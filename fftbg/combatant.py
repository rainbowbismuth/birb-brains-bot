from math import floor
from typing import List

import numpy as np

import fftbg.ability as ability
import fftbg.equipment as equipment
from fftbg.ability import SKILL_TAG
from fftbg.patch import Patch, Ability

PER_COMBATANTS = ['Name', 'Gender', 'Sign', 'Class', 'ActionSkill', 'SupportSkill', 'MoveSkill',
                  'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']
NUMERIC = ['Brave', 'Faith', 'HP', 'MP', 'Speed', 'Range', 'Ability-Range',
           'PA', 'MA', 'Move', 'Jump', 'Physical Evade', 'Magical Evade']
CATEGORICAL = PER_COMBATANTS + ['Color', 'Side', 'Map']

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


class Combatant:
    def __init__(self, combatant: dict, patch: Patch):
        self.raw_combatant: dict = combatant
        self.sign: str = combatant['Sign']
        self.job: str = combatant['Class']
        self.gender: str = combatant['Gender']
        self.stats = patch.get_base_stats(self.job, self.gender)
        self.mainhand = patch.get_equipment(combatant['Mainhand'])
        self.offhand = patch.get_equipment(combatant['Offhand'])
        self.headgear = patch.get_equipment(combatant['Head'])
        self.armor = patch.get_equipment(combatant['Armor'])
        self.accessory = patch.get_equipment(combatant['Accessory'])
        self.raw_brave: float = combatant['Brave'] / 100.0
        self.raw_faith: float = combatant['Faith'] / 100.0
        self.skills = {combatant['ReactionSkill'], combatant['SupportSkill']}
        self.skills.update(self.stats.innates)

        self.abilities: List[Ability] = []
        for ability_name in self.raw_combatant['ClassSkills']:
            self.abilities.append(patch.get_ability(ability_name))
        for ability_name in self.raw_combatant['ExtraSkills']:
            self.abilities.append(patch.get_ability(ability_name))
        # TODO: Ugh, why am I still mixing this two words up?
        for ability_name in self.stats.skills:
            self.abilities.append(patch.get_ability(ability_name))

        self.ct: int = 0
        self.charging: bool = False
        self.speed_mod: int = 0
        self.pa_mod: int = 0
        self.ma_mod: int = 0

    @property
    def all_equips(self):
        return [self.mainhand, self.offhand, self.headgear, self.armor, self.accessory]

    @property
    def max_hp(self) -> int:
        return self.stats.hp + sum([e.hp_bonus for e in self.all_equips])

    @property
    def max_mp(self) -> int:
        return self.stats.mp + sum([e.mp_bonus for e in self.all_equips])

    @property
    def speed(self) -> int:
        return self.stats.speed + self.speed_mod + sum([e.speed_bonus for e in self.all_equips])

    @property
    def brave(self) -> float:
        return self.raw_brave

    @property
    def faith(self) -> float:
        return self.raw_faith

    @property
    def evasion_multiplier(self) -> float:
        if self.charging:
            return 0.0
        elif 'Abandon' in self.skills:
            return 2.0
        else:
            return 1.0

    @property
    def class_evasion(self) -> float:
        return self.evasion_multiplier * (self.stats.c_ev / 100.0)

    @property
    def weapon_evasion(self) -> float:
        if 'Parry' not in self.skills:
            return 0.0
        # TODO: Pretty sure this is wrong
        return self.evasion_multiplier * (max([e.w_ev for e in self.all_equips]) / 100.0)

    @property
    def move(self) -> int:
        move = self.stats.move + sum([e.move_bonus for e in self.all_equips])
        if self.raw_combatant['MoveSkill'].startswith('Move+'):
            move += int(self.raw_combatant['MoveSkill'][-1])
        return move

    @property
    def jump(self) -> int:
        jump = self.stats.jump + sum([e.jump_bonus for e in self.all_equips])
        if self.raw_combatant['MoveSkill'].startswith('Jump+'):
            jump += int(self.raw_combatant['MoveSkill'][-1])
        elif self.raw_combatant['MoveSkill'] == 'Ignore Height':
            jump = 20
        elif self.raw_combatant['MoveSkill'].startswith('Teleport'):
            jump = 20
        elif 'Fly' in self.stats.innates or 'Fly' == self.raw_combatant['MoveSkill']:
            jump = 20
        return jump

    @property
    def physical_shield_evasion(self) -> float:
        return self.evasion_multiplier * (sum([e.phys_ev for e in (self.mainhand, self.offhand)]) / 100.0)

    @property
    def magical_shield_evasion(self) -> float:
        return self.evasion_multiplier * (sum([e.phys_ev for e in (self.mainhand, self.offhand)]) / 100.0)

    @property
    def physical_accessory_evasion(self) -> float:
        return self.evasion_multiplier * (self.accessory.phys_ev / 100.0)

    @property
    def magical_accessory_evasion(self) -> float:
        return self.evasion_multiplier * (self.accessory.magic_ev / 100.0)

    @property
    def pa_bang(self) -> int:
        return self.stats.pa + self.pa_mod

    @property
    def ma_bang(self) -> int:
        return self.stats.ma + self.ma_mod

    @property
    def pa(self) -> int:
        return self.pa_bang + sum([e.pa_bonus for e in self.all_equips])

    @property
    def ma(self) -> int:
        return self.ma_bang + sum([e.ma_bonus for e in self.all_equips])

    def zodiac(self, other: 'Combatant') -> float:
        s1 = ZODIAC_INDEX[self.sign]
        s2 = ZODIAC_INDEX[other.sign]
        if ZODIAC_CHART[s1][s2] == 'O':
            return 1.0
        elif ZODIAC_CHART[s1][s2] == '+':
            return 1.25
        elif ZODIAC_CHART[s1][s2] == '-':
            return 0.75
        elif ZODIAC_CHART[s1][s2] == '?':
            if self.gender == 'Monster' or other.gender == 'Monster':
                return 0.75
            elif self.gender != other.gender:
                return 1.5
            else:
                return 0.5
        else:
            raise Exception(f"Missing case in zodiac compatibility calculation\
             {self.sign} {self.gender} vs {other.sign} {other.gender}")

    def absorbs(self, element) -> bool:
        return element in self.stats.absorbs or any((element in e.absorbs for e in self.all_equips))

    def halves(self, element) -> bool:
        return element in self.stats.halves or any((element in e.halves for e in self.all_equips))

    def weak(self, element) -> bool:
        return element in self.stats.weaknesses or any((element in e.weaknesses for e in self.all_equips))

    def strengthens(self, element) -> bool:
        return any((element in e.strengthens for e in self.all_equips))

    def immune_to(self, element) -> bool:
        return any((element in e.immune_to for e in self.all_equips))


