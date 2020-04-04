use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK, TARGET_NOT_SELF};
use crate::sim::common::{
    mod_2_formula_xa, mod_3_formula_xa, AddConditionSpellImpl, ElementalDamageSpellImpl,
};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, HITS_ALLIES_ONLY,
    HITS_FOES_ONLY, NOT_ALIVE_OK, SILENCEABLE, TARGET_SELF_ONLY,
};

pub const PUNCH_ART_ABILITIES: &[Ability] = &[
    // Spin Fist: 0 range, 1 AoE. Effect: Damage ((PA + 1) / 2 * PA).
    Ability {
        name: "Spin Fist",
        // TODO: Pretty sure this could hit allies, need to add 'DOESNT HIT SELF'
        flags: HITS_FOES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: Some(1),
        implementation: &DamagePunchArt {
            element: Element::None,
            pa_plus: 1,
            range: 0,
        },
    },
    // TODO: Pummel: 1 range, 0 AoE. Effect: Damage (Random(1-9) * PA * 3 / 2).
    // TODO: Wave Fist: 3 range, 0 AoE. Element: Wind. Effect: Damage ((PA + 2) / 2 * PA).
    Ability {
        name: "Wave Fist",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &DamagePunchArt {
            element: Element::Wind,
            pa_plus: 2,
            range: 3,
        },
    },
    // Earth Slash: 8 range, 8 AoE (line). Element: Earth. Effect: Damage (PA / 2 * PA).
    // Secret Fist: 1 range, 0 AoE. Hit: (MA + 50)%. Effect: Add Death Sentence.
    Ability {
        name: "Secret Fist",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &SecretFistImpl { base_chance: 50 },
    },
    // Purification: 0 range, 1 AoE. Hit: (PA + 80)%. Effect: Cancel Petrify, Darkness, Confusion, Silence, Blood Suck, Berserk, Frog, Poison, Sleep, Don't Move, Don't Act.
    Ability {
        name: "Purification",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: Some(1),
        implementation: &PurificationImpl {
            base_chance: 80,
            cancels: &[
                Condition::Petrify,
                Condition::Darkness,
                Condition::Confusion,
                Condition::Silence,
                Condition::BloodSuck,
                Condition::Berserk,
                Condition::Frog,
                Condition::Poison,
                Condition::Sleep,
                Condition::DontMove,
                Condition::DontAct,
            ],
        },
    },
    // Chakra: 0 range, 1 AoE. Effect: Heal (PA * 5); HealMP ((PA * 5) / 2).
    Ability {
        name: "Chakra",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: Some(1),
        implementation: &ChakraImpl {
            hp_multiplier: 5,
            mp_multiplier: 5,
        },
    },
    // Revive: 1 range, 0 AoE. Effect: Hit: (PA + 70)%. Effect: Cancel Death; If successful Heal (25)%.
    Ability {
        name: "Revive",
        flags: ALLY_OK | NOT_ALIVE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &ReviveImpl {
            base_chance: 70,
            heal_amount: 0.25,
        },
    },
];

struct DamagePunchArt {
    element: Element,
    pa_plus: i16,
    range: i8,
}

impl AbilityImpl for DamagePunchArt {
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
            range: 1,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        if sim.do_physical_evade(user, target, Source::Ability) {
            return;
        }

        let xa = mod_2_formula_xa(
            sim,
            user.pa() as i16,
            user,
            target,
            self.element,
            false,
            true,
            false,
        );

        let mut damage = ((xa + self.pa_plus) / 2) * user.pa_bang() as i16;
        if target.halves(self.element) {
            damage /= 2;
        }
        if target.weak(self.element) {
            damage *= 2;
        }
        if target.absorbs(self.element) {
            damage = -damage;
        }

        sim.change_target_hp(target_id, damage, Source::Ability);
    }
}

struct SecretFistImpl {
    base_chance: i16,
}

impl AbilityImpl for SecretFistImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if target.death_sentence() {
            return;
        }
        actions.push(Action {
            ability,
            range: 1,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let xa = mod_3_formula_xa(user.ma() as i16, user, target, true, false);
        let chance =
            ((self.base_chance as f32 + xa as f32) / 100.0) * user.zodiac_compatibility(target);
        if sim.roll_auto_succeed() < chance {
            sim.add_condition(target_id, Condition::DeathSentence, Source::Ability);
        }
    }
}

struct ChakraImpl {
    hp_multiplier: i16,
    mp_multiplier: i16,
}

impl AbilityImpl for ChakraImpl {
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
            range: 0,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let mut pa = user.pa() as i16;
        if user.martial_arts() {
            pa = (pa * 3) / 2;
        }

        sim.change_target_hp(target_id, self.hp_multiplier * -pa, Source::Ability);
        sim.change_target_mp(target_id, (self.mp_multiplier * -pa) / 2, Source::Ability);
    }
}

struct ReviveImpl {
    base_chance: i16,
    heal_amount: f32,
}

impl AbilityImpl for ReviveImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if !target.dead() {
            return;
        }

        actions.push(Action {
            ability,
            range: 1,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        if !target.dead() {
            return;
        }

        let xa = mod_3_formula_xa(user.pa() as i16, user, target, true, true);
        let mut chance = (self.base_chance as f32 + xa as f32) / 100.0;
        chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < chance {
            let heal_amount = (target.max_hp() as f32 * self.heal_amount) as i16;
            sim.change_target_hp(target_id, -heal_amount, Source::Ability);
        }
    }
}

struct PurificationImpl {
    base_chance: i16,
    cancels: &'static [Condition],
}

impl AbilityImpl for PurificationImpl {
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
            range: 0,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let xa = mod_3_formula_xa(user.pa() as i16, user, target, true, true);
        let mut chance = (self.base_chance as f32 + xa as f32) / 100.0;
        chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < chance {
            for condition in self.cancels {
                sim.cancel_condition(target_id, *condition, Source::Ability);
            }
        }
    }
}
