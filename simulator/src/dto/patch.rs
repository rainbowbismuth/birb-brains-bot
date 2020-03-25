use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::sim::{Condition, Element, WeaponType};

#[derive(Serialize, Deserialize)]
pub struct Patch {
    pub time: f32,
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
    pub range: i16,
    pub aoe: Option<isize>,
    pub ct: i16,
    pub mp: i16,
    pub ma_constant: Option<isize>,
    pub adds: Vec<Condition>,
    pub cancels: Vec<Condition>,
    pub chance_to_add: Vec<Condition>,
    pub chance_to_cancel: Vec<Condition>,
}

#[derive(Serialize, Deserialize)]
pub struct HitChance {
    pub ma_plus: Option<i16>,
    pub pa_plus: Option<i16>,
    pub speed_plus: Option<i16>,
    pub pa_wp_plus: Option<i16>,
    pub times_faith: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentData {
    pub by_name: HashMap<String, Equipment>
}

#[derive(Serialize, Deserialize)]
pub struct Equipment {
    pub name: String,
    pub hp_bonus: i16,
    pub mp_bonus: i16,
    pub speed_bonus: i16,
    pub pa_bonus: i16,
    pub ma_bonus: i16,
    pub wp: i16,
    pub range: i16,
    pub w_ev: i16,
    pub phys_ev: i16,
    pub magic_ev: i16,
    pub weapon_type: Option<WeaponType>,
    pub weapon_element: Option<Element>,
    pub move_bonus: i16,
    pub jump_bonus: i16,
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
    pub hp: i16,
    pub mp: i16,
    #[serde(rename = "move")]
    pub movement: i16,
    pub jump: i16,
    pub speed: i16,
    pub pa: i16,
    pub ma: i16,
    pub c_ev: i16,
    pub innates: Vec<String>,
    pub skills: Vec<String>,
    pub absorbs: Vec<Element>,
    pub halves: Vec<Element>,
    pub weaknesses: Vec<Element>,
    pub cancels: Vec<Element>,
}