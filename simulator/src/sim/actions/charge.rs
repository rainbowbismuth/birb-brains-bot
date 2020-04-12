use crate::sim::actions::attack::do_single_weapon_attack;

use crate::sim::actions::{
    Ability, AbilityImpl, Action, AoE, ALLY_OK, DONT_MOVE_WHILE_CHARGING, FOE_OK, NO_SHORT_CHARGE,
    TARGET_NOT_SELF,
};
use crate::sim::{
    Combatant, CombatantId, Condition, EquipSlot, Event, Simulation, Source, WeaponType,
};

pub const CHARGE_ABILITIES: &[Ability] = &[
    // Charge+1: weapon range, 0 AoE, 3 CT. Effect: Normal Attack with +1 Charge.
    Ability {
        name: "Charge+1",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 1, ct: 3 },
    },
    // Charge+2: weapon range, 0 AoE, 4 CT. Effect: Normal Attack with +2 Charge.
    Ability {
        name: "Charge+2",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 2, ct: 4 },
    },
    // Charge+3: weapon range, 0 AoE, 5 CT. Effect: Normal Attack with +3 Charge.
    Ability {
        name: "Charge+3",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 3, ct: 5 },
    },
    // Charge+4: weapon range, 0 AoE, 6 CT. Effect: Normal Attack with +4 Charge.
    Ability {
        name: "Charge+4",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 4, ct: 6 },
    },
    // Charge+5: weapon range, 0 AoE, 7 CT. Effect: Normal Attack with +5 Charge.
    Ability {
        name: "Charge+5",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 5, ct: 7 },
    },
    // Charge+7: weapon range, 0 AoE, 9 CT. Effect: Normal Attack with +7 Charge.
    Ability {
        name: "Charge+7",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 7, ct: 9 },
    },
    // Charge+10: weapon range, 0 AoE, 12 CT. Effect: Normal Attack with +10 Charge.
    Ability {
        name: "Charge+10",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 10, ct: 12 },
    },
    // Charge+20: weapon range, 0 AoE, 20 CT. Effect: Normal Attack with +20 Charge.
    Ability {
        name: "Charge+20",
        flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | DONT_MOVE_WHILE_CHARGING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ChargeImpl { k: 20, ct: 20 },
    },
];

struct ChargeImpl {
    k: i16,
    ct: u8,
}

impl AbilityImpl for ChargeImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        let ct_remaining = 0.max(100 - target.ct);
        let speed = if target.haste() {
            // TODO: Real AI doesn't account for this, but, since I haven't implemented
            //  tile targeting, I'm going to only target those that will certainly hit
            (target.speed() * 3) / 2
        } else {
            target.speed()
        };
        let ticks_left = ct_remaining / speed.max(1) as u8;
        if self.ct >= ticks_left {
            return;
        }
        actions.push(Action::target_panel(
            ability,
            user.main_hand().map_or(1, |eq| eq.range),
            Some(self.ct),
            target.location,
        ));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let weapon = user.main_hand();
        do_single_weapon_attack(sim, user_id, weapon, target_id, self.k);
    }
}
