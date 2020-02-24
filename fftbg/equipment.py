import re
from dataclasses import dataclass
from typing import Optional

import fftbg.config as config

EQUIPMENT_MAP = {}

WEAPON_RE = re.compile(r'(.+?): (\d+) WP(.+)?, (\d+) range, (\d+)% evade, (.+?)\.(.*)')
ARMOR_RE = re.compile(r'(.+?): \+(\d+) HP.*(\d+) MP,.(.*)')
ACCESSORY_RE = re.compile(r'(.+?):(.*)')
SPEED_EFFECT_RE = re.compile(r'\+(\d) Speed')
BONUS_PA_RE = re.compile(r'\+(\d+) PA')
BONUS_MA_RE = re.compile(r'\+(\d+) MA')
PHYS_EVADE_RE = re.compile(r'(\d+)% phys evade')
MAGIC_EVADE_RE = re.compile(r'(\d+)% magic evade')
MOVE_RE = re.compile(r'\+(\d+) Move')
JUMP_RE = re.compile(r'\+(\d+) Jump')


@dataclass(frozen=True)
class Equipment:
    name: str
    hp_bonus: int = 0
    mp_bonus: int = 0
    speed_bonus: int = 0
    pa_bonus: int = 0
    ma_bonus: int = 0
    wp: int = 0
    range: int = 0
    w_ev: int = 0
    phys_ev: int = 0
    magic_ev: int = 0
    weapon_type: Optional[str] = None
    move_bonus: int = 0
    jump_bonus: int = 0


EMPTY = Equipment(name='')


def try_int(regex, s: str, default: int = 0):
    matches = regex.findall(s)
    if matches:
        return int(matches[0])
    return default


def parse_equipment():
    items = config.INFO_ITEM_PATH.read_text().splitlines()
    for item in items:
        armor_match = ARMOR_RE.match(item)
        if armor_match:
            name, hp_bonus, mp_bonus, everything_else = armor_match.groups()
            speed_bonus = try_int(SPEED_EFFECT_RE, everything_else)
            pa_bonus = try_int(BONUS_PA_RE, everything_else)
            ma_bonus = try_int(BONUS_MA_RE, everything_else)
            move_bonus = try_int(MOVE_RE, everything_else)
            jump_bonus = try_int(JUMP_RE, everything_else)
            EQUIPMENT_MAP[name] = Equipment(name=name,
                                            hp_bonus=int(hp_bonus),
                                            mp_bonus=int(mp_bonus),
                                            speed_bonus=speed_bonus,
                                            pa_bonus=pa_bonus,
                                            ma_bonus=ma_bonus,
                                            move_bonus=move_bonus,
                                            jump_bonus=jump_bonus)
            continue

        weapon_match = WEAPON_RE.match(item)
        if weapon_match:
            name, wp, modifier, w_range, w_ev, weapon_type, everything_else = weapon_match.groups()
            speed_bonus = try_int(SPEED_EFFECT_RE, everything_else)
            pa_bonus = try_int(BONUS_PA_RE, everything_else)
            ma_bonus = try_int(BONUS_MA_RE, everything_else)
            move_bonus = try_int(MOVE_RE, everything_else)
            EQUIPMENT_MAP[name] = Equipment(name=name,
                                            speed_bonus=speed_bonus,
                                            pa_bonus=pa_bonus,
                                            ma_bonus=ma_bonus,
                                            wp=int(wp),
                                            range=int(w_range),
                                            w_ev=int(w_ev),
                                            weapon_type=weapon_type,
                                            move_bonus=move_bonus)
            continue

        if 'Accessory' in item:
            accessory_match = ACCESSORY_RE.match(item)
            name, everything_else = accessory_match.groups()
            speed_bonus = try_int(SPEED_EFFECT_RE, everything_else)
            pa_bonus = try_int(BONUS_PA_RE, everything_else)
            ma_bonus = try_int(BONUS_MA_RE, everything_else)
            phys_ev = try_int(PHYS_EVADE_RE, everything_else)
            magic_ev = try_int(MAGIC_EVADE_RE, everything_else)
            move_bonus = try_int(MOVE_RE, everything_else)
            jump_bonus = try_int(JUMP_RE, everything_else)
            EQUIPMENT_MAP[name] = Equipment(name=name,
                                            speed_bonus=speed_bonus,
                                            pa_bonus=pa_bonus,
                                            ma_bonus=ma_bonus,
                                            phys_ev=phys_ev,
                                            magic_ev=magic_ev,
                                            move_bonus=move_bonus,
                                            jump_bonus=jump_bonus)


def get_equipment(name: str) -> Equipment:
    if not EQUIPMENT_MAP:
        parse_equipment()
    return EQUIPMENT_MAP.get(name, EMPTY)
