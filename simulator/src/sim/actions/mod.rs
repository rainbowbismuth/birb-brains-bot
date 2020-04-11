use crate::sim::{Combatant, CombatantId, Event, Facing, Location, Simulation};

pub mod attack;
pub mod battle_skill;
pub mod black_magic;
pub mod charge;
pub mod common;
pub mod draw_out;
pub mod item;
pub mod jump;
pub mod monster;
pub mod punch_art;
pub mod steal;
pub mod summon_magic;
pub mod throw;
pub mod time_magic;
pub mod white_magic;
pub mod yin_yang_magic;

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
pub const NO_SHORT_CHARGE: AbilityFlags = 1 << 6;
pub const HITS_FOES_ONLY: AbilityFlags = 1 << 7;
pub const HITS_ALLIES_ONLY: AbilityFlags = 1 << 8;
pub const TARGET_SELF_ONLY: AbilityFlags = 1 << 9;
pub const JUMPING: AbilityFlags = 1 << 10;
pub const TARGET_NOT_SELF: AbilityFlags = 1 << 11;
pub const FROG_OK: AbilityFlags = 1 << 12;
pub const DONT_MOVE_WHILE_CHARGING: AbilityFlags = 1 << 13;
pub const CAN_BE_REFLECTED: AbilityFlags = 1 << 14;
pub const CAN_BE_CALCULATED: AbilityFlags = 1 << 15;

#[derive(Copy, Clone)]
pub enum AoE {
    None,
    Diamond(u8),
    Line,
    TriLine, // Like that Tiamat ability
}

impl AoE {
    pub fn is_line(self) -> bool {
        match self {
            AoE::None => false,
            AoE::Diamond(_size) => false,
            AoE::Line => true,
            AoE::TriLine => true,
        }
    }
}

pub struct Ability<'a> {
    pub flags: AbilityFlags,
    pub mp_cost: i16,
    pub aoe: AoE,
    // TODO: Refactor this, consider if it needs to be in the ability impl itself?
    pub implementation: &'a (dyn AbilityImpl + 'a),
    pub name: &'a str,
}

#[derive(Clone, Copy)]
pub enum ActionTarget {
    Id(CombatantId),
    Panel(Location),
}

impl ActionTarget {
    pub fn to_location(self, sim: &Simulation) -> Location {
        match self {
            ActionTarget::Id(target_id) => sim.combatant(target_id).location,
            ActionTarget::Panel(location) => location,
        }
    }

    pub fn to_location_combatant_slice(self, combatants: &[Combatant]) -> Location {
        match self {
            ActionTarget::Id(target_id) => combatants[target_id.index()].location,
            ActionTarget::Panel(location) => location,
        }
    }

    pub fn to_target_id(self, sim: &Simulation) -> Option<CombatantId> {
        match self {
            ActionTarget::Id(target_id) => Some(target_id),
            ActionTarget::Panel(location) => sim.combatant_on_panel(location),
        }
    }

    pub fn to_target_id_only(self) -> Option<CombatantId> {
        match self {
            ActionTarget::Id(target_id) => Some(target_id),
            ActionTarget::Panel(_location) => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Action<'a> {
    pub ability: &'a Ability<'a>,
    pub range: i8,
    pub ctr: Option<u8>,
    pub target: ActionTarget,
}

impl<'a> Action<'a> {
    pub fn new(
        ability: &'a Ability<'a>,
        range: i8,
        ctr: Option<u8>,
        target_id: CombatantId,
    ) -> Action<'a> {
        Action {
            ability,
            range,
            ctr,
            target: ActionTarget::Id(target_id),
        }
    }

    pub fn target_panel(
        ability: &'a Ability<'a>,
        range: i8,
        ctr: Option<u8>,
        location: Location,
    ) -> Action<'a> {
        Action {
            ability,
            range,
            ctr,
            target: ActionTarget::Panel(location),
        }
    }
}

