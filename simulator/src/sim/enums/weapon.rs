use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_repr::Serialize_repr;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize_repr)]
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


impl<'de> Deserialize<'de> for WeaponType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct WeaponTypeVisitor;

        impl<'de> Visitor<'de> for WeaponTypeVisitor {
            type Value = WeaponType;

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
                match WeaponType::parse(name) {
                    Some(cond) => Ok(cond),
                    None => Err(de::Error::custom(String::from(name)))
                }
            }
        }

        deserializer.deserialize_any(WeaponTypeVisitor)
    }
}
