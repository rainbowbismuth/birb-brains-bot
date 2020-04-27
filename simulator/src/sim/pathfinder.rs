use crate::sim::{tile_height_from_direction, Arena, Combatant, Location, Panel, OFFSETS};

const MAX_DISTANCE: u8 = 254;
const OCCUPIED: u8 = 255;

pub struct Pathfinder<'a> {
    arena: &'a Arena,
    distance_lower: Vec<u8>,
    distance_upper: Vec<u8>,
    open_set: Vec<Panel>,
    reachable: Vec<Panel>,
}

pub struct MovementInfo {
    pub movement: u8,
    pub vertical_jump: u8,
    pub horizontal_jump: u8,
    pub fly_teleport: bool,
    pub water_ok: bool,
}

impl MovementInfo {
    pub fn new(combatant: &Combatant) -> MovementInfo {
        let movement = if combatant.dont_move() {
            0
        } else {
            combatant.movement()
        };
        let jump = combatant.jump();
        let vertical_jump = if combatant.ignore_height() { 100 } else { jump };
        let horizontal_jump = jump / 2;
        MovementInfo {
            movement,
            vertical_jump,
            horizontal_jump,
            fly_teleport: combatant.fly() || combatant.teleport(),
            water_ok: !combatant.landlocked(),
        }
    }
}

impl<'a> Pathfinder<'a> {
    pub fn new(arena: &'a Arena) -> Pathfinder {
        let area = arena.width as usize * arena.height as usize;
        let mut pathfinder = Pathfinder {
            arena,
            distance_lower: Vec::with_capacity(area),
            distance_upper: Vec::with_capacity(area),
            open_set: Vec::with_capacity(255),
            reachable: Vec::with_capacity(255),
        };
        pathfinder.distance_lower.resize(area, MAX_DISTANCE);
        pathfinder.distance_upper.resize(area, MAX_DISTANCE);
        pathfinder
    }

    pub fn reset_all(&mut self) {
        for i in 0..self.distance_lower.len() {
            self.distance_lower[i] = MAX_DISTANCE;
            self.distance_upper[i] = MAX_DISTANCE;
        }
        self.open_set.clear();
        self.reachable.clear();
    }

    pub fn reset(&mut self) {
        for i in 0..self.distance_lower.len() {
            if self.distance_lower[i] != OCCUPIED {
                self.distance_lower[i] = MAX_DISTANCE;
            }
            if self.distance_upper[i] != OCCUPIED {
                self.distance_upper[i] = MAX_DISTANCE;
            }
        }
        self.open_set.clear();
        self.reachable.clear();
    }

    pub fn set_distance(&mut self, panel: Panel, dist: u8) {
        let idx = self.to_index(panel);
        if panel.layer() {
            self.distance_upper[idx] = dist;
        } else {
            self.distance_lower[idx] = dist;
        }
    }

    pub fn distance(&self, panel: Panel) -> u8 {
        let idx = self.to_index(panel);
        if panel.layer() {
            self.distance_upper[idx]
        } else {
            self.distance_lower[idx]
        }
    }

    pub fn set_occupied(&mut self, panel: Panel) {
        self.set_distance(panel, OCCUPIED);
    }

    fn to_index(&self, panel: Panel) -> usize {
        self.arena
            .panel_to_index(panel)
            .expect("panel out of bounds")
    }

    pub fn reachable_set(&self) -> &[Panel] {
        &self.reachable
    }

    pub fn is_reachable(&self, panel: Panel) -> bool {
        if !self.inside_map(panel) {
            return false;
        }
        self.distance(panel) < MAX_DISTANCE
    }

    pub fn can_reach_and_end_turn_on(&self, info: &MovementInfo, panel: Panel) -> bool {
        self.is_reachable(panel) && self.can_end_on(info, panel)
    }

    pub fn calculate_reachable(&mut self, info: &MovementInfo, start: Panel) {
        self.reset();
        self.calculate_reachable_no_reset(info, start);
    }

