use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize_repr, Deserialize_repr)]
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

pub type ElementFlags = u16;

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

    pub fn flag(self) -> ElementFlags {
        1 << (self as u16)
    }
}