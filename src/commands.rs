use std::{
    fs::{read_to_string, write},
    io,
    num::ParseIntError,
};

use crate::parse::{export_map, parse_map};

pub(crate) enum State {
    Empty,
    Open { map: Vec<Vec<i32>>, path: String },
}

pub(crate) enum CommandError {
    ParseError(ParseIntError),
    IOError(io::Error),
    Empty,
}

impl State {
    pub(crate) fn open(&mut self, path: &str) -> Result<(), CommandError> {
        Ok(*self = State::Open {
            map: parse_map(&read_to_string(path).map_err(CommandError::IOError)?)
                .map_err(CommandError::ParseError)?,
            path: path.to_owned(),
        })
    }

    pub(crate) fn save(&self) -> Result<(), CommandError> {
        match self {
            State::Empty => Err(CommandError::Empty),
            State::Open { map, path } => {
                write(path, export_map(map)).map_err(CommandError::IOError)
            }
        }
    }

    pub(crate) fn display(&self) {
        println!(
            "{}",
            match self {
                State::Empty => "(empty)".to_owned(),
                State::Open { map, .. } => export_map(map),
            }
        )
    }
}
