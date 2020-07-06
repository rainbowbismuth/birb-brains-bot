use crate::sim::actions::{Ability, AbilityImpl, Action, AoE, FOE_OK};

use crate::sim::{
    Combatant, CombatantId, Condition, Element, Event, Simulation, Source, ALLY_OK,
    CAN_BE_CALCULATED, CAN_BE_REFLECTED, MISS_SLEEPING, SILENCEABLE,
};

pub const TALK_SKILL_ABILITIES: &[Ability] = &[
    // TODO: Rehabilitate: 4 range, 0 AoE. Effect: HealMP (MA * 3).
    // Invitation: 4 range, 0 AoE. Hit: (MA + 35)%. Effect: Add Confusion, Charm (Random).
    Ability {
        name: "Invitation",
        flags: FOE_OK | SILENCEABLE | MISS_SLEEPING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionTalkSkillImpl {
            range: 4,
            base_chance: 35,
            add_conditions: &[Condition::Confusion, Condition::Charm],
        },
    },
    // TODO: Persuade: 4 range, 0 AoE. Hit: (MA + 32)%. Effect: Set CT to 0.
    // Praise: 4 range, 0 AoE. Hit: (MA + 80)%. Effect: +5 Brave.
    Ability {
        name: "Praise",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | MISS_SLEEPING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &BraveFaithTalkSkillImpl {
            range: 4,
            base_chance: 80,
            brave_mod: 5,
            faith_mod: 0,
        },
    },
    // Threaten: 4 range, 0 AoE. Hit: (MA + 89)%. Effect: -20 Brave.
    Ability {
        name: "Threaten",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | MISS_SLEEPING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &BraveFaithTalkSkillImpl {
            range: 4,
            base_chance: 89,
            brave_mod: -20,
            faith_mod: 0,
        },
    },
    // Preach: 4 range, 0 AoE. Hit: (MA + 80)%. Effect: +5 Faith.
    Ability {
        name: "Preach",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | MISS_SLEEPING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &BraveFaithTalkSkillImpl {
            range: 4,
            base_chance: 80,
            brave_mod: 0,
            faith_mod: 5,
        },
    },
    // Solution: 4 range, 0 AoE. Hit: (MA + 89)%. Effect: -20 Faith.
    Ability {
        name: "Solution",
        flags: ALLY_OK | FOE_OK | SILENCEABLE | MISS_SLEEPING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &BraveFaithTalkSkillImpl {
            range: 4,
            base_chance: 89,
            brave_mod: 0,
            faith_mod: -20,
        },
    },
    // Death Sentence: 4 range, 0 AoE. Hit: (MA + 32)%. Effect: Add Death Sentence.
    Ability {
        name: "Death Sentence",
        flags: FOE_OK | SILENCEABLE | MISS_SLEEPING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionTalkSkillImpl {
            range: 4,
            base_chance: 32,
            add_conditions: &[Condition::DeathSentence],
        },
    },
    // TODO: Steal Status: 2 range, 0 AoE. Hit: Faith(MA + 163)%. Effect: Cancel statuses on target and Add them to self.
    // Insult: 4 range, 0 AoE. Hit: (MA + 40)%. Effect: Add Berserk.
    Ability {
        name: "Insult",
        flags: FOE_OK | SILENCEABLE | MISS_SLEEPING,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionTalkSkillImpl {
            range: 4,
            base_chance: 40,
            add_conditions: &[Condition::Berserk],
        },
    },
    // Mimic Daravon: 3 range, 1 AoE. Hit: (MA + 40)%. Effect: Add Sleep.
    Ability {
        name: "Mimic Daravon",
        flags: FOE_OK | SILENCEABLE,
        mp_cost: 0,
        aoe: AoE::Diamond(1, Some(2)),
        implementation: &ConditionTalkSkillImpl {
            range: 3,
            base_chance: 40,
            add_conditions: &[Condition::Sleep],
        },
    },
];

pub struct ConditionTalkSkillImpl {
    pub range: u8,
    pub base_chance: i16,
    pub add_conditions: &'static [Condition],
}

impl AbilityImpl for ConditionTalkSkillImpl {
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

        let mut chance = (user.ma() + self.base_chance) as f32 / 100.0;
        chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < chance {
            let index = sim.roll_inclusive(1, self.add_conditions.len() as i16) - 1;
            let condition = self.add_conditions[index as usize];
            sim.add_condition(target_id, condition, Source::Ability);
        }
    }
}

pub struct BraveFaithTalkSkillImpl {
    pub range: u8,
    pub base_chance: i16,
    pub brave_mod: i8,
    pub faith_mod: i8,
}

impl AbilityImpl for BraveFaithTalkSkillImpl {
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

        let mut chance = (user.ma() + self.base_chance) as f32 / 100.0;
        chance *= user.zodiac_compatibility(target);

        if sim.roll_auto_succeed() < chance {
            sim.change_unit_brave(target_id, self.brave_mod, Source::Ability);
            sim.change_unit_faith(target_id, self.faith_mod, Source::Ability);
        }
    }
}
