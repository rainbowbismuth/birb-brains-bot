use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::common::{do_hp_heal, mod_2_formula_xa, mod_5_formula_xa};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY,
};

pub const CHOCOBO_ABILITIES: &[Ability] = &[
    // TODO: Choco Attack: 1 range, 0 AoE. Effect: Normal Attack.
    // Choco Ball: 4 range, 0 AoE. Element: Water. Effect: Damage (PA / 2 * PA).
    Ability {
        name: "Choco Ball",
        flags: FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: None,
        implementation: &ChocoBallImpl {
            element: Element::Water,
            range: 4,
        },
    },
    // Choco Meteor: 5 range, 0 AoE. Effect: Damage (MA * 4).
    Ability {
        name: "Choco Meteor",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: None,
        implementation: &ChocoMeteorImpl {
            ma_factor: 4,
            range: 5,
        },
    },
    // Choco Esuna: 0 range, 1 AoE. Effect: Cancel Petrify, Darkness, Silence, Poison, Stop, Don't Move, Don't Act.
    Ability {
        name: "Choco Esuna",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: Some(1),
        implementation: &ChocoEsunaImpl {
            cures: &[
                Condition::Petrify,
                Condition::Darkness,
                Condition::Silence,
                Condition::Poison,
                Condition::Stop,
                Condition::DontMove,
                Condition::DontAct,
            ],
        },
    },
    // Choco Cure: 0 range, 1 AoE. Effect: Heal (MA * 3).
    Ability {
        name: "Choco Cure",
        flags: ALLY_OK | TARGET_SELF_ONLY,
        mp_cost: 0,
        aoe: Some(1),
        implementation: &ChocoCureImpl { ma_factor: 3 },
    },
];

struct ChocoEsunaImpl {
    cures: &'static [Condition],
}

impl AbilityImpl for ChocoEsunaImpl {
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
        for condition in self.cures {
            sim.cancel_condition(target_id, *condition, Source::Ability);
        }
    }
}

struct ChocoBallImpl {
    element: Element,
    range: i8,
}

impl AbilityImpl for ChocoBallImpl {
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

        // TODO: Roll crit.
        let crit = sim.roll_auto_fail() <= 0.05;

        let xa = mod_2_formula_xa(
            sim,
            user.pa() as i16,
            user,
            target,
            self.element,
            crit,
            false,
            false,
        );
        sim.change_target_hp(
            target_id,
            (xa / 2) * (user.pa_bang() as i16),
            Source::Ability,
        );
    }
}

struct ChocoMeteorImpl {
    ma_factor: i16,
    range: i8,
}

impl AbilityImpl for ChocoMeteorImpl {
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
        let xa = mod_5_formula_xa(user.ma() as i16, user, target, Element::None, false);
        sim.change_target_hp(target_id, xa * self.ma_factor, Source::Ability);
    }
}

struct ChocoCureImpl {
    ma_factor: i16,
}

impl AbilityImpl for ChocoCureImpl {
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
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        let xa = mod_5_formula_xa(user.ma() as i16, user, target, Element::None, true);
        do_hp_heal(sim, target_id, xa * self.ma_factor, true);
    }
}
