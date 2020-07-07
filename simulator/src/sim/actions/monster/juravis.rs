use crate::sim::actions::{Ability, AbilityImpl, Action, FOE_OK};

use crate::sim::common::mod_5_formula_xa;
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, TARGET_NOT_SELF,
};

pub const JURAVIS_ABILITIES: &[Ability] = &[
    // Beak: 1 range, 0 AoE. Hit: (MA + 37)%. Effect: Add Petrify.
    Ability {
        name: "Beak",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &BeakImpl { base_chance: 37 },
    },
    // Shine Lover: 3 range, 0 AoE. Hit: (PA + 65)%. Effect: DamageMP (65)%
    Ability {
        name: "Shine Lover",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ShineLoverImpl { base_chance: 65 },
    },
    // Feather Bomb: 5 range, 0 AoE. Element: Wind. Effect: Damage (MA * 2).
    Ability {
        name: "Feather Bomb",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &FeatherBombImpl {
            range: 5,
            ma_factor: 2,
        },
    },
    // Peck: 3 range, 0 AoE. Hit: (MA + 55)%. Effect: -2 PA.
    Ability {
        name: "Peck",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PeckImpl { base_chance: 55 },
    },
];

struct PeckImpl {
    base_chance: i16,
}

impl AbilityImpl for PeckImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 1, None, target.id()))
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let mut chance = (user.ma() + self.base_chance) as f32 / 100.0;
        chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < chance {
            sim.change_unit_pa(target_id, -2, Source::Ability);
        } else {
            sim.try_countergrasp(user_id, target_id);
        }
    }
}

struct FeatherBombImpl {
    range: u8,
    ma_factor: i16,
}

impl AbilityImpl for FeatherBombImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, None, target.id()))
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let xa = mod_5_formula_xa(user.ma(), user, target, Element::Wind, false);
        let amount = xa * self.ma_factor;
        sim.change_target_hp(target_id, amount, Source::Ability);
    }
}

struct BeakImpl {
    base_chance: i16,
}

impl AbilityImpl for BeakImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 1, None, target.id()))
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let mut chance = (user.ma() + self.base_chance) as f32 / 100.0;
        chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < chance {
            sim.add_condition(target_id, Condition::Petrify, Source::Ability);
        } else {
            sim.try_countergrasp(user_id, target_id);
        }
    }
}

struct ShineLoverImpl {
    base_chance: i16,
}

impl AbilityImpl for ShineLoverImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 1, None, target.id()))
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        if !sim.do_physical_evade(user, target, None, Source::Ability) {
            let mut chance = (user.pa() + self.base_chance) as f32 / 100.0;
            chance *= user.zodiac_compatibility(target);

            if sim.roll_auto_succeed() < chance {
                let amount = (target.mp() / 3) * 2;
                sim.change_target_mp(target_id, amount, Source::Ability);
            }
        }
        sim.try_countergrasp(user_id, target_id);
    }
}
