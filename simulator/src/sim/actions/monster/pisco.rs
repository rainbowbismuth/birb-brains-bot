use crate::sim::actions::common::AddConditionSpellImpl;
use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK, SILENCEABLE};
use crate::sim::common::{mod_5_formula_xa, mod_6_formula, ElementalDamageSpellImpl, EmpowerImpl};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Event, Simulation, Source, CASTER_IMMUNE,
    STATS_ABILITY, TARGET_NOT_SELF, TARGET_SELF_ONLY,
};

pub const PISCO_ABILITIES: &[Ability] = &[
    // Tentacle: 1 range, 0 AoE. Effect: Normal Attack.
    // Black Ink: 2 range, 2 AoE (line). Hit: (MA + 50)%. Effect: Add Darkness, Oil (All).
    // Odd Soundwave: 0 range, 2 AoE. Effect: Cancel Charging, Performing, Float, Reraise, Transparent, Regen, Protect, Shell, Haste, Faith, Reflect.
    // Mind Blast: 3 range, 1 AoE. Hit: (MA + 40)%. Effect: Add Confusion, Berserk (Random).
    Ability {
        name: "Mind Blast",
        flags: FOE_OK | CASTER_IMMUNE,
        mp_cost: 0,
        aoe: AoE::Diamond(1, Some(1)),
        implementation: &MindBlastImpl {
            conditions: &[Condition::Confusion, Condition::Berserk],
            base_chance: 40,
            range: 3,
        },
    },
    // Level Blast: 3 range, 0 AoE. Effect: +1 Brave, +1 PA, +1 MA, +1 Speed.
    Ability {
        name: "Level Blast",
        flags: ALLY_OK | STATS_ABILITY,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &EmpowerImpl {
            range: 3,
            ctr: None,
            brave_mod: 1,
            pa_buff: 1,
            ma_buff: 1,
            speed_buff: 1,
        },
    },
];

struct MindBlastImpl {
    conditions: &'static [Condition],
    base_chance: i16,
    range: u8,
}

impl AbilityImpl for MindBlastImpl {
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
        let target = sim.combatant(target_id);
        if sim.do_magical_evade(user, target, Source::Ability) {
            return;
        }
        let success_chance = mod_6_formula(user, target, Element::None, self.base_chance, false);
        if !(sim.roll_auto_succeed() < success_chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }
        let random = sim.roll_inclusive(0, (self.conditions.len() - 1) as i16);
        let cond = self.conditions[random as usize];
        sim.add_condition(target_id, cond, Source::Ability);
    }
}
