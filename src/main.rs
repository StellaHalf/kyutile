use std::io::{self};

use state::State;

mod bar;
mod map;
mod parse;
mod state;
mod ui;

fn main() {
    match launch() {
        Ok(_) => (),
        Err(err) => eprintln!("An IO error has occurred: {}.", err),
    }
}

fn launch() -> Result<(), io::Error> {
    let mut terminal = ratatui::init();
    let mut state: State = State::new()?;

    while !state.exit() {
        terminal.draw(|frame| state.draw(frame))?;
        state.handle_events()?
    }

    ratatui::restore();
    Ok(())
}
