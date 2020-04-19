use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};

use crate::sim::common::{mod_2_formula_xa, mod_5_formula_xa};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Event, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY,
};

pub const DRAGON_ABILITIES: &[Ability] = &[
    // Tail Swing: 1 range, 0 AoE. Effect: Damage (Random(1-15) * PA); Chance to Knockback.
    Ability {
        name: "Tail Swing",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &TailSwingImpl {
            min_factor: 1,
            max_factor: 15,
        },
    },
    // Ice Breath: 2 range, 2 AoE (line). Element: Ice. Effect: Damage (MA * 5).
    Ability {
        name: "Ice Breath",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::Line,
        implementation: &ElementalBreathImpl {
            element: Element::Ice,
            ma_factor: 5,
            range: 2,
        },
    },
    // Fire Breath: 2 range, 2 AoE (line). Element: Fire. Effect: Damage (MA * 5).
    Ability {
        name: "Fire Breath",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::Line,
        implementation: &ElementalBreathImpl {
            element: Element::Fire,
            ma_factor: 5,
            range: 2,
        },
    },
    // Thunder Breath: 2 range, 2 AoE (line). Element: Lightning. Effect: Damage (MA * 5).
    Ability {
        name: "Thunder Breath",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::Line,
        implementation: &ElementalBreathImpl {
            element: Element::Lightning,
            ma_factor: 5,
            range: 2,
        },
    },
];

struct ElementalBreathImpl {
    element: Element,
    ma_factor: i16,
    range: i8,
}

impl AbilityImpl for ElementalBreathImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 1, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let xa = mod_5_formula_xa(
            user.ma(),
            user,
            sim.combatant(target_id),
            self.element,
            false,
        );
        let damage = xa * self.ma_factor;
        sim.change_target_hp(target_id, damage, Source::Ability);
    }
}

struct TailSwingImpl {
    min_factor: i16,
    max_factor: i16,
}

impl AbilityImpl for TailSwingImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 1, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let xa = mod_2_formula_xa(
            sim,
            user.pa(),
            user,
            sim.combatant(target_id),
            Element::None,
            false,
            false,
            false,
        );
        let damage = sim.roll_inclusive(self.min_factor, self.max_factor) * xa;
        sim.change_target_hp(target_id, damage, Source::Ability);
        if sim.roll_auto_fail() < 0.5 {
            sim.do_knockback(user_id, target_id);
        }
    }
}
