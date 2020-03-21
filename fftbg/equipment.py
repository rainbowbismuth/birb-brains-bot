import re
from dataclasses import dataclass
from typing import Optional, Tuple

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
WEAPON_ELEMENT_RE = re.compile(r'Element: (\w+)')

STRENGTHEN_RE = re.compile(r'Strengthen ([\w,\s]+)')
ABSORB_RE = re.compile(r'Absorb ([\w,\s]+)')
HALF_RE = re.compile(r'Half ([\w,\s]+)')
WEAK_RE = re.compile(r'Weak ([\w,\s]+)')
CANCEL_RE = re.compile(r'Cancel ([\w,\s]+)')

INITIAL_RE = re.compile(r'Initial ([\w,\s\']+)', re.IGNORECASE)
CHANCE_TO_ADD_RE = re.compile(r'Chance to Add ([\w,\s\']+)', re.IGNORECASE)
CHANCE_TO_CANCEL_RE = re.compile(r'Chance to Cancel ([\w,\s\']+)', re.IGNORECASE)
IMMUNE_TO_RE = re.compile(r'Immune ([\w,\s\']+)')


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
    weapon_element: Optional[str] = None
    move_bonus: int = 0
    jump_bonus: int = 0
    strengthens: Tuple[str] = tuple()
    absorbs: Tuple[str] = tuple()
    halves: Tuple[str] = tuple()
    weaknesses: Tuple[str] = tuple()
    cancels: Tuple[str] = tuple()
    initial: Tuple[str] = tuple()
    chance_to_add: Tuple[str] = tuple()
    chance_to_cancel: Tuple[str] = tuple()
    immune_to: Tuple[str] = tuple()


EMPTY = Equipment(name='')


@dataclass
class EquipmentData:
    by_name: {str: Equipment}

    def get_equipment(self, name: str) -> Equipment:
        return self.by_name.get(name, EMPTY)


def try_int(regex, s: str, default: int = 0):
    matches = regex.findall(s)
    if matches:
        return int(matches[0])
    return default


def try_str(regex, s: str):
    matches = regex.findall(s)
    if matches:
        return matches[0]
    return None


def try_list(regex, s: str):
    matches = regex.findall(s)
    if matches:
        return tuple(sorted(e.strip() for e in matches[0].split(',')))
    return tuple()


def parse_equipment(info_item_path) -> EquipmentData:
    by_name = {}
    items = info_item_path.read_text().splitlines()
    for item in items:
        strengthens = try_list(STRENGTHEN_RE, item)
        absorbs = try_list(ABSORB_RE, item)
        halves = try_list(HALF_RE, item)
        weaknesses = try_list(WEAK_RE, item)
        cancels = try_list(CANCEL_RE, item)

        initial = try_list(INITIAL_RE, item)
        chance_to_add = try_list(CHANCE_TO_ADD_RE, item)
        chance_to_cancel = try_list(CHANCE_TO_CANCEL_RE, item)
        immune_to = try_list(IMMUNE_TO_RE, item)

        armor_match = ARMOR_RE.match(item)
        if armor_match:
            name, hp_bonus, mp_bonus, everything_else = armor_match.groups()
            speed_bonus = try_int(SPEED_EFFECT_RE, everything_else)
            pa_bonus = try_int(BONUS_PA_RE, everything_else)
            ma_bonus = try_int(BONUS_MA_RE, everything_else)
            move_bonus = try_int(MOVE_RE, everything_else)
            jump_bonus = try_int(JUMP_RE, everything_else)
            by_name[name] = Equipment(name=name,
                                      hp_bonus=int(hp_bonus),
                                      mp_bonus=int(mp_bonus),
                                      speed_bonus=speed_bonus,
                                      pa_bonus=pa_bonus,
                                      ma_bonus=ma_bonus,
                                      move_bonus=move_bonus,
                                      jump_bonus=jump_bonus,
                                      strengthens=strengthens,
                                      absorbs=absorbs,
                                      halves=halves,
                                      weaknesses=weaknesses,
                                      cancels=cancels,
                                      initial=initial,
                                      chance_to_add=chance_to_add,
                                      chance_to_cancel=chance_to_cancel,
                                      immune_to=immune_to)
            continue

        weapon_match = WEAPON_RE.match(item)
        if weapon_match:
            name, wp, modifier, w_range, w_ev, weapon_type, everything_else = weapon_match.groups()
            speed_bonus = try_int(SPEED_EFFECT_RE, everything_else)
            pa_bonus = try_int(BONUS_PA_RE, everything_else)
            ma_bonus = try_int(BONUS_MA_RE, everything_else)
            move_bonus = try_int(MOVE_RE, everything_else)
            weapon_element = try_str(WEAPON_ELEMENT_RE, everything_else)
            by_name[name] = Equipment(name=name,
                                      speed_bonus=speed_bonus,
                                      pa_bonus=pa_bonus,
                                      ma_bonus=ma_bonus,
                                      wp=int(wp),
                                      range=int(w_range),
                                      w_ev=int(w_ev),
                                      weapon_type=weapon_type,
                                      weapon_element=weapon_element,
                                      move_bonus=move_bonus,
                                      strengthens=strengthens,
                                      absorbs=absorbs,
                                      halves=halves,
                                      weaknesses=weaknesses,
                                      cancels=cancels,
                                      initial=initial,
                                      chance_to_add=chance_to_add,
                                      chance_to_cancel=chance_to_cancel,
                                      immune_to=immune_to)
            continue

        if 'Accessory' in item or 'Shield' in item:
            accessory_match = ACCESSORY_RE.match(item)
            name, everything_else = accessory_match.groups()
            speed_bonus = try_int(SPEED_EFFECT_RE, everything_else)
            pa_bonus = try_int(BONUS_PA_RE, everything_else)
            ma_bonus = try_int(BONUS_MA_RE, everything_else)
            phys_ev = try_int(PHYS_EVADE_RE, everything_else)
            magic_ev = try_int(MAGIC_EVADE_RE, everything_else)
            move_bonus = try_int(MOVE_RE, everything_else)
            jump_bonus = try_int(JUMP_RE, everything_else)
            by_name[name] = Equipment(name=name,
                                      speed_bonus=speed_bonus,
                                      pa_bonus=pa_bonus,
                                      ma_bonus=ma_bonus,
                                      phys_ev=phys_ev,
                                      magic_ev=magic_ev,
                                      move_bonus=move_bonus,
                                      jump_bonus=jump_bonus,
                                      strengthens=strengthens,
                                      absorbs=absorbs,
                                      halves=halves,
                                      weaknesses=weaknesses,
                                      cancels=cancels,
                                      initial=initial,
                                      chance_to_add=chance_to_add,
                                      chance_to_cancel=chance_to_cancel,
                                      immune_to=immune_to)

    return EquipmentData(by_name)
