use crate::sim::actions::{Ability, AbilityImpl, Action, AoE, ALLY_OK, FOE_OK};
use crate::sim::common::{do_hp_heal, should_heal_ally, should_heal_foe};
use crate::sim::{Combatant, CombatantId, Condition, Simulation, Source, NOT_ALIVE_OK};

pub const ITEM_ABILITIES: &[Ability] = &[
    Ability {
        name: "Potion",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PotionAbilityImpl {
            hp_amount: 100,
            mp_amount: 0,
        },
    },
    Ability {
        name: "Hi-Potion",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PotionAbilityImpl {
            hp_amount: 120,
            mp_amount: 0,
        },
    },
    Ability {
        name: "X-Potion",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PotionAbilityImpl {
            hp_amount: 150,
            mp_amount: 0,
        },
    },
    Ability {
        name: "Ether",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PotionAbilityImpl {
            hp_amount: 0,
            mp_amount: 20,
        },
    },
    Ability {
        name: "Hi-Ether",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PotionAbilityImpl {
            hp_amount: 0,
            mp_amount: 50,
        },
    },
    Ability {
        name: "Elixir",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PotionAbilityImpl {
            hp_amount: 999,
            mp_amount: 999,
        },
    },
    Ability {
        name: "Phoenix Down",
        flags: ALLY_OK | FOE_OK | NOT_ALIVE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &PhoenixDownImpl {},
    },
    Ability {
        name: "Antidote",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Poison],
        },
    },
    Ability {
        name: "Eye Drop",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Darkness],
        },
    },
    Ability {
        name: "Echo Grass",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Silence],
        },
    },
    Ability {
        name: "Maiden's Kiss",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Frog],
        },
    },
    Ability {
        name: "Soft",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Petrify],
        },
    },
    Ability {
        name: "Holy Water",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Undead],
        },
    },
    Ability {
        name: "Remedy",
        flags: ALLY_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionCureItemImpl {
            cures: &[
                Condition::Petrify,
                Condition::Darkness,
                Condition::Confusion,
                Condition::Silence,
                Condition::Oil,
                Condition::Frog,
                Condition::Poison,
                Condition::Sleep,
            ],
        },
    },
];

struct PotionAbilityImpl {
    hp_amount: i16,
    mp_amount: i16,
}

impl AbilityImpl for PotionAbilityImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.ally(target) && !should_heal_ally(target, true) {
            return;
        }
        if user.foe(target) && !should_heal_foe(target, true) {
            return;
        }
        actions.push(Action::new(ability, item_range(user), None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        if self.hp_amount > 0 {
            do_hp_heal(sim, target_id, self.hp_amount, true);
        }
        if self.mp_amount > 0 {
            sim.change_target_mp(target_id, -self.mp_amount, Source::Ability);
        }
    }
}

struct ConditionCureItemImpl {
    cures: &'static [Condition],
}

impl AbilityImpl for ConditionCureItemImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if !self.cures.iter().any(|cond| target.has_condition(*cond)) {
            return;
        }
        actions.push(Action::new(ability, item_range(user), None, target.id()));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        for condition in self.cures {
            sim.cancel_condition(target_id, *condition, Source::Ability);
        }
    }
}

fn item_range(user: &Combatant) -> i8 {
    if user.throw_item() {
        4
    } else {
        1
    }
}

struct PhoenixDownImpl {}

impl AbilityImpl for PhoenixDownImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.foe(target) && !target.dead() && target.undead() {
            actions.push(Action::new(ability, item_range(user), None, target.id()));
        } else if user.ally(target) && !target.undead() && target.dead() && !target.reraise() {
            actions.push(Action::new(ability, item_range(user), None, target.id()));
        }
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        let target = sim.combatant(target_id);
        if target.undead() && !target.dead() {
            sim.change_target_hp(target_id, target.max_hp(), Source::Ability);
        } else if !target.undead() && target.dead() {
            let heal_amount = sim.roll_inclusive(1, 20);
            sim.change_target_hp(target_id, -heal_amount, Source::Ability);
        }
    }
}
