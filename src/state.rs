use std::{
    collections::{HashMap, HashSet},
    fs::{read_to_string, write},
    io,
};

use itertools::Itertools;
use ratatui::crossterm::event::KeyCode;

use crate::{bar::Input, map::draw_all, parse::parse_tile_data};
use crate::{
    map::dot,
    parse::{export_map, parse_map},
};

#[derive(PartialEq, Eq)]
pub(crate) enum Bar {
    Closed,
    Input(Input),
    Err(String),
}

const CONFIG_PATH: &str = "config/";
const TILE_PATH: &str = "tiles.toml";

#[derive(PartialEq, Eq)]
pub(crate) enum Mode {
    Normal,
    Draw,
}

pub(crate) enum Brush {
    Add,
    Subtract,
    Tile(i32),
}

pub(crate) struct TileData {
    pub(crate) names: HashMap<i32, String>,
    pub(crate) colors: HashMap<i32, u32>,
}

pub(crate) struct State {
    pub(crate) bar: Bar,
    pub(crate) cursorx: usize,
    pub(crate) cursory: usize,
    pub(crate) mode: Mode,
    pub(crate) select: HashSet<(usize, usize)>,
    pub(crate) data: TileData,
    pub(crate) exit: bool,
    pub(crate) map: Option<Vec<Vec<i32>>>,
    pub(crate) argument: usize,
    pub(crate) modified: bool,
    pub(crate) path: Option<String>,
    pub(crate) brush: Brush,
}

struct Command {
    name: &'static str,
    aliases: &'static [&'static str],
    argsmin: usize,
    argsmax: usize,
    function: fn(&mut State, &[&str]) -> Result<(), String>,
}

impl Command {
    const fn new(
        name: &'static str,
        aliases: &'static [&'static str],
        argsmin: usize,
        argsmax: usize,
        function: fn(&mut State, &[&str]) -> Result<(), String>,
    ) -> Self {
        Command {
            name,
            aliases,
            argsmin,
            argsmax,
            function,
        }
    }
}

enum Direction {
    Left,
    Down,
    Up,
    Right,
}

fn parse_direction(arg: &str) -> Result<Direction, String> {
    match arg.to_lowercase().as_str() {
        "left" | "h" => Ok(Direction::Left),
        "down" | "j" => Ok(Direction::Down),
        "up" | "k" => Ok(Direction::Up),
        "right" | "l" => Ok(Direction::Right),
        _ => Err(format!(
            "Parse error: {} is not a direction, options are Left, Down, Up, Right.",
            arg
        )),
    }
}

fn parse_usize(arg: &str) -> Result<usize, String> {
    arg.parse()
        .map_err(|_| format!("Parse error: {} is not an integer.", arg))
}

impl State {
    pub(crate) fn new() -> Result<State, io::Error> {
        Ok(State {
            data: parse_tile_data(&read_to_string(CONFIG_PATH.to_owned() + TILE_PATH)?),
            exit: false,
            map: None,
            path: None,
            mode: Mode::Normal,
            cursorx: 0,
            cursory: 0,
            modified: false,
            argument: 0,
            brush: Brush::Tile(0),
            select: HashSet::new(),
            bar: Bar::Closed,
        })
    }

    pub(crate) fn append_argument(&mut self, digit: u8) {
        self.argument = (if let Some(p) = self.argument.checked_mul(10) {
            p.checked_add(digit.into())
        } else {
            None
        })
        .unwrap_or(self.argument)
    }

    pub(crate) fn open(&mut self, args: &[&str]) -> Result<(), String> {
        let path = args[0];
        self.map = Some(
            parse_map(&read_to_string(path).map_err(|_| format!("Could not open file {}.", path))?)
                .map_err(|e| format!("Could not parse map: {}", e))?,
        );
        self.path = Some(path.to_owned());
        self.modified = false;
        Ok(())
    }

