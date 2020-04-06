use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK, TARGET_NOT_SELF};

use crate::sim::{
    Combatant, CombatantId, Condition, Element, EquipSlot, Event, Simulation, Source,
    HITS_ALLIES_ONLY, HITS_FOES_ONLY, NOT_ALIVE_OK, SILENCEABLE, TARGET_SELF_ONLY,
};

pub const STEAL_ABILITIES: &[Ability] = &[
    // Gil Taking: 2 range, 0 AoE. Hit: (Speed + 62)%. Effect: Add Defending, Berserk (Random).
    Ability {
        name: "Gil Taking",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &GilTakingImpl {
            base_chance: 62,
            range: 2,
        },
    },
    // Steal Heart: 3 range, 0 AoE. Hit: Non-Matching Sex; (MA + 44)%. Effect: Add Charm.
    Ability {
        name: "Steal Heart",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &StealHeartImpl {
            base_chance: 44,
            range: 3,
        },
    },
    // Steal Helmet: 1 range, 0 AoE. Hit: (Speed + 50)%. Effect: Steal target's head equipment.
    Ability {
        name: "Steal Helmet",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &StealImpl {
            base_chance: 50,
            equip_slot: EquipSlot::Head,
        },
    },
    // Steal Armor: 1 range, 0 AoE. Hit: (Speed + 40)%. Effect: Steal target's body equipment.
    Ability {
        name: "Steal Armor",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &StealImpl {
            base_chance: 40,
            equip_slot: EquipSlot::Body,
        },
    },
    // Steal Shield: 1 range, 0 AoE. Hit: (Speed + 35)%. Effect: Steal target's shield.
    Ability {
        name: "Steal Shield",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &StealImpl {
            base_chance: 35,
            equip_slot: EquipSlot::Shield,
        },
    },
    // Steal Weapon: 1 range, 0 AoE. Hit: (Speed + 45)%. Effect: Steal target's weapon.
    Ability {
        name: "Steal Weapon",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &StealImpl {
            base_chance: 45,
            equip_slot: EquipSlot::Weapon,
        },
    },
    // Steal Accessory: 1 range, 0 AoE. Hit: (Speed + 55)%. Effect: Steal target's accessory.
    Ability {
        name: "Steal Accessory",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &StealImpl {
            base_chance: 55,
            equip_slot: EquipSlot::Accessory,
        },
    },
];

struct StealImpl {
    base_chance: i16,
    equip_slot: EquipSlot,
}

impl AbilityImpl for StealImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if target.monster() {
            return;
        }
        match self.equip_slot {
            EquipSlot::Weapon => {
                if target.main_hand().is_none() {
                    return;
                }
            }
            EquipSlot::Shield => {
                if target.off_hand().is_none() {
                    return;
                }
            }
            EquipSlot::Head => {
                if target.headgear().is_none() {
                    return;
                }
            }
            EquipSlot::Body => {
                if target.armor().is_none() {
                    return;
                }
            }
            EquipSlot::Accessory => {
                if target.accessory().is_none() {
                    return;
                }
            }
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

        if sim.do_physical_evade(user, target, Source::Ability) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        let chance = mod_4_formula(user, target, self.base_chance as f32 / 100.0);

        if !(sim.roll_auto_succeed() < chance) {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
            return;
        }

        let target = sim.combatant_mut(target_id);
        let old_equip = target.get_equip(self.equip_slot).unwrap();
        target.break_equip(self.equip_slot);
        sim.log_event(Event::Broke(target_id, old_equip));
    }
}

struct GilTakingImpl {
    base_chance: i16,
    range: i8,
}

impl AbilityImpl for GilTakingImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if target.berserk() {
            return;
        }
        actions.push(Action {
            ability,
            range: self.range,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let mut base_chance = (user.speed() as f32 + self.base_chance as f32) / 100.0;
        base_chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < base_chance {
            if sim.roll_inclusive(1, 2) == 1 {
                // TODO: Implement Defending first..
                // sim.add_condition(target_id, Condition::Defending, Source::Ability);
            } else {
                sim.add_condition(target_id, Condition::Berserk, Source::Ability);
            }
        } else {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
        }
    }
}

struct StealHeartImpl {
    base_chance: i16,
    range: i8,
}

impl AbilityImpl for StealHeartImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.gender() == target.gender() {
            // ;-;
            return;
        }
        actions.push(Action {
            ability,
            range: self.range,
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        if user.gender() == target.gender() {
            return;
        }

        let mut base_chance = (user.ma() as f32 + self.base_chance as f32) / 100.0;
        base_chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < base_chance {
            sim.add_condition(target_id, Condition::Charm, Source::Ability);
        } else {
            sim.log_event(Event::AbilityMissed(user_id, target_id));
        }
    }
}

fn mod_4_formula(user: &Combatant, target: &Combatant, k: f32) -> f32 {
    let mut speed = user.speed() as i16;

    //    1. If caster has Attack UP, then (Sp1 = [Sp0 * 4/3]), else Sp1 = Sp0
    if user.attack_up() {
        speed = (speed * 4) / 3;
    }

    //    2. If caster has Martial Arts, then (Sp2 = [Sp1 * 3/2]), else Sp2 = Sp1
    if user.martial_arts() {
        speed = (speed * 3) / 2;
    }
    //    3. If target has Defense UP, then (Sp3 = [Sp2 * 2/3]), else Sp3 = Sp2
    if target.defense_up() {
        speed = (speed * 2) / 3;
    }
    //    4. If target has Protect, then (Sp4 = [Sp3 * 2/3]), else Sp4 = Sp3
    if target.protect() {
        speed = (speed * 2) / 3;
    }
    //    5. If target is Charging, then (Sp5 = [Sp4 * 3/2]), else Sp5 = Sp4
    if target.charging() {
        speed = (speed * 3) / 2;
    }
    //    6. If target is Sleeping, then (Sp6 = [Sp5 * 3/2]), else Sp6 = Sp5
    if target.sleep() {
        speed = (speed * 3) / 2;
    }
    //    7. If target is a Frog and/or Chicken, then (Sp7 = [Sp6 * 3/2]), else
    //       Sp7 = Sp6
    if target.frog() || target.chicken() {
        speed = (speed * 3) / 2;
    }

    //    8. Calculate Z (zodiac addend):
    //         If compatibility is 'Good', then Z = [Sp7 / 4] + [K / 4]
    //         ElseIf compatibility is 'Bad', then Z = -[Sp7 / 4] - [K / 4]
    //         ElseIf compatibility is 'Best', then Z = [Sp7 / 2] + [K / 2]
    //         ElseIf compatibility is 'Worst', then Z = -[Sp7 / 2] - [K / 2]
    //         Else, Z = 0
    //    9. Success% = (Sp7 + K + Z)
    //   10. Consider physical evasion multipliers, if applicable.
    let mut chance = (speed as f32 / 100.0) + k;
    chance *= user.zodiac_compatibility(target);

    chance
}
