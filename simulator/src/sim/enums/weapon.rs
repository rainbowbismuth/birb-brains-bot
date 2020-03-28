use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WeaponType {
    Knife = 1,
    NinjaSword,
    Bow,
    KnightSword,
    Katana,
    Sword,
    Pole,
    Spear,
    Crossbow,
    Staff,
    Flail,
    Bag,
    Cloth,
    Harp,
    Book,
    Gun,
}

impl WeaponType {
    pub fn parse(name: &str) -> Option<WeaponType> {
        match name {
            "Knife" => Some(WeaponType::Knife),
            "Ninja Sword" | "Ninja Blade" => Some(WeaponType::NinjaSword),
            "Longbow" | "Bow" => Some(WeaponType::Bow),
            "Knight Sword" => Some(WeaponType::KnightSword),
            "Katana" => Some(WeaponType::Katana),
            "Sword" => Some(WeaponType::Sword),
            "Rod" | "Pole" => Some(WeaponType::Pole),
            "Spear" => Some(WeaponType::Spear),
            "Crossbow" => Some(WeaponType::Crossbow),
            "Staff" | "Stick" => Some(WeaponType::Staff),
            "Flail" | "Axe" => Some(WeaponType::Flail),
            "Bag" => Some(WeaponType::Bag),
            "Cloth" | "Fabric" => Some(WeaponType::Cloth),
            "Musical Instrument" | "Harp" => Some(WeaponType::Harp),
            "Dictionary" | "Book" => Some(WeaponType::Book),
            "Gun" => Some(WeaponType::Gun),
            _ => None
        }
    }
}