    pub(crate) fn write(&mut self, args: &[&str]) -> Result<(), String> {
        if let Some(&path) = args.first() {
            self.path = Some(path.to_owned())
        }
        match &self.path {
            None => Err("No path set (use :w <path>).".to_owned()),
            Some(path) => match &self.map {
                None => Err("No map in buffer (use :o <path> to open a map or :n <width> <height> to create a new one).".to_owned()),
                Some(map) => { write(path, export_map(map))
                .map_err(|_| format!("Could not write to file {}.", path))?; self.modified = false; Ok(()) },
            }
        }
    }

    pub(crate) fn quit(&mut self, _: &[&str]) -> Result<(), String> {
        if self.modified {
            Err(
                "Unsaved changes (use :q! to discard them and quit or :wq to save and quit)."
                    .to_owned(),
            )
        } else {
            self.exit = true;
            Ok(())
        }
    }

    pub(crate) fn quit_force(&mut self, _: &[&str]) -> Result<(), String> {
        self.exit = true;
        Ok(())
    }

    pub(crate) fn write_quit(&mut self, args: &[&str]) -> Result<(), String> {
        self.write(args)?;
        self.quit(&[])?;
        Ok(())
    }
    pub(crate) fn bucket(&mut self, _: &[&str]) -> Result<(), String> {
        if let Some(map) = &mut self.map {
            if let Brush::Tile(tile) = self.brush {
                if draw_all(map, &mut self.select.iter(), tile) {
                    self.modified = true;
                }
            }
        };
        Ok(())
    }

    pub(crate) fn dot(&mut self, _: &[&str]) -> Result<(), String> {
        if let Some(map) = &mut self.map {
            match self.brush {
                Brush::Tile(tile) => {
                    if dot(map, self.cursory, self.cursorx, tile) {
                        self.modified = true
                    }
                }
                Brush::Add => {
                    self.select.insert((self.cursory, self.cursorx));
                }
                Brush::Subtract => {
                    self.select.remove(&(self.cursory, self.cursorx));
                }
            }
        };
        Ok(())
    }

    pub(crate) fn brush(&mut self, args: &[&str]) -> Result<(), String> {
        match args[0].to_lowercase().as_str() {
            "add" => Ok(self.brush = Brush::Add),
            "subtract" => Ok(self.brush = Brush::Subtract),
            tile => match self
                .data
                .names
                .iter()
                .find(|(_, v)| tile.to_lowercase() == *v.to_lowercase())
            {
                Some((k, _)) => Ok(self.brush = Brush::Tile(*k)),
                None => match tile.parse::<i32>() {
                    Err(_) => Err(format!("Tile {} not found.", tile)),
                    Ok(i) => {
                        if self.data.colors.contains_key(&i) {
                            Ok(self.brush = Brush::Tile(i))
                        } else {
                            Err(format!("Tile number {} not found", tile))
                        }
                    }
                },
            },
        }
    }

    pub(crate) fn mode(&mut self, args: &[&str]) -> Result<(), String> {
        match args[0].to_lowercase().as_str() {
            "normal" => Ok(self.mode = Mode::Normal),
            "draw" => Ok(self.mode = Mode::Draw),
            _ => Err(format!(
                "Mode {} not found, options are Normal, Draw.",
                args[0]
            )),
        }
    }

