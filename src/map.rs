pub(crate) fn dot(map: &mut [Vec<i32>], x: usize, y: usize, tile: i32) -> bool {
    if map[x][y] == tile {
        false
    } else {
        map[x][y] = tile;
        true
    }
}

pub(crate) fn draw_all<'a, I>(map: &mut [Vec<i32>], positions: &mut I, tile: i32) -> bool
where
    I: Iterator<Item = &'a (usize, usize)>,
{
    positions
        .map(|&(i, j)| dot(map, i, j, tile))
        .reduce(|p, q| p || q)
        .unwrap_or(false)
}
