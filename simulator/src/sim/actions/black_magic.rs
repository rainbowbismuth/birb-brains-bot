use crate::sim::actions::{Ability, ALLY_OK, FOE_OK};
use crate::sim::common::{AddConditionSpellImpl, ElementalDamageSpellImpl};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, NOT_ALIVE_OK, SILENCEABLE,
};

pub const BLACK_MAGIC_ABILITIES: &[Ability] = &[
    // Fire: 5 range, 1 AoE, 3 CT, 6 MP. Element: Fire. Effect: Damage Faith(MA * 16).
    // Fire 2: 5 range, 1 AoE, 4 CT, 12 MP. Element: Fire. Effect: Damage Faith(MA * 20).
    // Fire 3: 5 range, 1 AoE, 6 CT, 24 MP. Element: Fire. Effect: Damage Faith(MA * 28).
    // Fire 4: 5 range, 2 AoE, 8 CT, 48 MP. Element: Fire. Effect: Damage Faith(MA * 36).
    Ability {
        name: "Fire",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 6,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Fire,
            q: 16,
            ctr: 3,
            range: 5,
        },
    },
    Ability {
        name: "Fire 2",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 12,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Fire,
            q: 20,
            ctr: 4,
            range: 5,
        },
    },
    Ability {
        name: "Fire 3",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 24,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Fire,
            q: 28,
            ctr: 6,
            range: 5,
        },
    },
    Ability {
        name: "Fire 4",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 48,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Fire,
            q: 36,
            ctr: 8,
            range: 5,
        },
    },
    // Bolt: 5 range, 1 AoE, 3 CT, 6 MP. Element: Lightning. Effect: Damage Faith(MA * 16).
    // Bolt 2: 5 range, 1 AoE, 4 CT, 12 MP. Element: Lightning. Effect: Damage Faith(MA * 20).
    // Bolt 3: 5 range, 1 AoE, 6 CT, 24 MP. Element: Lightning. Effect: Damage Faith(MA * 28).
    // Bolt 4: 5 range, 2 AoE, 8 CT, 48 MP. Element: Lightning. Effect: Damage Faith(MA * 36).
    Ability {
        name: "Bolt",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 6,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Lightning,
            q: 16,
            ctr: 3,
            range: 5,
        },
    },
    Ability {
        name: "Bolt 2",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 12,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Lightning,
            q: 20,
            ctr: 4,
            range: 5,
        },
    },
    Ability {
        name: "Bolt 3",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 24,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Lightning,
            q: 28,
            ctr: 6,
            range: 5,
        },
    },
    Ability {
        name: "Bolt 4",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 48,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Lightning,
            q: 36,
            ctr: 8,
            range: 5,
        },
    },
    // Ice: 5 range, 1 AoE, 3 CT, 6 MP. Element: Ice. Effect: Damage Faith(MA * 16).
    // Ice 2: 5 range, 1 AoE, 4 CT, 12 MP. Element: Ice. Effect: Damage Faith(MA * 20).
    // Ice 3: 5 range, 1 AoE, 6 CT, 24 MP. Element: Ice. Effect: Damage Faith(MA * 28).
    // Ice 4: 5 range, 2 AoE, 8 CT, 48 MP. Element: Ice. Effect: Damage Faith(MA * 36).
    Ability {
        name: "Ice",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 6,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Ice,
            q: 16,
            ctr: 3,
            range: 5,
        },
    },
    Ability {
        name: "Ice 2",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 12,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Ice,
            q: 20,
            ctr: 4,
            range: 5,
        },
    },
    Ability {
        name: "Ice 3",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 24,
        aoe: Some(1),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Ice,
            q: 28,
            ctr: 6,
            range: 5,
        },
    },
    Ability {
        name: "Ice 4",
        flags: ALLY_OK | FOE_OK | SILENCEABLE,
        mp_cost: 48,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Ice,
            q: 36,
            ctr: 8,
            range: 5,
        },
    },
    // Poison: 5 range, 1 AoE, 3 CT, 6 MP. Hit: Faith(MA + 190)%. Effect: Add Poison.
    Ability {
        name: "Poison",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 6,
        aoe: Some(1),
        implementation: &AddConditionSpellImpl {
            condition: Condition::Poison,
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 190,
            ctr: 3,
            range: 5,
        },
    },
    // Frog: 4 range, 0 AoE, 5 CT, 12 MP. Hit: Faith(MA + 120)%. Effect: Add Frog.
    Ability {
        name: "Frog",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 12,
        aoe: None,
        implementation: &AddConditionSpellImpl {
            condition: Condition::Frog,
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 120,
            ctr: 5,
            range: 4,
        },
    },
    // Death: 5 range, 0 AoE, 10 CT, 24 MP. Hit: Faith(MA + 110)%. Effect: Damage (100)%
    // TODO: Heals undead!!
    Ability {
        name: "Death",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 24,
        aoe: None,
        implementation: &AddConditionSpellImpl {
            condition: Condition::Death,
            can_be_evaded: true,
            ignore_magic_def: false,
            base_chance: 110,
            ctr: 10,
            range: 5,
        },
    },
    // Flare: 6 range, 0 AoE, 7 CT, 60 MP. Effect: Damage Faith(MA * 49).
    Ability {
        name: "Flare",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 60,
        aoe: None,
        implementation: &ElementalDamageSpellImpl {
            element: Element::None,
            q: 49,
            ctr: 7,
            range: 6,
        },
    },
];
