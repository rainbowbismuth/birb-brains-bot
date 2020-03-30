use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::{Combatant, CombatantId, Condition, Simulation, Source, NOT_ALIVE_OK};
use std::path::Component::CurDir;

pub const ITEM_ABILITIES: &[Ability] = &[
    Ability {
        name: "Potion",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        implementation: &PotionAbilityImpl {
            hp_amount: 100,
            mp_amount: 0,
        },
    },
    Ability {
        name: "Hi-Potion",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        implementation: &PotionAbilityImpl {
            hp_amount: 120,
            mp_amount: 0,
        },
    },
    Ability {
        name: "X-Potion",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        implementation: &PotionAbilityImpl {
            hp_amount: 150,
            mp_amount: 0,
        },
    },
    Ability {
        name: "Ether",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        implementation: &PotionAbilityImpl {
            hp_amount: 0,
            mp_amount: 20,
        },
    },
    Ability {
        name: "Hi-Ether",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        implementation: &PotionAbilityImpl {
            hp_amount: 0,
            mp_amount: 50,
        },
    },
    Ability {
        name: "Elixir",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        implementation: &PotionAbilityImpl {
            hp_amount: 999,
            mp_amount: 999,
        },
    },
    Ability {
        name: "Phoenix Down",
        flags: ALLY_OK | FOE_OK | NOT_ALIVE_OK,
        mp_cost: 0,
        implementation: &PhoenixDownImpl {},
    },
    Ability {
        name: "Antidote",
        flags: ALLY_OK,
        mp_cost: 0,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Poison],
        },
    },
    Ability {
        name: "Eye Drop",
        flags: ALLY_OK,
        mp_cost: 0,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Darkness],
        },
    },
    Ability {
        name: "Echo Grass",
        flags: ALLY_OK,
        mp_cost: 0,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Silence],
        },
    },
    Ability {
        name: "Maiden's Kiss",
        flags: ALLY_OK,
        mp_cost: 0,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Frog],
        },
    },
    Ability {
        name: "Soft",
        flags: ALLY_OK,
        mp_cost: 0,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Petrify],
        },
    },
    Ability {
        name: "Holy Water",
        flags: ALLY_OK,
        mp_cost: 0,
        implementation: &ConditionCureItemImpl {
            cures: &[Condition::Undead],
        },
    },
    Ability {
        name: "Remedy",
        flags: ALLY_OK,
        mp_cost: 0,
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
        if user.ally(target) && !should_item_heal_ally(target) {
            return;
        }
        if user.foe(target) && !should_item_heal_foe(target) {
            return;
        }
        actions.push(Action {
            ability,
            range: item_range(user),
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        let mut heal_amount = self.hp_amount;
        if sim.combatant(target_id).undead() {
            heal_amount = -heal_amount;
        }
        sim.change_target_hp(target_id, -heal_amount, Source::Constant("Item"));

        if self.mp_amount > 0 {
            sim.change_target_mp(target_id, -self.mp_amount, Source::Constant("Item"));
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
        actions.push(Action {
            ability,
            range: item_range(user),
            ctr: None,
            target_id: target.id(),
        });
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        for condition in self.cures {
            sim.cancel_condition(target_id, *condition, Source::Constant("Item"));
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

fn should_item_heal_foe(target: &Combatant) -> bool {
    target.undead()
}

fn should_item_heal_ally(target: &Combatant) -> bool {
    if target.undead() {
        false
    } else {
        target.hp_percent() <= 0.50
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
            actions.push(Action {
                ability,
                range: item_range(user),
                ctr: None,
                target_id: target.id(),
            });
        } else if user.ally(target) && !target.undead() && target.dead() && !target.reraise() {
            actions.push(Action {
                ability,
                range: item_range(user),
                ctr: None,
                target_id: target.id(),
            });
        }
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, _user_id: CombatantId, target_id: CombatantId) {
        let target = sim.combatant(target_id);
        if target.undead() && !target.dead() {
            sim.change_target_hp(target_id, target.max_hp(), Source::Constant("Phoenix Down"));
        } else if !target.undead() && target.dead() {
            let heal_amount = sim.roll_inclusive(1, 20);
            sim.change_target_hp(target_id, -heal_amount, Source::Constant("Phoenix Down"));
        }
    }
}