    pub fn calculate_reachable_no_reset(&mut self, info: &MovementInfo, start: Panel) {
        assert!(self.inside_map(start));
        self.open_set.push(start);
        self.reachable.push(start);
        self.set_distance(start, 0);

        while let Some(start) = self.open_set.pop() {
            let distance = self.distance(start);
            for direction in &OFFSETS {
                let end = start.plus(*direction);
                if !self.inside_map(end) {
                    continue;
                }
                let new_distance = distance + 1;
                let old_distance = self.distance(end);
                if new_distance >= old_distance || OCCUPIED == old_distance {
                    continue;
                }
                if self.can_transition(info, start, end) {
                    self.set_distance(end, new_distance);
                    if new_distance >= info.movement {
                        continue;
                    }
                    self.open_set.push(end);
                    if self.can_end_on(info, end) {
                        self.reachable.push(end);
                    }
                } else {
                    self.horizontal_jump_search(info, start, *direction);
                }
            }
        }
    }
    pub fn horizontal_jump_search(
        &mut self,
        info: &MovementInfo,
        start: Panel,
        direction: Location,
    ) {
        if info.horizontal_jump == 0 {
            return;
        }

        let start_point = start.location();
        let end_point = start_point + direction * (info.horizontal_jump as i16 + 1);
        let towards_direction = start_point.facing_towards(end_point);
        let start_tile = self.arena.tile(start);
        let distance = self.distance(start);

        for (i, end_location) in start_point.line(end_point).enumerate().skip(1) {
            if !self.inside_map_location(end_location) {
                return;
            }
            let end = start.on_same_layer(end_location);
            let end_tile = self.arena.tile(end);
            let end_tile_height = tile_height_from_direction(&end_tile, towards_direction);
            let highest_part = end_tile.height + end_tile.slope_height;
            let start_tile_height =
                tile_height_from_direction(&start_tile, towards_direction.opposite());

            if highest_part > start_tile_height {
                return;
            }
            if end_tile_height < start_tile_height {
                continue;
            }
            if !self.can_end_on(info, end) {
                continue;
            }

            let new_distance = distance + i as u8;
            self.set_distance(end, new_distance);
            if new_distance >= info.movement {
                continue;
            }
            self.open_set.push(end);
            self.reachable.push(end);
        }
    }

    pub fn inside_map(&self, panel: Panel) -> bool {
        !(panel.x() >= self.arena.width || panel.y() >= self.arena.height)
    }

    pub fn inside_map_location(&self, location: Location) -> bool {
        !(location.x < 0
            || location.y < 0
            || location.x >= self.arena.width as i16
            || location.y >= self.arena.height as i16)
    }

    fn can_transition(&self, info: &MovementInfo, start: Panel, end: Panel) -> bool {
        let start_tile = self.arena.tile(start);
        let end_tile = self.arena.tile(end);

        if info.fly_teleport {
            return true;
        }
        if end_tile.no_walk {
            return false;
        }

        // TODO: Still not entirely sure of this logic
        let towards_direction = start.facing_towards(end);
        let end_tile_height = tile_height_from_direction(&end_tile, towards_direction);
        let start_tile_height =
            tile_height_from_direction(&start_tile, towards_direction.opposite());
        let height_diff = (start_tile_height as i16 - end_tile_height as i16).abs() as u8;

        if height_diff > info.vertical_jump {
            return false;
        }
        true
    }

