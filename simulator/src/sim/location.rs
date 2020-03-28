#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Location {
    pub x: i8
}

pub type Distance = i8;

impl Location {
    pub fn new(location: i8) -> Location {
        Location { x: location }
    }

    pub fn distance(self, other: &Self) -> Distance {
        (self.x - other.x).abs()
    }
}
