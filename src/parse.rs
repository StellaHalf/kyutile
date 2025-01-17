use std::num::ParseIntError;

use itertools::Itertools;

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
