from math import floor

from fftbg.equipment import Equipment
from fftbg.simulation.combatant import Combatant


# Miscellaneous properties of JUMP:
#
# > Physical attack
# > non-elemental, regardless of weapon
# > Cannot be Reflected
# > Cannot be evaded
# > Triggers Countergrasp reactions
# > Triggers Counter Flood
# > Does not trigger Counter Magic
# > Affected by Protect and Defense UP
# > NOT affected by Attack UP or Martial Arts
# > Can only target panels, not specific units
# > The (PA * WP) component of the damage formula remains the same regardless
#   of the weapon equipped -- even if that weapon does not use this formula
#   to calculate its ATTACK damage (e.g. staves, dictionaries, knives, guns).
# > Jump +X and Ignore Height have NO effect on the JUMP command -- these
#   abilities influence the Jump statistic, which is completely different
#   from the JUMP action ability.

def jump_damage(user: Combatant, weapon: Equipment, target: Combatant):
    """

    :param user: The user making the JUMP
    :param weapon: The weapon the user is using
    :param target: The target of the JUMP command
    :return: The total damage done
    """
    pa = user.pa

    #  1. If target has Defense UP, then (PA1 = [PA0 * 2/3]), else PA1 = PA0
    if target.defense_up:
        pa = (pa * 2) // 3

    #  2. If target has Protect, then (PA2 = [PA1 * 2/3]), else PA2 = PA1
    if target.protect:
        pa = (pa * 2) // 3

    #  3. If target is Charging, then (PA3 = [PA2 * 3/2]), else PA3 = PA2
    if target.charging:
        pa = (pa * 3) // 2

    #  4. If target is Sleeping, then (PA4 = [PA3 * 3/2]), else PA4 = PA3
    if target.sleep:
        pa = (pa * 3) // 2

    #  5. If target is a Frog and/or Chicken, then (PA5 = [PA4 * 3/2]), else
    #     PA5 = PA4
    if target.frog or target.chicken:
        pa = (pa * 3) // 2

    #  6. If caster is equipped with a spear, then (PA6 = [PA5 * 3/2]), else
    #     PA6 = PA5
    if weapon.weapon_type == 'Spear':
        pa = (pa * 3) // 2

    #  7. Factor in zodiac compatibility:
    #           If compatibility is 'Good', then (PA7 = PA6 + [(PA6)/4]))
    #           ElseIf compatibility is 'Bad', then (PA7 = PA6 - [(PA6)/4])
    #           ElseIf compatibility is 'Best', then (PA7 = PA6 + [(PA6)/2])
    #           ElseIf compatibility is 'Worst', then (PA7 = PA6 - [(PA6)/2])
    #           Else PA7 = PA6
    pa = floor(pa * user.zodiac_compatibility(target))

    #  7. Damage = PA7 * WP (armed) or damage = PA7 * [PA0 * Br/100] (unarmed)
    if user.barehanded:
        damage = floor(pa * user.brave) * user.pa_bang
    else:
        damage = pa * weapon.wp

    return damage
