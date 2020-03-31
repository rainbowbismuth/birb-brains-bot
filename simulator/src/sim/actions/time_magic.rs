use crate::sim::actions::{Ability, ALLY_OK, FOE_OK};
use crate::sim::common::{AddConditionSpellImpl, ElementalDamageSpellImpl};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, NOT_ALIVE_OK, SILENCEABLE,
};

pub const TIME_MAGIC_ABILITIES: &[Ability] = &[
    // Haste: 4 range, 1 AoE, 2 CT, 8 MP. Hit: Faith(MA + 180)%. Effect: Add Haste.
    Ability {
        name: "Haste",
        flags: ALLY_OK | SILENCEABLE,
        mp_cost: 8,
        aoe: Some(1),
        implementation: &AddConditionSpellImpl {
            condition: Condition::Haste,
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 180,
            range: 4,
            ctr: 2,
        },
    },
    // Haste 2: 4 range, 1 AoE, 5 CT, 20 MP. Hit: Faith(MA + 240)%. Effect: Add Haste.
    Ability {
        name: "Haste 2",
        flags: ALLY_OK | SILENCEABLE,
        mp_cost: 20,
        aoe: Some(1),
        implementation: &AddConditionSpellImpl {
            condition: Condition::Haste,
            can_be_evaded: false,
            ignore_magic_def: true,
            base_chance: 240,
            range: 4,
            ctr: 2,
        },
    },
    // Slow: 4 range, 1 AoE, 2 CT, 8 MP. Hit: Faith(MA + 180)%. Effect: Add Slow.
    Ability {
        name: "Slow",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 8,
        aoe: Some(1),
        implementation: &AddConditionSpellImpl {
            condition: Condition::Slow,
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 180,
            range: 4,
            ctr: 2,
        },
    },
    // Slow 2: 4 range, 1 AoE, 5 CT, 20 MP. Hit: Faith(MA + 240)%. Effect: Add Slow.
    Ability {
        name: "Slow 2",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 20,
        aoe: Some(1),
        implementation: &AddConditionSpellImpl {
            condition: Condition::Slow,
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 240,
            range: 4,
            ctr: 5,
        },
    },
    // Stop: 4 range, 1 AoE, 7 CT, 14 MP. Hit: Faith(MA + 130)%. Effect: Add Stop.
    Ability {
        name: "Stop",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 14,
        aoe: Some(1),
        implementation: &AddConditionSpellImpl {
            condition: Condition::Stop,
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 130,
            range: 4,
            ctr: 7,
        },
    },
    // Immobilize: 5 range, 1 AoE, 3 CT, 10 MP. Hit: Faith(MA + 190)%. Effect: Add Don't Move.
    Ability {
        name: "Immobilize",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 10,
        aoe: Some(1),
        implementation: &AddConditionSpellImpl {
            condition: Condition::DontMove,
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 190,
            range: 5,
            ctr: 3,
        },
    },
    // TODO: Float: 5 range, 1 AoE, 2 CT, 8 MP. Hit: Faith(MA + 170)%. Effect: Add Float.
    // TODO: Reflect: 5 range, 0 AoE, 2 CT, 12 MP. Hit: Faith(MA + 180)%. Effect: Add Reflect.
    // TODO: Quick: 5 range, 0 AoE, 4 CT, 24 MP. Hit: Faith(MA + 140)%. Effect: Set CT to Max.
    // TODO: Demi: 5 range, 1 AoE, 3 CT, 20 MP. Hit: Faith(MA + 205)%. Effect: Damage (25)%.
    // TODO: Demi 2: 5 range, 1 AoE, 6 CT, 40 MP. Hit: Faith(MA + 165)%. Effect: Damage (50)%.
    // Meteor: 5 range, 3 AoE, 13 CT, 70 MP. Effect: Damage Faith(MA * 60).
    Ability {
        name: "Meteor",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 70,
        aoe: Some(3),
        implementation: &ElementalDamageSpellImpl {
            // TODO: There should be a 'neutral' element.
            element: Element::None,
            q: 60,
            ctr: 13,
            range: 5,
        },
    },
];
