use crate::sim::actions::common::{mod_2_formula_xa, EmpowerImpl};
use crate::sim::actions::{Ability, AbilityImpl, Action, AoE, ALLY_OK, FOE_OK, TARGET_NOT_SELF};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, EquipSlot, Event, Simulation, Source, WeaponType,
    TARGET_SELF_ONLY,
};

pub const BASIC_SKILL_ABILITIES: &[Ability] = &[
    // Accumulate: 0 range, 0 AoE. Effect: +1 PA.
    Ability {
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &EmpowerImpl {
            range: 0,
            ctr: None,
            brave_mod: 0,
            pa_buff: 1,
            ma_buff: 0,
            speed_buff: 0,
        },
        name: "Accumulate",
    },
    // Throw Stone: 4 range, 0 AoE. Effect: Damage (Random(1-2) * PA); Chance to Knockback.
    Ability {
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &DashImpl {
            rand_min: 1,
            rand_max: 2,
            range: 4,
        },
        name: "Throw Stone",
    },
    DASH_ABILITY,
    // Heal: 1 range, 0 AoE. Effect: Cancel Darkness, Silence, Oil, Poison, Sleep, Don't Act.
    // Yell: 3 range, 0 AoE. Effect: +1 Speed.
    Ability {
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &EmpowerImpl {
            range: 3,
            ctr: None,
            brave_mod: 0,
            pa_buff: 0,
            ma_buff: 0,
            speed_buff: 1,
        },
        name: "Yell",
    },
    // Cheer Up: 3 range, 0 AoE. Hit: (MA + 85)%. Effect: Add Defending, Float, Reraise, Regen, Faith (Separate).
    // Wish: 1 range, 0 AoE. Effect: Heal (CasterMaxHP * 2 / 5); DamageCaster (CasterMaxHP / 5).
    // Scream: 0 range, 0 AoE, 2 CT. Effect: +3 Brave, +1 PA, +1 MA, +1 Speed.
    Ability {
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &EmpowerImpl {
            range: 0,
            ctr: Some(2),
            brave_mod: 3,
            pa_buff: 1,
            ma_buff: 1,
            speed_buff: 1,
        },
        name: "Scream",
    },
];

// Dash: 1 range, 0 AoE. Effect: Damage (Random(1-4) * PA); Chance to Knockback.
pub const DASH_ABILITY: Ability = Ability {
    flags: FOE_OK | TARGET_NOT_SELF,
    mp_cost: 0,
    aoe: AoE::None,
    implementation: &DashImpl {
        rand_min: 1,
        rand_max: 4,
        range: 1,
    },
    name: "Dash",
};

struct DashImpl {
    rand_min: i16,
    rand_max: i16,
    range: u8,
}

impl AbilityImpl for DashImpl {
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
        if sim.do_physical_evade(user, target, Source::Ability) {
            return;
        }
        let xa = mod_2_formula_xa(
            sim,
            user.pa() as i16,
            user,
            target,
            Element::None,
            false,
            false,
            false,
        );
        let damage = sim.roll_inclusive(self.rand_min, self.rand_max) * xa;
        sim.change_target_hp(target_id, damage, Source::Ability);
        if sim.roll_auto_fail() <= 0.5 {
            sim.do_knockback(user_id, target_id);
        }
    }
}
