use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};

use crate::sim::common::mod_5_formula_xa;
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

// TODO: Line of sight.
pub const REAPER_ABILITIES: &[Ability] = &[
    // TODO: Knife Hand: 1 range, 0 AoE. Effect: Normal Attack; Chance to Add Undead.
    // Thunder Soul: 3 range, 0 AoE. Element: Lightning. Effect: Damage (MA * 3).
    Ability {
        name: "Thunder Soul",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &SoulAbility {
            ma_factor: 3,
            range: 3,
            element: Element::Lightning,
        },
    },
    // Aqua Soul: 3 range, 0 AoE. Element: Water. Effect: Damage (MA * 3).
    Ability {
        name: "Aqua Soul",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &SoulAbility {
            ma_factor: 3,
            range: 3,
            element: Element::Water,
        },
    },
    // Ice Soul: 3 range, 0 AoE. Element: Ice. Effect: Damage (MA * 3).
    Ability {
        name: "Ice Soul",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &SoulAbility {
            ma_factor: 3,
            range: 3,
            element: Element::Ice,
        },
    },
    // Wind Soul: 3 range, 0 AoE. Element: Wind. Effect: Damage (MA * 4).
    Ability {
        name: "Wind Soul",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &SoulAbility {
            ma_factor: 4,
            range: 3,
            element: Element::Wind,
        },
    },
];

struct SoulAbility {
    ma_factor: i16,
    range: u8,
    element: Element,
}

impl AbilityImpl for SoulAbility {
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
        let xa = mod_5_formula_xa(user.ma() as i16, user, target, self.element, false);
        sim.change_target_hp(target_id, xa * self.ma_factor, Source::Ability);
    }
}
