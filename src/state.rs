use std::{
    collections::{HashMap, HashSet},
    fs::{read_to_string, write},
    io,
};

use itertools::Itertools;
use ratatui::crossterm::event::KeyCode;

use crate::{
    bar::Input,
    map::{create, draw_all, in_bounds, validate},
    tiles::TILES,
};
use crate::{
    files::{export_map, parse_map},
    map::dot,
};

#[derive(PartialEq, Eq)]
pub(crate) enum Bar {
    Closed,
    Input(Input),
    Err(String),
}

#[derive(PartialEq, Eq)]
pub(crate) enum Pen {
    Up,
    Down,
}

pub(crate) enum Brush {
    Add,
    Subtract,
    Tile(i32),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Map {
    pub(crate) map: Vec<Vec<i32>>,
    pub(crate) select: HashSet<(usize, usize)>,
}

pub(crate) struct Clipboard {
    pub(crate) content: HashMap<(usize, usize), i32>,
    pub(crate) offsetx: usize,
    pub(crate) offsety: usize,
}

pub(crate) struct State {
    pub(crate) argument: usize,
    pub(crate) bar: Bar,
    pub(crate) brush: Brush,
    pub(crate) clipboard: Option<Clipboard>,
    pub(crate) cursorx: usize,
    pub(crate) cursory: usize,
    pub(crate) exit: bool,
    pub(crate) last_saved: Option<Vec<Vec<i32>>>,
    pub(crate) map: Map,
    pub(crate) path: Option<String>,
    pub(crate) pen: Pen,
    redo_stack: Vec<Map>,
    undo_stack: Vec<Map>,
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
fn parse_tile(tile: &str) -> Result<i32, String> {
    match TILES
        .tiles
        .iter()
        .find(|v| v.1.to_lowercase() == tile.to_lowercase())
    {
        Some(t) => Ok(t.0 as i32),
        None => {
            if let Ok(i) = parse_usize(tile) {
                if TILES.tiles.iter().any(|t| t.0 as usize == i) {
                    Ok(i as i32)
                } else {
                    Err(format!("Parse error: invalid tile number {}.", i))
                }
            } else {
                Err(format!("Parse error: {} is not a valid tile.", tile))
            }
        }
    }
}

impl State {
    pub(crate) fn new() -> Result<State, io::Error> {
        Ok(State {
            map: Map {
                map: create(11, 11, 0),
                select: HashSet::new(),
            },
            clipboard: None,
            last_saved: None,
            exit: false,
            path: None,
            pen: Pen::Up,
            cursorx: 0,
            cursory: 0,
            argument: 0,
            brush: Brush::Tile(0),
            bar: Bar::Closed,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub(crate) fn modified(&self) -> bool {
        match &self.last_saved {
            None => false,
            Some(saved) => self.map.map != *saved,
        }
    }

    fn push_undo(&mut self, map_clone: Map) {
        self.undo_stack.push(map_clone);
        self.redo_stack.clear();
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
        if self.modified() {
            Err(
                "Unsaved changes (use :o! to discard them and open another file or :w to save them).".to_owned(),
            )
        } else {
            self.open_force(args)
        }
    }

    fn open_force(&mut self, args: &[&str]) -> Result<(), String> {
        let path = args[0];
        let map =
            parse_map(&read_to_string(path).map_err(|_| format!("Could not open file {}.", path))?)
                .map_err(|_| "Could not parse map.")?;
        match validate(&map) {
            Ok(_) => {
                self.map = Map {
                    map: map.clone(),
                    select: HashSet::new(),
                };
                self.path = Some(path.to_owned());
                self.last_saved = Some(map);
                Ok(())
            }
            Err(err) => Err(format!("Could not validate map: {}", err)),
        }
    }

    pub(crate) fn write(&mut self, args: &[&str]) -> Result<(), String> {
        if let Some(&path) = args.first() {
            self.path = Some(path.to_owned())
        }
        match &self.path {
            None => Err("No path set (use :w <path>).".to_owned()),
            Some(path) => {
                write(path, export_map(&self.map.map))
                    .map_err(|_| format!("Could not write to file {}.", path))?;
                self.last_saved = Some(self.map.map.clone());
                Ok(())
            }
        }
    }

    pub(crate) fn quit(&mut self, _: &[&str]) -> Result<(), String> {
        if self.modified() {
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
        let map_clone = self.map.clone();
        if let Brush::Tile(tile) = self.brush {
            if draw_all(&mut self.map.map, self.map.select.clone(), tile) {
                self.push_undo(map_clone);
            }
        }
        Ok(())
    }

    pub(crate) fn dot(&mut self, _: &[&str]) -> Result<(), String> {
        let map_clone = self.map.clone();
        match self.brush {
            Brush::Tile(tile) => {
                if dot(&mut self.map.map, self.cursory, self.cursorx, tile) {
                    self.push_undo(map_clone);
                }
            }
            Brush::Add => {
                if self.map.select.insert((self.cursory, self.cursorx)) {
                    self.push_undo(map_clone);
                }
            }
            Brush::Subtract => {
                if self.map.select.remove(&(self.cursory, self.cursorx)) {
                    self.push_undo(map_clone);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn brush(&mut self, args: &[&str]) -> Result<(), String> {
        match args[0].to_lowercase().as_str() {
            "add" => {
                self.brush = Brush::Add;
            }
            "subtract" => {
                self.brush = Brush::Subtract;
            }
            tile => self.brush = Brush::Tile(parse_tile(tile)?),
        };
        Ok(())
    }

    pub(crate) fn pen(&mut self, args: &[&str]) -> Result<(), String> {
        match args[0].to_lowercase().as_str() {
            "up" => {
                self.pen = Pen::Up;
                Ok(())
            }
            "down" => {
                self.pen = Pen::Down;
                Ok(())
            }
            _ => Err(format!(
                "Pen mode {} not found, options are up, down.",
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
                let ny = (self.cursory + distance).min(self.map.map.len() - 1);
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
                let nx = (self.cursorx + distance).min(self.map.map[0].len() - 1);
                (
                    nx,
                    self.cursory,
                    (self.cursorx + 1..=nx).map(|x| (self.cursory, x)).collect(),
                )
            }
        };
        self.cursorx = nx;
        self.cursory = ny;
        if self.pen == Pen::Down {
            let map_clone = self.map.clone();
            match self.brush {
                Brush::Tile(tile) => {
                    for &(i, j) in &positions {
                        dot(&mut self.map.map, i, j, tile);
                    }
                }
                Brush::Add => self.map.select.extend(positions),
                Brush::Subtract => {
                    for p in positions {
                        self.map.select.remove(&p);
                    }
                }
            }
            if self.map != map_clone {
                self.push_undo(map_clone);
            }
        }
        Ok(())
    }

    pub(crate) fn edge(&mut self, args: &[&str]) -> Result<(), String> {
        match parse_direction(args[0])? {
            Direction::Left => self.move_cursor(Direction::Left, self.cursorx),
            Direction::Down => {
                self.move_cursor(Direction::Down, self.map.map[0].len() - self.cursory)
            }
            Direction::Up => self.move_cursor(Direction::Up, self.cursory),
            Direction::Right => {
                self.move_cursor(Direction::Right, self.map.map[0].len() - self.cursorx)
            }
        }
    }

    pub(crate) fn goto(&mut self, args: &[&str]) -> Result<(), String> {
        let i = parse_usize(args[0])?;
        let j = parse_usize(args[1])?;
        if in_bounds(self.map.map.len(), self.map.map[0].len(), i, j) {
            self.cursorx = i;
            self.cursory = j;
            Ok(())
        } else {
            Err("Out of bounds.".to_owned())
        }
    }

    pub(crate) fn pick(&mut self, _: &[&str]) -> Result<(), String> {
        self.brush = Brush::Tile(self.map.map[self.cursory][self.cursorx]);
        Ok(())
    }

    pub(crate) fn select(&mut self, args: &[&str]) -> Result<(), String> {
        let map_clone = self.map.clone();
        match args[0].to_lowercase().as_str() {
            "all" => {
                self.map.select =
                    ((0..self.map.map.len()).cartesian_product(0..self.map.map[0].len())).collect();
                Ok(())
            }
            "none" => {
                self.map.select.clear();
                Ok(())
            }
            "invert" => {
                self.map.select = ((0..self.map.map.len())
                    .cartesian_product(0..self.map.map[0].len()))
                .filter(|p| !self.map.select.contains(p))
                .collect();
                Ok(())
            }
            arg => match parse_tile(arg) {
                Ok(tile) => {
                    let positions = (0..self.map.map.len())
                        .cartesian_product(0..self.map.map[0].len())
                        .filter(|&(i, j)| self.map.map[i][j] == tile);
                    match self.brush {
                        Brush::Add => self.map.select.extend(positions),
                        Brush::Subtract => {
                            for p in positions {
                                self.map.select.remove(&p);
                            }
                        }
                        _ => self.map.select = positions.collect(),
                    }
                    Ok(())
                }
                _ => Err("Invalid selection argument, options are all, none, invert and <tile>."),
            },
        }?;
        if self.map.select != map_clone.select {
            self.push_undo(map_clone);
        };
        Ok(())
    }

    pub(crate) fn draw_shape<F, I>(&mut self, args: &[&str], shape: F) -> Result<(), String>
    where
        F: FnOnce(&[&str]) -> Result<I, String>,
        I: IntoIterator<Item = (usize, usize)>,
    {
        let (lx, ly) = (self.map.map.len(), self.map.map[0].len());
        let positions = shape(args)?
            .into_iter()
            .filter(|(x, y)| in_bounds(lx, ly, *x, *y));
        let map_clone = self.map.clone();
        match &self.brush {
            Brush::Add => {
                self.map.select.extend(positions);
                if self.map.select != map_clone.select {
                    self.push_undo(map_clone);
                }
            }
            Brush::Subtract => {
                for p in positions {
                    self.map.select.remove(&p);
                }
                if self.map.select != map_clone.select {
                    self.push_undo(map_clone);
                }
            }
            Brush::Tile(tile) => {
                if draw_all(&mut self.map.map, positions, *tile) {
                    self.push_undo(map_clone);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn fuzzy(&mut self, args: &[&str]) -> Result<(), String> {
        let map = self.map.map.clone();
        let cursorx = self.cursorx;
        let cursory = self.cursory;
        self.draw_shape(args, |args| {
            let mut reached = HashSet::new();
            let mut frontier = HashSet::new();
            let mut new_frontier = HashSet::new();
            frontier.insert((cursory, cursorx));
            let tile = map[cursory][cursorx];
            let mut i = match args.get(0) {
                Some(arg) => parse_usize(arg)? as isize,
                None => -1,
            };
            while !frontier.is_empty() && i != 0 {
                i -= 1;
                reached.extend(&frontier);
                new_frontier.clear();
                for (i, j) in frontier.clone() {
                    for (di, dj) in vec![(-1, 0), (0, -1), (1, 0), (0, 1)] {
                        let ni = i as isize + di;
                        let nj = j as isize + dj;
                        if ni >= 0
                            && ni < map.len() as isize
                            && nj >= 0
                            && nj < map[0].len() as isize
                            && !reached.contains(&(ni as usize, nj as usize))
                            && map[ni as usize][nj as usize] == tile
                        {
                            new_frontier.insert((ni as usize, nj as usize));
                        }
                    }
                    frontier.clear();
                    frontier.extend(new_frontier.clone());
                }
            }
            Ok(reached)
        })
    }

    pub(crate) fn r#box(&mut self, args: &[&str]) -> Result<(), String> {
        self.draw_shape::<_, Vec<_>>(args, |args| {
            let (x0, y0, x1, y1) = (
                parse_usize(args[0])?,
                parse_usize(args[1])?,
                parse_usize(args[2])?,
                parse_usize(args[3])?,
            );
            let fill = if let Some(arg) = args.get(4) {
                if *arg == "fill" || *arg == "true" {
                    true
                } else {
                    return Err("Invalid argument, the only option is fill (optional).".to_owned());
                }
            } else {
                false
            };
            Ok(if fill {
                (y0..=y1).cartesian_product(x0..=x1).collect()
            } else {
                (y0..=y1)
                    .map(|x| (x, x0))
                    .chain((y0..=y1).map(|x| (x, x1)))
                    .chain((x0 + 1..x1).map(|y| (y0, y)))
                    .chain((x0 + 1..x1).map(|y| (y1, y)))
                    .collect()
            })
        })
    }

    //adapted from http://members.chello.at/~easyfilter/bresenham.html
    pub(crate) fn ellipse(&mut self, args: &[&str]) -> Result<(), String> {
        self.draw_shape(args, |args| {
            let (x0, y0, x1, y1) = (
                parse_usize(args[0])? as isize,
                parse_usize(args[1])? as isize,
                parse_usize(args[2])? as isize,
                parse_usize(args[3])? as isize,
            );
            let fill = if let Some(arg) = args.get(4) {
                if *arg == "fill" || *arg == "true" {
                    true
                } else {
                    return Err("Invalid argument, the only option is fill (optional)".to_owned());
                }
            } else {
                false
            };
            let mut positions = Vec::new();
            let a = (x1 - x0).abs();
            let b = (y1 - y0).abs();
            let bp = b & 1;
            let mut dx = 4 * (1 - a) * b * b;
            let mut dy = 4 * (bp + 1) * a * a;
            let mut e = dx + dy + bp * a * a;
            let (mut x0, mut x1) = if x0 > x1 { (x1, x0) } else { (x0, x1) };
            let mut y0 = if y1 > y0 { y0 } else { y1 };
            y0 += (b + 1) / 2;
            let mut y1 = y0 - bp;
            let a = 8 * a * a;
            let mut e2;
            while x0 <= x1 {
                let x0u = x0 as usize;
                let x1u = x1 as usize;
                let y0u = y0 as usize;
                let y1u = y1 as usize;
                if fill {
                    positions.extend(
                        (x0u..=x1u)
                            .map(|x| (x, y0u))
                            .chain((x0u..=x1u).map(|x| (x, y1u))),
                    )
                } else {
                    positions.extend_from_slice(&[(x1u, y0u), (x0u, y0u), (x0u, y1u), (x1u, y1u)])
                };
                e2 = 2 * e;
                if e2 <= dy {
                    y0 += 1;
                    y1 -= 1;
                    dy += a;
                    e += dy;
                }
                if e2 >= dx || 2 * e > dy {
                    x0 += 1;
                    x1 -= 1;
                    dx += 8 * b * b;
                    e += dx;
                }
            }
            Ok(positions)
        })
    }

    pub(crate) fn create(&mut self, args: &[&str]) -> Result<(), String> {
        let new_map = Map {
            map: create(parse_usize(args[1])?, parse_usize(args[0])?, 0),
            select: HashSet::new(),
        };
        if self.map != new_map {
            self.push_undo(self.map.clone());
            self.map = new_map;
        }
        self.reset_cursor();
        Ok(())
    }

    pub(crate) fn reset_cursor(&mut self) {
        if !in_bounds(
            self.map.map[0].len(),
            self.map.map.len(),
            self.cursorx,
            self.cursory,
        ) {
            self.cursorx = 0;
            self.cursory = 0;
        }
    }

    pub(crate) fn undo(&mut self, _: &[&str]) -> Result<(), String> {
        match self.undo_stack.pop() {
            None => Err("Undo stack is empty.".to_owned()),
            Some(map) => {
                self.redo_stack.push(self.map.clone());
                self.map = map;
                self.reset_cursor();
                Ok(())
            }
        }
    }

    pub(crate) fn redo(&mut self, _: &[&str]) -> Result<(), String> {
        match self.redo_stack.pop() {
            None => Err("Redo stack is empty".to_owned()),
            Some(map) => {
                self.undo_stack.push(self.map.clone());
                self.map = map;
                self.reset_cursor();
                Ok(())
            }
        }
    }

    pub(crate) fn parse_command(&mut self, text: &str) -> Result<(), String> {
        if let Some((name, args)) = text
            .split(" ")
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .split_first()
        {
            match COMMANDS
                .iter()
                .find(|c| c.name == *name || c.aliases.contains(name))
            {
                None => Err(format!("Command {} not found.", name)),
                Some(command) => {
                    if args.len() >= command.argsmin && args.len() <= command.argsmax {
                        (command.function)(self, args)
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
            "Path: {}{}, Pen: {}, Brush: {}, Cursor: ({},{}), Argument: {}",
            self.path.clone().unwrap_or("[-]".to_owned()),
            if self.modified() { "(*)" } else { "" },
            match self.pen {
                Pen::Up => "Up",
                Pen::Down => "Down",
            },
            match self.brush {
                Brush::Add => "add",
                Brush::Subtract => "subtract",
                Brush::Tile(tile) => TILES.tiles.iter().find(|t| t.0 as i32 == tile).unwrap().1,
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

    pub(crate) fn copy(&mut self, _: &[&str]) -> Result<(), String> {
        self.clipboard = Some(Clipboard {
            content: self
                .map
                .select
                .iter()
                .map(|(i, j)| ((*i, *j), self.map.map[*i][*j]))
                .collect(),
            offsetx: self.cursorx,
            offsety: self.cursory,
        });
        Ok(())
    }

    pub(crate) fn paste(&mut self, _: &[&str]) -> Result<(), String> {
        let ly = self.map.map[0].len();
        let lx = self.map.map.len();
        let map_clone = self.map.clone();
        if let Some(clipboard) = &self.clipboard {
            for (i, j, tile) in clipboard
                .content
                .iter()
                .map(|((i, j), tile)| {
                    (
                        (*i as isize + self.cursory as isize - (clipboard.offsety as isize))
                            as usize,
                        (*j as isize + self.cursorx as isize - (clipboard.offsetx as isize))
                            as usize,
                        tile,
                    )
                })
                .filter(|(i, j, _)| in_bounds(lx, ly, *i, *j))
            {
                self.map.map[i][j] = *tile;
            }
            if self.map.map != map_clone.map {
                self.push_undo(map_clone);
            }
            Ok(())
        } else {
            Err("Clipboard is empty".to_owned())
        }
    }

    fn move_with(&mut self, direction: Direction) -> Result<(), String> {
        self.move_cursor(direction, self.argument.max(1))?;
        self.argument = 0;
        Ok(())
    }
    pub(crate) fn receive_key_closed(&mut self, code: KeyCode) {
        let _ = match &code {
            KeyCode::Char(':') => {
                self.bar = Bar::Input(Input::empty());
                Ok(())
            }
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
            KeyCode::Char('i') => self.pen(&["down"]),
            KeyCode::Char('I') => self.pen(&["up"]),
            KeyCode::Char('A') => self.select(&["all"]),
            KeyCode::Char('S') => self.select(&["none"]),
            KeyCode::Char('F') => self.select(&["invert"]),
            KeyCode::Esc => {
                self.argument = 0;
                Ok(())
            }
            KeyCode::Char('f') => self.bucket(&[]),
            KeyCode::Char('p') => self.pick(&[]),
            KeyCode::Char('u') => self.undo(&[]),
            KeyCode::Char('U') => self.redo(&[]),
            KeyCode::Char('o') => self.copy(&[]),
            KeyCode::Char('O') => self.paste(&[]),
            KeyCode::Char(c) => {
                if let Some(i) = c.to_digit(10) {
                    self.append_argument(i as u8)
                };
                Ok(())
            }
            _ => Ok(()),
        };
    }
}

const COMMANDS: [Command; 23] = [
    Command::new("open", &["o"], 1, 1, State::open),
    Command::new("open!", &["o!"], 1, 1, State::open_force),
    Command::new("write", &["w"], 0, 1, State::write),
    Command::new("quit", &["q"], 0, 0, State::quit),
    Command::new("quit!", &["q!"], 0, 0, State::quit_force),
    Command::new("write-quit", &["wq"], 0, 1, State::write_quit),
    Command::new("brush", &["tile", "t"], 1, 1, State::brush),
    Command::new("dot", &[], 0, 0, State::dot),
    Command::new("bucket", &[], 0, 0, State::bucket),
    Command::new("move", &[], 1, 2, State::r#move),
    Command::new("pick", &[], 0, 0, State::pick),
    Command::new("pen", &[], 1, 1, State::pen),
    Command::new("edge", &[], 1, 1, State::edge),
    Command::new("goto", &["g"], 2, 2, State::goto),
    Command::new("select", &["s"], 1, 1, State::select),
    Command::new("undo", &[], 0, 0, State::undo),
    Command::new("redo", &[], 0, 0, State::redo),
    Command::new("create", &["n"], 2, 2, State::create),
    Command::new("box", &["b"], 4, 5, State::r#box),
    Command::new("ellipse", &["e"], 4, 5, State::ellipse),
    Command::new("fuzzy", &["f"], 0, 1, State::fuzzy),
    Command::new("copy", &[], 0, 0, State::copy),
    Command::new("paste", &[], 0, 0, State::paste),
];
