use crate::sim::actions::common::AddConditionSpellImpl;
use crate::sim::actions::punch_art::Pummel;
use crate::sim::actions::talk_skill::ConditionTalkSkillImpl;
use crate::sim::actions::{Ability, AbilityImpl, Action, ALLY_OK, FOE_OK};
use crate::sim::attack::AttackImpl;
use crate::sim::common::{do_hp_heal, mod_2_formula_xa, mod_5_formula_xa};
use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, TARGET_NOT_SELF,
    TARGET_SELF_ONLY, TRIGGERS_HAMEDO,
};

pub const TRENT_ABILITIES: &[Ability] = &[];
