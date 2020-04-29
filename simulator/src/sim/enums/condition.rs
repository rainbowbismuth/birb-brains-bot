use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Condition {
    Stop = 0,
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
    Evil,
    Transparent,
}

pub type ConditionFlags = u64;

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
    "Evil",
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
        Condition::Evil,
        Condition::Transparent,
    ];
}

pub const DAMAGE_CANCELS: [Condition; 3] =
    [Condition::Charm, Condition::Confusion, Condition::Sleep];

pub const DEATH_CANCELS: [Condition; 22] = [
    Condition::Berserk,
    Condition::BloodSuck,
    Condition::Confusion,
    Condition::Charm,
    Condition::Charging,
    Condition::DeathSentence,
    Condition::Defending,
    Condition::DontMove,
    Condition::DontAct,
    Condition::Faith,
    Condition::Float,
    Condition::Haste,
    Condition::Innocent,
    Condition::Performing,
    Condition::Poison,
    Condition::Protect,
    Condition::Reflect,
    Condition::Regen,
    Condition::Shell,
    Condition::Slow,
    Condition::Stop,
    Condition::Transparent,
];

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
            "Undead" => Some(Condition::Undead),
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
            "Evil" => Some(Condition::Evil),
            "Transparent" => Some(Condition::Transparent),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        CONDITION_NAMES[self.index()]
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
            _ => None,
        }
    }

    pub const fn flag(self) -> u64 {
        1 << (self as u64)
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn is_timed_condition(self) -> bool {
        self.index() < TIMED_CONDITIONS_LEN
    }

    pub fn cancels(self) -> &'static [Condition] {
        match self {
            Condition::Haste => &[Condition::Slow],
            Condition::Slow => &[Condition::Haste],
            Condition::Poison => &[Condition::Regen],
            Condition::Regen => &[Condition::Poison],
            Condition::Petrify => &[
                Condition::Charging,
                Condition::DeathSentence,
                Condition::Transparent,
            ],
            Condition::Faith => &[Condition::Innocent],
            Condition::Innocent => &[Condition::Faith],
            Condition::Sleep => &[Condition::Charging],
            Condition::Charm => &[Condition::Charging],
            Condition::Frog => &[Condition::Charging],
            Condition::Berserk => &[Condition::Charging],
            Condition::Chicken => &[Condition::Charging],
            _ => &[],
        }
    }
}
