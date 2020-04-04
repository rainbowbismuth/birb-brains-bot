use crate::sim::actions::{Ability, ALLY_OK, FOE_OK};
use crate::sim::common::{CureSpellImpl, ElementalDamageSpellImpl};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, NOT_ALIVE_OK, SILENCEABLE,
    TARGET_SELF_ONLY,
};

pub const CHOCOBO_ABILITIES: &[Ability] = &[
    // TODO: Choco Attack: 1 range, 0 AoE. Effect: Normal Attack.
    // TODO: Choco Ball: 4 range, 0 AoE. Element: Water. Effect: Damage (PA / 2 * PA).
    // Choco Meteor: 5 range, 0 AoE. Effect: Damage (MA * 4).
    Ability {
        name: "Choco Meteor",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &ElementalDamageSpellImpl {
            q: 4,
            element: Element::None,
            ctr: None,
            range: 4,
            evadable: false,
        },
    },
    // TODO: Choco Esuna: 0 range, 1 AoE. Effect: Cancel Petrify, Darkness, Silence, Poison, Stop, Don't Move, Don't Act.
    // Choco Cure: 0 range, 1 AoE. Effect: Heal (MA * 3).
    Ability {
        name: "Choco Cure",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: Some(1),
        implementation: &CureSpellImpl {
            q: 3,
            ctr: None,
            range: 0,
        },
    },
];
