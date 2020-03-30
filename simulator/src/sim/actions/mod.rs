use crate::sim::{Combatant, CombatantId, Simulation};

pub mod attack;
pub mod common;
pub mod item;
pub mod white_magic;

pub trait AbilityImpl: Sync {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    );
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId);
}

pub type AbilityFlags = u16;

pub const BERSERK_OK: AbilityFlags = 1;
pub const ALLY_OK: AbilityFlags = 1 << 1;
pub const FOE_OK: AbilityFlags = 1 << 2;
pub const NOT_ALIVE_OK: AbilityFlags = 1 << 3;
pub const PETRIFY_OK: AbilityFlags = 1 << 4;
pub const SILENCEABLE: AbilityFlags = 1 << 5;

pub struct Ability<'a> {
    pub flags: AbilityFlags,
    pub mp_cost: i16,
    pub implementation: &'a (dyn AbilityImpl + 'a),
    pub name: &'a str,
}

#[derive(Copy, Clone)]
pub struct Action<'a> {
    pub ability: &'a Ability<'a>,
    pub range: i8,
    pub ctr: Option<u8>,
    pub target_id: CombatantId,
}

fn filter_ability_level(user: &Combatant, ability: &Ability) -> bool {
    let flags = ability.flags;
    if flags & BERSERK_OK == 0 && user.berserk() {
        false
    } else if flags & SILENCEABLE != 0 && user.silence() {
        false
    } else if ability.mp_cost > 0 && user.mp() < ability.mp_cost {
        false
    } else {
        true
    }
}

fn filter_target_level(user: &Combatant, ability: &Ability, target: &Combatant) -> bool {
    let flags = ability.flags;
    if target.crystal() {
        false
    } else if flags & ALLY_OK == 0 && user.ally(target) {
        false
    } else if flags & FOE_OK == 0 && user.foe(target) {
        false
    } else if flags & NOT_ALIVE_OK == 0 && !target.alive() {
        false
    } else if flags & PETRIFY_OK == 0 && target.petrify() {
        false
    } else {
        true
    }
}

pub fn ai_consider_actions<'a>(
    actions: &mut Vec<Action<'a>>,
    sim: &Simulation<'a>,
    user: &Combatant<'a>,
    targets: &[Combatant<'a>],
) {
    for ability in user.abilities() {
        if !filter_ability_level(user, ability) {
            continue;
        }
        for target in targets {
            if !filter_target_level(user, ability, target) {
                continue;
            }
            ability
                .implementation
                .consider(actions, ability, sim, user, target);
        }
    }
}

pub fn perform_action<'a>(sim: &mut Simulation<'a>, user_id: CombatantId, action: Action<'a>) {
    let ability = action.ability;
    let user = sim.combatant(user_id);
    if !filter_ability_level(user, ability) {
        // TODO: Log some sort of event for failing to perform an ability
        return;
    }
    let target = sim.combatant(action.target_id);
    if !filter_target_level(user, ability, target) {
        // TODO: Log some sort of event for failing to perform an ability
        return;
    }
    ability
        .implementation
        .perform(sim, user_id, action.target_id);
}
