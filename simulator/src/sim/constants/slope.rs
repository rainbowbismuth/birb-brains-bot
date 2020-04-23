use crate::dto::rust::Tile;
use crate::sim::Facing;

pub const SLOPE_FLAT_0: u8 = 0;
pub const SLOPE_INCLINE_N: u8 = 133;
pub const SLOPE_INCLINE_E: u8 = 82;
pub const SLOPE_INCLINE_S: u8 = 37;
pub const SLOPE_INCLINE_W: u8 = 88;
pub const SLOPE_CONVEX_NE: u8 = 65;
pub const SLOPE_CONVEX_SE: u8 = 17;
pub const SLOPE_CONVEX_SW: u8 = 20;
pub const SLOPE_CONVEX_NW: u8 = 68;
pub const SLOPE_CONCAVE_NE: u8 = 150;
pub const SLOPE_CONCAVE_SE: u8 = 102;
pub const SLOPE_CONCAVE_SW: u8 = 105;
pub const SLOPE_CONCAVE_NW: u8 = 153;

pub fn tile_height_from_direction(tile: &Tile, direction: Facing) -> u8 {
    // TODO: Not sure how to handle convex?
    match (tile.slope_type, direction) {
        (SLOPE_FLAT_0, _) => tile.height,
        (SLOPE_INCLINE_N, Facing::South) => tile.height + tile.slope_height,
        (SLOPE_INCLINE_E, Facing::West) => tile.height + tile.slope_height,
        (SLOPE_INCLINE_S, Facing::North) => tile.height + tile.slope_height,
        (SLOPE_INCLINE_W, Facing::East) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_NE, Facing::South) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_NE, Facing::West) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_SE, Facing::North) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_SE, Facing::West) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_SW, Facing::North) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_SW, Facing::East) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_NW, Facing::South) => tile.height + tile.slope_height,
        (SLOPE_CONCAVE_NW, Facing::East) => tile.height + tile.slope_height,
        _ => tile.height,
    }
}
