use crate::dto::rust::Equipment;
use crate::sim::actions::attack::do_single_weapon_attack;
use crate::sim::actions::common::{do_hp_damage, do_hp_heal, mod_2_formula_xa, mod_3_formula_xa};
use crate::sim::actions::{
    Ability, AbilityImpl, Action, ALLY_OK, FOE_OK, HITS_FOES_ONLY, TARGET_NOT_SELF,
};
use crate::sim::{
    Combatant, CombatantId, Condition, EquipSlot, Event, Simulation, Source, WeaponType,
};

pub const BATTLE_SKILL_ABILITIES: &[Ability] = &[
    // Head Break: weapon range, 0 AoE. Hit: (PA + WP + 45)%. Effect: Break target's head equipment; If none, attack instead.
    Ability {
        name: "Head Break",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &BreakEquipImpl {
            base_chance: 45,
            equip_slot: EquipSlot::Head,
        },
    },
    // Armor Break: weapon range, 0 AoE. Hit: (PA + WP + 40)%. Effect: Break target's body equipment; If none, attack instead.
    Ability {
        name: "Armor Break",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &BreakEquipImpl {
            base_chance: 40,
            equip_slot: EquipSlot::Body,
        },
    },
    // Shield Break: weapon range, 0 AoE. Hit: (PA + WP + 55)%. Effect: Break target's shield; If none, attack instead.
    Ability {
        name: "Shield Break",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &BreakEquipImpl {
            base_chance: 55,
            equip_slot: EquipSlot::Shield,
        },
    },
    // Weapon Break: weapon range, 0 AoE. Hit: (PA + WP + 30)%. Effect: Break target's weapon; If none, attack instead.
    Ability {
        name: "Weapon Break",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &BreakEquipImpl {
            base_chance: 30,
            equip_slot: EquipSlot::Weapon,
        },
    },
    // Magic Break: weapon range, 0 AoE. Hit: (PA + 50)%. Effect: DamageMP (50)%.
    Ability {
        name: "Magic Break",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &MagicBreakImpl {
            base_chance: 50,
            mp_percent: 0.5,
        },
    },
    // Speed Break: weapon range, 0 AoE. Hit: (PA + 50)%. Effect: -2 Speed.
    // Power Break: weapon range, 0 AoE. Hit: (PA + 50)%. Effect: -3 PA.
    // Mind Break: weapon range, 0 AoE. Hit: (PA + 50)%. Effect: -3 MA.

    // Stasis Sword: 1 range, 1 AoE, 2 CT, 20 MP. Effect: Damage (PA * (WP + 1)); Chance to Add Stop.
    Ability {
        name: "Stasis Sword",
        flags: FOE_OK | HITS_FOES_ONLY,
        mp_cost: 20,
        aoe: Some(1),
        implementation: &ChanceToAddSwordImpl {
            wp_plus: 1,
            chance_to_add: Condition::Stop,
            range: 1,
            ctr: 2,
        },
    },
    // Justice Sword: 2 range, 0 AoE, 2 CT, 22 MP. Effect: Damage (PA * (WP + 2)); Chance to Add Death Sentence.
    Ability {
        name: "Justice Sword",
        flags: FOE_OK,
        mp_cost: 22,
        aoe: None,
        implementation: &ChanceToAddSwordImpl {
            wp_plus: 2,
            chance_to_add: Condition::DeathSentence,
            range: 2,
            ctr: 2,
        },
    },
    // Surging Sword: 1 range, 1 AoE, 3 CT, 24 MP. Effect: Damage (PA * (WP + 2)); Chance to Add Silence.
    Ability {
        name: "Surging Sword",
        flags: FOE_OK | HITS_FOES_ONLY,
        mp_cost: 24,
        aoe: Some(1),
        implementation: &ChanceToAddSwordImpl {
            wp_plus: 2,
            chance_to_add: Condition::Silence,
            range: 1,
            ctr: 3,
        },
    },
    // Explosion Sword: 4 range, 4 AoE (line), 2 CT, 26 MP. Effect: Damage (PA * (WP + 3)); Chance to Add Confusion.
    // Shellburst Stab: 2 range, 0 AoE, 15 MP. Effect: Break target's body equipment; If successful Damage (PA * WP).
    // Dark Sword: 2 range, 0 AoE, 2 CT, 18 MP. Effect: AbsorbMP (PA * WP).
    Ability {
        name: "Dark Sword",
        flags: FOE_OK,
        mp_cost: 18,
        aoe: None,
        implementation: &AbsorbSwordImpl {
            hp_not_mp: false,
            range: 2,
            ctr: 2,
        },
    },
    // Night Sword: 2 range, 0 AoE, 3 CT, 22 MP. Effect: AbsorbHP (PA * WP).
    Ability {
        name: "Night Sword",
        flags: FOE_OK,
        mp_cost: 22,
        aoe: None,
        implementation: &AbsorbSwordImpl {
            hp_not_mp: true,
            range: 2,
            ctr: 3,
        },
    },
];

struct AbsorbSwordImpl {
    hp_not_mp: bool,
    range: i8,
    ctr: u8,
}

