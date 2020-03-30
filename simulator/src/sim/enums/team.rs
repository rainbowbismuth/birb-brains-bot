#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Left,
    Right,
}

impl Team {
    pub fn opposite(self) -> Team {
        match self {
            Team::Left => Team::Right,
            Team::Right => Team::Left,
        }
    }
}
