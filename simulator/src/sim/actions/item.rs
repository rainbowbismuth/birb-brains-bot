use crate::sim::{Combatant, CombatantId, Simulation};
use crate::sim::actions::Action;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Potion,
    HiPotion,
    XPotion,
    PhoenixDown,
}

pub fn consider_item(sim: &Simulation, user: &Combatant, target: &Combatant) -> Vec<Action> {
    vec![]
}

pub fn perform_item(sim: &mut Simulation, user: CombatantId, target: CombatantId, item: Item) {}