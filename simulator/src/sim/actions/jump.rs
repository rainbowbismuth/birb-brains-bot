use crate::sim::actions::{
    Ability, AbilityImpl, Action, AoE, ALLY_OK, FOE_OK, NO_SHORT_CHARGE, TARGET_NOT_SELF,
};
use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, WeaponType, DAMAGE_CANCELS,
    HITS_ALLIES_ONLY, HITS_FOES_ONLY, JUMPING, NOT_ALIVE_OK, SILENCEABLE, TARGET_SELF_ONLY,
};

pub const JUMP_ABILITIES: &[Ability] = &[Ability {
    flags: TARGET_NOT_SELF | ALLY_OK | FOE_OK | NO_SHORT_CHARGE | JUMPING,
    mp_cost: 0,
    aoe: AoE::None,
    implementation: &JumpImpl {},
    name: "Jump",
}];

struct JumpImpl {}

impl AbilityImpl for JumpImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.ally(target)
            && !DAMAGE_CANCELS
                .iter()
                .any(|condition| target.has_condition(*condition))
        {
            return;
        }
        let ct_remaining = 0.max(100 - target.ct.min(100));
        let speed = if target.haste() {
            // TODO: Real AI doesn't account for this, but, since I haven't implemented
            //  tile targeting, I'm going to only target those that will certainly hit
            (target.speed() * 3) / 2
        } else {
            target.speed()
        };
        let ticks_left = ct_remaining / speed.max(1) as u8;
        let jump_ticks = 50 / user.speed().max(1) as u8;

        if jump_ticks >= ticks_left {
            return;
        }

        actions.push(Action::new(
            ability,
            user.info.horizontal_jump as u8,
            Some(jump_ticks),
            target.id(),
        ));
    }
    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        sim.cancel_condition(user_id, Condition::Jumping, Source::Ability);

        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);

        let mut xa = user.pa() as i16;

        if target.defense_up() {
            xa = (xa * 2) / 3;
        }

        if target.protect() {
            xa = (xa * 2) / 3;
        }

        if target.charging() {
            xa = (xa * 3) / 2;
        }

        if target.sleep() {
            xa = (xa * 3) / 2;
        }

        if target.frog() || target.chicken() {
            xa = (xa * 3) / 2;
        }

        if user
            .main_hand()
            .and_then(|wp| wp.weapon_type)
            .map_or(false, |ty| ty == WeaponType::Spear)
        {
            xa = (xa * 3) / 2;
        }

        xa = (xa as f32 * user.zodiac_compatibility(target)) as i16;

        let damage = if user.barehanded() {
            xa * (user.pa_bang() as f32 * user.brave_percent()) as i16
        } else {
            xa * user.main_hand().unwrap().wp as i16
        };

        sim.change_target_hp(target_id, damage, Source::Ability);
        sim.try_countergrasp(user_id, target_id);
    }
}
