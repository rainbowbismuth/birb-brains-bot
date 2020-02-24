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


@dataclass
class Ability:
    name: str
    multiplier: Optional[str] = None
    damage: bool = False
    heals: bool = False
    element: Optional[str] = None
    range: Optional[int] = None
    aoe: Optional[int] = None

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
            return ability * (pa + ma / 2.0)
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

        range = None
        range_match = RANGE_RE.findall(desc)
        if range_match:
            range = int(range_match[0])

        aoe = None
        aoe_match = AOE_RE.findall(desc)
        if aoe_match:
            aoe = int(aoe_match[0])

        element = None
        element_match = ELEMENT_RE.findall(desc)
        if element_match:
            element = element_match[0]

        ABILITY_MAP[name.lower()] = Ability(
            name=name,
            multiplier=multiplier,
            damage=damage,
            heals=heals,
            range=range,
            aoe=aoe,
            element=element)


def get_ability(name: str) -> Ability:
    if not ABILITY_MAP:
        parse_abilities()
    return ABILITY_MAP.get(name.lower(), DEFAULT_ABILITY)


if __name__ == '__main__':
    parse_abilities()
    for ability in ABILITY_MAP.values():
        print(ability)
