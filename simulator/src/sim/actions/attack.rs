use crate::dto::rust::Equipment;
use crate::sim::actions::{Action, ActionKind};
use crate::sim::{Combatant, CombatantId, Simulation, Source, WeaponType, DAMAGE_CANCELS};

fn should_attack_ally(user: &Combatant, target: &Combatant) -> bool {
    if DAMAGE_CANCELS
        .iter()
        .any(|condition| target.has_condition(*condition))
    {
        return true;
    }
    if let Some(element) = user.main_hand().and_then(|w| w.weapon_element) {
        target.absorbs(element)
    } else {
        false
    }
}

fn should_attack_foe(user: &Combatant, target: &Combatant) -> bool {
    if let Some(element) = user.main_hand().and_then(|w| w.weapon_element) {
        if target.absorbs(element) {
            return false;
        }
    }
    // TODO: There should be more complicated logic here.
    !target.charm()
}

pub fn consider_attack(
    actions: &mut Vec<Action>,
    _sim: &Simulation,
    user: &Combatant,
    target: &Combatant,
) {
    if target.dead() || target.crystal() || target.petrify() {
        return;
    }
    if user.same_team(target) && !should_attack_ally(user, target) {
        return;
    }
    if user.different_team(target) && !should_attack_foe(user, target) {
        return;
    }

    if user.frog() || user.berserk() && user.monster() {
        actions.push(Action {
            kind: ActionKind::FrogAttack,
            range: 1,
            target_id: target.id(),
        });
    } else {
        actions.push(Action {
            kind: ActionKind::Attack,
            range: user.main_hand().map_or(1, |eq| eq.range),
            target_id: target.id(),
        });
    }
}

pub fn perform_attack(sim: &mut Simulation, user_id: CombatantId, target_id: CombatantId) {
    let weapon1 = sim.combatant(user_id).main_hand();
    let weapon2 = sim.combatant(user_id).off_hand();
    let (mut damage, mut crit) = do_single_weapon_attack(sim, user_id, weapon1, target_id);
    if sim.combatant(user_id).dual_wield() && weapon2.is_some() {
        if sim.roll_auto_succeed() < 0.05 {
            let pair = do_single_weapon_attack(sim, user_id, weapon2, target_id);
            damage = pair.0;
            crit = pair.1;
        }
    }
    if damage > 0 {
        sim.after_damage_reaction(user_id, target_id, damage);
    }
}

pub fn perform_frog_attack(sim: &mut Simulation, user_id: CombatantId, target_id: CombatantId) {
    let pa = sim.combatant(user_id).pa_bang();
    sim.change_target_hp(target_id, pa.into(), Source::Weapon(user_id, None));
    sim.after_damage_reaction(user_id, target_id, pa.into());
}

fn do_single_weapon_attack<'a, 'b>(
    sim: &'a mut Simulation<'b>,
    user_id: CombatantId,
    weapon: Option<&'b Equipment>,
    target_id: CombatantId,
) -> (i16, bool) {
    let user = sim.combatant(user_id);
    let target = sim.combatant(target_id);
    let src = Source::Weapon(user_id, weapon);
    if sim.do_physical_evade(user, target, src) {
        return (0, false);
    }
    let (damage, crit) = calculate_damage(sim, user, weapon, target, 0);
    sim.change_target_hp(target_id, damage, src);
    sim.weapon_chance_to_add_or_cancel_status(user_id, weapon, target_id);
    (damage, crit)
}

fn calculate_damage<'a, 'b>(
    sim: &'a Simulation<'b>,
    user: &Combatant,
    weapon: Option<&'b Equipment>,
    target: &Combatant,
    k: i16,
) -> (i16, bool) {
    // FIXME: These modifiers do not apply to magical guns
    let mut critical_hit = false;
    let mut xa = sim.calculate_weapon_xa(user, weapon, k);
    let mut damage = 0;

    if sim.roll_auto_fail() <= 0.05 {
        xa += sim.roll_inclusive(1, xa.max(1)) - 1;
        critical_hit = true;
    }

    if let Some(element) = weapon.and_then(|w| w.weapon_element) {
        if user.strengthens(element) {
            xa = (xa * 5) / 4;
        }
    }

    if user.attack_up() {
        xa = (xa * 4) / 3;
    }

    if user.barehanded() && user.martial_arts() {
        xa = (xa * 3) / 2;
    }

    if user.berserk() {
        xa = (xa * 3) / 2;
    }

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

    if target.chicken() || target.frog() {
        xa = (xa * 3) / 2;
    }

    // TODO: ZODIAC
    //     #   11. Apply zodiac multipliers:
    //     #           If compatibility is 'Good', then (XA11 = XA10 + [(XA10)/4]))
    //     #           elseIf compatibility is 'Bad', then (XA11 = XA10 - [(XA10)/4])
    //     #           elseIf compatibility is 'Best', then (XA11 = XA10 + [(XA10)/2])
    //     #           elseIf compatibility is 'Worst', then (XA11 = XA10 - [(XA10)/2])
    //     #           else XA11 = XA10
    //     xa = floor(xa * user.zodiac_compatibility(target))

    if user.barehanded() {
        damage = xa * user.pa_bang() as i16;
    } else {
        let weapon = weapon.unwrap();
        damage = xa * weapon.wp as i16;

        if user.double_hand() && weapon.weapon_type != Some(WeaponType::Gun) {
            damage *= 2;
        }
    }

    if let Some(element) = weapon.and_then(|w| w.weapon_element) {
        if user.weak(element) {
            damage *= 2;
        }
        if target.halves(element) {
            damage /= 2;
        }
        if target.absorbs(element) {
            damage = -damage;
        }
    }

    (damage, critical_hit)
}
