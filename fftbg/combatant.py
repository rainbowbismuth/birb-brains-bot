from math import floor

import numpy as np

import fftbg.ability as ability
import fftbg.base_stats as base_stats
import fftbg.equipment as equipment
from fftbg.ability import SKILL_TAG

PER_COMBATANTS = ['Name', 'Gender', 'Sign', 'Class', 'ActionSkill', 'SupportSkill', 'MoveSkill',
                  'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']
NUMERIC = ['Brave', 'Faith', 'HP', 'MP', 'Speed', 'Range', 'Ability-Range',
           'PA', 'MA', 'Move', 'Jump', 'Physical Evade', 'Magical Evade']
CATEGORICAL = PER_COMBATANTS + ['Color', 'Side', 'Map']


def combatant_to_dict(combatant: dict):
    output = dict(combatant)
    del output['ClassSkills']
    del output['ExtraSkills']

    # Compute stats
    stats = base_stats.get_base_stats(combatant['Class'], combatant['Gender'])
    mainhand = equipment.get_equipment(combatant['Mainhand'])
    offhand = equipment.get_equipment(combatant['Offhand'])
    headgear = equipment.get_equipment(combatant['Head'])
    armor = equipment.get_equipment(combatant['Armor'])
    accessory = equipment.get_equipment(combatant['Accessory'])
    all_equips = [mainhand, offhand, headgear, armor, accessory]

    output['HP'] = stats.hp + sum([e.hp_bonus for e in all_equips])
    output['MP'] = stats.mp + sum([e.mp_bonus for e in all_equips])
    output['Speed'] = stats.speed + sum([e.speed_bonus for e in all_equips])
    output['Range'] = max([e.range for e in all_equips])
    output['W-EV'] = max([e.w_ev for e in all_equips]) / 100.0
    output['C-EV'] = stats.c_ev / 100.0
    output['Move'] = stats.move + sum([e.move_bonus for e in all_equips])
    output['Jump'] = stats.jump + sum([e.jump_bonus for e in all_equips])
    output['Physical S-EV'] = sum([e.phys_ev for e in (mainhand, offhand)]) / 100.0
    output['Magical S-EV'] = sum([e.phys_ev for e in (mainhand, offhand)]) / 100.0

    if combatant['MoveSkill'][:-1] == 'Move+':
        output['Move'] += int(combatant['MoveSkill'][-1])
        output['MoveSkill'] = ''
    if combatant['MoveSkill'][:-1] == 'Jump+':
        output['Jump'] += int(combatant['MoveSkill'][-1])
        output['MoveSkill'] = ''

    output['PA!'] = stats.pa
    output['PA'] = stats.pa + sum([e.pa_bonus for e in all_equips])
    output['MA!'] = stats.ma
    output['MA'] = stats.ma + sum([e.ma_bonus for e in all_equips])
    output['Physical A-EV'] = accessory.phys_ev / 100.0
    output['Magical A-EV'] = accessory.magic_ev / 100.0

    if output['ReactionSkill'] == 'Abandon':
        output['Physical S-EV'] *= 2
        output['Magical S-EV'] *= 2
        output['Physical A-EV'] *= 2
        output['Magical A-EV'] *= 2
        output['C-EV'] *= 2
        output['ReactionSkill'] = ''

    if output['ReactionSkill'] != 'Parry':
        output['W-EV'] = 0.0
    else:
        output['ReactionSkill'] = ''

    physical_evasions = [
        output['W-EV'], output['C-EV'] * 3.0 / 4.0, output['Physical S-EV'] / 2.0, output['Physical A-EV']]
    output['Physical Evade'] = 1.0 - np.product([1 - p for p in physical_evasions])
    assert 0 <= output['Physical Evade'] < 1.0

    magical_evasions = [
        output['Magical S-EV'] / 2.0, output['Magical A-EV']]
    output['Magical Evade'] = 1.0 - np.product([1 - p for p in magical_evasions])
    assert 0 <= output['Magical Evade'] < 1.0

    absorbs = set(stats.absorbs)
    halves = set(stats.halves)
    weaknesses = set(stats.weaknesses)
    cancels = set(stats.cancels)
    strengthens = set()
    chance_to_add = set()
    chance_to_cancel = set()
    immune_to = set()

    for equip in all_equips:
        absorbs.update(equip.absorbs)
        halves.update(equip.halves)
        weaknesses.update(equip.weaknesses)
        cancels.update(equip.cancels)
        strengthens.update(equip.strengthens)
        chance_to_add.update(equip.chance_to_add)
        chance_to_cancel.update(equip.chance_to_cancel)
        immune_to.update(equip.immune_to)

    for element in absorbs:
        output[f'Absorb-{element}'] = 1.0
    for element in halves:
        output[f'Half-{element}'] = 1.0
    for element in weaknesses:
        output[f'Weak-{element}'] = 1.0
    for element in cancels:
        output[f'Cancel-{element}'] = 1.0
    for element in strengthens:
        output[f'Strengthen-{element}'] = 1.0
    for status in chance_to_add:
        output[f'Chance-To-Add-{status}'] = 1.0
    for status in immune_to:
        output[f'Immune-{status}'] = 1.0

    damage_1 = damage_calculation(output, mainhand)
    damage_2 = 0
    if offhand.weapon_type is not None:
        damage_2 = damage_calculation(output, offhand)

    output['Attack-Damage'] = damage_1 + damage_2

    # Skill effectiveness calculations:
    brave = output['Brave'] / 100.0
    faith = output['Faith'] / 100.0
    pa = output['PA']
    pa_bang = output['PA!']
    ma = output['MA']
    wp = max(mainhand.wp, offhand.wp)  # no idea here, ask B.M.G.
    speed = output['Speed']

    output['Ability-Range'] = 0.0
    for skill in combatant['ClassSkills'] + combatant['ExtraSkills']:
        calc = ability.get_ability(skill)
        output['Ability-Range'] = max(output['Ability-Range'], calc.range)
        output[SKILL_TAG + skill] = calc.multiply(brave, faith, pa, pa_bang, ma, wp, speed)

    output[SKILL_TAG + combatant['ReactionSkill']] = brave
    del output['ReactionSkill']

    return output


