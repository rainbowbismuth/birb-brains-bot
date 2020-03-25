use serde::Deserialize;

#[derive(Deserialize)]
pub struct MatchUp {
    tournament_id: isize,
    modified: f64,
    left: Team,
    right: Team,
    left_wins: Option<bool>,
    game_map: String,
}

#[derive(Deserialize)]
pub struct Team {
    color: String,
    combatants: Vec<Combatant>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Combatant {
    name: String,
    gender: String,
    sign: String,
    brave: isize,
    faith: isize,
    class: String,
    action_skill: String,
    reaction_skill: String,
    support_skill: String,
    move_skill: String,
    mainhand: String,
    offhand: String,
    head: String,
    armor: String,
    accessory: String,
    class_skills: Vec<String>,
    extra_skills: Vec<String>,
}
