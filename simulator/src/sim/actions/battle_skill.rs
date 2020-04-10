use crate::dto::rust::Equipment;
use crate::sim::actions::attack::do_single_weapon_attack;
use crate::sim::actions::common::mod_3_formula_xa;
use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK, TARGET_NOT_SELF};
use crate::sim::{Combatant, CombatantId, Condition, EquipSlot, Event, Simulation, Source};

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
    // Speed Break: weapon range, 0 AoE. Hit: (PA + 50)%. Effect: -2 Speed.
    // Power Break: weapon range, 0 AoE. Hit: (PA + 50)%. Effect: -3 PA.
    // Mind Break: weapon range, 0 AoE. Hit: (PA + 50)%. Effect: -3 MA.
];

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
