use std::{
    collections::{HashMap, HashSet},
    fs::{read_to_string, write},
    io,
};

use crate::{bar::Input, map::fill, parse::parse_tile_data};
use crate::{
    map::set,
    parse::{export_map, parse_map},
};

#[derive(PartialEq, Eq)]
pub(crate) enum Bar {
    Closed,
    Input(Input),
    Err(String),
}

const TILES_PATH: &str = "data/tiles.toml";

pub(crate) enum Mode {
    Normal,
    Draw,
    Add,
    Subtract,
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
    modified: bool,
    path: Option<String>,
    pub(crate) tile: i32,
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

impl State {
    pub(crate) fn new() -> Result<State, io::Error> {
        Ok(State {
            data: parse_tile_data(&read_to_string(TILES_PATH)?),
            exit: false,
            map: None,
            path: None,
            mode: Mode::Normal,
            cursorx: 0,
            cursory: 0,
            modified: false,
            tile: 0,
            select: HashSet::new(),
            bar: Bar::Closed,
        })
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

    pub(crate) fn save(&mut self, _args: &[&str]) -> Result<(), String> {
        match &self.path {
            None => Err("No path set (use :w <path>).".to_owned()),
            Some(path) => match &self.map {
                None => Err("No map in buffer (use :o <path> to open a map or :n <width> <height> to create a new one).".to_owned()),
                Some(map) => { write(path, export_map(map))
                .map_err(|_| format!("Could not write to file {}.", path))?; self.modified = false; Ok(()) },
            }
        }
    }

    pub(crate) fn quit(&mut self, _args: &[&str]) -> Result<(), String> {
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

    pub(crate) fn fill(&mut self) {
        if let Some(map) = &mut self.map {
            fill(map, &self.select, self.tile)
        }
    }

    pub(crate) fn set(&mut self) {
        if let Some(map) = &mut self.map {
            set(map, self.cursorx, self.cursory, self.tile)
        }
    }

    pub(crate) fn tile(&mut self, args: &[&str]) -> Result<(), String> {
        let tile = args[0];
        match self.data.names.iter().find(|(_, v)| tile == *v) {
            Some((k, _)) => Ok(self.tile = *k),
            None => match tile.parse::<i32>() {
                Err(_) => Err("Tile name not found.".to_owned()),
                Ok(i) => {
                    if self.data.colors.contains_key(&i) {
                        Ok(self.tile = i)
                    } else {
                        Err("Tile number not found.".to_owned())
                    }
                }
            },
        }
    }

    pub(crate) fn r#move(&mut self, direction: &str, distance: usize) -> Result<(), String> {
        if let Some(map) = &self.map {
            let (dx, dy) = match direction.to_lowercase().as_str() {
                "up" | "u" => (0, -1),
                "down" | "d" => (0, 1),
                "left" | "l" => (-1, 0),
                "right" | "r" => (1, 0),
                _ => Err(format!("{} is not a direction.", direction))?,
            };
            self.cursorx = (self.cursorx as isize + dx * distance as isize)
                .clamp(0, map.len() as isize) as usize;
            self.cursory = (self.cursory as isize + dy * distance as isize)
                .clamp(0, map[0].len() as isize) as usize;
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
}

const COMMANDS: [Command; 2] = [
    Command::new("open", &["o"], 1, 1, State::open),
    Command::new("write", &["w"], 0, 0, State::save),
];