def combatant_to_dict(combatant: dict, patch: Patch):
    output = dict(combatant)
    del output['ClassSkills']
    del output['ExtraSkills']

    if combatant['SupportSkill'].startswith('Equip '):
        combatant['SupportSkill'] = ''

    # Compute stats
    stats = patch.get_base_stats(combatant['Class'], combatant['Gender'])
    mainhand = patch.get_equipment(combatant['Mainhand'])
    offhand = patch.get_equipment(combatant['Offhand'])
    headgear = patch.get_equipment(combatant['Head'])
    armor = patch.get_equipment(combatant['Armor'])
    accessory = patch.get_equipment(combatant['Accessory'])
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
    if combatant['MoveSkill'] == 'Ignore Height':
        output['Jump'] = 20
        output['MoveSkill'] = ''
    if combatant['MoveSkill'] == 'Lava Walking':
        output['MoveSkill'] = ''
    if combatant['MoveSkill'].startswith('Teleport'):
        output['Jump'] = 20

    if 'Landlocked' in stats.innates:
        output[SKILL_TAG + 'Landlocked'] = 1.0
    if 'Fly' in stats.innates or 'Fly' == combatant['MoveSkill']:
        output[SKILL_TAG + 'Fly'] = 1.0
        output['Jump'] = 20

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
        output['W-EV'], output['C-EV'] * 1.0 / 3.0, output['Physical S-EV'] * 3.0 / 4.0, output['Physical A-EV']]
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
    output['WP'] = wp
    speed = output['Speed']

    output['Ability-Range'] = 0.0
    for skill in combatant['ClassSkills'] + combatant['ExtraSkills'] + list(stats.skills):
        calc = patch.get_ability(skill)
        output['Ability-Range'] = max(output['Ability-Range'], calc.range)
        output[SKILL_TAG + skill] = calc.multiply(brave, faith, pa, pa_bang, ma, wp, speed)

    output[SKILL_TAG + combatant['ReactionSkill']] = brave
    del output['ReactionSkill']

    return output


