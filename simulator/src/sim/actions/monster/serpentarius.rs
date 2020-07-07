use crate::sim::actions::{Ability, AbilityImpl, Action, FOE_OK};

use crate::sim::common::mod_5_formula_xa;
use crate::sim::{AoE, Combatant, CombatantId, Condition, Element, Simulation, Source};

pub const SERPENTARIUS_ABILITIES: &[Ability] = &[
    // Snake Carrier: 1 range, 0 AoE. Effect: AbsorbHP (25)%; Add Undead, Darkness, Confusion, Blood Suck, Oil, Frog, Poison, Charm, Sleep, Death Sentence (Random).
    Ability {
        name: "Snake Carrier",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &SnakeCarrierImpl {
            conditions: &[
                Condition::Undead,
                Condition::Darkness,
                Condition::Confusion,
                Condition::BloodSuck,
                Condition::Oil,
                Condition::Frog,
                Condition::Poison,
                Condition::Charm,
                Condition::Sleep,
                Condition::DeathSentence,
            ],
        },
    },
    // Toxic Frog: 4 range, 2 AoE, 3 CT. Hit: Faith(MA + 106)%. Effect: Add Frog, Poison (All).
    Ability {
        name: "Toxic Frog",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(1)),
        implementation: &ToxicFrogImpl {
            range: 4,
            ctr: Some(3),
            base_chance: 106,
            conditions: &[Condition::Frog, Condition::Poison],
        },
    },
    // Midgar Swarm: 4 range, 2 AoE, 4 CT, 37 MP. Effect: Damage (MA * 16).
    Ability {
        name: "Midgar Swarm",
        flags: FOE_OK,
        mp_cost: 37,
        aoe: AoE::Diamond(2, Some(2)),
        implementation: &MidgarSwarmImpl {
            ma_factor: 16,
            range: 4,
            ctr: Some(4),
        },
    },
];

struct SnakeCarrierImpl {
    conditions: &'static [Condition],
}

impl AbilityImpl for SnakeCarrierImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, 1, None, target.id()))
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let target = sim.combatant(target_id);
        let amount = target.hp() / 4;
        sim.change_target_hp(target_id, amount, Source::Ability);
        sim.change_target_hp(user_id, -amount, Source::Ability);
        for condition in self.conditions {
            if sim.roll_auto_succeed() < 0.25 {
                sim.add_condition(target_id, *condition, Source::Ability);
            }
        }
        sim.try_countergrasp(user_id, target_id);
    }
}

struct ToxicFrogImpl {
    range: u8,
    ctr: Option<u8>,
    base_chance: i16,
    conditions: &'static [Condition],
}

impl AbilityImpl for ToxicFrogImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, self.ctr, target.id()))
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let mut chance = (user.ma() + self.base_chance) as f32 / 100.0;
        chance *= user.faith_percent();
        chance *= target.faith_percent();
        chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < chance {
            for condition in self.conditions {
                sim.add_condition(target_id, *condition, Source::Ability);
            }
        }
    }
}

pub(crate) struct MidgarSwarmImpl {
    pub(crate) ma_factor: i16,
    pub(crate) range: u8,
    ctr: Option<u8>,
}

impl AbilityImpl for MidgarSwarmImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        _user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        actions.push(Action::new(ability, self.range, self.ctr, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        let xa = mod_5_formula_xa(user.ma() as i16, user, target, Element::None, false);
        sim.change_target_hp(target_id, xa * self.ma_factor, Source::Ability);
    }
}
