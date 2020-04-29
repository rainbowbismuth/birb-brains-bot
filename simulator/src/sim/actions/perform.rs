use crate::sim::actions::common::{mod_2_formula_xa, EmpowerImpl};
use crate::sim::actions::{
    Ability, AbilityImpl, Action, ActionTarget, AoE, ALLY_OK, FOE_OK, TARGET_NOT_SELF,
};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, EquipSlot, Event, Simulation, Source, WeaponType,
    HITS_ALLIES_ONLY, HITS_FOES_ONLY, NO_SHORT_CHARGE, PERFORMANCE, STATS_ABILITY,
    TARGET_SELF_ONLY,
};

pub const PERFORMANCE_ABILITIES: &[Ability] = &[
    // Angel Song: 0 range, 255 AoE, 5 CT. Effect: HealMP (20 + MA).
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Angel Song",
        implementation: &HealSongImpl {
            ct: 5,
            hp_not_mp: false,
            bonus: 20,
        },
    },
    // Life Song: 0 range, 255 AoE, 5 CT. Effect: Heal (20 + MA).
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Life Song",
        implementation: &HealSongImpl {
            ct: 5,
            hp_not_mp: true,
            bonus: 20,
        },
    },
    // Cheer Song: 0 range, 255 AoE, 6 CT. Hit: 50% per target. Effect: +1 Speed.
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Cheer Song",
        implementation: &StatPerformanceImpl {
            hit_chance: 0.50,
            ct: 6,
            speed_buff: 1,
            pa_buff: 0,
            ma_buff: 0,
        },
    },
    // Battle Song: 0 range, 255 AoE, 7 CT. Hit: 50% per target. Effect: +1 PA.
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Battle Song",
        implementation: &StatPerformanceImpl {
            hit_chance: 0.50,
            ct: 7,
            speed_buff: 0,
            pa_buff: 1,
            ma_buff: 0,
        },
    },
    // Magic Song: 0 range, 255 AoE, 7 CT. Hit: 50% per target. Effect: +1 MA.
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Magic Song",
        implementation: &StatPerformanceImpl {
            hit_chance: 0.50,
            ct: 7,
            speed_buff: 0,
            pa_buff: 0,
            ma_buff: 1,
        },
    },
    // Nameless Song: 0 range, 255 AoE, 9 CT, 5 MP. Hit: 50% per target. Effect: Add Reraise, Regen, Protect, Shell, Haste (Random).
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Nameless Song",
        implementation: &NamelessImpl {
            hit_chance: 0.50,
            ct: 9,
            conditions: &[
                Condition::Reraise,
                Condition::Regen,
                Condition::Protect,
                Condition::Shell,
                Condition::Haste,
            ],
        },
    },
    // TODO: Last Song: 0 range, 255 AoE, 8 CT. Hit: 25% per target. Effect: Set CT to Max.
    // Witch Hunt: 0 range, 255 AoE, 5 CT. Effect: DamageMP (PA + (PA * Brave) / 100).
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_FOES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Witch Hunt",
        implementation: &HurtDanceImpl {
            ct: 5,
            hp_not_mp: false,
        },
    },
    // Wiznaibus: 0 range, 255 AoE, 5 CT. Effect: Damage (PA + (PA * Brave) / 100).
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_FOES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Wiznaibus",
        implementation: &HurtDanceImpl {
            ct: 5,
            hp_not_mp: true,
        },
    },
    // Slow Dance: 0 range, 255 AoE, 6 CT. Hit: 50% per target. Effect: -1 Speed.
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_FOES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Slow Dance",
        implementation: &StatPerformanceImpl {
            hit_chance: 0.50,
            ct: 6,
            speed_buff: -1,
            pa_buff: 0,
            ma_buff: 0,
        },
    },
    // Polka Polka: 0 range, 255 AoE, 7 CT. Hit: 50% per target. Effect: -1 PA.
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_FOES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Polka Polka",
        implementation: &StatPerformanceImpl {
            hit_chance: 0.50,
            ct: 7,
            speed_buff: 0,
            pa_buff: -1,
            ma_buff: 0,
        },
    },
    // Disillusion: 0 range, 255 AoE, 7 CT. Hit: 50% per target. Effect: -1 MA.
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_FOES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Disillusion",
        implementation: &StatPerformanceImpl {
            hit_chance: 0.50,
            ct: 7,
            speed_buff: 0,
            pa_buff: 0,
            ma_buff: -1,
        },
    },
    // Nameless Dance: 0 range, 255 AoE, 10 CT, 5 MP. Hit: 50% per target. Effect: Add Darkness, Confusion, Silence, Frog, Poison, Slow, Stop, Sleep (Random).
    Ability {
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_FOES_ONLY | NO_SHORT_CHARGE | PERFORMANCE,
        mp_cost: 0,
        aoe: AoE::Global,
        name: "Nameless Dance",
        implementation: &NamelessImpl {
            hit_chance: 0.50,
            ct: 10,
            conditions: &[
                Condition::Darkness,
                Condition::Confusion,
                Condition::Silence,
                Condition::Frog,
                Condition::Poison,
                Condition::Slow,
                Condition::Stop,
                Condition::Sleep,
            ],
        },
    },
    // TODO: Last Dance: 0 range, 255 AoE, 8 CT. Hit: 25% per target. Effect: Set CT to 0.
];

