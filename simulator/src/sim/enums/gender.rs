use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_repr::Serialize_repr;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize_repr)]
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
            _ => None
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Gender::Male => "Male",
            Gender::Female => "Female",
            Gender::Monster => "Monster"
        }
    }
}


impl<'de> Deserialize<'de> for Gender {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct GenderVisitor;

        impl<'de> Visitor<'de> for GenderVisitor {
            type Value = Gender;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Gender must be a string or u8")
            }

            fn visit_u8<E>(self, code: u8) -> Result<Self::Value, E>
                where E: de::Error
            {
                unsafe {
                    Ok(std::mem::transmute_copy(&code))
                }
            }

            fn visit_str<E>(self, name: &str) -> Result<Self::Value, E>
                where E: de::Error
            {
                match Gender::parse(name) {
                    Some(cond) => Ok(cond),
                    None => Err(de::Error::custom(String::from(name)))
                }
            }
        }

        deserializer.deserialize_any(GenderVisitor)
    }
}
