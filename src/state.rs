use std::{
    collections::{HashMap, HashSet},
    fs::{read_to_string, write},
    io,
};

use crate::{bar::Input, map::bucket, parse::parse_tile_data};
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

pub(crate) fn parse_arg(arg: &str) -> Result<usize, String> {
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

    pub(crate) fn write(&mut self, _: &[&str]) -> Result<(), String> {
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

    pub(crate) fn bucket(&mut self, _: &[&str]) -> Result<(), String> {
        if let Some(map) = &mut self.map {
            if let Brush::Tile(tile) = self.brush {
                bucket(map, &self.select, tile)
            }
        };
        Ok(())
    }

    pub(crate) fn dot(&mut self, _: &[&str]) -> Result<(), String> {
        if let Some(map) = &mut self.map {
            match self.brush {
                Brush::Tile(tile) => dot(map, self.cursorx, self.cursory, tile),
                Brush::Add => {
                    self.select.insert((self.cursorx, self.cursory));
                }
                Brush::Subtract => {
                    self.select.remove(&(self.cursorx, self.cursory));
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
            _ => Err(format!("Mode {} not found.", args[0])),
        }
    }

    pub(crate) fn r#move(&mut self, args: &[&str]) -> Result<(), String> {
        let distance = args
            .get(1)
            .map(|s| {
                s.parse()
                    .map_err(|_| format!("Parse error: {} is not an integer.", s))
            })
            .transpose()?
            .unwrap_or(1);
        if let Some(map) = &self.map {
            let (dx, dy) = match args[0].to_lowercase().as_str() {
                "up" | "k" => (-1, 0),
                "down" | "j" => (1, 0),
                "left" | "h" => (0, -1),
                "right" | "l" => (0, 1),
                d => Err(format!("{} is not a direction.", d))?,
            };
            self.cursorx = (self.cursorx as isize + dx * distance as isize)
                .clamp(0, map.len() as isize - 1) as usize;
            self.cursory = (self.cursory as isize + dy * distance as isize)
                .clamp(0, map[0].len() as isize - 1) as usize;
            if self.mode == Mode::Draw {
                let _ = self.dot(&[]);
            }
        }
        Ok(())
    }

    pub(crate) fn edge(&mut self, args: &[&str]) -> Result<(), String> {
        Ok(if let Some(map) = &self.map {
            match args[0].to_lowercase().as_str() {
                "up" | "k" => self.cursorx = 0,
                "down" | "j" => self.cursorx = map.len() - 1,
                "left" | "h" => self.cursory = 0,
                "right" | "l" => self.cursory = map[0].len() - 1,
                d => Err(format!("{} is not a direction.", d))?,
            };
        })
    }

    pub(crate) fn goto(&mut self, args: &[&str]) -> Result<(), String> {
        if let Some(map) = &self.map {
            let i = parse_arg(args[0])?;
            let j = parse_arg(args[1])?;
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
            self.brush = Brush::Tile(map[self.cursorx][self.cursory]);
        }
        Ok(())
    }

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
}

const COMMANDS: [Command; 11] = [
    Command::new("open", &["o"], 1, 1, State::open),
    Command::new("write", &["w"], 0, 0, State::write),
    Command::new("quit", &["q"], 0, 0, State::quit),
    Command::new("brush", &["tile", "t"], 1, 1, State::brush),
    Command::new("dot", &[], 0, 0, State::dot),
    Command::new("bucket", &[], 0, 0, State::bucket),
    Command::new("move", &[], 1, 2, State::r#move),
    Command::new("pick", &[], 0, 0, State::pick),
    Command::new("mode", &[], 1, 1, State::mode),
    Command::new("edge", &[], 1, 1, State::edge),
    Command::new("goto", &["g"], 2, 2, State::goto),
];
