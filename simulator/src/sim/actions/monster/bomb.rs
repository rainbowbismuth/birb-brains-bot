use crate::sim::actions::{Ability, AbilityImpl, Action, FOE_OK};

use crate::sim::actions::attack::AttackImpl;
use crate::sim::actions::common::ElementalDamageSpellImpl;
use crate::sim::actions::monster::ChocoMeteorImpl;
use crate::sim::common::mod_5_formula_xa;
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, ALLY_OK, TARGET_NOT_SELF,
    TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const BOMB_ABILITIES: &[Ability] = &[
    // Bite: 1 range, 0 AoE. Effect: Normal Attack; Chance to Add Oil.
    Ability {
        name: "Bite",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF | TRIGGERS_HAMEDO,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &AttackImpl {
            condition: Some(Condition::Oil),
        },
    },
    // Small Bomb: 4 range, 0 AoE. Element: Dark. Effect: Damage Faith(MA * 9).
    Ability {
        name: "Small Bomb",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ElementalDamageSpellImpl {
            element: Element::Dark,
            q: 9,
            range: 4,
            ctr: None,
            evadable: true,
        },
    },
    // Self Destruct: 0 range, 2 AoE. Effect: Adds Death to Caster; Damage (CasterMaxHP - CasterCurrentHP); Adds Oil.
    Ability {
        name: "Self Destruct",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::None, // Handled inside ability
        implementation: &SelfDestructImpl,
    },
    // Flame Attack: 3 range, 0 AoE. Element: Fire. Effect: Damage (MA * 4).
    Ability {
        name: "Flame Attack",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChocoMeteorImpl {
            ma_factor: 4,
            range: 3,
            element: Element::Fire,
        },
    },
    // Spark: 0 range, 2 AoE. Element: Fire. Effect: Damage (MA * 3).
    Ability {
        name: "Spark",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(1)),
        implementation: &ChocoMeteorImpl {
            ma_factor: 3,
            range: 0,
            element: Element::Fire,
        },
    },
];

struct SelfDestructImpl;

impl AbilityImpl for SelfDestructImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 0, None, target.id()))
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, _target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let hp_diff = user.max_hp() - user.hp();
        for target_panel in user.panel.diamond(2) {
            if let Some(target_id) = sim.combatant_on_panel(target_panel) {
                if user_id == target_id {
                    continue;
                }
                sim.add_condition(target_id, Condition::Oil, Source::Ability);
                sim.change_target_hp(target_id, hp_diff, Source::Ability);
            }
        }
        sim.add_condition(user_id, Condition::Death, Source::Ability);
    }
}
