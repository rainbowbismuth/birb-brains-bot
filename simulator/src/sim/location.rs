#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Location {
    pub x: i16
}

pub type Distance = i16;

impl Location {
    pub fn new(location: i16) -> Location {
        Location { x: location }
    }

    pub fn distance(self, other: &Self) -> Distance {
        (self.x - other.x).abs()
    }
}
