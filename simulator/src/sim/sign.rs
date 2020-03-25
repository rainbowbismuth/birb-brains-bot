use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::Serialize;

#[derive(PartialEq, Eq, Debug, Serialize)]
#[repr(u8)]
pub enum Sign {
    Aries = 0,
    Taurus = 1,
    Gemini = 2,
    Cancer = 3,
    Leo = 4,
    Virgo = 5,
    Libra = 6,
    Scorpio = 7,
    Sagittarius = 8,
    Capricorn = 9,
    Aquarius = 10,
    Pisces = 11,
    Serpentarius = 12,
}

impl Sign {
    pub fn parse(name: &str) -> Option<Sign> {
        match name {
            "Aries" => Some(Sign::Aries),
            "Taurus" => Some(Sign::Taurus),
            "Gemini" => Some(Sign::Gemini),
            "Cancer" => Some(Sign::Cancer),
            "Leo" => Some(Sign::Leo),
            "Virgo" => Some(Sign::Virgo),
            "Libra" => Some(Sign::Libra),
            "Scorpio" => Some(Sign::Scorpio),
            "Sagittarius" => Some(Sign::Sagittarius),
            "Capricorn" => Some(Sign::Capricorn),
            "Aquarius" => Some(Sign::Aquarius),
            "Pisces" => Some(Sign::Pisces),
            "Serpentarius" => Some(Sign::Serpentarius),
            _ => None
        }
    }
}


impl<'de> Deserialize<'de> for Sign {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct SignVisitor;

        impl<'de> Visitor<'de> for SignVisitor {
            type Value = Sign;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Sign must be a string")
            }

            fn visit_str<E>(self, name: &str) -> Result<Self::Value, E>
                where E: de::Error
            {
                match Sign::parse(name) {
                    Some(cond) => Ok(cond),
                    None => Err(de::Error::custom(String::from(name)))
                }
            }
        }

        deserializer.deserialize_any(SignVisitor)
    }
}
