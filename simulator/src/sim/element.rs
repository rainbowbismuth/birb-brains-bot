use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_repr::Serialize_repr;

#[derive(PartialEq, Eq, Debug, Serialize_repr)]
#[repr(u8)]
pub enum Element {
    Weapon = 1,
    Fire,
    Ice,
    Lightning,
    Water,
    Earth,
    Wind,
    Dark,
    Holy,
}

impl Element {
    pub fn parse(name: &str) -> Option<Element> {
        match name {
            "Weapon" => Some(Element::Weapon),
            "Fire" => Some(Element::Fire),
            "Ice" => Some(Element::Ice),
            "Lightning" => Some(Element::Lightning),
            "Water" => Some(Element::Water),
            "Earth" => Some(Element::Earth),
            "Wind" => Some(Element::Wind),
            "Dark" => Some(Element::Dark),
            "Holy" => Some(Element::Holy),
            _ => None
        }
    }
}


impl<'de> Deserialize<'de> for Element {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ElementVisitor;

        impl<'de> Visitor<'de> for ElementVisitor {
            type Value = Element;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("WeaponType must be a string or u8")
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
                match Element::parse(name) {
                    Some(cond) => Ok(cond),
                    None => Err(de::Error::custom(String::from(name)))
                }
            }
        }

        deserializer.deserialize_any(ElementVisitor)
    }
}
