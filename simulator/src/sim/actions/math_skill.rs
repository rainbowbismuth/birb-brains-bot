use crate::sim::actions::{Ability, AbilityImpl, Action, AoE, ALLY_OK};
use crate::sim::{
    ActionTarget, CalcAlgorithm, CalcAttribute, Combatant, CombatantId, Condition, EquipSlot,
    Event, Simulation, Source, WeaponType, CAN_BE_CALCULATED, SILENCEABLE, TARGET_SELF_ONLY,
};

pub const MATH_SKILL_ABILITY: Ability = Ability {
    flags: ALLY_OK | TARGET_SELF_ONLY | SILENCEABLE,
    mp_cost: 0,
    aoe: AoE::None,
    implementation: &MathSkillImpl {},
    name: "Math Skill",
};

struct MathSkillImpl {}

impl AbilityImpl for MathSkillImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        _ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        _target: &Combatant<'a>,
    ) {
        for ability in user.abilities() {
            if ability.flags & CAN_BE_CALCULATED == 0 {
                continue;
            }
            if user.info.known_calc_attributes & CalcAttribute::Height.flag() != 0 {
                add_with_attr(actions, ability, user, CalcAttribute::Height);
            }
            if user.info.known_calc_attributes & CalcAttribute::CT.flag() != 0 {
                add_with_attr(actions, ability, user, CalcAttribute::CT);
            }
        }
    }

    fn perform<'a>(
        &self,
        _sim: &mut Simulation<'a>,
        _user_id: CombatantId,
        _target_id: CombatantId,
    ) {
        panic!("Math Skill itself is never performed");
    }
}

fn add_with_attr<'a>(
    actions: &mut Vec<Action<'a>>,
    ability: &'a Ability<'a>,
    user: &Combatant<'a>,
    attr: CalcAttribute,
) {
    try_add_with_algo(actions, ability, user, attr, CalcAlgorithm::Prime);
    try_add_with_algo(actions, ability, user, attr, CalcAlgorithm::M5);
    try_add_with_algo(actions, ability, user, attr, CalcAlgorithm::M4);
    try_add_with_algo(actions, ability, user, attr, CalcAlgorithm::M3);
}

fn try_add_with_algo<'a>(
    actions: &mut Vec<Action<'a>>,
    ability: &'a Ability<'a>,
    user: &Combatant<'a>,
    attr: CalcAttribute,
    algo: CalcAlgorithm,
) {
    if user.info.known_calc_algorithms & algo.flag() != 0 {
        actions.push(Action {
            ability,
            range: 0,
            ctr: None,
            target: ActionTarget::Math(attr, algo),
        })
    }
}
