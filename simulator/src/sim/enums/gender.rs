use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Gender {
    Male = 0,
    Female = 1,
    Monster = 2,
}

impl Gender {
    pub fn parse(name: &str) -> Option<Gender> {
        match name {
            "Male" => Some(Gender::Male),
            "Female" => Some(Gender::Female),
            "Monster" => Some(Gender::Monster),
            _ => None,
        }
    }
}
