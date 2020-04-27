use crate::sim::{tile_height_from_direction, Arena, Combatant, Location, Panel, OFFSETS};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::usize;

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: u8,
    panel: Panel,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.panel.x().cmp(&other.panel.x()))
            .then_with(|| self.panel.y().cmp(&other.panel.y()))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const MAX_DISTANCE: u8 = 250;
const IN_OPEN_SET_FLAG: u8 = 1;
const OCCUPIED_FLAG: u8 = 1 << 1;
const IMPASSABLE_FLAG: u8 = 1 << 2;
const REACHABLE_FLAG: u8 = 1 << 3;

#[derive(Copy, Clone, Debug)]
struct TileMarking {
    distance: u8,
    flags: u8,
}

impl TileMarking {
    const fn default() -> TileMarking {
        TileMarking {
            distance: MAX_DISTANCE,
            flags: 0,
        }
    }
}

pub struct Pathfinder<'a> {
    arena: &'a Arena,
    lower: Vec<TileMarking>,
    upper: Vec<TileMarking>,
    open_set: BinaryHeap<State>,
    reachable: Vec<Panel>,
}

#[derive(Clone)]
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
            lower: Vec::with_capacity(area),
            upper: Vec::with_capacity(area),
            open_set: BinaryHeap::with_capacity(255),
            reachable: Vec::with_capacity(255),
        };
        pathfinder.lower.resize(area, TileMarking::default());
        pathfinder.upper.resize(area, TileMarking::default());
        pathfinder
    }

    pub fn reset(&mut self) {
        for i in 0..self.lower.len() {
            self.lower[i].distance = MAX_DISTANCE;
            self.lower[i].flags &= OCCUPIED_FLAG | IMPASSABLE_FLAG;
            self.upper[i].distance = MAX_DISTANCE;
            self.upper[i].flags &= OCCUPIED_FLAG | IMPASSABLE_FLAG;
        }
        self.open_set.clear();
        self.reachable.clear();
    }

    pub fn reset_all(&mut self) {
        for i in 0..self.lower.len() {
            self.lower[i] = TileMarking::default();
            self.upper[i] = TileMarking::default();
        }
        self.open_set.clear();
        self.reachable.clear();
    }

    fn tile_marking(&self, panel: Panel) -> &TileMarking {
        let idx = self.to_index(panel);
        if panel.layer() {
            &self.upper[idx]
        } else {
            &self.lower[idx]
        }
    }

    fn tile_marking_mut(&mut self, panel: Panel) -> &mut TileMarking {
        let idx = self.to_index(panel);
        if panel.layer() {
            &mut self.upper[idx]
        } else {
            &mut self.lower[idx]
        }
    }

    pub fn set_distance(&mut self, panel: Panel, dist: u8) {
        self.tile_marking_mut(panel).distance = dist;
    }

    pub fn distance(&self, panel: Panel) -> u8 {
        self.tile_marking(panel).distance
    }

    pub fn set_occupied(&mut self, panel: Panel) {
        self.tile_marking_mut(panel).flags |= OCCUPIED_FLAG;
    }

    pub fn set_impassable(&mut self, panel: Panel) {
        self.tile_marking_mut(panel).flags |= IMPASSABLE_FLAG;
    }

    fn in_open_set(&self, panel: Panel) -> bool {
        self.tile_marking(panel).flags & IN_OPEN_SET_FLAG != 0
    }

    fn remove_from_open_set(&mut self, panel: Panel) {
        self.tile_marking_mut(panel).flags &= !IN_OPEN_SET_FLAG
    }

    pub fn is_occupied(&self, panel: Panel) -> bool {
        self.tile_marking(panel).flags & OCCUPIED_FLAG != 0
    }

    pub fn is_impassable(&self, panel: Panel) -> bool {
        self.tile_marking(panel).flags & IMPASSABLE_FLAG != 0
    }

    fn to_index(&self, panel: Panel) -> usize {
        self.arena
            .panel_to_index(panel)
            .unwrap_or_else(|| panic!("panel out of bounds: {:?}", panel.location()))
        // .expect("panel out of bounds")
    }

    pub fn reachable_set(&self) -> &[Panel] {
        &self.reachable
    }

    pub fn is_reachable(&self, panel: Panel) -> bool {
        if !self.inside_map(panel) {
            return false;
        }
        self.tile_marking(panel).distance < MAX_DISTANCE
    }

    pub fn can_reach_and_end_turn_on(&self, panel: Panel) -> bool {
        if !self.inside_map(panel) {
            return false;
        }
        self.tile_marking(panel).flags & REACHABLE_FLAG != 0
    }

    pub fn calculate_reachable(&mut self, info: &MovementInfo, start: Panel) {
        self.reset();
        self.calculate_reachable_no_reset(info, start);
    }

    pub fn path_find_no_reset(&mut self, info: &MovementInfo, start: Panel, goal: Panel) -> Panel {
        assert!(self.inside_map(start));
        assert!(self.inside_map(goal));

        let mut copied_info = info.clone();
        copied_info.movement = MAX_DISTANCE - 1;
        self.calculate_reachable_no_reset(&copied_info, goal);
        // TODO: Implement an actual pathfinding algorithm
        let copied_lower = self.lower.clone();
        let copied_upper = self.upper.clone();
        self.reset();
        self.calculate_reachable_with_goal_no_reset(&info, start, Some(goal));
        self.reachable
            .iter()
            .map(|panel| {
                let idx = self.arena.panel_to_index(*panel).unwrap();
                let dist = if panel.layer() {
                    copied_upper[idx].distance
                } else {
                    copied_lower[idx].distance
                };
                (dist, panel)
            })
            .min_by_key(|p| p.0)
            .map(|p| *p.1)
            .unwrap()
    }

    pub fn calculate_reachable_no_reset(&mut self, info: &MovementInfo, start: Panel) {
        self.calculate_reachable_with_goal_no_reset(info, start, None);
    }

    pub fn calculate_reachable_with_goal_no_reset(
        &mut self,
        info: &MovementInfo,
        start: Panel,
        goal: Option<Panel>,
    ) {
        assert!(self.inside_map(start));
        self.expand_open_set(start, 0);
        self.expand_reachable(start);
        self.set_distance(start, 0);

        while let Some(State { panel: start, cost }) = self.open_set.pop() {
            self.remove_from_open_set(start);

            if let Some(goal_panel) = goal {
                if goal_panel == start {
                    return;
                }
            }

            if cost > self.distance(start) {
                continue;
            }

            for direction in &OFFSETS {
                let end_lower = start.plus(*direction).lower();
                if !self.inside_map(end_lower) {
                    continue;
                }

                let end_upper = end_lower.upper();
                let end_lower_tile = self.arena.tile(end_lower);
                let end_upper_tile = self.arena.tile(end_upper);

                if self.try_move_to(info, start, end_lower) {
                    continue;
                } else if end_upper_tile.height > end_lower_tile.height
                    && self.try_move_to(info, start, end_upper)
                {
                    continue;
                } else {
                    self.horizontal_jump_search(info, start, *direction);
                }
            }
        }
    }

    fn try_move_to(&mut self, info: &MovementInfo, start: Panel, end: Panel) -> bool {
        let distance = self.distance(start);
        let new_distance = distance + 1;
        let old_distance = self.distance(end);
        if new_distance >= old_distance {
            return false;
        }
        if self.can_transition(info, start, end) {
            self.set_distance(end, new_distance);
            if new_distance >= info.movement {
                return false;
            }
            self.expand_open_set(end, new_distance);
            if self.can_end_on(info, end) {
                self.expand_reachable(end);
            }
            true
        } else {
            false
        }
    }

    fn horizontal_jump_search(&mut self, info: &MovementInfo, start: Panel, direction: Location) {
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

            // TODO: This really is a mess and makes so sense. Sit and think about it.
            if highest_part > start_tile_height {
                return;
            }
            let height_diff = (start_tile_height as i16 - end_tile_height as i16).abs() as u8;
            if height_diff > info.vertical_jump {
                continue;
            }
            if !self.can_end_on(info, end) {
                continue;
            }

            let new_distance = distance + i as u8;
            if new_distance >= self.distance(end) {
                continue;
            }
            self.set_distance(end, new_distance);
            if new_distance >= info.movement {
                continue;
            }
            self.expand_open_set(end, new_distance);
            self.expand_reachable(end);
        }
    }

    fn expand_open_set(&mut self, panel: Panel, distance: u8) {
        if !self.in_open_set(panel) {
            self.open_set.push(State {
                panel,
                cost: distance,
            });
            self.tile_marking_mut(panel).flags |= IN_OPEN_SET_FLAG;
        }
    }

    fn expand_reachable(&mut self, panel: Panel) {
        if self.tile_marking(panel).flags & REACHABLE_FLAG == 0 {
            self.reachable.push(panel);
            self.tile_marking_mut(panel).flags |= REACHABLE_FLAG;
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
        if end_tile.no_walk || self.is_impassable(end) {
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
        if self.is_impassable(panel) {
            return false;
        }
        if self.is_occupied(panel) {
            return false;
        }
        true
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::dto::rust::Tile;
    use crate::sim::{Facing, SLOPE_FLAT_0, SLOPE_INCLINE_E, SLOPE_INCLINE_W};

    fn tile_no_walk() -> Tile {
        Tile {
            height: 0,
            depth: 0,
            slope_type: 0,
            surface_type: 0,
            slope_height: 0,
            no_cursor: true,
            no_walk: true,
        }
    }

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
        let mut upper = vec![];
        upper.resize(100, tile_no_walk());
        Arena {
            lower: tiles,
            upper,
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
            for panel in center.diamond(movement) {
                assert_eq!(pathfinder.is_reachable(panel), true);
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
        let no_walk = tile_no_walk();
        let mut tiles = vec![];
        for _i in 0..2 {
            tiles.push(walk);
        }
        tiles.push(no_walk);
        for _i in 0..2 {
            tiles.push(walk);
        }
        let mut upper = vec![];
        upper.resize(5, no_walk);
        Arena {
            lower: tiles,
            upper,
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
        let mut upper = vec![];
        upper.resize(5, tile_no_walk());
        Arena {
            lower: tiles,
            upper,
            width: 5,
            height: 1,
            starting_locations: vec![],
        }
    }

    fn make_bridge_map() -> Arena {
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
        let middle = Tile {
            height: 8,
            depth: 0,
            slope_type: SLOPE_FLAT_0,
            surface_type: 0,
            slope_height: 0,
            no_cursor: false,
            no_walk: false,
        };
        let tiles = vec![walk, up_east, walk, walk, up_west, walk];
        let mut upper = vec![];
        upper.resize(6, tile_no_walk());
        upper[2] = middle;
        upper[3] = middle;
        Arena {
            lower: tiles,
            upper,
            width: 6,
            height: 1,
            starting_locations: vec![],
        }
    }

    #[test]
    pub fn test_walk_over_bridge() {
        let arena = make_bridge_map();
        let mut pathfinder = Pathfinder::new(&arena);
        let start = Panel::coords(0, 0, false);
        let up_path = Panel::coords(1, 0, false);
        let middle_up = Panel::coords(2, 0, true);
        let middle_down = Panel::coords(2, 0, false);
        let end = Panel::coords(5, 0, false);
        let movement_info = MovementInfo {
            movement: 10,
            vertical_jump: 2,
            horizontal_jump: 1,
            fly_teleport: false,
            water_ok: false,
        };
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(pathfinder.is_reachable(middle_up), true);
        assert_eq!(pathfinder.is_reachable(middle_down), false);
        assert_eq!(pathfinder.is_reachable(up_path), true);
        assert_eq!(pathfinder.is_reachable(end), true);

        assert_eq!(pathfinder.can_reach_and_end_turn_on(middle_up), true);
        assert_eq!(pathfinder.can_reach_and_end_turn_on(end), true);
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
        assert_eq!(pathfinder.can_reach_and_end_turn_on(middle), false);
        assert_eq!(pathfinder.can_reach_and_end_turn_on(end), true);
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
        dbg!(&pathfinder.lower);
        assert_eq!(pathfinder.is_reachable(end), true);
        assert_eq!(pathfinder.is_reachable(middle), false);
        assert_eq!(pathfinder.can_reach_and_end_turn_on(middle), false);
        assert_eq!(pathfinder.can_reach_and_end_turn_on(end), true);
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
        assert_eq!(pathfinder.can_reach_and_end_turn_on(middle), false);
    }

    #[test]
    pub fn cant_cross_impassable() {
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
            pathfinder.set_impassable(start.plus(facing.offset()));
        }
        pathfinder.calculate_reachable(&movement_info, start);
        assert_eq!(pathfinder.reachable_set(), &[start]);
        assert_eq!(pathfinder.is_reachable(outside), false);
        assert_eq!(pathfinder.can_reach_and_end_turn_on(outside), false);
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

        dbg!(&pathfinder.lower);
        assert_eq!(pathfinder.is_reachable(end), true);
        assert_eq!(pathfinder.is_reachable(middle), true);
        assert_eq!(pathfinder.can_reach_and_end_turn_on(middle), false);
    }
}
