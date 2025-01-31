use std::collections::HashSet;

pub(crate) fn set(map: &mut [Vec<i32>], x: usize, y: usize, tile: i32) {
    map[x][y] = tile
}

pub(crate) fn fill(map: &mut [Vec<i32>], positions: &HashSet<(usize, usize)>, tile: i32) {
    for &(i, j) in positions {
        set(map, i, j, tile)
    }
}
