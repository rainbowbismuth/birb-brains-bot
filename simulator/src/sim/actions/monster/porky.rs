use crate::sim::actions::steal::StealHeartImpl;

use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};

use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, CAN_BE_REFLECTED,
    CASTER_IMMUNE, NOT_ALIVE_OK, SILENCEABLE, TARGET_NOT_SELF, TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const PORKY_ABILITIES: &[Ability] = &[
    // Straight Dash: 1 range, 0 AoE. Effect: Normal Attack.
    // Snort: 2 range, 0 AoE. Hit: Non-Matching Sex; (MA + 40)%. Effect: Add Charm.
    Ability {
        name: "Snort",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &StealHeartImpl {
            base_chance: 40,
            range: 2,
        },
    },
    // Oink: 2 range, 0 AoE. Hit: (PA + 71)%. Effect: Cancel Death; If successful Heal (75)%.
    Ability {
        name: "Oink",
        flags: ALLY_OK | NOT_ALIVE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &OinkImpl,
    },
    // Toot: 2 range, 0 AoE. Effect: Add Confusion, Sleep (Random).
    Ability {
        name: "Toot",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &TootImpl {
            conditions: &[Condition::Confusion, Condition::Sleep],
        },
    },
    // Bequeath Bacon: 2 range, 0 AoE. Effect: Heal (CasterMaxHP * 2 / 5); DamageCaster (CasterMaxHP / 5).
    Ability {
        name: "Bequeath Bacon",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &Energize { range: 2 },
    },
];

pub struct Energize {
    pub range: u8,
}

impl AbilityImpl for Energize {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let max_hp = user.max_hp();
        let heal_amount = (max_hp * 2) / 5;
        let dmg_amount = max_hp / 5;
        sim.change_target_hp(target_id, -heal_amount, Source::Ability);
        sim.change_target_hp(user_id, dmg_amount, Source::Ability);
    }
}

struct OinkImpl;

impl AbilityImpl for OinkImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if !target.dead() {
            return;
        }
        actions.push(Action::new(ability, 2, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        let max_hp = target.max_hp();
        let compat = user.zodiac_compatibility(target);
        let chance = ((user.pa() + 71) as f32 * compat) / 100.0;
        if sim.roll_auto_succeed() < chance {
            sim.change_target_hp(target_id, -((max_hp / 4) * 3), Source::Ability);
        }
    }
}

struct TootImpl {
    conditions: &'static [Condition],
}

impl AbilityImpl for TootImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 2, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        let idx = sim.roll_inclusive(0, (self.conditions.len() - 1) as i16);
        sim.add_condition(target_id, self.conditions[idx as usize], Source::Ability);
    }
}
