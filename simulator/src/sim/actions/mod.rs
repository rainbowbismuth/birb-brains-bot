use crate::sim::{
    Combatant, CombatantId, Condition, Event, Facing, Location, Simulation, Source, COMBATANT_IDS,
};

pub mod attack;
pub mod basic_skill;
pub mod battle_skill;
pub mod black_magic;
pub mod charge;
pub mod common;
pub mod draw_out;
pub mod elemental;
pub mod item;
pub mod jump;
pub mod math_skill;
pub mod monster;
pub mod punch_art;
pub mod steal;
pub mod summon_magic;
pub mod talk_skill;
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

pub type AbilityFlags = u32;

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
pub const USE_ON_CRITICAL_ONLY: AbilityFlags = 1 << 16;
pub const UNDER_50_PERCENT_HP_ONLY: AbilityFlags = 1 << 17;

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
#[repr(u8)]
pub enum CalcAttribute {
    CT = 0,
    Height,
}

impl CalcAttribute {
    pub fn flag(self) -> u8 {
        1 << self as u8
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum CalcAlgorithm {
    Prime = 0,
    M3,
    M4,
    M5,
}

impl CalcAlgorithm {
    pub fn flag(self) -> u8 {
        1 << self as u8
    }
}

#[derive(Clone, Copy)]
pub enum ActionTarget {
    Id(CombatantId),
    Panel(Location),
    Math(CalcAttribute, CalcAlgorithm),
}

impl ActionTarget {
    pub fn is_math(self) -> bool {
        match self {
            ActionTarget::Math(_, _) => true,
            _ => false,
        }
    }

    pub fn to_location(self, sim: &Simulation) -> Option<Location> {
        match self {
            ActionTarget::Id(target_id) => Some(sim.combatant(target_id).location),
            ActionTarget::Panel(location) => Some(location),
            ActionTarget::Math(_, _) => None,
        }
    }

    pub fn to_location_combatant_slice(self, combatants: &[Combatant]) -> Option<Location> {
        match self {
            ActionTarget::Id(target_id) => Some(combatants[target_id.index()].location),
            ActionTarget::Panel(location) => Some(location),
            ActionTarget::Math(_, _) => None,
        }
    }

    pub fn to_target_id(self, sim: &Simulation) -> Option<CombatantId> {
        match self {
            ActionTarget::Id(target_id) => Some(target_id),
            ActionTarget::Panel(location) => sim.combatant_on_panel(location),
            ActionTarget::Math(_, _) => None,
        }
    }

    pub fn to_target_id_only(self) -> Option<CombatantId> {
        match self {
            ActionTarget::Id(target_id) => Some(target_id),
            ActionTarget::Panel(_location) => None,
            ActionTarget::Math(_, _) => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Action<'a> {
    pub ability: &'a Ability<'a>,
    pub range: u8,
    pub ctr: Option<u8>,
    pub target: ActionTarget,
}

impl<'a> Action<'a> {
    pub fn new(
        ability: &'a Ability<'a>,
        range: u8,
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
        range: u8,
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
    } else if flags & USE_ON_CRITICAL_ONLY != 0 && !target.critical() {
        false
    } else if flags & UNDER_50_PERCENT_HP_ONLY != 0 && target.hp_percent() > 0.50 {
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
    let mut action_target = action.target;
    let user = sim.combatant_mut(user_id);
    if !action_target.is_math() && ability.mp_cost > 0 && !user.no_mp() {
        let mp_cost = if user.halve_mp() {
            1.max(ability.mp_cost / 2)
        } else {
            ability.mp_cost
        };
        let new_mp = user.mp() - mp_cost;
        user.set_mp_within_bounds(new_mp);
    }

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

    if ability.flags & JUMPING != 0 {
        sim.cancel_condition(user_id, Condition::Jumping, Source::Ability);
    }

    if let ActionTarget::Math(attr, algo) = action_target {
        handle_math_ability(sim, user_id, ability, attr, algo);
    } else {
        handle_normal_ability(sim, user_id, action, ability, action_target);
    }
    sim.end_of_action_checks(user_id);
}

fn handle_math_ability(
    sim: &mut Simulation,
    user_id: CombatantId,
    ability: &Ability,
    attr: CalcAttribute,
    algo: CalcAlgorithm,
) {
    for cid in &COMBATANT_IDS {
        if !math_match(sim, *cid, attr, algo) {
            continue;
        }
        let target = sim.combatant(*cid);
        if ability.flags & NOT_ALIVE_OK == 0 && !target.alive() {
            return;
        }
        if ability.flags & PETRIFY_OK == 0 && target.petrify() {
            return;
        }
        ability.implementation.perform(sim, user_id, *cid);
    }
}

const PRIME_NUMBERS: &[u8] = &[
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
];

fn math_match(
    sim: &Simulation,
    combatant_id: CombatantId,
    attr: CalcAttribute,
    algo: CalcAlgorithm,
) -> bool {
    let combatant = sim.combatant(combatant_id);
    if combatant.crystal() {
        return false;
    }
    let val = match attr {
        CalcAttribute::CT => combatant.ct,
        CalcAttribute::Height => {
            let height = sim.combatant_height(combatant_id);
            if height.floor() != height.ceil() {
                return false;
            } else {
                height as u8
            }
        }
    };
    match algo {
        CalcAlgorithm::Prime => PRIME_NUMBERS.binary_search(&val).is_ok(),
        CalcAlgorithm::M5 => val % 5 == 0,
        CalcAlgorithm::M4 => val % 4 == 0,
        CalcAlgorithm::M3 => val % 3 == 0,
    }
}

fn handle_normal_ability(
    sim: &mut Simulation,
    user_id: CombatantId,
    action: Action,
    ability: &Ability,
    action_target: ActionTarget,
) {
    match ability.aoe {
        AoE::None => {
            if let Some(target_id) = action.target.to_target_id(sim) {
                let user = sim.combatant(user_id);
                let target = sim.combatant(target_id);
                // TODO: Not a great place for this.. re: MP costs.
                if !target.jumping() && filter_target_level(user, ability, target) {
                    ability.implementation.perform(sim, user_id, target_id);
                } else {
                    // TODO: Log some sort of event for failing to perform an ability
                }
            } else {
                // TODO: Something about the ability missing.
            }
        }
        AoE::Diamond(size) => {
            for target_panel in action
                .target
                .to_location(sim)
                .expect("should only be none if math")
                .diamond(size)
            {
                perform_aoe_on_panel(sim, user_id, ability, target_panel)
            }
        }
        AoE::Line => {
            let user = sim.combatant(user_id);
            let target_location = action_target
                .to_location(sim)
                .expect("should only be none if math");
            let user_location = user.location;
            let facing = Facing::towards(user.location, target_location);
            for i in 1..=action.range {
                let target_panel = user_location + facing.offset() * i as i16;
                perform_aoe_on_panel(sim, user_id, ability, target_panel);
            }
        }
        AoE::TriLine => {
            let user = sim.combatant(user_id);
            let target_location = action_target
                .to_location(sim)
                .expect("should only be none if math");
            let facing = Facing::towards(user.location, target_location);
            let user_location = user.location;
            let left_facing = facing.rotate(3);
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
