use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::common::{mod_6_formula, ElementalDamageSpellImpl};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Event, Simulation, Source, HITS_ALLIES_ONLY,
    HITS_FOES_ONLY, NOT_ALIVE_OK, PETRIFY_OK, SILENCEABLE,
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
            ctr: Some(4),
            range: 4,
            evadable: false,
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
            ctr: Some(4),
            range: 4,
            evadable: false,
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
            ctr: Some(4),
            range: 4,
            evadable: false,
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
            ctr: Some(5),
            range: 4,
            evadable: false,
        },
    },
    // TODO: Golem: 0 range, 0 AoE, 4 CT, 40 MP. Hit: ((Faith/100) * (MA + 200))%. Effect: Set Golem on party equal to Caster HP, which takes all physical damage for party until destroyed.
    // Carbunkle: 4 range, 2 AoE, 7 CT, 30 MP. Hit: Faith(MA + 140)%. Effect: Cancel Death, Undead, Petrify, Blood Suck, Charm, Death Sentence; If successful Heal (25)%.
    Ability {
        name: "Carbunkle",
        flags: ALLY_OK | NOT_ALIVE_OK | PETRIFY_OK | SILENCEABLE | HITS_ALLIES_ONLY,
        mp_cost: 30,
        aoe: Some(2),
        implementation: &CarbunkleImpl {
            base_chance: 140,
            heal_percent: 0.25,
            conditions: &[
                Condition::Undead,
                Condition::Petrify,
                Condition::BloodSuck,
                Condition::Charm,
                Condition::DeathSentence,
            ],
            range: 4,
            ctr: 7,
        },
    },
    // Bahamut: 4 range, 3 AoE, 10 CT, 60 MP. Element: Dark. Effect: Damage Faith(MA * 46).
    Ability {
        name: "Bahamut",
        flags: FOE_OK | SILENCEABLE | HITS_FOES_ONLY,
        mp_cost: 60,
        aoe: Some(3),
        implementation: &ElementalDamageSpellImpl {
            element: Element::Dark,
            q: 46,
            ctr: Some(10),
            range: 4,
            evadable: false,
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
            ctr: Some(9),
            range: 4,
            evadable: false,
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
            ctr: Some(9),
            range: 4,
            evadable: false,
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
            ctr: Some(9),
            range: 4,
            evadable: false,
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
            ctr: Some(7),
            range: 4,
            evadable: false,
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
            ctr: Some(11),
            range: 4,
            evadable: false,
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
            ctr: Some(12),
            range: 4,
            evadable: false,
        },
    },
];

struct CarbunkleImpl {
    base_chance: i16,
    heal_percent: f32,
    conditions: &'static [Condition],
    range: i8,
    ctr: u8,
}

impl AbilityImpl for CarbunkleImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: self.range,
            ctr: Some(self.ctr),
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        let success_chance = mod_6_formula(user, target, Element::None, self.base_chance, true);
        if !(sim.roll_auto_succeed() < success_chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        if target.dead() {
            let heal_amount = (target.max_hp() as f32 * self.heal_percent) as i16;
            sim.change_target_hp(target_id, -heal_amount, Source::Ability);
        }
        for condition in self.conditions.iter() {
            sim.cancel_condition(target_id, *condition, Source::Ability);
        }
    }
}
