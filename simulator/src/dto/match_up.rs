use serde::{Deserialize, Serialize};

use crate::sim::{Gender, Sign};

#[derive(Serialize, Deserialize)]
pub struct MatchUp {
    pub tournament_id: isize,
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
    pub gender: Gender,
    pub sign: Sign,
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
