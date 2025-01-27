use std::{
    fmt,
    fs::{read_to_string, write},
    io,
    num::ParseIntError,
};

use ratatui::crossterm::event::KeyCode;

use crate::bar::Input;
use crate::parse::{export_map, parse_map};

pub(crate) enum Bar {
    Closed,
    Input(Input),
    Err(String),
}

pub(crate) struct State {
    exit: bool,
    map: Option<Vec<Vec<i32>>>,
    path: Option<String>,
    modified: bool,
    tile: i32,
    pub(crate) bar: Bar,
}

impl State {
    pub(crate) const NEW: State = State {
        exit: false,
        map: None,
        path: None,
        modified: false,
        tile: 0,
        bar: Bar::Closed,
    };

    pub(crate) fn exit(&self) -> bool {
        self.exit
    }

    pub(crate) fn map(&self) -> Box<&Option<Vec<Vec<i32>>>> {
        Box::new(&self.map)
    }

    pub(crate) fn clear_bar(&mut self) {
        self.bar = Bar::Closed
    }

    pub(crate) fn begin_input(&mut self) {
        self.bar = Bar::Input(Input::empty())
    }

    pub(crate) fn open(&mut self, path: &str) -> Result<(), String> {
        self.map = Some(
            parse_map(&read_to_string(path).map_err(|_| format!("Could not open file {}.", path))?)
                .map_err(|e| format!("Could not parse map: {}", e))?,
        );
        self.path = Some(path.to_owned());
        self.modified = false;
        Ok(())
    }

    pub(crate) fn save(&mut self) -> Result<(), String> {
        match &self.path {
            None => Err("No path set (use :w <path>).".to_owned()),
            Some(path) => match &self.map {
                None => Err("No map in buffer (use :o <path> to open a map or :n <width> <height> to create a new one).".to_owned()),
                Some(map) => { write(path, export_map(map))
                .map_err(|_| format!("Could not write to file {}.", path))?; self.modified = false; Ok(()) },
            }
        }
    }

    pub(crate) fn quit(&mut self) -> Result<(), String> {
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
    pub(crate) fn parse_command(&mut self, text: &str) -> Result<(), String> {
        let args: Vec<_> = text.split(" ").collect();
        if args.len() > 0 {
            match args[0] {
                "quit" | "q" => {
                    if args.len() == 1 {
                        self.quit()
                    } else {
                        Err(":quit takes no arguments.".to_owned())
                    }
                }
                "open" | "o" => {
                    if args.len() == 2 {
                        self.open(args[1])
                    } else {
                        Err("incorrect number of arguments for :open".to_owned())
                    }
                }
                "write" | "w" => self.save(),
                _ => Err("command not found".to_owned()),
            }
        } else {
            Ok(())
        }
    }
}
