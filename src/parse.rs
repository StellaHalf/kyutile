use std::{collections::HashMap, num::ParseIntError};

use itertools::Itertools;
use toml::Table;

use crate::state::TileData;

pub(crate) fn parse_map(input: &str) -> Result<Vec<Vec<i32>>, ParseIntError> {
    input
        .trim()
        .split('\n')
        .map(|l| l.split(',').map(|c| c.parse()).collect())
        .collect()
}

pub(crate) fn export_map(map: &[Vec<i32>]) -> String {
    map.iter()
        .map(|r| r.iter().map(|i| i.to_string()).join(","))
        .join("\n")
}

pub(crate) fn parse_tile_data(input: &str) -> TileData {
    let parsed = input.parse::<Table>().unwrap();
    TileData {
        names: parsed["names"]
            .as_table()
            .unwrap()
            .iter()
            .map(|(i, s)| (i.parse().unwrap(), s.as_str().unwrap().to_owned()))
            .collect::<HashMap<_, _>>(),
        colors: parsed["colors"]
            .as_table()
            .unwrap()
            .iter()
            .map(|(i, s)| (i.parse().unwrap(), s.as_integer().unwrap() as u32))
            .collect(),
    }
}
