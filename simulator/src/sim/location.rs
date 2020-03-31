use std::ops::Add;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Location {
    pub x: i16,
    pub y: i16,
}

pub type Distance = i16;

impl Location {
    pub fn new(x: i16, y: i16) -> Location {
        Location { x, y }
    }

    pub fn zero() -> Location {
        Location { x: 0, y: 0 }
    }

    pub fn distance(self, other: Self) -> Distance {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn diamond(self, size: u8) -> DiamondIterator {
        DiamondIterator {
            size,
            idx: 0,
            constant: self,
        }
    }
}

impl Add for Location {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

pub struct DiamondIterator {
    size: u8,
    idx: u8,
    constant: Location,
}

// TODO: This implementation is going to be bare bones simple for now
impl Iterator for DiamondIterator {
    type Item = Location;

    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    fn next(&mut self) -> Option<Self::Item> {
        let length = self.size as u8 * 2 + 1;
        let squared = length * length;
        while self.idx < squared {
            let x = (self.idx % length) as i8 - self.size as i8;
            let y = (self.idx / length) as i8 - self.size as i8;
            let loc = Location::new(x as i16, y as i16);
            self.idx += 1;
            if Location::zero().distance(loc) <= self.size as i16 {
                return Some(loc + self.constant);
            }
        }
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_diamond_0() {
        let expected = vec![Location::zero()];
        let result: Vec<_> = Location::zero().diamond(0).collect();
        assert_eq!(expected, result);
    }

    #[test]
    pub fn test_diamond_1() {
        let expected = vec![
            Location::new(0, -1),
            Location::new(-1, 0),
            Location::zero(),
            Location::new(1, 0),
            Location::new(0, 1),
        ];
        let result: Vec<_> = Location::zero().diamond(1).collect();
        assert_eq!(expected, result);
    }
}
