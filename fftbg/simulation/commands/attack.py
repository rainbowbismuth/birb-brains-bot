import random
from math import floor

from fftbg.equipment import Equipment
from fftbg.simulation.abc.simulation import AbstractSimulation
from fftbg.simulation.action import Action
from fftbg.simulation.combatant import Combatant
from fftbg.simulation.status import DAMAGE_CANCELS


def calculate_damage(user: Combatant, weapon: Equipment, target: Combatant, k=0):
    """

    :param user: The user making the ATTACK
    :param weapon: The weapon being used
    :param target: The target of the ATTACK
    :param k: Archer's CHARGE command uses this same formula, with k equal to charge amount
    :return: The total damage done, and a critical hit flag
    """
    # FIXME: These modifiers do not apply to magical guns
    xa = user.calculate_weapon_xa(weapon, k)
    critical_hit = False

    # 1. If this is a critical hit, then XA1 = XA0 + (1..XA0) - 1
    #       else XA1 = XA0
    #       (See section 2.1 for details)
    if random.randint(1, 20) == 20:
        xa += random.randint(1, xa) - 1
        critical_hit = True

    #    2. If the weapon is endowed with an element, and the attacker has
    #       equipment that 'Strengthens' that element, then (XA2 = [XA1 * 5/4]),
    #       else XA2 = XA1
    if user.strengthens(weapon.weapon_element):
        xa = (xa * 5) // 4

    #    3. If attacker has Attack UP, then (XA3 = [XA2 * 4/3]), else XA3 = XA2
    if user.attack_up:
        xa = (xa * 4) // 3

    #    4. If attacker has Martial Arts and is barehanded, then
    #       (XA4 = [XA3 * 3/2]), else XA4 = XA3
    if user.barehanded and user.martial_arts:
        xa = (xa * 3) // 2

    #    5. If attacker is Berserk, then (XA5 = [XA4 * 3/2]), else XA5 = XA4
    if user.berserk:
        xa = (xa * 3) // 2

    #    6. If target has Defense UP, then (XA6 = [XA5 * 2/3]), else XA6 = XA5
    if target.defense_up:
        xa = (xa * 2) // 3

    #    7. If target has Protect, then (XA7 = [XA6 * 2/3]), else XA7 = XA6
    if target.protect:
        xa = (xa * 2) // 3

    #    8. If target is Charging, then (XA8 = [XA7 * 3/2]), else XA8 = XA7
    if target.charging:
        xa = (xa * 3) // 2

    #    9. If target is Sleeping, then (XA9 = [XA8 * 3/2]), else XA9 = XA8
    if target.sleep:
        xa = (xa * 3) // 2

    #   10. If target is a Chicken and/or a Frog, then (XA10 = [XA9 * 3/2]),
    #       else XA10 = XA9
    if target.chicken or target.frog:
        xa = (xa * 3) // 2

    #   11. Apply zodiac multipliers:
    #           If compatibility is 'Good', then (XA11 = XA10 + [(XA10)/4]))
    #           elseIf compatibility is 'Bad', then (XA11 = XA10 - [(XA10)/4])
    #           elseIf compatibility is 'Best', then (XA11 = XA10 + [(XA10)/2])
    #           elseIf compatibility is 'Worst', then (XA11 = XA10 - [(XA10)/2])
    #           else XA11 = XA10
    xa = floor(xa * user.zodiac_compatibility(target))

    #   12. Apply weapon's damage formula using XA = XA11 (if there is more
    #       than one instance of XA, only set _one_ instance to XA11 and
    #       leave the other as XA0 (see above). The result of the formula
    #       is equal to damage0.
    if weapon.weapon_type is None:
        damage = xa * user.pa_bang
    else:
        damage = xa * weapon.wp

        if user.double_hand and weapon.weapon_type != 'Gun':
            damage *= 2

    #   13. If target is 'Weak' against the weapon's element, then
    #         damage1 = damage0 * 2
    #       Else, damage1 = damage0
    if target.weak(weapon.weapon_element):
        damage *= 2

    #   14. If target has 'Half' against the weapon's element, then
    #         damage2 = [damage1 / 2]
    #       Else, damage2 = damage1
    if target.halves(weapon.weapon_element):
        damage //= 2

    #   15. If target has 'Absorb' against the weapon's element, then
    #         damage3 = -(damage2)
    #       Else, damage3 = damage2
    if target.absorbs(weapon.weapon_element):
        damage = -damage

    #   16. The damage done by the attack will be equal to damage3.
    return damage, critical_hit


def should_attack_ally(user: Combatant, target: Combatant) -> bool:
    if any(target.has_status(status) for status in DAMAGE_CANCELS):
        return True
    if target.absorbs(user.mainhand.weapon_element):
        return True
    return False


def should_attack_foe(user: Combatant, target: Combatant) -> bool:
    if target.absorbs(user.mainhand.weapon_element):
        return False
    if target.charm:
        # TODO: There should be more complicated logic here.
        return False
    return True


def consider_attack(sim: AbstractSimulation, user: Combatant, target: Combatant):
    if target.dead or target.crystal or target.petrified:
        return

    if user.is_friend(target) and not should_attack_ally(user, target):
        return

    if user.is_foe(target) and not should_attack_foe(user, target):
        return

    yield Action(
        range=user.mainhand.range,
        target=target,
        perform=lambda: do_cmd_attack(sim, user, target))


def do_cmd_attack(sim: AbstractSimulation, user: Combatant, target: Combatant):
    user.acted_during_active_turn = True
    damage, crit = do_single_weapon_attack(sim, user, user.mainhand, target)
    if user.dual_wield and user.has_offhand_weapon and target.healthy:
        if crit and random.randint(1, 2) == 1:
            sim.unit_report(target, 'was pushed out of range of a second attack')
        else:
            damage, crit = do_single_weapon_attack(sim, user, user.offhand, target)
    if damage > 0:
        sim.after_damage_reaction(target, user, damage)


def do_single_weapon_attack(
        sim: AbstractSimulation, user: Combatant, weapon: Equipment, target: Combatant) -> (int, bool):
    if not sim.in_range(user, weapon.range, target):
        sim.unit_report(target, 'not in range!')
        return 0, False

    if sim.do_physical_evade(user, weapon, target):
        return 0, False

    damage, crit = calculate_damage(user, weapon, target)
    if not crit:
        src = f'{user.name}\'s {weapon.weapon_name}'
    else:
        src = f'{user.name}\'s {weapon.weapon_name} (critical!)'
    sim.change_target_hp(target, damage, src)
    sim.weapon_chance_to_add_or_cancel_status(user, weapon, target)
    return damage, crit
