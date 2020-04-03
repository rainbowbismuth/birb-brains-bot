use crate::sim::Location;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Facing {
    North = 0,
    East,
    South,
    West,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum RelativeFacing {
    Front = 0,
    Side,
    Back,
}

const OFFSETS: [Location; 4] = [
    Location::new(0, -1),
    Location::new(1, 0),
    Location::new(0, 1),
    Location::new(-1, 0),
];

impl Facing {
    pub fn towards(from: Location, to: Location) -> Facing {
        let north = (Facing::North.offset() + from).distance_squared(to);
        let east = (Facing::East.offset() + from).distance_squared(to);
        let south = (Facing::South.offset() + from).distance_squared(to);
        let west = (Facing::West.offset() + from).distance_squared(to);

        if north <= east && north <= south && north <= west {
            Facing::North
        } else if east <= south && east <= west {
            Facing::East
        } else if south <= west {
            Facing::South
        } else {
            Facing::West
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn offset(self) -> Location {
        OFFSETS[self.index()]
    }

    pub fn rotate(self, amount: u8) -> Facing {
        unsafe { std::mem::transmute_copy(&(((self as u8) + amount) % 4)) }
    }

    pub fn relative(self, target_loc: Location, from: Location) -> RelativeFacing {
        let front = target_loc + Facing::North.rotate(self as u8).offset();
        let right = target_loc + Facing::East.rotate(self as u8).offset();
        let back = target_loc + Facing::South.rotate(self as u8).offset();
        let left = target_loc + Facing::West.rotate(self as u8).offset();

        let front_dist = from.distance_squared(front);
        let right_dist = from.distance_squared(right);
        let back_dist = from.distance_squared(back);
        let left_dist = from.distance_squared(left);

        if front_dist <= right_dist && front_dist <= left_dist && front_dist <= back_dist {
            RelativeFacing::Front
        } else if left_dist <= back_dist || right_dist <= back_dist {
            RelativeFacing::Side
        } else {
            RelativeFacing::Back
        }
    }
}

impl RelativeFacing {
    pub fn is_front(self) -> bool {
        match self {
            RelativeFacing::Front => true,
            _ => false,
        }
    }

    pub fn is_front_or_side(self) -> bool {
        match self {
            RelativeFacing::Front => true,
            RelativeFacing::Side => true,
            _ => false,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn facing_tests() {
        let target = Location::zero();
        let target_facing = Facing::East;
        assert_eq!(
            RelativeFacing::Side,
            target_facing.relative(target, Location::new(-1, -1))
        );
        assert_eq!(
            RelativeFacing::Side,
            target_facing.relative(target, Location::new(-1, 1))
        );
        assert_eq!(
            RelativeFacing::Back,
            target_facing.relative(target, Location::new(-1, 0))
        );
        assert_eq!(
            RelativeFacing::Front,
            target_facing.relative(target, Location::new(1, 1))
        );
        assert_eq!(
            RelativeFacing::Front,
            target_facing.relative(target, Location::new(1, -1))
        );
    }

    #[test]
    pub fn facing_tests_at_range() {
        let target = Location::zero();
        let target_facing = Facing::North;
        assert_eq!(
            RelativeFacing::Back,
            target_facing.relative(target, Location::new(0, 1))
        );
        assert_eq!(
            RelativeFacing::Side,
            target_facing.relative(target, Location::new(-1, 0))
        );
        assert_eq!(
            RelativeFacing::Side,
            target_facing.relative(target, Location::new(1, 0))
        );
        assert_eq!(
            RelativeFacing::Front,
            target_facing.relative(target, Location::new(0, -1))
        );
    }

    #[test]
    pub fn facing_towards_tests() {
        assert_eq!(
            Facing::East,
            Facing::towards(Location::zero(), Location::new(5, 2))
        );
        assert_eq!(
            Facing::West,
            Facing::towards(Location::zero(), Location::new(-5, 2))
        );
        assert_eq!(
            Facing::North,
            Facing::towards(Location::zero(), Location::new(5, -200))
        );
        assert_eq!(
            Facing::South,
            Facing::towards(Location::zero(), Location::new(0, 100))
        );
    }
}
