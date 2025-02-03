use std::collections::HashSet;

pub(crate) fn dot(map: &mut [Vec<i32>], x: usize, y: usize, tile: i32) {
    map[x][y] = tile
}

pub(crate) fn bucket(map: &mut [Vec<i32>], positions: &HashSet<(usize, usize)>, tile: i32) {
    for &(i, j) in positions {
        dot(map, i, j, tile)
    }
}
