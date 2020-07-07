use crate::sim::actions::punch_art::DamagePunchArt;
use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};

use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, CASTER_IMMUNE,
    TARGET_NOT_SELF, TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const GOBLIN_ABILITIES: &[Ability] = &[
    // Goblin Punch: 1 range, 0 AoE. Hit: (MA + 43)%. Effect: Damage (CasterMaxHP - CasterCurrentHP).
    Ability {
        name: "Goblin Punch",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &GoblinPunchImpl { base_chance: 43 },
    },
    // Turn Punch: 0 range, 1 AoE. Effect: Damage ((PA + 4) / 2 * PA).
    Ability {
        name: "Turn Punch",
        flags: ALLY_OK | TARGET_SELF_ONLY | CASTER_IMMUNE,
        mp_cost: 0,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &DamagePunchArt {
            element: Element::None,
            pa_plus: 4,
            range: 0,
        },
    },
    // Eye Gouge: 1 range, 0 AoE. Hit: (MA + 55)%. Effect: Add Darkness, Confusion (All).
    Ability {
        name: "Eye Gouge",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &EyeGougeImpl { base_chance: 55 },
    },
    // Mutilate: 1 range, 0 AoE. Hit: (MA + 29)%. Effect: AbsorbHP (66)%.
    Ability {
        name: "Mutilate",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &EyeGougeImpl { base_chance: 29 },
    },
];

struct GoblinPunchImpl {
    base_chance: i16,
}

impl AbilityImpl for GoblinPunchImpl {
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
            let hp_diff = user.max_hp() - user.hp();
            sim.change_target_hp(target_id, hp_diff, Source::Ability);
        }

        sim.try_countergrasp(user_id, target_id)
    }
}

struct EyeGougeImpl {
    base_chance: i16,
}

impl AbilityImpl for EyeGougeImpl {
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
            sim.add_condition(target_id, Condition::Darkness, Source::Ability);
            sim.add_condition(target_id, Condition::Confusion, Source::Ability);
        } else {
            sim.try_countergrasp(user_id, target_id)
        }
    }
}

struct MutilateImpl {
    base_chance: i16,
}

impl AbilityImpl for MutilateImpl {
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
            let amount = (target.hp() / 3) * 2;
            sim.change_target_hp(target_id, amount, Source::Ability);
            sim.change_target_hp(user_id, -amount, Source::Ability);
        }
    }
}
