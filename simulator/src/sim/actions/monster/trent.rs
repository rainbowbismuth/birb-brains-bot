use crate::sim::actions::common::AddConditionSpellImpl;
use crate::sim::actions::monster::ChocoMeteorImpl;
use crate::sim::actions::punch_art::Pummel;
use crate::sim::actions::talk_skill::ConditionTalkSkillImpl;
use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::attack::AttackImpl;
use crate::sim::common::{do_hp_heal, mod_2_formula_xa, mod_5_formula_xa};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, HITS_ALLIES_ONLY,
    HITS_FOES_ONLY, TARGET_NOT_SELF, TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const TRENT_ABILITIES: &[Ability] = &[
    // Leaf Dance: 0 range, 1 AoE. Effect: Damage (MA * 5).
    Ability {
        name: "Leaf Dance",
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_FOES_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(1, Some(0)),
        implementation: &ChocoMeteorImpl {
            ma_factor: 5,
            range: 0,
        },
    },
    // Protect Spirit: 0 range, 2 AoE. Hit: (MA + 60)%. Effect: Add Defending, Protect (All).
    Ability {
        name: "Protect Spirit",
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(0)),
        implementation: &SpiritCondition {
            range: 0,
            base_chance: 60,
            add_conditions: &[Condition::Defending, Condition::Protect],
        },
    },
    // Calm Spirit: 0 range, 2 AoE. Hit: (MA + 60)%. Effect: Add Defending, Shell (All).
    Ability {
        name: "Calm Spirit",
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(0)),
        implementation: &SpiritCondition {
            range: 0,
            base_chance: 60,
            add_conditions: &[Condition::Defending, Condition::Shell],
        },
    },
    // Life Spirit: 0 range, 2 AoE. Effect: Heal (MA * 4).
    Ability {
        name: "Life Spirit",
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(0)),
        implementation: &ChocoMeteorImpl {
            ma_factor: -4,
            range: 0,
        },
    },
    // Spirit of Life: 0 range, 2 AoE. Effect: Heal (MA * 4).
    Ability {
        name: "Spirit of Life",
        flags: TARGET_SELF_ONLY | ALLY_OK | HITS_ALLIES_ONLY,
        mp_cost: 0,
        aoe: AoE::Diamond(2, Some(0)),
        implementation: &ChocoMeteorImpl {
            ma_factor: -4,
            range: 0,
        },
    },
    // TODO: Magic Spirit: 0 range, 2 AoE. Effect: HealMP (MA * 2).
];

pub struct SpiritCondition {
    pub range: u8,
    pub base_chance: i16,
    pub add_conditions: &'static [Condition],
}

impl AbilityImpl for SpiritCondition {
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
            for condition in self.add_conditions {
                sim.add_condition(target_id, *condition, Source::Ability);
            }
        }
    }
}
