use crate::sim::{Combatant, CombatantId, Simulation};

pub mod attack;
pub mod item;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionKind {
    FrogAttack,
    Attack,
    Item(item::Item),
}

#[derive(Copy, Clone, Debug)]
pub struct Action {
    pub kind: ActionKind,
    pub range: i16,
    pub target_id: CombatantId,
}

pub fn ai_consider_actions(sim: &Simulation, user: &Combatant, targets: &[Combatant]) -> Vec<Action> {
    let mut actions = vec![];
    for target in targets {
        actions.extend(attack::consider_attack(sim, user, target));
        actions.extend(item::consider_item(sim, user, target));
    }
    actions
}

pub fn perform_action(sim: &mut Simulation, user_id: CombatantId, action: Action) {
    match action.kind {
        ActionKind::FrogAttack =>
            attack::perform_frog_attack(sim, user_id, action.target_id),

        ActionKind::Attack =>
            attack::perform_attack(sim, user_id, action.target_id),

        ActionKind::Item(item) =>
            item::perform_item(sim, user_id, action.target_id, item)
    }
}