use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::attack::AttackImpl;

use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Simulation, Source, CASTER_IMMUNE, TARGET_NOT_SELF,
    TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const MOLBORO_ABILITIES: &[Ability] = &[
    // Tendrils: 1 range, 0 AoE. Effect: Normal Attack; Chance to Add Slow.
    Ability {
        name: "Tendrils",
        flags: FOE_OK | TARGET_NOT_SELF | TRIGGERS_HAMEDO,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &AttackImpl {
            condition: Some(Condition::Slow),
        },
    },
    // TODO: Lick: 1 range, 0 AoE. Effect: Add Reflect.
    // Goo: 2 range, 0 AoE. Hit: (MA + 60)%. Effect: Add Stop, Don't Act, Don't Move (All).
    Ability {
        name: "Goo",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &GooImpl {
            base_chance: 60,
            range: 2,
            conditions: &[Condition::Stop, Condition::DontAct, Condition::DontMove],
        },
    },
    // Bad Breath: 0 range, 2 AoE. Effect: Add Petrify, Darkness, Confusion, Silence, Oil, Frog, Poison, Sleep (Separate).
    Ability {
        name: "Bad Breath",
        flags: ALLY_OK | TARGET_SELF_ONLY | CASTER_IMMUNE,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(0)),
        implementation: &BadBreathImpl {
            conditions: &[
                Condition::Petrify,
                Condition::Darkness,
                Condition::Confusion,
                Condition::Silence,
                Condition::Oil,
                Condition::Frog,
                Condition::Poison,
                Condition::Sleep,
            ],
            range: 0,
        },
    },
    // TODO: Moldball Virus: 1 range, 0 AoE. Hit: (MA + 6)%. Effect: Transform Human into a Malboro.
];

struct GooImpl {
    base_chance: i16,
    range: u8,
    conditions: &'static [Condition],
}

impl AbilityImpl for GooImpl {
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
        // TODO: Not sure if Zodiac affects this.. need to ask Nacho. In base game it was MOD 0.
        let user = sim.combatant(user_id);
        let chance = (user.ma() + self.base_chance) as f32 / 100.0;
        if sim.roll_auto_succeed() < chance {
            for condition in self.conditions {
                sim.add_condition(target_id, *condition, Source::Ability);
            }
        }
        sim.try_countergrasp(user_id, target_id)
    }
}

pub(crate) struct BadBreathImpl {
    pub(crate) conditions: &'static [Condition],
    pub(crate) range: u8,
}

impl AbilityImpl for BadBreathImpl {
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
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        for condition in self.conditions {
            // TODO: Not sure what the proc rate is.
            if sim.roll_auto_succeed() < 0.25 {
                sim.add_condition(target_id, *condition, Source::Ability);
            }
        }
    }
}
