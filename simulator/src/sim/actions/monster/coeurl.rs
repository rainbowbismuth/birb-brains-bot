use crate::sim::actions::punch_art::Pummel;
use crate::sim::actions::talk_skill::ConditionTalkSkillImpl;
use crate::sim::actions::{Ability, ALLY_OK, FOE_OK};

use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const COEURL_ABILITIES: &[Ability] = &[
    // TODO: Scratch: 1 range, 0 AoE. Effect: Normal Attack.
    // Cat Kick: 1 range, 0 AoE. Effect: Damage (Random(1,8) * PA); Chance to Knockback.
    Ability {
        name: "Cat Kick",
        flags: ALLY_OK | FOE_OK | TARGET_NOT_SELF,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &Pummel {
            max: 8,
            pa_mult: 1.0,
            knockback_chance: true,
        },
    },
    // Blaster: 3 range, 0 AoE. Hit: (MA + 35)%. Effect: Add Petrify, Stop, Sleep (Random).
    Ability {
        name: "Blaster",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionTalkSkillImpl {
            range: 3,
            base_chance: 35,
            add_conditions: &[Condition::Petrify, Condition::Stop, Condition::Sleep],
        },
    },
    // Poison Nail: 1 range, 0 AoE. Hit: (MA + 55)%. Effect: Add Poison.
    Ability {
        name: "Poison Nail",
        flags: FOE_OK,
        mp_cost: 0,
        aoe: AoE::None,
        implementation: &ConditionTalkSkillImpl {
            range: 1,
            base_chance: 55,
            add_conditions: &[Condition::Poison],
        },
    },
    // TODO: Blood Suck: 1 range, 0 AoE. Effect: AbsorbHP (25)%; Chance to Add Blood Suck.
];
