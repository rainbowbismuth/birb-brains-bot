use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MatchUp {
    pub tournament_id: u64,
    pub modified: f64,
    pub left: Team,
    pub right: Team,
    pub left_wins: Option<bool>,
    pub game_map: String,
}

#[derive(Serialize, Deserialize)]
pub struct Team {
    pub color: String,
    pub combatants: Vec<Combatant>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Combatant {
    pub name: String,
    pub gender: String,
    pub sign: String,
    pub brave: i8,
    pub faith: i8,
    pub class: String,
    pub action_skill: String,
    pub reaction_skill: String,
    pub support_skill: String,
    pub move_skill: String,
    pub mainhand: String,
    pub offhand: String,
    pub head: String,
    pub armor: String,
    pub accessory: String,
    pub class_skills: Vec<String>,
    pub extra_skills: Vec<String>,
}

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
    pub element: Option<String>,
    pub range: u8,
    pub aoe: Option<u8>,
    pub ct: i8,
    pub mp: i8,
    pub ma_constant: Option<i8>,
    pub adds: Vec<String>,
    pub cancels: Vec<String>,
    pub chance_to_add: Vec<String>,
    pub chance_to_cancel: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HitChance {
    pub ma_plus: Option<u8>,
    pub pa_plus: Option<u8>,
    pub speed_plus: Option<u8>,
    pub pa_wp_plus: Option<u8>,
    pub times_faith: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentData {
    pub by_name: HashMap<String, Equipment>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Equipment {
    pub name: String,
    pub hp_bonus: i16,
    pub mp_bonus: i16,
    pub speed_bonus: i8,
    pub pa_bonus: i8,
    pub ma_bonus: i8,
    pub wp: i8,
    pub range: i8,
    pub w_ev: i8,
    pub phys_ev: i8,
    pub magic_ev: i8,
    pub move_bonus: i8,
    pub jump_bonus: i8,
    pub weapon_type: Option<String>,
    pub weapon_element: Option<String>,
    pub strengthens: Vec<String>,
    pub absorbs: Vec<String>,
    pub halves: Vec<String>,
    pub weaknesses: Vec<String>,
    pub cancels: Vec<String>,
    pub cancels_element: Vec<String>,
    pub initial: Vec<String>,
    pub chance_to_add: Vec<String>,
    pub chance_to_cancel: Vec<String>,
    pub immune_to: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BaseStatsData {
    pub by_job_gender: HashMap<String, BaseStats>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseStats {
    pub job: String,
    pub gender: String,
    pub hp: i16,
    pub mp: i16,
    #[serde(rename = "move")]
    pub movement: i8,
    pub jump: i8,
    pub speed: i8,
    pub pa: i8,
    pub ma: i8,
    pub c_ev: i8,
    pub innates: Vec<String>,
    pub skills: Vec<String>,
    pub absorbs: Vec<String>,
    pub halves: Vec<String>,
    pub weaknesses: Vec<String>,
    pub cancels: Vec<String>,
}