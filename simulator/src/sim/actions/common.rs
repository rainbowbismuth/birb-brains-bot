use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::{Combatant, CombatantId, Condition, Simulation, Source, NOT_ALIVE_OK};

pub fn should_heal_foe(target: &Combatant, hurts_undead: bool) -> bool {
    hurts_undead && target.undead()
}

pub fn should_heal_ally(target: &Combatant, hurts_undead: bool) -> bool {
    if hurts_undead && target.undead() {
        false
    } else {
        target.hp_percent() <= 0.50
    }
}

pub fn do_hp_heal(
    sim: &mut Simulation,
    target_id: CombatantId,
    mut amount: i16,
    hurts_undead: bool,
) {
    let target = sim.combatant(target_id);
    if target.undead() {
        amount = -amount;
    }
    sim.change_target_hp(target_id, amount, Source::Ability);
}
