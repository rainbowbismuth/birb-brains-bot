import re
from dataclasses import dataclass

import config

EQUIPMENT_MAP = {}

ARMOR_RE = re.compile(r'(.+?): \+(\d+) HP.*(\d+) MP,.(.*)')


@dataclass(frozen=True)
class Equipment:
    name: str
    hp_bonus: int
    mp_bonus: int


EMPTY = Equipment(name='', hp_bonus=0, mp_bonus=0)


def parse_equipment():
    items = config.INFO_ITEM_PATH.read_text().splitlines()
    for item in items:
        armor_match = ARMOR_RE.match(item)
        if armor_match:
            name, hp_bonus, mp_bonus, _everything_else = armor_match.groups()
            EQUIPMENT_MAP[name] = Equipment(name=name, hp_bonus=int(hp_bonus), mp_bonus=int(mp_bonus))


def get_equipment(name: str) -> Equipment:
    if not EQUIPMENT_MAP:
        parse_equipment()
    return EQUIPMENT_MAP.get(name, EMPTY)