def damage_calculation(combatant, weapon: equipment.Equipment):
    pa = combatant['PA']
    ma = combatant['MA']
    pa_bang = combatant['PA!']
    br = combatant['Brave'] / 100.0
    wp = weapon.wp
    sp = combatant['Speed']

    element = weapon.weapon_element
    if element and combatant.get(f'Strengthen-{element}'):
        pa = (pa * 5) // 4
        ma = (ma * 5) // 4

    if combatant['SupportSkill'] == 'Doublehand' and weapon.weapon_type not in ('Gun',):
        wp *= 2
    if combatant['SupportSkill'] == 'Attack UP':
        pa = (pa * 4) // 3

    # TODO: Fix magical guns, they are special.

    if weapon.weapon_type is None or combatant['Gender'] == 'Monster':
        # Bare Hands case
        if combatant['SupportSkill'] == 'Martial Arts':
            pa_bang = (pa_bang * 3) // 2
        return floor(pa * br) * pa_bang
    if weapon.weapon_type in ('Knife', 'Ninja Sword', 'Ninja Blade', 'Longbow', 'Bow'):
        return ((pa + sp) // 2) * wp
    if weapon.weapon_type in ('Knight Sword', 'Katana'):
        return floor(pa * br) * wp
    if weapon.weapon_type in ('Sword', 'Rod', 'Pole', 'Spear', 'Crossbow'):
        return pa * wp
    if weapon.weapon_type in ('Staff', 'Stick'):
        return ma * wp
    if weapon.weapon_type in ('Flail', 'Axe', 'Bag'):
        return (pa / 2) * wp
    if weapon.weapon_type in ('Cloth', 'Fabric', 'Musical Instrument', 'Harp', 'Dictionary', 'Book'):
        return ((pa + ma) // 2) * wp
    if weapon.weapon_type == 'Gun':
        return wp * wp

    raise Exception('Missing weapon type in damage calc: ' + weapon.weapon_type)


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


def can_heal(actor, victim, patch: Patch):
    weapon = patch.get_equipment(actor['Mainhand'])
    amount = 0
    z_compatibility = zodiac_compat(actor, victim)
    if has_absorb(victim, weapon.weapon_element):
        amount = max(amount, actor['Attack-Damage'] * z_compatibility)

    for ability_name in actor.keys():
        if not ability_name.startswith(SKILL_TAG):
            continue
        ab = patch.get_ability(ability_name)

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

def can_hurt(actor, victim, patch: Patch):
    weapon = patch.get_equipment(actor['Mainhand'])
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
        ab = patch.get_ability(ability_name)
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


def lethality(actor, victim, patch: Patch):
    damage = can_hurt(actor, victim, patch)
    hp = victim['HP']
    ev_avg = 1.0 - (victim['Physical Evade'] + victim['Magical Evade']) / 3.0
    damage *= ev_avg
    damage = max(5, damage)
    return max(0.01, hp / damage)


def can_cause(actor, victim, status, patch: Patch):
    if is_immune(victim, status):
        return 0.0

    effectiveness = 0.0
    z_compatibility = zodiac_compat(actor, victim)

    physical_guard = 1.0 - victim['Physical Evade']
    weapon = patch.get_equipment(actor['Mainhand'])
    if status in weapon.chance_to_add:
        effectiveness = 0.19 * physical_guard

    weapon2 = patch.get_equipment(actor['Offhand'])
    if status in weapon2.chance_to_add:
        chance = 0.19 * physical_guard
        effectiveness = (chance + effectiveness) - (chance * effectiveness)

    caster_ma = actor['MA']
    caster_pa = actor['PA']
    caster_speed = actor['Speed']
    caster_wp = actor['WP']
    caster_faith = actor['Faith'] / 100.0
    victim_faith = victim['Faith'] / 100.0

    for ab in patch.get_ability_by_adds(status):
        if ab.name_with_tag not in actor:
            continue

        hit = ab.hit_chance.chance(caster_ma, caster_pa, caster_speed, caster_wp, caster_faith, victim_faith)

        magic_guard = 1.0
        if ab.hit_chance.times_faith:
            magic_guard = 1.0 - victim['Magical Evade']

        hit *= z_compatibility * magic_guard
        effectiveness = max(hit, effectiveness)

    assert effectiveness >= 0
    return min(effectiveness, 1.0)


def can_cancel(actor, victim, status, patch: Patch):
    effectiveness = 0.0
    z_compatibility = zodiac_compat(actor, victim)

    caster_ma = actor['MA']
    caster_pa = actor['PA']
    caster_speed = actor['Speed']
    caster_wp = actor['WP']
    caster_faith = actor['Faith'] / 100.0
    victim_faith = victim['Faith'] / 100.0

    for ab in patch.get_ability_by_cancels(status):
        if ab.name_with_tag not in actor:
            continue

        hit = ab.hit_chance.chance(caster_ma, caster_pa, caster_speed, caster_wp, caster_faith, victim_faith)
        hit *= z_compatibility
        effectiveness = max(hit, effectiveness)

    assert effectiveness >= 0
    return min(effectiveness, 1.0)
