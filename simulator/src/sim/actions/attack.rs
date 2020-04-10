use crate::dto::rust::Equipment;
use crate::sim::actions::common::mod_5_formula_pass_ma;
use crate::sim::actions::{
    Ability, AbilityImpl, Action, ALLY_OK, BERSERK_OK, FOE_OK, FROG_OK, TARGET_NOT_SELF,
};
use crate::sim::{Combatant, CombatantId, Simulation, Source, WeaponType, DAMAGE_CANCELS};

pub const ATTACK_ABILITY: Ability = Ability {
    flags: BERSERK_OK | FROG_OK | ALLY_OK | FOE_OK | TARGET_NOT_SELF,
    mp_cost: 0,
    aoe: None,
    implementation: &AttackImpl {},
    name: "Attack",
};

pub struct AttackImpl {}

impl AbilityImpl for AttackImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        _sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        if user.ally(target) && !should_attack_ally(user, target) {
            return;
        }
        if user.foe(target) && !should_attack_foe(user, target) {
            return;
        }
        if user.frog() || user.berserk() && user.monster() {
            actions.push(Action {
                ability,
                range: attack_range(user),
                ctr: None,
                target_id: target.id(),
            });
        } else {
            actions.push(Action {
                ability,
                range: attack_range(user),
                ctr: None,
                target_id: target.id(),
            });
        }
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        if user.frog() || user.berserk() && user.monster() {
            perform_frog_attack(sim, user_id, target_id);
        } else {
            perform_attack(sim, user_id, target_id);
        }
    }
}

pub fn attack_range(user: &Combatant) -> i8 {
    if user.frog() || user.berserk() && user.monster() {
        1
    } else {
        user.main_hand().map_or(1, |eq| eq.range)
    }
}

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
    true
}

fn perform_attack(sim: &mut Simulation, user_id: CombatantId, target_id: CombatantId) {
    let weapon1 = sim.combatant(user_id).main_hand();
    let weapon2 = sim.combatant(user_id).off_hand();
    let (mut damage, mut crit) = do_single_weapon_attack(sim, user_id, weapon1, target_id);
    let mut knockback = false;

    if crit && sim.roll_auto_fail() < 0.50 {
        sim.do_knockback(user_id, target_id);
        knockback = true;
    }

    if sim.combatant(user_id).dual_wield()
        && weapon2.is_some()
        // FIXME: This condition is a little bit of a cheat :)
        && (!knockback || attack_range(sim.combatant(user_id)) > 1)
    {
        let pair = do_single_weapon_attack(sim, user_id, weapon2, target_id);
        damage = pair.0;
        crit = pair.1;
    }

    sim.try_countergrasp(user_id, target_id);
    if damage > 0 {
        sim.after_damage_reaction(user_id, target_id, damage);
    }
}

fn perform_frog_attack(sim: &mut Simulation, user_id: CombatantId, target_id: CombatantId) {
    let pa = sim.combatant(user_id).pa_bang();
    sim.change_target_hp(target_id, pa.into(), Source::Weapon(user_id, None));
    sim.try_countergrasp(user_id, target_id);
    sim.after_damage_reaction(user_id, target_id, pa.into());
}

fn do_single_weapon_attack<'a, 'b>(
    sim: &'a mut Simulation<'b>,
    user_id: CombatantId,
    weapon: Option<&'b Equipment>,
    target_id: CombatantId,
) -> (i16, bool) {
    let is_gun = weapon.map_or(false, |eq| eq.weapon_type == Some(WeaponType::Gun));

    if let Some(weapon) = weapon {
        if is_gun && weapon.weapon_element.is_some() {
            return do_single_magical_gun_attack(sim, user_id, weapon, target_id);
        }
    }

    let user = sim.combatant(user_id);
    let target = sim.combatant(target_id);
    let src = Source::Weapon(user_id, weapon);

    if !is_gun && sim.do_physical_evade(user, target, src) {
        return (0, false);
    }

    let (damage, crit) = calculate_damage(sim, user, weapon, target, 0);
    sim.change_target_hp(target_id, damage, src);
    sim.weapon_chance_to_add_or_cancel_status(user_id, weapon, target_id);
    if weapon.map_or(false, |eq| eq.absorbs_hp) {
        sim.change_target_hp(user_id, -damage, src);
    }
    (damage, crit)
}

fn calculate_damage<'a, 'b>(
    sim: &'a Simulation<'b>,
    user: &Combatant,
    weapon: Option<&'b Equipment>,
    target: &Combatant,
    k: i16,
) -> (i16, bool) {
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

    xa = (xa as f32 * user.zodiac_compatibility(target)).floor() as i16;

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

// Blaze Gun: 13 WP, 7 range, 4% evade, Gun. Element: Fire.
// Glacier Gun: 13 WP, 7 range, 4% evade, Gun. Element: Ice.
// Blast Gun: 13 WP, 7 range, 4% evade, Gun. Element: Lightning.
fn do_single_magical_gun_attack<'a, 'b>(
    sim: &'a mut Simulation<'b>,
    user_id: CombatantId,
    weapon: &'b Equipment,
    target_id: CombatantId,
) -> (i16, bool) {
    let user = sim.combatant(user_id);
    let target = sim.combatant(target_id);

    let spell_strength = sim.roll_inclusive(1, 10);
    let q = if spell_strength <= 6 {
        14
    } else if spell_strength <= 9 {
        18
    } else {
        24
    };

    let damage = mod_5_formula_pass_ma(
        weapon.wp as i16,
        user,
        target,
        weapon.weapon_element.unwrap(),
        q,
    );
    (damage, false)
}
