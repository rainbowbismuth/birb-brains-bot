use crate::dto::rust::Arena;
use crate::sim::{Combatant, Location, OFFSETS};

const MAX_DISTANCE: u8 = 254;
const OCCUPIED: u8 = 255;

pub struct Pathfinder<'a> {
    arena: &'a Arena,
    distance: Vec<u8>,
    open_set: Vec<Location>,
    reachable: Vec<Location>,
}

pub struct MovementInfo {
    pub movement: u8,
    pub jump: u8,
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
        let jump = if combatant.ignore_height() {
            100
        } else {
            combatant.jump()
        };
        MovementInfo {
            movement,
            jump,
            fly_teleport: combatant.fly() || combatant.teleport(),
            water_ok: !combatant.landlocked(),
        }
    }
}

impl<'a> Pathfinder<'a> {
    pub fn new(arena: &'a Arena) -> Pathfinder {
        let mut pathfinder = Pathfinder {
            arena,
            distance: Vec::with_capacity(arena.width as usize * arena.height as usize),
            open_set: Vec::with_capacity(255),
            reachable: Vec::with_capacity(255),
        };
        for _i in 0..pathfinder.distance.capacity() {
            pathfinder.distance.push(MAX_DISTANCE);
        }
        pathfinder
    }

    pub fn reset_all(&mut self) {
        for i in 0..self.distance.len() {
            self.distance[i] = MAX_DISTANCE;
        }
        self.open_set.clear();
        self.reachable.clear();
    }

    pub fn reset(&mut self) {
        for i in 0..self.distance.len() {
            if self.distance[i] != OCCUPIED {
                self.distance[i] = MAX_DISTANCE;
            }
        }
        self.open_set.clear();
        self.reachable.clear();
    }

    pub fn set_occupied(&mut self, panel: Location) {
        let idx = self.to_index(panel);
        self.distance[idx] = OCCUPIED;
    }

    fn to_index(&self, panel: Location) -> usize {
        self.arena.width as usize * panel.y as usize + panel.x as usize
    }

    pub fn reachable_set(&self) -> &[Location] {
        &self.reachable
    }

    pub fn is_reachable(&self, end: Location) -> bool {
        if !self.inside_map(end) {
            return false;
        }
        let end_idx = self.to_index(end);
        self.distance[end_idx] < MAX_DISTANCE
    }

    pub fn can_reach_and_end_turn_on(&self, info: &MovementInfo, end: Location) -> bool {
        self.is_reachable(end) && self.can_end_on(info, end)
    }

    pub fn calculate_reachable(&mut self, info: &MovementInfo, start: Location) {
        self.reset();
        self.calculate_reachable_no_reset(info, start);
    }

    pub fn calculate_reachable_no_reset(&mut self, info: &MovementInfo, start: Location) {
        assert!(self.inside_map(start));
        self.open_set.push(start);
        self.reachable.push(start);
        let start_idx = self.to_index(start);
        self.distance[start_idx] = 0;

        while let Some(start) = self.open_set.pop() {
            let start_idx = self.to_index(start);
            let distance = self.distance[start_idx];
            for direction in &OFFSETS {
                let end = start + *direction;
                if !self.inside_map(end) {
                    continue;
                }
                let end_idx = self.to_index(end);
                let new_distance = distance + 1;
                if new_distance > self.distance[end_idx] {
                    continue;
                }
                if OCCUPIED == self.distance[end_idx] {
                    continue;
                }
                if !self.can_transition(info, start, end) {
                    continue;
                }
                self.distance[end_idx] = new_distance;
                if new_distance >= info.movement {
                    continue;
                }
                self.open_set.push(end);
                if self.can_end_on(info, end) {
                    self.reachable.push(end);
                }
            }
        }
    }

    pub fn inside_map(&self, end: Location) -> bool {
        if end.x < 0 || end.y < 0 {
            return false;
        }
        if end.x >= (self.arena.width as i16) {
            return false;
        }
        if end.y >= (self.arena.height as i16) {
            return false;
        }
        true
    }

    fn can_transition(&self, info: &MovementInfo, start: Location, end: Location) -> bool {
        let start_idx = self.to_index(start);
        let end_idx = self.to_index(end);
        let start_tile = self.arena.lower[start_idx];
        let end_tile = self.arena.lower[end_idx];
        if info.fly_teleport {
            return true;
        }
        if end_tile.no_walk {
            return false;
        }
        // TODO: Slopes matter because it's based on the edges.. oh gosh.
        let height_diff = (start_tile.height as i16 - end_tile.height as i16).abs() as u8;
        if height_diff > info.jump {
            return false;
        }
        true
    }

    fn can_end_on(&self, info: &MovementInfo, end: Location) -> bool {
        let end_idx = self.to_index(end);
        let end_tile = self.arena.lower[end_idx];
        if end_tile.no_walk {
            return false;
        }
        // TODO: ignoring lava for now.
        if !info.water_ok && end_tile.depth > 0 {
            return false;
        }
        return true;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::dto::rust::Tile;
    use crate::sim::Facing;

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
        }
    }

    #[test]
    pub fn matches_diamond() {
        let arena = make_simple_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let center = Location::new(5, 5);
        for movement in 0..4 {
            let movement_info = MovementInfo {
                movement,
                jump: 0,
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
        }
    }

    #[test]
    pub fn cant_cross_no_fly() {
        let arena = make_impassible_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let start = Location::new(0, 0);
        let middle = Location::new(2, 0);
        let end = Location::new(4, 0);
        let movement_info = MovementInfo {
            movement: 10,
            jump: 0,
            fly_teleport: false,
            water_ok: false,
        };
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(pathfinder.reachable_set(), &[start, Location::new(1, 0)]);
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
        let start = Location::new(1, 1);
        let outside = Location::new(3, 3);
        let movement_info = MovementInfo {
            movement: 10,
            jump: 0,
            fly_teleport: false,
            water_ok: false,
        };
        for facing in &[Facing::North, Facing::East, Facing::South, Facing::West] {
            pathfinder.set_occupied(start + facing.offset());
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
        let start = Location::new(0, 0);
        let middle = Location::new(2, 0);
        let end = Location::new(4, 0);
        let movement_info = MovementInfo {
            movement: 10,
            jump: 0,
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
