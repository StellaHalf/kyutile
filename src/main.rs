use std::{
    env,
    io::{self},
    path::Path,
};

use state::State;

mod bar;
mod map;
mod parse;
mod state;
mod ui;

const VERSION: &str = "1.0";
const HELP: &str = "Help!";

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.contains(&"--help".to_owned()) || args.contains(&"-h".to_owned()) {
        println!("{}", HELP)
    } else if args.contains(&"--version".to_owned()) | args.contains(&"-V".to_owned()) {
        println!("kyutile {}", VERSION)
    } else {
        match launch(&args.get(0)) {
            Ok(_) => (),
            Err(err) => eprintln!("An IO error has occurred: {}.", err),
        }
    }
}

fn launch(arg: &Option<&String>) -> Result<(), io::Error> {
    let mut terminal = ratatui::init();
    let mut state: State = State::new()?;
    if let Some(path) = arg {
        if Path::new(path.as_str()).exists() {
            let _ = state.open(&[path]);
        } else {
            state.path = Some((*path).clone());
            state.modified = true;
        }
    }

    while !state.exit {
        terminal.draw(|frame| state.draw(frame))?;
        state.handle_events()?
    }

    ratatui::restore();
    Ok(())
}
