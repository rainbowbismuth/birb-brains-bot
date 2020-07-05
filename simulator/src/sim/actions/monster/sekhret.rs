use crate::sim::actions::common::{AddConditionSpellImpl, EmpowerImpl};
use crate::sim::actions::monster::{ChocoMeteorImpl, ElementalBreathImpl};
use crate::sim::actions::punch_art::{DamagePunchArt, Pummel};
use crate::sim::actions::talk_skill::ConditionTalkSkillImpl;
use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::attack::AttackImpl;
use crate::sim::common::{do_hp_heal, mod_2_formula_xa, mod_5_formula_xa};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, CASTER_IMMUNE,
    STATS_ABILITY, TARGET_NOT_SELF, TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const SEKHRET_ABILITIES: &[Ability] = &[
    // Storm Around: 0 range, 1 AoE. Element: Lightning. Effect: Damage ((PA + 1) / 2 * PA).
    Ability {
        name: "Storm Around",
        flags: ALLY_OK | TARGET_SELF_ONLY | CASTER_IMMUNE,
        mp_cost: 0,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &DamagePunchArt {
            element: Element::Lightning,
            pa_plus: 1,
            range: 0,
        },
    },
    Ability {
        name: "Wave Around",
        flags: ALLY_OK | TARGET_SELF_ONLY | CASTER_IMMUNE,
        mp_cost: 0,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &DamagePunchArt {
            element: Element::None,
            pa_plus: 1,
            range: 0,
        },
    },
    // Mimic Titan: 0 range, 2 AoE. Element: Earth. Effect: Damage (MA * 3).
    Ability {
        name: "Mimic Titan",
        flags: ALLY_OK | TARGET_SELF_ONLY | CASTER_IMMUNE,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(1)),
        implementation: &ChocoMeteorImpl {
            ma_factor: 3,
            range: 0,
        },
    },
    // Gather Power: 0 range, 0 AoE. Effect: +2 PA.
    Ability {
        name: "Gather Power",
        flags: ALLY_OK | TARGET_SELF_ONLY | STATS_ABILITY,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &EmpowerImpl {
            range: 0,
            ctr: None,
            brave_mod: 0,
            pa_buff: 2,
            ma_buff: 0,
            speed_buff: 0,
        },
    },
    // Blow Fire: 3 range, 3 AoE (line). Element: Fire. Effect: Damage (MA * 4).
    Ability {
        name: "Blow Fire",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::Line(Some(2)),
        implementation: &ElementalBreathImpl {
            element: Element::Fire,
            ma_factor: 4,
            range: 3,
        },
    },
];
