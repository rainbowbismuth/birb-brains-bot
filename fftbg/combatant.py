from math import floor

import fftbg.ability as ability
import fftbg.base_stats as base_stats
import fftbg.equipment as equipment

PER_COMBATANTS = ['Name', 'Gender', 'Sign', 'Class', 'ActionSkill', 'SupportSkill', 'MoveSkill',
                  'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']
NUMERIC = ['Brave', 'Faith', 'HP', 'MP', 'Speed', 'Range', 'W-EV', 'C-EV', 'PA', 'PA!', 'MA', 'MA!',
           'Physical A-EV', 'Magical A-EV',
           'Effective HP', 'Attack-Damage']
CATEGORICAL = PER_COMBATANTS + ['Color', 'Side', 'Map']
SKILL_TAG = '⭒ '


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
    output['W-EV'] = max([e.w_ev for e in all_equips])
    output['C-EV'] = stats.c_ev

    output['PA!'] = stats.pa
    output['PA'] = stats.pa + sum([e.pa_bonus for e in all_equips])
    output['MA!'] = stats.ma
    output['MA'] = stats.ma + sum([e.ma_bonus for e in all_equips])
    output['Physical A-EV'] = sum([e.phys_ev for e in all_equips])
    output['Magical A-EV'] = sum([e.magic_ev for e in all_equips])

    # An estimate here, against physical attacks
    w_ev = 1 - (output['W-EV'] / 150.0)
    c_ev = 1 - (output['C-EV'] / 200.0)
    a_ev = 1 - (output['Physical A-EV'] / 100.0)
    summary_ev = w_ev * c_ev * a_ev
    output['Effective HP'] = output['HP'] * (1 / summary_ev)

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

    for skill in combatant['ClassSkills'] + combatant['ExtraSkills']:
        calc = ability.get_ability(skill)
        output[SKILL_TAG + skill] = calc.multiply(1, brave, faith, pa, pa_bang, ma, wp, speed)

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

    raise Exception('missing weapon type in damage calc: ' + weapon.weapon_type)