    pub(crate) fn r#move(&mut self, args: &[&str]) -> Result<(), String> {
        let distance = args
            .get(1)
            .map(|s| parse_usize(s))
            .transpose()?
            .unwrap_or(1);
        self.move_cursor(parse_direction(args[0])?, distance)?;
        Ok(())
    }

    fn move_cursor(&mut self, direction: Direction, distance: usize) -> Result<(), String> {
        Ok(if let Some(map) = &mut self.map {
            let (nx, ny, positions) = match direction {
                Direction::Left => {
                    let nx = (self.cursorx as isize - distance as isize).max(0) as usize;
                    (
                        nx,
                        self.cursory,
                        (nx..self.cursorx)
                            .map(|x| (self.cursory, x))
                            .collect::<Vec<_>>(),
                    )
                }
                Direction::Down => {
                    let ny = (self.cursory + distance).min(map.len() - 1);
                    (
                        self.cursorx,
                        ny,
                        (self.cursory + 1..=ny).map(|y| (y, self.cursorx)).collect(),
                    )
                }
                Direction::Up => {
                    let ny = (self.cursory as isize - distance as isize).max(0) as usize;
                    (
                        self.cursorx,
                        ny,
                        (ny..self.cursory).map(|y| (y, self.cursorx)).collect(),
                    )
                }
                Direction::Right => {
                    let nx = (self.cursorx + distance).min(map[0].len() - 1);
                    (
                        nx,
                        self.cursory,
                        (self.cursorx + 1..=nx).map(|x| (self.cursory, x)).collect(),
                    )
                }
            };
            self.cursorx = nx;
            self.cursory = ny;
            if self.mode == Mode::Draw {
                match self.brush {
                    Brush::Tile(tile) => {
                        for &(i, j) in &positions {
                            if dot(map, i, j, tile) {
                                self.modified = true;
                            }
                        }
                    }
                    Brush::Add => self.select.extend(positions.iter()),
                    Brush::Subtract => {
                        for p in positions {
                            self.select.remove(&p);
                        }
                    }
                }
            }
        })
    }

    pub(crate) fn edge(&mut self, args: &[&str]) -> Result<(), String> {
        if let Some(map) = &self.map {
            match parse_direction(args[0])? {
                Direction::Left => self.move_cursor(Direction::Left, self.cursorx),
                Direction::Down => self.move_cursor(Direction::Down, map[0].len() - self.cursory),
                Direction::Up => self.move_cursor(Direction::Up, self.cursory),
                Direction::Right => self.move_cursor(Direction::Left, map.len() - self.cursorx),
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn goto(&mut self, args: &[&str]) -> Result<(), String> {
        if let Some(map) = &self.map {
            let i = parse_usize(args[0])?;
            let j = parse_usize(args[1])?;
            if i < map.len() && j < map[0].len() {
                self.cursorx = i;
                self.cursory = j;
                Ok(())
            } else {
                Err("Out of bounds.".to_owned())
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn pick(&mut self, _: &[&str]) -> Result<(), String> {
        if let Some(map) = &self.map {
            self.brush = Brush::Tile(map[self.cursory][self.cursorx]);
        }
        Ok(())
    }

    pub(crate) fn select(&mut self, args: &[&str]) -> Result<(), String> {
        if let Some(map) = &self.map {
            match args[0].to_lowercase().as_str() {
                "all" => {
                    Ok(self.select = ((0..map.len()).cartesian_product(0..map[0].len())).collect())
                }
                "none" => Ok(self.select.clear()),
                "invert" => Ok(
                    self.select = ((0..map.len()).cartesian_product(0..map[0].len()))
                        .filter(|p| !self.select.contains(p))
                        .collect(),
                ),
                _ => Err(format!(
                    "Option {} not found. Options are all, none, invert.",
                    args[0]
                )),
            }
        } else {
            Ok(())
        }
    }

    //TODO: Fuzzy, Select-tile, circle, square, undo

    pub(crate) fn parse_command(&mut self, text: &str) -> Result<(), String> {
        if let Some((name, args)) = text.split(" ").collect::<Vec<_>>().split_first() {
            match COMMANDS
                .iter()
                .find(|c| c.name == *name || c.aliases.contains(&name))
            {
                None => Err(format!("Command {} not found.", name)),
                Some(command) => {
                    if args.len() >= command.argsmin && args.len() <= command.argsmax {
                        (command.function)(self, &args)
                    } else {
                        Err(format!(
                            "Incorrect number of arguments for {}: expected {}, found {}.",
                            command.name,
                            if command.argsmin == command.argsmax {
                                command.argsmin.to_string()
                            } else {
                                format!("{}-{}", command.argsmin, command.argsmax)
                            },
                            args.len()
                        ))
                    }
                }
            }
        } else {
            Ok(())
        }
    }

    pub(crate) fn info_bar(&self) -> String {
        format!(
            "Path: {}{}, Mode: {}, Brush: {}, Cursor: ({},{}), Argument: {}",
            self.path.clone().unwrap_or("[-]".to_owned()),
            if self.modified { "(*)" } else { "" },
            match self.mode {
                Mode::Normal => "normal",
                Mode::Draw => "draw",
            },
            match self.brush {
                Brush::Add => "add",
                Brush::Subtract => "subtract",
                Brush::Tile(tile) => &self.data.names[&tile],
            },
            self.cursorx,
            self.cursory,
            if self.argument > 0 {
                self.argument.to_string()
            } else {
                "".to_owned()
            }
        )
    }

    fn move_with(&mut self, direction: Direction) -> Result<(), String> {
        self.move_cursor(direction, self.argument.max(1))?;
        self.argument = 0;
        Ok(())
    }
    pub(crate) fn receive_key_closed(&mut self, code: KeyCode) {
        let _ = match &code {
            KeyCode::Char(':') => Ok(self.bar = Bar::Input(Input::empty())),
            KeyCode::Char('h') | KeyCode::Left => self.move_with(Direction::Left),
            KeyCode::Char('j') | KeyCode::Down => self.move_with(Direction::Down),
            KeyCode::Char('k') | KeyCode::Up => self.move_with(Direction::Up),
            KeyCode::Char('l') | KeyCode::Right => self.move_with(Direction::Right),
            KeyCode::Char('H') => self.edge(&["left"]),
            KeyCode::Char('J') => self.edge(&["down"]),
            KeyCode::Char('K') => self.edge(&["up"]),
            KeyCode::Char('L') => self.edge(&["right"]),
            KeyCode::Char('d') => self.dot(&[]),
            KeyCode::Char('a') => self.brush(&["add"]),
            KeyCode::Char('s') => self.brush(&["subtract"]),
            KeyCode::Char('i') => self.mode(&["draw"]),
            KeyCode::Char('I') => self.mode(&["normal"]),
            KeyCode::Char('A') => self.select(&["all"]),
            KeyCode::Char('S') => self.select(&["none"]),
            KeyCode::Char('F') => self.select(&["invert"]),
            KeyCode::Esc => {
                self.argument = 0;
                Ok(())
            }
            KeyCode::Char('f') => self.bucket(&[]),
            KeyCode::Char('p') => self.pick(&[]),
            KeyCode::Char(c) => Ok(if let Some(i) = c.to_digit(10) {
                self.append_argument(i as u8)
            }),
            _ => Ok(()),
        };
    }
}

const COMMANDS: [Command; 14] = [
    Command::new("open", &["o"], 1, 1, State::open),
    Command::new("write", &["w"], 0, 1, State::write),
    Command::new("quit", &["q"], 0, 0, State::quit),
    Command::new("quit!", &["q!"], 0, 0, State::quit_force),
    Command::new("write-quit", &["wq"], 0, 1, State::write_quit),
    Command::new("brush", &["tile", "t"], 1, 1, State::brush),
    Command::new("dot", &[], 0, 0, State::dot),
    Command::new("bucket", &[], 0, 0, State::bucket),
    Command::new("move", &[], 1, 2, State::r#move),
    Command::new("pick", &[], 0, 0, State::pick),
    Command::new("mode", &[], 1, 1, State::mode),
    Command::new("edge", &[], 1, 1, State::edge),
    Command::new("goto", &["g"], 2, 2, State::goto),
    Command::new("select", &[], 1, 1, State::select),
];
