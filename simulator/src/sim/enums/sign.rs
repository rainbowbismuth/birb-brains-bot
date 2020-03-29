use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize_repr, Deserialize_repr)]
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
            _ => None,
        }
    }
}