    fn can_end_on(&self, info: &MovementInfo, panel: Panel) -> bool {
        let tile = self.arena.tile(panel);
        if tile.no_walk {
            return false;
        }
        // TODO: ignoring lava for now.
        if !info.water_ok && tile.depth > 0 {
            return false;
        }
        self.distance(panel) != OCCUPIED
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::dto::rust::Tile;
    use crate::sim::{Facing, SLOPE_FLAT_0, SLOPE_INCLINE_E, SLOPE_INCLINE_W};

    fn make_simple_map() -> Arena {
        let tile = Tile {
            height: 0,
            depth: 0,
            slope_type: 0,
            surface_type: 0,
            slope_height: 0,
            no_cursor: false,
            no_walk: false,
        };
        let mut tiles = vec![];
        for _i in 0..100 {
            tiles.push(tile);
        }
        Arena {
            lower: tiles,
            upper: vec![],
            width: 10,
            height: 10,
            starting_locations: vec![],
        }
    }

    #[test]
    pub fn matches_diamond() {
        let arena = make_simple_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let center = Panel::coords(5, 5, false);
        for movement in 0..4 {
            let movement_info = MovementInfo {
                movement,
                horizontal_jump: 0,
                vertical_jump: 0,
                fly_teleport: false,
                water_ok: false,
            };
            pathfinder.calculate_reachable(&movement_info, center);
            for location in center.diamond(movement) {
                assert_eq!(pathfinder.is_reachable(location), true);
            }
        }
    }

    fn make_impassible_map() -> Arena {
        let walk = Tile {
            height: 0,
            depth: 0,
            slope_type: 0,
            surface_type: 0,
            slope_height: 0,
            no_cursor: false,
            no_walk: false,
        };
        let no_walk = Tile {
            height: 0,
            depth: 0,
            slope_type: 0,
            surface_type: 0,
            slope_height: 0,
            no_cursor: false,
            no_walk: true,
        };
        let mut tiles = vec![];
        for _i in 0..2 {
            tiles.push(walk);
        }
        tiles.push(no_walk);
        for _i in 0..2 {
            tiles.push(walk);
        }
        Arena {
            lower: tiles,
            upper: vec![],
            width: 5,
            height: 1,
            starting_locations: vec![],
        }
    }

    fn make_high_slope_map() -> Arena {
        let walk = Tile {
            height: 0,
            depth: 0,
            slope_type: SLOPE_FLAT_0,
            surface_type: 0,
            slope_height: 0,
            no_cursor: false,
            no_walk: false,
        };
        let up_east = Tile {
            height: 0,
            depth: 0,
            slope_type: SLOPE_INCLINE_E,
            surface_type: 0,
            slope_height: 8,
            no_cursor: false,
            no_walk: false,
        };
        let up_west = Tile {
            height: 0,
            depth: 0,
            slope_type: SLOPE_INCLINE_W,
            surface_type: 0,
            slope_height: 8,
            no_cursor: false,
            no_walk: false,
        };
        let tiles = vec![walk, up_east, walk, up_west, walk];
        Arena {
            lower: tiles,
            upper: vec![],
            width: 5,
            height: 1,
            starting_locations: vec![],
        }
    }

    #[test]
    pub fn test_walk_up_and_jump_slope() {
        let arena = make_high_slope_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let start = Panel::coords(0, 0, false);
        let middle = Panel::coords(2, 0, false);
        let end = Panel::coords(4, 0, false);
        let movement_info = MovementInfo {
            movement: 10,
            vertical_jump: 2,
            horizontal_jump: 1,
            fly_teleport: false,
            water_ok: false,
        };
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(pathfinder.is_reachable(end), true);
        assert_eq!(pathfinder.is_reachable(middle), false);
        assert_eq!(
            pathfinder.can_reach_and_end_turn_on(&movement_info, middle),
            false
        );
        assert_eq!(
            pathfinder.can_reach_and_end_turn_on(&movement_info, end),
            true
        );
    }

    #[test]
    pub fn test_jump_over_no_walk() {
        let arena = make_impassible_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let start = Panel::coords(0, 0, false);
        let middle = Panel::coords(2, 0, false);
        let end = Panel::coords(4, 0, false);
        let movement_info = MovementInfo {
            movement: 10,
            vertical_jump: 2,
            horizontal_jump: 1,
            fly_teleport: false,
            water_ok: false,
        };
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(pathfinder.is_reachable(end), true);
        assert_eq!(pathfinder.is_reachable(middle), false);
        assert_eq!(
            pathfinder.can_reach_and_end_turn_on(&movement_info, middle),
            false
        );
        assert_eq!(
            pathfinder.can_reach_and_end_turn_on(&movement_info, end),
            true
        );
    }

    #[test]
    pub fn cant_cross_no_fly() {
        let arena = make_impassible_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let start = Panel::coords(0, 0, false);
        let middle = Panel::coords(2, 0, false);
        let end = Panel::coords(4, 0, false);
        let movement_info = MovementInfo {
            movement: 10,
            horizontal_jump: 0,
            vertical_jump: 0,
            fly_teleport: false,
            water_ok: false,
        };
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(
            pathfinder.reachable_set(),
            &[start, Panel::coords(1, 0, false)]
        );
        assert_eq!(pathfinder.is_reachable(end), false);
        assert_eq!(pathfinder.is_reachable(middle), false);
        assert_eq!(
            pathfinder.can_reach_and_end_turn_on(&movement_info, middle),
            false
        );
    }

    #[test]
    pub fn cant_cross_occupied() {
        let arena = make_simple_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let start = Panel::coords(1, 1, false);
        let outside = Panel::coords(3, 3, false);
        let movement_info = MovementInfo {
            movement: 10,
            horizontal_jump: 0,
            vertical_jump: 0,
            fly_teleport: false,
            water_ok: false,
        };
        for facing in &[Facing::North, Facing::East, Facing::South, Facing::West] {
            pathfinder.set_occupied(start.plus(facing.offset()));
        }
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(pathfinder.reachable_set(), &[start]);
        assert_eq!(pathfinder.is_reachable(outside), false);
        assert_eq!(
            pathfinder.can_reach_and_end_turn_on(&movement_info, outside),
            false
        );
    }

    #[test]
    pub fn can_cross_fly_teleport() {
        let arena = make_impassible_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let start = Panel::coords(0, 0, false);
        let middle = Panel::coords(2, 0, false);
        let end = Panel::coords(4, 0, false);
        let movement_info = MovementInfo {
            movement: 10,
            horizontal_jump: 0,
            vertical_jump: 0,
            fly_teleport: true,
            water_ok: false,
        };
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(pathfinder.is_reachable(end), true);
        assert_eq!(pathfinder.is_reachable(middle), true);
        assert_eq!(
            pathfinder.can_reach_and_end_turn_on(&movement_info, middle),
            false
        );
    }
}
