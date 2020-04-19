use crate::sim::actions::{Ability, AbilityImpl, Action, AoE, FOE_OK};

use crate::sim::{
    Combatant, CombatantId, Condition, Element, Event, Simulation, Source, CAN_BE_CALCULATED,
    CAN_BE_REFLECTED, SILENCEABLE,
};

pub const TALK_SKILL_ABILITIES: &[Ability] = &[
    // TODO: Rehabilitate: 4 range, 0 AoE. Effect: HealMP (MA * 3).
    // Invitation: 4 range, 0 AoE. Hit: (MA + 35)%. Effect: Add Confusion, Charm (Random).
    Ability {
        name: "Invitation",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionTalkSkillImpl {
            range: 4,
            base_chance: 35,
            add_conditions: &[Condition::Confusion, Condition::Charm],
        },
    },
    // TODO: Persuade: 4 range, 0 AoE. Hit: (MA + 32)%. Effect: Set CT to 0.
    // TODO: Praise: 4 range, 0 AoE. Hit: (MA + 80)%. Effect: +5 Brave.
    // TODO: Threaten: 4 range, 0 AoE. Hit: (MA + 89)%. Effect: -20 Brave.
    // TODO: Preach: 4 range, 0 AoE. Hit: (MA + 80)%. Effect: +5 Faith.
    // TODO: Solution: 4 range, 0 AoE. Hit: (MA + 89)%. Effect: -20 Faith.
    // Death Sentence: 4 range, 0 AoE. Hit: (MA + 32)%. Effect: Add Death Sentence.
    Ability {
        name: "Death Sentence",
        flags: FOE_OK,
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
        flags: FOE_OK,
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
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ConditionTalkSkillImpl {
            range: 3,
            base_chance: 40,
            add_conditions: &[Condition::Sleep],
        },
    },
];

struct ConditionTalkSkillImpl {
    range: u8,
    base_chance: i16,
    add_conditions: &'static [Condition],
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
