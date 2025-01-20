use std::{
    error,
    io::{self, stdout},
};

use commands::State;
use ratatui::{prelude::CrosstermBackend, Terminal};

mod commands;
mod map;
mod parse;
mod ui;

fn main() {
    match launch() {
        Ok(_) => (),
        Err(err) => eprintln!("An IO error has occurred: {}.", err),
    }
}

fn launch() -> Result<(), io::Error> {
    let mut terminal = ratatui::init();
    let mut state: State = State::NEW;

    while !state.is_exit() {
        terminal.draw(|frame| state.draw(frame))?;
        state.handle_events()?
    }

    Ok(())
}
