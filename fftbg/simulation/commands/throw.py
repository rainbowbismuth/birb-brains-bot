from math import floor

from fftbg.equipment import Equipment
from fftbg.simulation.combatant import Combatant


# Miscellaneous properties of THROW:
#
# > Physical attack
# > Acquires the elemental of the thrown weapon. THROW damage is not affected
#   if the caster has an elemental 'Strengthen' attribute, but is affected if
#   the target halves, blocks, absorbs, or is weak to the elemental of the
#   thrown weapon.
# > Cannot be Reflected
# > Can be evaded
# > Triggers Countergrasp reactions
# > Triggers Counter Flood
# > Triggers Catch: If Catch succeeds, the THROW directive is aborted,
#   a 'Caught' message is displayed, and the thrown weapon is added to
#   the catcher's inventory.
# > Does not trigger Counter Magic
# > Affected by Protect and Defense UP
# > NOT affected by Attack UP

def throw_damage(user: Combatant, weapon: Equipment, target: Combatant):
    """

    :param user: The user using the THROW command
    :param weapon: The weapon being thrown
    :param target: The target of the THROW command
    :return: The total damage done
    """
    speed = user.speed

    #   1. If target has Defense UP, then (Sp1 = [Sp0 * 2/3]), else Sp1 = Sp0
    if target.defense_up:
        speed = (speed * 2) // 3

    #   2. If target has Protect, then (Sp2 = [Sp1 * 2/3]), else Sp2 = Sp1
    if target.protect:
        speed = (speed * 2) // 3

    #   3. If target is Charging, then (Sp3 = [Sp2 * 3/2]), else Sp3 = Sp2
    if target.charging:
        speed = (speed * 3) // 2

    #   4. If target is Sleeping, then (Sp4 = [Sp3 * 3/2]), else Sp4 = Sp3
    if target.sleep:
        speed = (speed * 3) // 2

    #   5. If target is a Frog and/or Chicken, then (Sp5 = [Sp4 * 3/2]), else
    #      Sp5 = Sp4
    if target.frog or target.chicken:
        speed = (speed * 3) // 2

    #   6. Apply zodiac multipliers:
    #            If compatibility is 'Good', then (Sp6 = Sp5 + [(Sp5)/4]))
    #            ElseIf compatibility is 'Bad', then (Sp6 = Sp5 - [(Sp5)/4])
    #            ElseIf compatibility is 'Best', then (Sp6 = Sp5 + [(Sp5)/2])
    #            ElseIf compatibility is 'Worst', then (Sp6 = Sp5 - [(Sp5)/2])
    #            Else Sp6 = Sp5
    speed = floor(speed * user.zodiac_compatibility(target))

    #   7. damage0 = Sp6 * (ThrownWpnPwr)
    damage = speed * weapon.wp

    #   8. If target is 'Weak' against the weapon's element, then
    #          damage1 = damage0 * 2
    #        Else, damage1 = damage0
    if target.weak(weapon.weapon_element):
        damage *= 2

    #   9. If target has 'Half' against the weapon's element, then
    #          damage2 = [damage1 / 2]
    #        Else, damage2 = damage1
    if target.halves(weapon.weapon_element):
        damage //= 2

    #  10. If target has 'Absorb' against the weapon's element, then
    #          damage3 = -(damage2)
    #        Else, damage3 = damage2
    if target.absorbs(weapon.weapon_element):
        damage = -damage

    #  11. The damage done by the THROW attack will be equal to damage3.
    return damage
