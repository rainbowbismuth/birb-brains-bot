use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_repr::Serialize_repr;

#[derive(PartialEq, Eq, Debug, Serialize_repr)]
#[repr(u8)]
pub enum Condition {
    Stop = 1,
    Sleep,
    Slow,
    Shell,
    Regen,
    Reflect,
    Protect,
    Poison,
    Innocent,
    Haste,
    Faith,
    DontMove,
    DontAct,
    Charm,
    Chicken,
    Frog,
    Charging,
    Berserk,
    Petrify,
    Jumping,
    Undead,
    Silence,
    Oil,
    Reraise,
    Wall,
    Darkness,
    Death,
    BloodSuck,
    Confusion,
    Critical,
    DeathSentence,
    Defending,
    Float,
    Performing,
    Transparent,
}

impl Condition {
    pub fn parse(name: &str) -> Option<Condition> {
        match name {
            "Stop" => Some(Condition::Stop),
            "Sleep" => Some(Condition::Sleep),
            "Slow" => Some(Condition::Slow),
            "Shell" => Some(Condition::Shell),
            "Regen" => Some(Condition::Regen),
            "Reflect" => Some(Condition::Reflect),
            "Protect" => Some(Condition::Protect),
            "Poison" => Some(Condition::Poison),
            "Innocent" => Some(Condition::Innocent),
            "Haste" => Some(Condition::Haste),
            "Faith" => Some(Condition::Faith),
            "Don't Move" => Some(Condition::DontMove),
            "Don't Act" => Some(Condition::DontAct),
            "Charm" => Some(Condition::Charm),
            "Chicken" => Some(Condition::Chicken),
            "Frog" => Some(Condition::Frog),
            "Charging" => Some(Condition::Charging),
            "Berserk" => Some(Condition::Berserk),
            "Petrify" => Some(Condition::Petrify),
            "Jumping" => Some(Condition::Jumping),
            "Undead" => Some(Condition::Jumping),
            "Silence" => Some(Condition::Silence),
            "Oil" => Some(Condition::Oil),
            "Reraise" => Some(Condition::Reraise),
            "Wall" => Some(Condition::Wall),
            "Darkness" | "Blind" => Some(Condition::Darkness),
            "Death" => Some(Condition::Death),
            "Blood Suck" => Some(Condition::BloodSuck),
            "Confusion" => Some(Condition::Confusion),
            "Critical" => Some(Condition::Critical),
            "Death Sentence" => Some(Condition::DeathSentence),
            "Defending" => Some(Condition::Defending),
            "Float" => Some(Condition::Float),
            "Performing" => Some(Condition::Performing),
            "Transparent" => Some(Condition::Transparent),
            _ => None
        }
    }

    pub fn flag(self) -> u64 {
        1 << ((self as u64) - 1)
    }
}

impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ConditionVisitor;

        impl<'de> Visitor<'de> for ConditionVisitor {
            type Value = Condition;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Condition must be a string or u8")
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
                match Condition::parse(name) {
                    Some(cond) => Ok(cond),
                    None => Err(de::Error::custom(String::from(name)))
                }
            }
        }

        deserializer.deserialize_any(ConditionVisitor)
    }
}
