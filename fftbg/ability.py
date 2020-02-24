import re
from dataclasses import dataclass
from typing import Optional

import fftbg.config as config

ABILITY_MAP = {}

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


@dataclass
class Ability:
    name: str
    multiplier: Optional[str] = None
    damage: bool = False
    heals: bool = False
    element: Optional[str] = None
    range: Optional[int] = None
    aoe: Optional[int] = None
    ma_constant: Optional[int] = None

    def multiply(self, ability, brave, faith, pa, pa_bang, ma, wp, speed):
        if self.multiplier == MULT_BRAVE:
            return ability * brave
        elif self.multiplier == MULT_PA:
            return ability * pa
        elif self.multiplier == MULT_MA:
            return ability * ma
        elif self.multiplier == MULT_SPEED:
            return ability * speed
        elif self.multiplier == MULT_FAITH_MA:
            return ability * faith * ma
        elif self.multiplier == MULT_PA_PA_BANG:
            return ability * pa * pa_bang
        elif self.multiplier == MULT_PA_HALF_MA:
            return ability * (((pa + 2.0) / 2.0) * ma)
        elif self.multiplier == MULT_PA_PLUS_WP:
            return ability * (pa + wp)
        elif self.multiplier == MULT_PA_TIMES_WP:
            return ability * pa * wp
        elif self.multiplier is None:
            return ability
        else:
            raise Exception('Encountered unknown multiplier type: ' + self.multiplier)


DEFAULT_ABILITY = Ability('')
RANGE_RE = re.compile(r'(\d+) range')
AOE_RE = re.compile(r'(\d+) AoE')
ELEMENT_RE = re.compile(r'Element: (\w+)')


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

        range = try_int(RANGE_RE, desc)
        aoe = try_int(AOE_RE, desc)

        element = None
        element_match = ELEMENT_RE.findall(desc)
        if element_match:
            element = element_match[0]

        ma_constant = try_int(MA_CONSTANT_RE, desc)

        ABILITY_MAP[name.lower()] = Ability(
            name=name,
            multiplier=multiplier,
            damage=damage,
            heals=heals,
            range=range,
            aoe=aoe,
            element=element,
            ma_constant=ma_constant)


def try_int(regex, s):
    match = regex.findall(s)
    if match:
        return int(match[0])
    return None


def get_ability(name: str) -> Ability:
    if not ABILITY_MAP:
        parse_abilities()
    if name.startswith(SKILL_TAG):
        name = name[len(SKILL_TAG):]
    return ABILITY_MAP.get(name.lower(), DEFAULT_ABILITY)


if __name__ == '__main__':
    parse_abilities()
    for ab in ABILITY_MAP.values():
        print(ab)