impl AbilityImpl for AbsorbSwordImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.main_hand().map_or(true, |eq| {
            eq.weapon_type != Some(WeaponType::Sword)
                && eq.weapon_type != Some(WeaponType::KnightSword)
        }) {
            return;
        }

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

        if user.main_hand().map_or(true, |eq| {
            eq.weapon_type != Some(WeaponType::Sword)
                && eq.weapon_type != Some(WeaponType::KnightSword)
        }) {
            return;
        }

        // TODO: This is a weapon elemental attack sooo...
        let mut xa = mod_3_formula_xa(user.pa() as i16, user, target, false, false);
        if sim.roll_auto_fail() < 0.05 {
            xa += sim.roll_inclusive(1, xa.max(1)) - 1;
        }
        let damage = xa * user.main_hand().unwrap().wp as i16;

        if self.hp_not_mp {
            do_hp_damage(sim, target_id, damage, true);
            do_hp_heal(sim, user_id, damage, true);
        } else {
            sim.change_target_mp(target_id, damage, Source::Ability);
            sim.change_target_mp(user_id, -damage, Source::Ability);
        }
    }
}

struct ChanceToAddSwordImpl {
    wp_plus: i16,
    chance_to_add: Condition,
    range: i8,
    ctr: u8,
}

impl AbilityImpl for ChanceToAddSwordImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.main_hand().map_or(true, |eq| {
            eq.weapon_type != Some(WeaponType::Sword)
                && eq.weapon_type != Some(WeaponType::KnightSword)
        }) {
            return;
        }

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

        if user.main_hand().map_or(true, |eq| {
            eq.weapon_type != Some(WeaponType::Sword)
                && eq.weapon_type != Some(WeaponType::KnightSword)
        }) {
            return;
        }

        // TODO: This is a weapon elemental attack sooo...
        let mut xa = mod_3_formula_xa(user.pa() as i16, user, target, false, false);
        if sim.roll_auto_fail() < 0.05 {
            xa += sim.roll_inclusive(1, xa.max(1)) - 1;
        }
        let damage = xa * (user.main_hand().unwrap().wp as i16 + self.wp_plus);
        sim.change_target_hp(target_id, damage, Source::Ability);
        if sim.roll_auto_succeed() < 0.25 {
            sim.add_condition(target_id, self.chance_to_add, Source::Ability);
        }
    }
}

struct BreakEquipImpl {
    base_chance: i16,
    equip_slot: EquipSlot,
}

impl AbilityImpl for BreakEquipImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if target.monster() {
            return;
        }
        actions.push(Action {
            ability,
            range: user.main_hand().map_or(1, |eq| eq.range),
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let weapon = user.main_hand();
        if target.get_equip(self.equip_slot).is_none() {
            do_single_weapon_attack(sim, user_id, weapon, target_id);
        } else {
            try_break_equip(
                sim,
                self.base_chance,
                self.equip_slot,
                user_id,
                target_id,
                weapon,
            );
        }

        let user = sim.combatant(user_id);
        if user.dual_wield() && user.off_hand().is_some() {
            let weapon = user.off_hand();
            let target = sim.combatant(target_id);
            if target.get_equip(self.equip_slot).is_none() {
                do_single_weapon_attack(sim, user_id, weapon, target_id);
            } else {
                try_break_equip(
                    sim,
                    self.base_chance,
                    self.equip_slot,
                    user_id,
                    target_id,
                    weapon,
                );
            }
        }

        sim.try_countergrasp(user_id, target_id);
    }
}

fn try_break_equip<'a>(
    sim: &mut Simulation<'a>,
    base_chance: i16,
    equip_slot: EquipSlot,
    user_id: CombatantId,
    target_id: CombatantId,
    equip: Option<&'a Equipment>,
) {
    let user = sim.combatant(user_id);
    let target = sim.combatant(target_id);
    let wp = equip.map_or(0, |eq| eq.wp);
    let mod_pa = mod_3_formula_xa(user.pa() as i16, user, target, false, false);
    let mod_wp = mod_3_formula_xa(wp as i16, user, target, false, false);

    let mut chance = (mod_pa as f32 + mod_wp as f32 + base_chance as f32 / 100.0);
    chance *= user.zodiac_compatibility(target);

    if sim.do_physical_evade(user, target, Source::Ability) {
        sim.log_event(Event::AbilityMissed(user_id, target_id));
    } else if sim.roll_auto_succeed() < chance {
        let target = sim.combatant_mut(target_id);
        let old_equip = target.get_equip(equip_slot).unwrap();
        target.break_equip(equip_slot);
        sim.log_event(Event::Broke(target_id, old_equip));
    } else {
        sim.log_event(Event::AbilityMissed(user_id, target_id));
    }
}

struct MagicBreakImpl {
    base_chance: i16,
    mp_percent: f32,
}

impl AbilityImpl for MagicBreakImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action {
            ability,
            range: user.main_hand().map_or(1, |eq| eq.range),
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        // TODO: Ok, I know this probably happens twice with dual wield.. bluh.
        let xa = mod_3_formula_xa(user.pa() as i16, user, target, false, false);
        let mut chance = (xa as f32 + self.base_chance as f32) / 100.0;
        chance *= user.zodiac_compatibility(target);
        if sim.do_physical_evade(user, target, Source::Ability) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
        } else if sim.roll_auto_succeed() < chance {
            let mp_damage = (target.max_mp() as f32 * self.mp_percent) as i16;
            sim.change_target_mp(target_id, mp_damage, Source::Ability);
        } else {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
        }
        sim.try_countergrasp(user_id, target_id);
    }
}
