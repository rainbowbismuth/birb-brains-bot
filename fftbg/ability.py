import re
from dataclasses import dataclass
from typing import Optional, Tuple, List

import fftbg.config as config

ABILITY_MAP = {}
ABILITY_BY_ADDS = {}
ABILITY_BY_CANCELS = {}

MULT_BRAVE = 'BRAVE'
MULT_PA = 'PA'
MULT_MA = 'MA'
MULT_SPEED = 'SPEED'
MULT_FAITH_MA = 'FAITH_MA'
MULT_PA_PA_BANG = 'PA_PA_BANG'
MULT_PA_HALF_MA = 'PA_HALF_MA'
MULT_PA_PLUS_WP = 'PA_PLUS_WP'
MULT_PA_TIMES_WP = 'PA_TIMES_WP'

MA_CONSTANT_RE = re.compile(r'\(MA \* (\d+)\)')
SKILL_TAG = '⭒ '


@dataclass(frozen=True)
class HitChance:
    ma_plus: Optional[int] = None
    times_faith: bool = False

    def chance(self, caster_ma, caster_faith, target_faith):
        # Not including Zodiac right now..
        if not self.ma_plus:
            return 1.0

        chance = (caster_ma + self.ma_plus) / 100.0
        if self.times_faith:
            chance *= caster_faith * target_faith
        return chance


DEFAULT_HIT_CHANCE = HitChance()


@dataclass(frozen=True)
class Ability:
    name: str
    name_with_tag: str
    multiplier: Optional[str] = None
    hit_chance: HitChance = DEFAULT_HIT_CHANCE
    damage: bool = False
    heals: bool = False
    element: Optional[str] = None
    range: int = 0
    aoe: Optional[int] = None
    ma_constant: Optional[int] = None
    adds: Tuple[str] = tuple()
    cancels: Tuple[str] = tuple()

    def multiply(self, brave, faith, pa, pa_bang, ma, wp, speed):
        if self.multiplier == MULT_BRAVE:
            return brave
        elif self.multiplier == MULT_PA:
            return pa
        elif self.multiplier == MULT_MA:
            return ma
        elif self.multiplier == MULT_SPEED:
            return speed
        elif self.multiplier == MULT_FAITH_MA:
            return faith * ma
        elif self.multiplier == MULT_PA_PA_BANG:
            return pa * pa_bang
        elif self.multiplier == MULT_PA_HALF_MA:
            return ((pa + 2.0) / 2.0) * ma
        elif self.multiplier == MULT_PA_PLUS_WP:
            return pa + wp
        elif self.multiplier == MULT_PA_TIMES_WP:
            return pa * wp
        elif self.multiplier is None:
            return 1.0
        else:
            raise Exception('Encountered unknown multiplier type: ' + self.multiplier)


DEFAULT_ABILITY = Ability('', name_with_tag=SKILL_TAG)
RANGE_RE = re.compile(r'(\d+) range')
AOE_RE = re.compile(r'(\d+) AoE')
ELEMENT_RE = re.compile(r'Element: (\w+)')

HIT_MA_PLUS_RE = re.compile(r'Hit: \(MA \+ (\d+)\)%')
HIT_FAITH_MA_PLUS_RE = re.compile(r'Hit: Faith\(MA \+ (\d+)\)%')
ADD_STATUS_RE = re.compile(r'Add ([\w,\s\']+)')
CANCEL_STATUS_RE = re.compile(r'Cancel ([\w,\s\']+)')


def parse_hit_chance(desc) -> Optional[HitChance]:
    ma_plus = try_int(HIT_MA_PLUS_RE, desc)
    if ma_plus:
        return HitChance(ma_plus=ma_plus, times_faith=False)
    faith_ma_plus = try_int(HIT_FAITH_MA_PLUS_RE, desc)
    if faith_ma_plus:
        return HitChance(ma_plus=faith_ma_plus, times_faith=True)
    return DEFAULT_HIT_CHANCE


def parse_abilities():
    abilities = config.ABILITY_HELP_PATH.read_text().splitlines()
    for ability in abilities:
        name = ability[:ability.index(':')]
        desc = ability[ability.index(':'):]
        multiplier = None
        if 'Reaction' in desc:
            multiplier = MULT_BRAVE
        elif 'Faith' in desc and 'MA' in desc:
            multiplier = MULT_FAITH_MA
        elif '(Speed' in desc:
            multiplier = MULT_SPEED
        elif '((PA + 2) / 2 * MA)' in desc:
            multiplier = MULT_PA_HALF_MA
        elif '(PA + WP' in desc:
            multiplier = MULT_PA_PLUS_WP
        elif 'MA' in desc:
            multiplier = MULT_MA
        elif desc.count('PA') >= 2:
            if name == 'Chakra':
                multiplier = MULT_PA
            else:
                multiplier = MULT_PA_PA_BANG
        elif 'PA * (WP' in desc or 'PA * WP' in desc:
            multiplier = MULT_PA_TIMES_WP
        elif 'PA' in desc:
            multiplier = MULT_PA

        damage = False
        heals = False
        if ' Damage ' in desc:
            damage = True
        if ' Heal ' in desc:
            heals = True

        range = try_int(RANGE_RE, desc, 0)
        aoe = try_int(AOE_RE, desc)

        element = None
        element_match = ELEMENT_RE.findall(desc)
        if element_match:
            element = element_match[0]

        ma_constant = try_int(MA_CONSTANT_RE, desc)

        hit_chance = parse_hit_chance(desc)

        adds = tuple(try_list(ADD_STATUS_RE, desc))
        cancels = tuple(try_list(CANCEL_STATUS_RE, desc))

        ab = Ability(name=name,
                     name_with_tag=SKILL_TAG + name,
                     multiplier=multiplier,
                     hit_chance=hit_chance,
                     damage=damage,
                     heals=heals,
                     range=range,
                     aoe=aoe,
                     element=element,
                     ma_constant=ma_constant,
                     adds=adds,
                     cancels=cancels)
        ABILITY_MAP[name.lower()] = ab
        for status in ab.adds:
            ABILITY_BY_ADDS.setdefault(status, []).append(ab)
        for status in ab.cancels:
            ABILITY_BY_CANCELS.setdefault(status, []).append(ab)


def try_int(regex, s, default=None):
    match = regex.findall(s)
    if match:
        return int(match[0])
    return default


def try_list(regex, s):
    match = regex.findall(s)
    if match:
        return tuple(x.strip() for x in match[0].split(','))
    return tuple()


def get_ability(name: str) -> Ability:
    if not ABILITY_MAP:
        parse_abilities()
    if name.startswith(SKILL_TAG):
        name = name[len(SKILL_TAG):]
    return ABILITY_MAP.get(name.lower(), DEFAULT_ABILITY)


def get_ability_by_adds(status: str) -> List[Ability]:
    return ABILITY_BY_ADDS.get(status, [])


def get_ability_by_cancels(status: str) -> List[Ability]:
    return ABILITY_BY_CANCELS.get(status, [])


if __name__ == '__main__':
    parse_abilities()
    for ab in ABILITY_MAP.values():
        print(ab)
