use std::{
    io::{self, BufRead},
    process::exit,
};

use commands::State;

mod commands;
mod map;
mod parse;

fn main() {
    launch()
}

fn launch() {
    let mut state = State::Empty;
    let stdin = io::stdin().lock();
    for line in stdin.lines() {
        match line {
            Ok(s) => {
                if s.len() > 0 {
                    let args: Vec<_> = s.split(" ").collect();
                    match args[0] {
                        "o" => {
                            if args.len() >= 2 {
                                match state.open(args[1]) {
                                    Ok(_) => println!(
                                        "Opened map {} ({} bytes).",
                                        args[1],
                                        args[1].bytes().count()
                                    ),
                                    Err(_) => eprintln!("Error."),
                                }
                            }
                        }
                        "d" => state.display(),
                        _ => {
                            eprintln!("unsupported command");
                        }
                    }
                }
            }
            Err(_) => return (),
        }
    }
}
