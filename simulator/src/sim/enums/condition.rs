use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde_repr::Serialize_repr;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize_repr)]
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

const FIRST_CONDITION: Condition = Condition::Stop;
const LAST_CONDITION: Condition = Condition::Transparent;

pub const TIMED_CONDITIONS_LEN: usize = Condition::Charm.index() + 1;
pub const TIMED_CONDITIONS: [Condition; TIMED_CONDITIONS_LEN] = [
    Condition::Stop,
    Condition::Sleep,
    Condition::Slow,
    Condition::Shell,
    Condition::Regen,
    Condition::Reflect,
    Condition::Protect,
    Condition::Poison,
    Condition::Innocent,
    Condition::Haste,
    Condition::Faith,
    Condition::DontMove,
    Condition::DontAct,
    Condition::Charm,
];

const CONDITIONS_LEN: usize = LAST_CONDITION.index() + 1;
const CONDITION_NAMES: [&'static str; CONDITIONS_LEN] = [
    "Stop",
    "Sleep",
    "Slow",
    "Shell",
    "Regen",
    "Reflect",
    "Protect",
    "Poison",
    "Innocent",
    "Haste",
    "Faith",
    "Don't Move",
    "Don't Act",
    "Charm",
    "Chicken",
    "Frog",
    "Charging",
    "Berserk",
    "Petrify",
    "Jumping",
    "Undead",
    "Silence",
    "Oil",
    "Reraise",
    "Wall",
    "Darkness",
    "Death",
    "Blood Suck",
    "Confusion",
    "Critical",
    "Death Sentence",
    "Defending",
    "Float",
    "Performing",
    "Transparent",
];

lazy_static! {
    pub static ref ALL_CONDITIONS: Vec<Condition> = vec![
        Condition::Stop,
        Condition::Sleep,
        Condition::Slow,
        Condition::Shell,
        Condition::Regen,
        Condition::Reflect,
        Condition::Protect,
        Condition::Poison,
        Condition::Innocent,
        Condition::Haste,
        Condition::Faith,
        Condition::DontMove,
        Condition::DontAct,
        Condition::Charm,
        Condition::Chicken,
        Condition::Frog,
        Condition::Charging,
        Condition::Berserk,
        Condition::Petrify,
        Condition::Jumping,
        Condition::Undead,
        Condition::Silence,
        Condition::Oil,
        Condition::Reraise,
        Condition::Wall,
        Condition::Darkness,
        Condition::Death,
        Condition::BloodSuck,
        Condition::Confusion,
        Condition::Critical,
        Condition::DeathSentence,
        Condition::Defending,
        Condition::Float,
        Condition::Performing,
        Condition::Transparent,
    ];
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

    pub fn name(self) -> &'static str {
        CONDITION_NAMES[self.index()]
    }

    pub fn from_num(code: u8) -> Option<Condition> {
        if code >= FIRST_CONDITION as u8 && code <= LAST_CONDITION as u8 {
            unsafe { Some(std::mem::transmute_copy(&code)) }
        } else {
            None
        }
    }

    pub fn condition_duration(self) -> Option<i8> {
        match self {
            Condition::Charm => Some(32),
            Condition::DontAct => Some(24),
            Condition::DontMove => Some(24),
            Condition::Faith => Some(32),
            Condition::Haste => Some(32),
            Condition::Innocent => Some(32),
            Condition::Poison => Some(36),
            Condition::Protect => Some(32),
            Condition::Reflect => Some(32),
            Condition::Regen => Some(36),
            Condition::Shell => Some(32),
            Condition::Slow => Some(24),
            Condition::Sleep => Some(60),
            Condition::Stop => Some(20),
            _ => None
        }
    }

    pub const fn flag(self) -> u64 {
        1 << ((self as u64) - 1)
    }

    pub const fn index(self) -> usize { (self as usize) - 1 }

    pub const fn is_timed_condition(self) -> bool { self.index() < TIMED_CONDITIONS_LEN }
}

#[derive(Copy, Clone)]
pub struct ConditionBlock {
    pub status_flags: u64,
    pub timed_conditions: [i8; TIMED_CONDITIONS_LEN],
}

impl ConditionBlock {
    pub fn new() -> ConditionBlock {
        ConditionBlock {
            status_flags: 0,
            timed_conditions: [0; TIMED_CONDITIONS_LEN],
        }
    }

    pub fn add(&mut self, condition: Condition) {
        if let Some(duration) = condition.condition_duration() {
            self.timed_conditions[condition.index()] = duration;
        }
        self.status_flags |= condition.flag();
    }

    pub fn has(&self, condition: Condition) -> bool {
        self.status_flags & condition.flag() != 0
    }

    pub fn tick(&mut self, condition: Condition) -> Option<bool> {
        if condition.is_timed_condition() {
            let count = self.timed_conditions[condition.index()];
            if count > 0 {
                self.timed_conditions[condition.index()] -= 1;
            }
            let to_remove = count == 1;
            if to_remove {
                self.status_flags &= !condition.flag();
            }
            Some(to_remove)
        } else {
            None
        }
    }

    pub fn remove(&mut self, condition: Condition) {
        if condition.is_timed_condition() {
            self.timed_conditions[condition.index()] = 0;
        }
        self.status_flags &= !condition.flag();
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
                match Condition::from_num(code) {
                    Some(cond) => Ok(cond),
                    None => Err(de::Error::custom(format!("{} is not a valid Condition code", code)))
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

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    pub fn durations_only_for_timed_conditions() {
        for condition in &TIMED_CONDITIONS {
            if condition.is_timed_condition() {
                assert!(condition.condition_duration().is_some());
            } else {
                assert!(condition.condition_duration().is_none());
            }
        }
    }

    #[test]
    pub fn can_add_remove_any_condition() {
        let mut block = ConditionBlock::new();
        for condition in &TIMED_CONDITIONS {
            assert!(!block.has(*condition));
            block.add(*condition);
            assert!(block.has(*condition));
            block.remove(*condition);
            assert!(!block.has(*condition));
        }
    }


    #[test]
    pub fn tick_condition_status_until_removal() {
        for condition in &TIMED_CONDITIONS {
            let mut block = ConditionBlock::new();
            block.add(*condition);
            for _ in 0..condition.condition_duration().unwrap() - 1 {
                assert_eq!(block.tick(*condition), Some(false));
                assert!(block.has(*condition));
            }
            assert_eq!(block.tick(*condition), Some(true));
            assert!(!block.has(*condition));
        }
    }
}