struct NamelessImpl {
    hit_chance: f32,
    ct: u8,
    conditions: &'static [Condition],
}

impl AbilityImpl for NamelessImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        _target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: 255,
            ctr: Some(self.ct),
            target: ActionTarget::Panel(user.panel),
        });
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        let target = sim.combatant(target_id);
        if sim.roll_auto_succeed() <= self.hit_chance {
            // TODO: Make this not terrible
            for _ in 0..10 {
                let idx = sim.roll_inclusive(0, self.conditions.len() as i16 - 1) as usize;
                let condition = self.conditions[idx];
                if target.has_condition(condition) {
                    continue;
                }
                sim.add_condition(target_id, condition, Source::Ability);
                return;
            }
        }
    }
}

struct HurtDanceImpl {
    ct: u8,
    hp_not_mp: bool,
}

impl AbilityImpl for HurtDanceImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        _target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: 255,
            ctr: Some(self.ct),
            target: ActionTarget::Panel(user.panel),
        });
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let amount = user.pa() + (user.pa() as f32 * user.brave_percent()) as i16;
        if self.hp_not_mp {
            sim.change_target_hp(target_id, amount, Source::Ability);
        } else {
            sim.change_target_mp(target_id, amount, Source::Ability);
        }
    }
}

struct HealSongImpl {
    ct: u8,
    hp_not_mp: bool,
    bonus: i16,
}

impl AbilityImpl for HealSongImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        _target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: 255,
            ctr: Some(self.ct),
            target: ActionTarget::Panel(user.panel),
        });
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let amount = self.bonus + user.ma();
        if self.hp_not_mp {
            sim.change_target_hp(target_id, -amount, Source::Ability);
        } else {
            sim.change_target_mp(target_id, -amount, Source::Ability);
        }
    }
}

struct StatPerformanceImpl {
    hit_chance: f32,
    ct: u8,
    speed_buff: i8,
    pa_buff: i8,
    ma_buff: i8,
}

impl AbilityImpl for StatPerformanceImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        _target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: 255,
            ctr: Some(self.ct),
            target: ActionTarget::Panel(user.panel),
        });
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        if sim.roll_auto_succeed() <= self.hit_chance {
            sim.change_unit_pa(target_id, self.pa_buff, Source::Ability);
            sim.change_unit_ma(target_id, self.ma_buff, Source::Ability);
            sim.change_unit_speed(target_id, self.speed_buff, Source::Ability);
        }
    }
}