fn filter_ability_level(user: &Combatant, ability: &Ability) -> bool {
    let flags = ability.flags;
    if flags & BERSERK_OK == 0 && user.berserk() {
        false
    } else if flags & FROG_OK == 0 && user.frog() {
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
    if target.crystal() || target.jumping() {
        false
    } else if flags & TARGET_NOT_SELF != 0 && user.id() == target.id() {
        false
    } else if flags & TARGET_SELF_ONLY != 0 && user.id() != target.id() {
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
    let foes_have_non_disabled = sim.ai_foes_have_non_disabled_units(user);
    for ability in user.abilities() {
        if !filter_ability_level(user, ability) {
            continue;
        }
        for target in targets {
            if !filter_target_level(user, ability, target) {
                continue;
            }

            if user.foe(target) && foes_have_non_disabled {
                if target.confusion() || target.death_sentence() {
                    continue;
                }
            }

            ability
                .implementation
                .consider(actions, ability, sim, user, target);
        }
    }
}

pub fn perform_action_slow<'a>(sim: &mut Simulation<'a>, user_id: CombatantId, action: Action<'a>) {
    let ability = action.ability;
    let user = sim.combatant(user_id);

    // TODO: These are redundant with the entire check below..
    if action.ability.flags & SILENCEABLE != 0 && user.silence() {
        sim.log_event(Event::Silenced(user_id, action));
        return;
    } else if ability.mp_cost > 0 && user.mp() < ability.mp_cost {
        sim.log_event(Event::NoMP(user_id, action));
        return;
    }

    if !filter_ability_level(user, ability) {
        return;
    }

    perform_action(sim, user_id, action)
}

pub fn perform_action<'a>(sim: &mut Simulation<'a>, user_id: CombatantId, action: Action<'a>) {
    let ability = action.ability;

    if ability.mp_cost > 0 {
        let user = sim.combatant_mut(user_id);
        let mp_cost = if user.halve_mp() {
            1.max(ability.mp_cost / 2)
        } else {
            ability.mp_cost
        };
        let new_mp = user.mp() - mp_cost;
        user.set_mp_within_bounds(new_mp);
    }

    let mut action_target = action.target;
    if action.ability.flags & CAN_BE_REFLECTED != 0 {
        // TODO: FFT AI has some awesome decision making around the AI planning it's actions
        //  based on reflect, if the reflective unit can move by the time the spell goes off,
        //  etc.
        if let Some(target_id) = action_target.to_target_id(sim) {
            let target = sim.combatant(target_id);
            if target.reflect() {
                let user = sim.combatant(user_id);
                let direction = target.location - user.location;
                let new_location = direction * 2;
                action_target = ActionTarget::Panel(new_location);
                sim.log_event(Event::SpellReflected(target_id, new_location));
            }
        }
    }

    match ability.aoe {
        AoE::None => {
            if let Some(target_id) = action.target.to_target_id(sim) {
                let target = sim.combatant(target_id);
                if target.jumping() {
                    return;
                }

                let user = sim.combatant(user_id);
                // TODO: Not a great place for this.. re: MP costs.
                if !filter_target_level(user, ability, target) {
                    // TODO: Log some sort of event for failing to perform an ability
                    return;
                }

                ability.implementation.perform(sim, user_id, target_id);
            } else {
                // TODO: Something about the ability missing.
            }
        }
        AoE::Diamond(size) => {
            for target_panel in action.target.to_location(sim).diamond(size) {
                perform_aoe_on_panel(sim, user_id, ability, target_panel)
            }
        }
        AoE::Line => {
            let user = sim.combatant(user_id);
            let target_location = action_target.to_location(sim);
            let user_location = user.location;
            let facing = Facing::towards(user.location, target_location);
            for i in 1..=action.range {
                let target_panel = user_location + facing.offset() * i as i16;
                perform_aoe_on_panel(sim, user_id, ability, target_panel);
            }
        }
        AoE::TriLine => {
            let user = sim.combatant(user_id);
            let target_location = action_target.to_location(sim);
            let facing = Facing::towards(user.location, target_location);
            let user_location = user.location;
            let left_facing = facing.rotate(-1);
            let right_facing = facing.rotate(1);
            for i in 1..=action.range {
                let target_panel = user_location + facing.offset() * i as i16;
                perform_aoe_on_panel(sim, user_id, ability, target_panel);

                let target_panel = user_location + left_facing.offset() * i as i16;
                perform_aoe_on_panel(sim, user_id, ability, target_panel);

                let target_panel = user_location + right_facing.offset() * i as i16;
                perform_aoe_on_panel(sim, user_id, ability, target_panel);
            }
        }
    }
    sim.end_of_action_checks(user_id);
}

fn perform_aoe_on_panel(
    sim: &mut Simulation,
    user_id: CombatantId,
    ability: &Ability,
    location: Location,
) {
    if let Some(real_target_id) = sim.combatant_on_panel(location) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(real_target_id);
        if target.crystal() || target.jumping() {
            return;
        }
        if ability.flags & HITS_FOES_ONLY != 0 && !user.foe(target) {
            return;
        }
        if ability.flags & HITS_ALLIES_ONLY != 0 && !user.ally(target) {
            return;
        }
        if ability.flags & NOT_ALIVE_OK == 0 && !target.alive() {
            return;
        }
        if ability.flags & PETRIFY_OK == 0 && target.petrify() {
            return;
        }

        ability.implementation.perform(sim, user_id, real_target_id);
    }
}
