use std::collections::HashMap;

use serde::Deserialize;

use crate::sim::condition::Condition;

#[derive(Deserialize)]
pub struct Patch {
    ability: AbilityData,
    equipment: EquipmentData,
    base_stats: BaseStatsData,
}

#[derive(Deserialize)]
pub struct AbilityData {
    by_name: HashMap<String, Ability>
}

#[derive(Deserialize)]
pub struct Ability {
    name: String,
    name_with_tag: String,
    multiplier: Option<String>,
    hit_chance: HitChance,
    damage: bool,
    heals: bool,
    element: Option<String>,
    range: isize,
    aoe: Option<isize>,
    ct: isize,
    mp: isize,
    ma_constant: Option<isize>,
    adds: Vec<Condition>,
    cancels: Vec<Condition>,
    chance_to_add: Vec<Condition>,
    chance_to_cancel: Vec<Condition>,
}

#[derive(Deserialize)]
pub struct HitChance {
    ma_plus: Option<isize>,
    pa_plus: Option<isize>,
    speed_plus: Option<isize>,
    pa_wp_plus: Option<isize>,
    times_faith: bool,
}

#[derive(Deserialize)]
pub struct EquipmentData {
    by_name: HashMap<String, Equipment>
}

#[derive(Deserialize)]
pub struct Equipment {
    name: String,
    hp_bonus: isize,
    mp_bonus: isize,
    speed_bonus: isize,
    pa_bonus: isize,
    ma_bonus: isize,
    wp: isize,
    range: isize,
    w_ev: isize,
    phys_ev: isize,
    magic_ev: isize,
    weapon_type: Option<String>,
    weapon_element: Option<String>,
    move_bonus: isize,
    jump_bonus: isize,
    strengthens: Vec<String>,
    absorbs: Vec<String>,
    halves: Vec<String>,
    weaknesses: Vec<String>,
    cancels: Vec<Condition>,
    cancels_element: Vec<String>,
    initial: Vec<Condition>,
    chance_to_add: Vec<Condition>,
    chance_to_cancel: Vec<Condition>,
    immune_to: Vec<String>,
}

#[derive(Deserialize)]
pub struct BaseStatsData {
    by_job_gender: HashMap<String, BaseStats>
}

#[derive(Deserialize)]
pub struct BaseStats {
    job: String,
    gender: String,
    hp: isize,
    mp: isize,
    #[serde(rename = "move")]
    movement: isize,
    jump: isize,
    speed: isize,
    pa: isize,
    ma: isize,
    c_ev: isize,
    innates: Vec<String>,
    skills: Vec<String>,
    absorbs: Vec<String>,
    halves: Vec<String>,
    weaknesses: Vec<String>,
    cancels: Vec<String>,
}