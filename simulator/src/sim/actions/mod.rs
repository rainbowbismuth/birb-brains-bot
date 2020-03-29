use crate::sim::{Combatant, CombatantId, Simulation};

pub mod attack;
pub mod item;
pub mod white_magic;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionKind {
    FrogAttack,
    Attack,
    Item(item::Item),
    WhiteMagic(white_magic::WhiteMagic),
}

#[derive(Copy, Clone, Debug)]
pub struct Action {
    pub kind: ActionKind,
    pub range: i8,
    pub ctr: Option<u8>,
    pub target_id: CombatantId,
}

pub fn ai_consider_actions(
    actions: &mut Vec<Action>,
    sim: &Simulation,
    user: &Combatant,
    targets: &[Combatant],
) {
    for target in targets {
        attack::consider_attack(actions, sim, user, target);
        item::consider_item(actions, sim, user, target);
        white_magic::consider_white_magic(actions, sim, user, target);
    }
}

pub fn perform_action(sim: &mut Simulation, user_id: CombatantId, action: Action) {
    match action.kind {
        ActionKind::FrogAttack => attack::perform_frog_attack(sim, user_id, action.target_id),

        ActionKind::Attack => attack::perform_attack(sim, user_id, action.target_id),

        ActionKind::Item(item) => item::perform_item(sim, user_id, action.target_id, item),

        ActionKind::WhiteMagic(spell) => {
            white_magic::perform_white_magic(sim, user_id, action.target_id, spell)
        }
    }
}
