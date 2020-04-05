use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};

use crate::sim::{
    Combatant, CombatantId, Condition, Event, Simulation, Source, TARGET_NOT_SELF, TARGET_SELF_ONLY,
};

pub const WORK_ABILITIES: &[Ability] = &[
    // Destroy: 4 range, 0 AoE. Effect: Damage (PA * 10); DamageCaster ((PA * 10) / 4).
    Ability {
        name: "Destroy",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &WorkDamageImpl {
            pa_factor: 10,
            hurt_div: 4,
            range: 4,
        },
    },
    // Compress: 2 range, 0 AoE. Effect: Damage (PA * 15); DamageCaster ((PA * 15) / 3).
    Ability {
        name: "Compress",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &WorkDamageImpl {
            pa_factor: 15,
            hurt_div: 3,
            range: 2,
        },
    },
    // Dispose: 8 range, 0 AoE. Effect: Damage (PA * 5); DamageCaster ((PA * 5) / 5).
    Ability {
        name: "Dispose",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &WorkDamageImpl {
            pa_factor: 5,
            hurt_div: 5,
            range: 8,
        },
    },
    // Repair: 0 range, 0 AoE. Hit: (PA + 80)%. Effect: Add Oil; If successful Heal (40)%.
    Ability {
        name: "Repair",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: None,
        implementation: &RepairImpl { base_chance: 80 },
    },
];

struct WorkDamageImpl {
    range: i8,
    pa_factor: i16,
    hurt_div: i16,
}

impl AbilityImpl for WorkDamageImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: self.range,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let _target = sim.combatant(target_id);
        let pa = user.pa() as i16;
        let damage = pa * self.pa_factor;
        let self_damage = damage / self.hurt_div;
        sim.change_target_hp(target_id, damage, Source::Ability);
        sim.change_target_hp(user_id, self_damage, Source::Ability);
    }
}

struct RepairImpl {
    base_chance: i16,
}

impl AbilityImpl for RepairImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: 0,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);

        let chance = (user.pa() as f32 + self.base_chance as f32) / 100.0;
        if !(sim.roll_auto_succeed() < chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        let target = sim.combatant(target_id);
        if target.oil() {
            return;
        }
        let heal_amount = (target.max_hp() * 10) / 4;

        sim.add_condition(target_id, Condition::Oil, Source::Ability);
        sim.change_target_hp(target_id, -heal_amount, Source::Ability);
    }
}
