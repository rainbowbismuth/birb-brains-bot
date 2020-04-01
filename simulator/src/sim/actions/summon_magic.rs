use crate::sim::actions::{Ability, ALLY_OK, FOE_OK};
use crate::sim::common::{AddConditionSpellImpl, ElementalDamageSpellImpl};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, HITS_FOES_ONLY, NOT_ALIVE_OK,
    SILENCEABLE,
};

pub const SUMMON_MAGIC_ABILITES: &[Ability] = &[
    // TODO: Moogle: 4 range, 2 AoE, 3 CT, 12 MP. Hit: Faith(MA + 145)%. Effect: Cancel Petrify, Darkness, Confusion, Silence, Blood Suck, Berserk, Frog, Poison, Sleep, Don't Move, Don't Act; If successful, Heal (10)%
    // Shiva: 4 range, 2 AoE, 4 CT, 24 MP. Element: Ice. Effect: Damage Faith(MA * 24).
    Ability {
        name: "Shiva",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 24,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Ice,
            q: 24,
            ctr: 4,
            range: 4,
        },
    },
    // Ramuh: 4 range, 2 AoE, 4 CT, 24 MP. Element: Lightning. Effect: Damage Faith(MA * 24).
    Ability {
        name: "Ramuh",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 24,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Lightning,
            q: 24,
            ctr: 4,
            range: 4,
        },
    },
    // Ifrit: 4 range, 2 AoE, 4 CT, 24 MP. Element: Fire. Effect: Damage Faith(MA * 24).
    Ability {
        name: "Ifrit",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 24,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Fire,
            q: 24,
            ctr: 4,
            range: 4,
        },
    },
    // Titan: 4 range, 2 AoE, 5 CT, 30 MP. Element: Earth. Effect: Damage Faith(MA * 28).
    Ability {
        name: "Titan",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 30,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Earth,
            q: 28,
            ctr: 5,
            range: 4,
        },
    },
    // TODO: Golem: 0 range, 0 AoE, 4 CT, 40 MP. Hit: ((Faith/100) * (MA + 200))%. Effect: Set Golem on party equal to Caster HP, which takes all physical damage for party until destroyed.
    // TODO: Carbunkle: 4 range, 2 AoE, 7 CT, 30 MP. Hit: Faith(MA + 140)%. Effect: Cancel Death, Undead, Petrify, Blood Suck, Charm, Death Sentence; If successful Heal (25)%.
    // Bahamut: 4 range, 3 AoE, 10 CT, 60 MP. Element: Dark. Effect: Damage Faith(MA * 46).
    Ability {
        name: "Bahamut",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 60,
        aoe: Some(3),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Dark,
            q: 46,
            ctr: 10,
            range: 4,
        },
    },
    // Odin: 4 range, 3 AoE, 9 CT, 50 MP. Element: Holy. Effect: Damage Faith(MA * 40).
    Ability {
        name: "Odin",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 50,
        aoe: Some(3),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Holy,
            q: 40,
            ctr: 9,
            range: 4,
        },
    },
    // Leviathan: 4 range, 3 AoE, 9 CT, 48 MP. Element: Water. Effect: Damage Faith(MA * 38).
    Ability {
        name: "Leviathan",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 48,
        aoe: Some(3),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Water,
            q: 38,
            ctr: 9,
            range: 4,
        },
    },
    // Salamander: 4 range, 2 AoE, 9 CT, 44 MP. Element: Fire. Effect: Damage Faith(MA * 36); Chance to add Oil.
    // TODO: Chance to add Oil.
    Ability {
        name: "Salamander",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 44,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Fire,
            q: 36,
            ctr: 9,
            range: 4,
        },
    },
    // Silf: 4 range, 2 AoE, 7 CT, 36 MP. Element: Wind. Effect: Damage Faith(MA * 30); Chance to add Silence.
    // TODO: Chance to add Silence.
    Ability {
        name: "Silf",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 36,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Wind,
            q: 30,
            ctr: 7,
            range: 4,
        },
    },
    // TODO: Fairy: 4 range, 2 AoE, 6 CT, 28 MP. Effect: Heal Faith(MA * 24).
    // TODO: Lich: 4 range, 2 AoE, 9 CT, 40 MP. Element: Dark. Hit: Faith(MA + 160)%. Effect: Damage (60)%.
    // Cyclops: 4 range, 2 AoE, 11 CT, 62 MP. Effect: Damage Faith(MA * 50).
    Ability {
        name: "Cyclops",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 62,
        aoe: Some(2),
        implementation: &ElementalDamageSpellImpl {
            element: Element::None,
            q: 50,
            ctr: 11,
            range: 4,
        },
    },
    // Zodiac: 4 range, 3 AoE, 12 CT, 99 MP. Effect: Damage Faith(MA * 90).
    Ability {
        name: "Zodiac",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 99,
        aoe: Some(3),
        implementation: &ElementalDamageSpellImpl {
            element: Element::None,
            q: 90,
            ctr: 12,
            range: 4,
        },
    },
];