def damage_calculation(combatant, weapon: equipment.Equipment):
    pa = combatant['PA']
    pa_bang = combatant['PA!']
    ma_bang = combatant['MA!']
    br = combatant['Brave'] / 100.0
    wp = weapon.wp
    sp = combatant['Speed']

    element = weapon.weapon_element
    if element and combatant.get(f'Strengthen-{element}'):
        pa_bang = (pa_bang * 5) // 4
        ma_bang = (ma_bang * 5) // 4

    if combatant['SupportSkill'] == 'Doublehand':
        wp *= 2
    if combatant['SupportSkill'] == 'Attack UP':
        pa_bang = (pa_bang * 4) // 3

    # TODO: Fix magical guns, they are special.

    if weapon.weapon_type is None or combatant['Gender'] == 'Monster':
        # Bare Hands case
        if combatant['SupportSkill'] == 'Martial Arts':
            pa_bang = (pa_bang * 3) // 2
        return floor(pa * br) * pa_bang
    if weapon.weapon_type in ('Knife', 'Ninja Sword', 'Ninja Blade', 'Longbow', 'Bow'):
        return ((pa_bang + sp) // 2) * wp
    if weapon.weapon_type in ('Knight Sword', 'Katana'):
        return floor(pa_bang * br) * wp
    if weapon.weapon_type in ('Sword', 'Rod', 'Pole', 'Spear', 'Crossbow'):
        return pa_bang * wp
    if weapon.weapon_type in ('Staff', 'Stick'):
        return ma_bang * wp
    if weapon.weapon_type in ('Flail', 'Axe', 'Bag'):
        return (pa_bang / 2) * wp
    if weapon.weapon_type in ('Cloth', 'Fabric', 'Musical Instrument', 'Harp', 'Dictionary', 'Book'):
        return ((pa_bang + ma_bang) // 2) * wp
    if weapon.weapon_type == 'Gun':
        return wp * wp

    raise Exception('Missing weapon type in damage calc: ' + weapon.weapon_type)


ZODIAC_INDEX = {
    'Aries': 0,
    'Taurus': 1,
    'Gemini': 2,
    'Cancer': 3,
    'Leo': 4,
    'Virgo': 5,
    'Libra': 6,
    'Scorpio': 7,
    'Sagittarius': 8,
    'Capricorn': 9,
    'Aquarius': 10,
    'Pisces': 11,
    'Serpentarius': 12
}

ZODIAC_CHART = [
    'O  O  O  -  +  O  ?  O  +  -  O  O  O'.split('  '),
    'O  O  O  O  -  +  O  ?  O  +  -  O  O'.split('  '),
    'O  O  O  O  O  -  +  O  ?  O  +  -  O'.split('  '),
    '-  O  O  O  O  O  -  +  O  ?  O  +  O'.split('  '),
    '+  -  O  O  O  O  O  -  +  O  ?  O  O'.split('  '),
    'O  +  -  O  O  O  O  O  -  +  O  ?  O'.split('  '),
    '?  O  +  -  O  O  O  O  O  -  +  O  O'.split('  '),
    'O  ?  O  +  -  O  O  O  O  O  -  +  O'.split('  '),
    '+  O  ?  O  +  -  O  O  O  O  O  -  O'.split('  '),
    '-  +  O  ?  O  +  -  O  O  O  O  O  O'.split('  '),
    'O  -  +  O  ?  O  +  -  O  O  O  O  O'.split('  '),
    'O  O  -  +  O  ?  O  +  -  O  O  O  O'.split('  '),
    'O  O  O  O  O  O  O  O  O  O  O  O  O'.split('  '),
]


def zodiac_compat(c1, c2):
    s1 = ZODIAC_INDEX[c1['Sign']]
    s2 = ZODIAC_INDEX[c2['Sign']]
    g1 = c1['Gender']
    g2 = c2['Gender']

    if ZODIAC_CHART[s1][s2] == 'O':
        return 1.0
    elif ZODIAC_CHART[s1][s2] == '+':
        return 1.25
    elif ZODIAC_CHART[s1][s2] == '-':
        return 0.75
    elif ZODIAC_CHART[s1][s2] == '?':
        if g1 == 'Monster' or g2 == 'Monster':
            return 0.75
        elif g1 != g2:
            return 1.5
        else:
            return 0.5
    else:
        raise Exception(f"Missing case in zodiac compatibility calculation {c1['Sign']} {g1} vs {c2['Sign']} {g2}")


def has_absorb(actor, element) -> float:
    return actor.get(f'Absorb-{element}', 0.0)


def has_half(actor, element) -> float:
    return actor.get(f'Half-{element}', 0.0)


def has_weak(actor, element) -> float:
    return actor.get(f'Weak-{element}', 0.0)


def has_cancel(actor, element) -> float:
    return actor.get(f'Cancel-{element}', 0.0)


def has_strengthen(actor, element) -> float:
    return actor.get(f'Strengthen-{element}', 0.0)


def is_immune(actor, status) -> float:
    return actor.get(f'Immune-{status}', 0.0)


def can_heal(actor, victim):
    weapon = equipment.get_equipment(actor['Mainhand'])
    amount = 0
    z_compatibility = zodiac_compat(actor, victim)
    if has_absorb(victim, weapon.weapon_element):
        amount = max(amount, actor['Attack-Damage'] * z_compatibility)

    for ability_name in actor.keys():
        if not ability_name.startswith(SKILL_TAG):
            continue
        ab = ability.get_ability(ability_name)

        if has_cancel(victim, ab.element):
            continue

        bonus = 1.0
        if has_strengthen(actor, ab.element):
            bonus = 1.25

        if ab.heals and ab.ma_constant and not has_cancel(victim, ab.element):
            vfaith = victim['Faith'] / 100.0
            f = actor[ability_name] * vfaith * z_compatibility * ab.ma_constant * bonus
            amount = max(f, amount)

        if ab.damage and ab.ma_constant and has_absorb(victim, ab.element):
            if actor['SupportSkill'] == 'Magic Attack UP':
                bonus = bonus * (4.0 / 3.0)
            if victim['SupportSkill'] == 'Magic Defend UP':
                bonus = bonus * (2.0 / 3.0)

            vfaith = victim['Faith'] / 100.0
            f = actor[ability_name] * vfaith * z_compatibility * ab.ma_constant * bonus
            amount = max(f, amount)

        if ab.name == 'Potion':
            amount = max(100, amount)
        elif ab.name == 'Hi-Potion':
            amount = max(120, amount)
        elif ab.name == 'X-Potion':
            amount = max(150, amount)
        elif ab.name == 'Elixir':
            amount = max(victim['HP'], amount)

    assert amount >= 0
    return min(amount, 500)


# TODO: Break these calculations out into a single per skill function
#  can-heal can reuse 'if absorb'

def can_hurt(actor, victim):
    weapon = equipment.get_equipment(actor['Mainhand'])
    z_compatibility = zodiac_compat(actor, victim)
    amount = actor['Attack-Damage'] * z_compatibility

    if has_absorb(victim, weapon.weapon_element):
        amount = 0
    if has_cancel(victim, weapon.weapon_element):
        amount = 0
    if has_half(victim, weapon.weapon_element):
        amount = amount // 2
    if has_weak(victim, weapon.weapon_element):
        amount = amount * 2

    for ability_name in actor.keys():
        if not ability_name.startswith(SKILL_TAG):
            continue
        ab = ability.get_ability(ability_name)
        if ab.damage and (ab.ma_constant or ab.multiplier == ability.MULT_PA_HALF_MA):
            vfaith = victim['Faith'] / 100.0

            constant = 1.0
            if ab.ma_constant:
                constant = ab.ma_constant
            if ab.multiplier == ability.MULT_PA_HALF_MA:
                vfaith = 1.0

            f = actor[ability_name] * vfaith * z_compatibility * constant

            if has_strengthen(actor, ab.element):
                f = (f * 5) // 4

            if actor['SupportSkill'] == 'Magic Attack UP':
                f = (f * 4) // 3

            if victim['SupportSkill'] == 'Magic Defend UP':
                f = (f * 2) // 3

            if has_absorb(victim, ab.element):
                f = 0
            if has_cancel(victim, ab.element):
                f = 0
            if has_half(victim, ab.element):
                f = f // 2
            if has_weak(victim, ab.element):
                f = f * 2

            amount = max(f, amount)
        elif ab.damage and ab.multiplier == ability.MULT_PA_TIMES_WP:
            f = actor[ability_name] * z_compatibility
            amount = max(f, amount)

    assert amount >= 0
    return min(amount, 500)


def can_cause(actor, victim, status):
    if is_immune(victim, status):
        return 0.0

    effectiveness = 0.0
    z_compatibility = zodiac_compat(actor, victim)

    physical_guard = 1.0 - victim['Physical Evade']
    weapon = equipment.get_equipment(actor['Mainhand'])
    if status in weapon.chance_to_add:
        effectiveness = 0.19 * physical_guard

    weapon2 = equipment.get_equipment(actor['Offhand'])
    if status in weapon2.chance_to_add:
        chance = 0.19 * physical_guard
        effectiveness = (chance + effectiveness) - (chance * effectiveness)

    caster_ma = actor['MA']
    caster_faith = actor['Faith'] / 100.0
    victim_faith = victim['Faith'] / 100.0

    for ab in ability.get_ability_by_adds(status):
        if ab.name_with_tag not in actor:
            continue

        hit = ab.hit_chance.chance(caster_ma, caster_faith, victim_faith)

        magic_guard = 1.0
        if ab.hit_chance.times_faith:
            magic_guard = 1.0 - victim['Magical Evade']

        hit *= z_compatibility * magic_guard
        effectiveness = max(hit, effectiveness)

    assert effectiveness >= 0
    return min(effectiveness, 1.0)


def can_cancel(actor, victim, status):
    effectiveness = 0.0
    z_compatibility = zodiac_compat(actor, victim)

    caster_ma = actor['MA']
    caster_faith = actor['Faith'] / 100.0
    victim_faith = victim['Faith'] / 100.0

    for ab in ability.get_ability_by_cancels(status):
        if ab.name_with_tag not in actor:
            continue

        hit = ab.hit_chance.chance(caster_ma, caster_faith, victim_faith)
        hit *= z_compatibility
        effectiveness = max(hit, effectiveness)

    assert effectiveness >= 0
    return min(effectiveness, 1.0)
