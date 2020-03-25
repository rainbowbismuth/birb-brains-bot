use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::sim::condition::Condition;
use crate::sim::element::Element;
use crate::sim::weapon::WeaponType;

#[derive(Serialize, Deserialize)]
pub struct Patch {
    pub ability: AbilityData,
    pub equipment: EquipmentData,
    pub base_stats: BaseStatsData,
}

#[derive(Serialize, Deserialize)]
pub struct AbilityData {
    pub by_name: HashMap<String, Ability>
}

#[derive(Serialize, Deserialize)]
pub struct Ability {
    pub name: String,
    pub name_with_tag: String,
    pub multiplier: Option<String>,
    pub hit_chance: HitChance,
    pub damage: bool,
    pub heals: bool,
    pub element: Option<Element>,
    pub range: isize,
    pub aoe: Option<isize>,
    pub ct: isize,
    pub mp: isize,
    pub ma_constant: Option<isize>,
    pub adds: Vec<Condition>,
    pub cancels: Vec<Condition>,
    pub chance_to_add: Vec<Condition>,
    pub chance_to_cancel: Vec<Condition>,
}

#[derive(Serialize, Deserialize)]
pub struct HitChance {
    pub ma_plus: Option<isize>,
    pub pa_plus: Option<isize>,
    pub speed_plus: Option<isize>,
    pub pa_wp_plus: Option<isize>,
    pub times_faith: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentData {
    pub by_name: HashMap<String, Equipment>
}

#[derive(Serialize, Deserialize)]
pub struct Equipment {
    pub name: String,
    pub hp_bonus: isize,
    pub mp_bonus: isize,
    pub speed_bonus: isize,
    pub pa_bonus: isize,
    pub ma_bonus: isize,
    pub wp: isize,
    pub range: isize,
    pub w_ev: isize,
    pub phys_ev: isize,
    pub magic_ev: isize,
    pub weapon_type: Option<WeaponType>,
    pub weapon_element: Option<Element>,
    pub move_bonus: isize,
    pub jump_bonus: isize,
    pub strengthens: Vec<Element>,
    pub absorbs: Vec<Element>,
    pub halves: Vec<Element>,
    pub weaknesses: Vec<Element>,
    pub cancels: Vec<Condition>,
    pub cancels_element: Vec<Element>,
    pub initial: Vec<Condition>,
    pub chance_to_add: Vec<Condition>,
    pub chance_to_cancel: Vec<Condition>,
    pub immune_to: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BaseStatsData {
    pub by_job_gender: HashMap<String, BaseStats>
}

#[derive(Serialize, Deserialize)]
pub struct BaseStats {
    pub job: String,
    pub gender: String,
    pub hp: isize,
    pub mp: isize,
    #[serde(rename = "move")]
    pub movement: isize,
    pub jump: isize,
    pub speed: isize,
    pub pa: isize,
    pub ma: isize,
    pub c_ev: isize,
    pub innates: Vec<String>,
    pub skills: Vec<String>,
    pub absorbs: Vec<Element>,
    pub halves: Vec<Element>,
    pub weaknesses: Vec<Element>,
    pub cancels: Vec<Element>,
}