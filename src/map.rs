use std::iter::once;

pub(crate) fn validate(map: &[Vec<i32>]) -> Result<(), String> {
    if map.is_empty() {
        Err("Maps cannot be empty.".to_owned())
    } else {
        let len = map[0].len();
        if map.iter().skip(1).any(|r| r.len() != len) {
            Err("Maps must be rectangular.".to_owned())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn dot(map: &mut [Vec<i32>], x: usize, y: usize, tile: i32) -> bool {
    if map[x][y] == tile {
        false
    } else {
        map[x][y] = tile;
        true
    }
}

pub(crate) fn draw_all<I>(map: &mut [Vec<i32>], positions: I, tile: i32) -> bool
where
    I: IntoIterator<Item = (usize, usize)>,
{
    positions
        .into_iter()
        .map(|(i, j)| dot(map, i, j, tile))
        .reduce(|p, q| p || q)
        .unwrap_or(false)
}

pub(crate) fn create(x: usize, y: usize, tile: i32) -> Vec<Vec<i32>> {
    once(once(tile).cycle().take(y).collect())
        .cycle()
        .take(x)
        .collect()
}

pub(crate) fn in_bounds(lx: usize, ly: usize, x: usize, y: usize) -> bool {
    x < lx && y < ly
}
