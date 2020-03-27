use crate::sim::{Combatant, CombatantId, Simulation};
use crate::sim::actions::Action;

pub fn consider_attack(sim: &Simulation, user: &Combatant, target: &Combatant) -> Option<Action> {
    None
}

pub fn perform_attack(sim: &mut Simulation, user: CombatantId, target: CombatantId) {}

pub fn perform_frog_attack(sim: &mut Simulation, user: CombatantId, target: CombatantId) {}