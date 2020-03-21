import random
from math import floor
from typing import List

from fftbg import equipment as equipment
from fftbg.ability import Ability
from fftbg.combatant import ZODIAC_INDEX, ZODIAC_CHART
from fftbg.patch import Patch
from fftbg.simulation.status import TIME_STATUS_LEN, TIME_STATUS_INDEX, SLEEP, SHELL, PROTECT


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
        self.skills = {combatant['ReactionSkill'], combatant['SupportSkill'], combatant['MoveSkill']}
        self.skills.update(self.stats.innates)

        self.abilities: List[Ability] = []
        for ability_name in self.raw_combatant['ClassSkills']:
            self.abilities.append(patch.get_ability(ability_name))
        for ability_name in self.raw_combatant['ExtraSkills']:
            self.abilities.append(patch.get_ability(ability_name))
        # TODO: Ugh, why am I still mixing these two words up?
        for ability_name in self.stats.skills:
            self.abilities.append(patch.get_ability(ability_name))

        self.ct: int = 0
        self.charging: bool = False
        self.speed_mod: int = 0
        self.pa_mod: int = 0
        self.ma_mod: int = 0
        self.timed_status_conditions: List[int] = [0] * TIME_STATUS_LEN

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

    def status_time_remaining(self, status: str) -> int:
        return self.timed_status_conditions[TIME_STATUS_INDEX[status]]

    @property
    def berserk(self) -> bool:
        # TODO: Implement
        return False

    @property
    def sleep(self) -> bool:
        # TODO: Implement
        return self.status_time_remaining(SLEEP) > 0

    @property
    def shell(self) -> bool:
        # TODO: Implement
        return self.status_time_remaining(SHELL) > 0

    @property
    def protect(self) -> bool:
        # TODO: Implement
        return self.status_time_remaining(PROTECT) > 0

    @property
    def chicken(self) -> bool:
        # TODO: Implement
        return False

    @property
    def frog(self) -> bool:
        # TODO: Implement
        return False

    @property
    def attack_up(self) -> bool:
        return 'Attack UP' in self.skills

    @property
    def defense_up(self) -> bool:
        return 'Defense UP' in self.skills

    @property
    def martial_arts(self) -> bool:
        return 'Martial Arts' in self.skills

    @property
    def barehanded(self) -> bool:
        return self.mainhand.weapon_type is None or self.mainhand.weapon_type == 'Shield'

    @property
    def double_hand(self) -> bool:
        return 'Doublehand' in self.skills

    @property
    def dual_wield(self) -> bool:
        return 'Dual Wield' in self.skills

    def zodiac_compatibility(self, other: 'Combatant') -> float:
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

    def calculate_weapon_xa(self, weapon: equipment.Equipment, k=0):
        if self.barehanded:
            xa = floor((self.pa + k) * self.brave)
        elif weapon.weapon_type in ('Knife', 'Ninja Sword', 'Ninja Blade', 'Longbow', 'Bow'):
            xa = ((self.pa + k) + (self.speed + k)) // 2
        elif weapon.weapon_type in ('Knight Sword', 'Katana'):
            xa = floor((self.pa + k) * self.brave)
        elif weapon.weapon_type in ('Sword', 'Rod', 'Pole', 'Spear', 'Crossbow'):
            xa = self.pa + k
        elif weapon.weapon_type in ('Staff', 'Stick'):
            xa = self.ma + k
        elif weapon.weapon_type in ('Flail', 'Axe', 'Bag'):
            xa = random.randint(1, self.pa + k)
        elif weapon.weapon_type in ('Cloth', 'Fabric', 'Musical Instrument', 'Harp', 'Dictionary', 'Book'):
            xa = (self.pa + k + self.ma + k) // 2
        elif weapon.weapon_type == 'Gun':
            xa = weapon.wp + k
        else:
            raise Exception('Missing weapon type in damage calc: ' + weapon.weapon_type)
        return xa
