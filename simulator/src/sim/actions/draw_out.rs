use crate::sim::actions::{Ability, AbilityImpl, Action, AoE, ALLY_OK};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, FOE_OK, HITS_ALLIES_ONLY,
    HITS_FOES_ONLY, NOT_ALIVE_OK, SILENCEABLE, TARGET_NOT_SELF, TARGET_SELF_ONLY,
};

pub const DRAW_OUT_ABILITIES: &[Ability] = &[
    // Asura: 0 range, 2 AoE. Effect: Damage (MA * 8); Chance to Cancel Undead, Blood Suck, Float, Reraise, Transparent, Berserk, Regen, Protect, Shell, Haste, Faith, Innocent, Reflect.
    Ability {
        name: "Asura",
        flags: HITS_FOES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: 8,
            range: 0,
            damage_hp_not_mp: true,
            chance_to_cancel: &[
                Condition::Undead,
                Condition::BloodSuck,
                Condition::Float,
                Condition::Reraise,
                Condition::Transparent,
                Condition::Berserk,
                Condition::Regen,
                Condition::Protect,
                Condition::Shell,
                Condition::Haste,
                Condition::Faith,
                Condition::Innocent,
                Condition::Reflect,
            ],
            chance_to_add_random: &[],
        },
    },
    // Koutetsu: 0 range, 2 AoE, Effect: Damage (MA * 10); Chance to Add Oil, Darkness (Random).
    Ability {
        name: "Koutetsu",
        flags: HITS_FOES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: 10,
            range: 0,
            damage_hp_not_mp: true,
            chance_to_cancel: &[],
            chance_to_add_random: &[Condition::Oil, Condition::Darkness],
        },
    },
    // Bizen Boat: 0 range, 2 AoE. Effect: DamageMP (MA * 5).
    Ability {
        name: "Bizen Boat",
        flags: HITS_FOES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: 5,
            range: 0,
            damage_hp_not_mp: false,
            chance_to_cancel: &[],
            chance_to_add_random: &[],
        },
    },
    // Murasame: 0 range, 2 AoE. Effect: Heal (MA * 9).
    Ability {
        name: "Murasame",
        flags: HITS_ALLIES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: -9,
            range: 0,
            damage_hp_not_mp: true,
            chance_to_cancel: &[],
            chance_to_add_random: &[],
        },
    },
    // Heaven's Cloud: 0 range, 2 AoE. Effect: Damage (MA * 11); Chance to Add Slow.
    Ability {
        name: "Heaven's Cloud",
        flags: HITS_FOES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: 11,
            range: 0,
            damage_hp_not_mp: true,
            chance_to_cancel: &[],
            chance_to_add_random: &[Condition::Slow],
        },
    },
    // Kiyomori: 0 range, 2 AoE. Effect: Add Protect, Shell (Random).
    Ability {
        name: "Kiyomori",
        flags: HITS_ALLIES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutBuffImpl {
            add_random: &[Condition::Protect, Condition::Shell],
        },
    },
    // Muramasa: 0 range, 2 AoE. Effect: Damage (MA * 14); Chance to Add Confusion, Death Sentence (Random).
    Ability {
        name: "Muramasa",
        flags: HITS_FOES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: 14,
            range: 0,
            damage_hp_not_mp: true,
            chance_to_cancel: &[],
            chance_to_add_random: &[Condition::Confusion, Condition::DeathSentence],
        },
    },
    // Kikuichimoji: 6 range, 6 AoE (line). Effect: Damage (MA * 12).
    Ability {
        name: "Kikuichimoji",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::Line(Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: 12,
            range: 6,
            damage_hp_not_mp: true,
            chance_to_cancel: &[],
            chance_to_add_random: &[],
        },
    },
    // Masamune: 0 range, 2 AoE. Effect: Add Regen, Haste (Random).
    Ability {
        name: "Masamune",
        flags: HITS_ALLIES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutBuffImpl {
            add_random: &[Condition::Regen, Condition::Haste],
        },
    },
    // Chirijiraden: 0 range, 2 AoE. Effect: Damage (MA * 18).
    Ability {
        name: "Chirijiraden",
        flags: HITS_FOES_ONLY | ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(3)),
        implementation: &DrawOutDamageImpl {
            ma_factor: 18,
            range: 0,
            damage_hp_not_mp: true,
            chance_to_cancel: &[],
            chance_to_add_random: &[],
        },
    },
];

struct DrawOutDamageImpl {
    ma_factor: i16,
    range: u8,
    damage_hp_not_mp: bool,
    chance_to_add_random: &'static [Condition],
    chance_to_cancel: &'static [Condition],
}

impl AbilityImpl for DrawOutDamageImpl {
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

        let mut ma = user.ma() as i16;
        // If caster has Magic AttackUP, then (MA2 = [MA1 * 4/3]), else MA2 = MA1
        if user.magic_attack_up() {
            ma = (ma * 4) / 3;
        }

        if self.ma_factor > 0 {
            if target.magic_defense_up() {
                ma = (ma * 2) / 3;
            }
            if target.shell() {
                ma = (ma * 2) / 3;
            }
        }

        ma = (ma as f32 * user.zodiac_compatibility(target)) as i16;
        let damage = ma * self.ma_factor;
        if self.damage_hp_not_mp {
            sim.change_target_hp(target_id, damage, Source::Ability);
        } else {
            sim.change_target_mp(target_id, damage, Source::Ability);
        }

        if !self.chance_to_add_random.is_empty() && sim.roll_auto_fail() < 0.20 {
            let length = (self.chance_to_add_random.len() - 1) as i16;
            let condition = self.chance_to_add_random[sim.roll_inclusive(0, length) as usize];
            sim.add_condition(target_id, condition, Source::Ability);
        }

        if !self.chance_to_cancel.is_empty() && sim.roll_auto_fail() < 0.20 {
            for condition in self.chance_to_cancel {
                sim.cancel_condition(target_id, *condition, Source::Ability);
            }
        }
    }
}

struct DrawOutBuffImpl {
    add_random: &'static [Condition],
}

impl AbilityImpl for DrawOutBuffImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 0, None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        // TODO: Do we ever add more than one?
        let length = (self.add_random.len() - 1) as i16;
        let condition = self.add_random[sim.roll_inclusive(0, length) as usize];
        sim.add_condition(target_id, condition, Source::Ability);
    }
}
