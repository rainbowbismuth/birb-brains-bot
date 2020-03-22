import random
from math import floor
from typing import List, Set

from fftbg import equipment as equipment
from fftbg.ability import Ability
from fftbg.combatant import ZODIAC_INDEX, ZODIAC_CHART
from fftbg.patch import Patch
from fftbg.simulation.status import TIME_STATUS_LENGTHS, TIME_STATUS_LEN, TIME_STATUS_INDEX, BERSERK, CHARGING, SLEEP, \
    SHELL, PROTECT, HASTE, SLOW, FROG, CHICKEN, PETRIFY, REGEN, POISON, CRITICAL, STOP, CONFUSION, CHARM, TRANSPARENT


class Combatant:
    def __init__(self, combatant: dict, patch: Patch, team: int):
        self.raw_combatant: dict = combatant
        self.team = team
        self.name: str = combatant['Name']
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

        self.hp: int = self.max_hp
        self.mp: int = self.max_mp
        self.ct: int = 0
        self.pa_mod: int = 0
        self.ma_mod: int = 0
        self.speed_mod: int = 0
        self.timed_status_conditions: List[int] = [0] * TIME_STATUS_LEN
        self.other_status: Set[str] = set()

        self.ctr: int = 0
        self.ctr_action = None

        self.on_active_turn = False
        self.moved_during_active_turn = False
        self.acted_during_active_turn = False
        self.took_damage_during_active_turn = False

        self.location: int = 0

        for e in self.all_equips:
            for status in e.initial:
                self.add_status(status)

    def __repr__(self):
        return f'<{self.name} ({self.hp} HP) team: {self.team}>'

    def is_friend(self, other: 'Combatant'):
        return self.team == other.team

    def is_foe(self, other: 'Combatant'):
        return self.team != other.team

    def distance(self, other: 'Combatant'):
        return abs(self.location - other.location)

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
        # TODO: This is how sleep works right?
        if self.charging or self.sleep:
            return 0.0
        elif self.abandon:
            return 2.0
        else:
            return 1.0

    @property
    def class_evasion(self) -> float:
        return self.evasion_multiplier * (self.stats.c_ev / 100.0)

    @property
    def weapon_evasion(self) -> float:
        if not self.parry:
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

    def add_status(self, status: str):
        # NOTE: This doesn't handle opposing statuses (like Haste/Slow)
        if status in TIME_STATUS_LENGTHS:
            self.timed_status_conditions[TIME_STATUS_INDEX[status]] = TIME_STATUS_LENGTHS[status]
        else:
            self.other_status.add(status)

    def has_status(self, status: str):
        if status in TIME_STATUS_LENGTHS:
            return self.status_time_remaining(status) > 0

        if status == CHARGING:
            return self.ctr_action is not None

        if status == CRITICAL:
            return self.hp <= self.max_hp // 5

        return status in self.other_status

    def cancel_status(self, status: str):
        if status in TIME_STATUS_LENGTHS:
            self.timed_status_conditions[TIME_STATUS_INDEX[status]] = 0
            return

        if status == CHARGING:
            self.ctr_action = None
            self.ctr = 0
            return

        self.other_status.discard(status)

    @property
    def healthy(self) -> bool:
        return self.hp > 0 and not self.petrified

    @property
    def critical(self) -> bool:
        return self.has_status(CRITICAL)

    @property
    def charging(self) -> bool:
        return self.has_status(CHARGING)

    @property
    def transparent(self) -> bool:
        return self.has_status(TRANSPARENT)

    @property
    def berserk(self) -> bool:
        return self.has_status(BERSERK)

    @property
    def sleep(self) -> bool:
        return self.has_status(SLEEP)

    @property
    def shell(self) -> bool:
        return self.has_status(SHELL)

    @property
    def protect(self) -> bool:
        return self.has_status(PROTECT)

    @property
    def haste(self) -> bool:
        return self.has_status(HASTE)

    @property
    def slow(self) -> bool:
        return self.has_status(SLOW)

    @property
    def stop(self) -> bool:
        return self.has_status(STOP)

    @property
    def regen(self) -> bool:
        return self.has_status(REGEN)

    @property
    def poison(self) -> bool:
        return self.has_status(POISON)

    @property
    def chicken(self) -> bool:
        return self.has_status(CHICKEN)

    @property
    def frog(self) -> bool:
        return self.has_status(FROG)

    @property
    def petrified(self) -> bool:
        return self.has_status(PETRIFY)

    @property
    def charm(self) -> bool:
        return self.has_status(CHARM)

    @property
    def confusion(self) -> bool:
        return self.has_status(CONFUSION)

    @property
    def abandon(self) -> bool:
        return 'Abandon' in self.skills

    @property
    def parry(self) -> bool:
        return 'Parry' in self.skills

    @property
    def attack_up(self) -> bool:
        return 'Attack UP' in self.skills

    @property
    def defense_up(self) -> bool:
        return 'Defense UP' in self.skills

    @property
    def concentrate(self) -> bool:
        return 'Concentrate' in self.skills

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
    def auto_potion(self) -> bool:
        return 'Auto Potion' in self.skills

    @property
    def dual_wield(self) -> bool:
        return 'Dual Wield' in self.skills

    @property
    def mana_shield(self) -> bool:
        return 'Mana Shield' in self.skills

    @property
    def damage_split(self) -> bool:
        return 'Damage Split' in self.skills

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
