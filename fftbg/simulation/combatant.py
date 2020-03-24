import random
from math import floor
from typing import List

from fftbg import equipment as equipment
from fftbg.ability import Ability
from fftbg.combatant import ZODIAC_INDEX, ZODIAC_CHART
from fftbg.patch import Patch
from fftbg.simulation.status import *


class Combatant:
    def __init__(self, combatant: dict, patch: Patch, team: int, index: int):
        self.raw_combatant: dict = combatant
        self.team = team
        self.index = index
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

        self.ability_by_name = {}
        for ability in self.abilities:
            self.ability_by_name[ability.name] = ability

        self.raw_hp: int = self.max_hp
        self.raw_mp: int = self.max_mp
        self.ct: int = 0
        self.pa_mod: int = 0
        self.ma_mod: int = 0
        self.speed_mod: int = 0

        self.status_flags: int = 0
        self.timed_status_conditions: List[int] = [0] * TIME_STATUS_LEN
        self.broken_items: int = 0

        self.ctr: int = 0
        self.ctr_action = None

        self.on_active_turn = False
        self.moved_during_active_turn = False
        self.acted_during_active_turn = False
        self.took_damage_during_active_turn = False

        self.crystal_counter: int = 4

        self.location: int = 0

        self.num_mp_using_abilities: int = len([ab for ab in self.abilities if ab.mp > 0])
        self.lowest_mp_cost_ability: int = min([999] + [ab.mp for ab in self.abilities if ab.mp > 0])

        self.target_value: float = 0.0

        for e in self.all_equips:
            for status in e.initial:
                self.add_status_flag(status)

    def __repr__(self):
        return f'<{self.name} ({self.hp} HP) team: {self.team} loc: {self.location}>'

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
    def hp(self) -> int:
        return self.raw_hp

    @hp.setter
    def hp(self, new_hp: int):
        self.raw_hp = max(0, min(new_hp, self.max_hp))
        # TODO: Calculate death statuses here?

    @property
    def hp_percent(self) -> float:
        return self.raw_hp / self.max_hp

    @property
    def mp(self) -> int:
        return self.raw_mp

    @mp.setter
    def mp(self, new_mp: int):
        self.raw_mp = max(0, min(new_mp, self.max_mp))

    @property
    def mp_percent(self) -> float:
        return self.raw_mp / self.max_mp

    @property
    def can_cast_mp_ability(self) -> bool:
        if self.num_mp_using_abilities == 0:
            return False
        return self.raw_mp >= self.lowest_mp_cost_ability

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

    def has_ability(self, name: str) -> bool:
        return name in self.ability_by_name

    def status_time_remaining(self, status: str) -> int:
        # TODO: No longer called? Could be useful though.
        return self.timed_status_conditions[TIME_STATUS_INDEX[status]]

    def add_status_flag(self, status: str):
        # NOTE: This doesn't handle opposing statuses (like Haste/Slow)
        # NOTE: This doesn't handle Death :(
        if status in TIME_STATUS_LENGTHS:
            self.timed_status_conditions[TIME_STATUS_INDEX[status]] = TIME_STATUS_LENGTHS[status]
        self.status_flags |= STATUS_FLAGS[status]

    def has_status(self, status: str):
        if status == CHARGING:
            return self.ctr_action is not None

        if status == CRITICAL:
            return self.hp <= self.max_hp // 5

        if status == DEATH:
            return self.dead

        return self.status_flags & STATUS_FLAGS[status] != 0

    @property
    def all_statuses(self):
        statuses = []
        for status in ALL_CONDITIONS:
            if self.has_status(status):
                statuses.append(status)
        return statuses

    def cancel_status(self, status: str):
        if status == CHARGING:
            self.ctr_action = None
            self.ctr = 0
            return

        if status in TIME_STATUS_LENGTHS:
            self.timed_status_conditions[TIME_STATUS_INDEX[status]] = 0

        self.status_flags &= ~STATUS_FLAGS[status]

    @property
    def healthy(self) -> bool:
        return self.hp > 0 and not self.petrified

    @property
    def dead(self) -> bool:
        return self.hp == 0

    @property
    def crystal(self) -> bool:
        return self.crystal_counter == 0

    @property
    def undead(self) -> bool:
        return self.status_flags & UNDEAD_FLAG != 0

    @property
    def death_sentence(self) -> bool:
        return self.status_flags & DEATH_SENTENCE_FLAG != 0

    @property
    def reraise(self) -> bool:
        return self.status_flags & RERAISE_FLAG != 0

    @property
    def critical(self) -> bool:
        return self.has_status(CRITICAL)

    @property
    def dont_move(self) -> bool:
        return self.status_flags & DONT_MOVE_FLAG != 0

    @property
    def dont_act(self) -> bool:
        return self.status_flags & DONT_ACT_FLAG != 0

    @property
    def silence(self) -> bool:
        return self.status_flags & SILENCE_FLAG != 0

    @property
    def innocent(self) -> bool:
        return self.status_flags & INNOCENT_FLAG != 0

    @property
    def reflect(self) -> bool:
        return self.status_flags & REFLECT_FLAG != 0

    @property
    def charging(self) -> bool:
        return self.has_status(CHARGING)

    @property
    def transparent(self) -> bool:
        return self.status_flags & TRANSPARENT_FLAG != 0

    @property
    def berserk(self) -> bool:
        return self.status_flags & BERSERK_FLAG != 0

    @property
    def blood_suck(self) -> bool:
        return self.status_flags & BLOOD_SUCK_FLAG != 0

    @property
    def oil(self) -> bool:
        return self.status_flags & OIL_FLAG != 0

    @property
    def float(self) -> bool:
        return self.status_flags & FLOAT_FLAG != 0

    @property
    def sleep(self) -> bool:
        return self.status_flags & SLEEP_FLAG != 0

    @property
    def shell(self) -> bool:
        return self.status_flags & SHELL_FLAG != 0

    @property
    def protect(self) -> bool:
        return self.status_flags & PROTECT_FLAG != 0

    @property
    def wall(self) -> bool:
        return self.status_flags & WALL_FLAG != 0

    @property
    def haste(self) -> bool:
        return self.status_flags & HASTE_FLAG != 0

    @property
    def slow(self) -> bool:
        return self.status_flags & SLOW_FLAG != 0

    @property
    def stop(self) -> bool:
        return self.status_flags & STOP_FLAG != 0

    @property
    def regen(self) -> bool:
        return self.status_flags & REGEN_FLAG != 0

    @property
    def poison(self) -> bool:
        return self.status_flags & POISON_FLAG != 0

    @property
    def chicken(self) -> bool:
        return self.status_flags & CHICKEN_FLAG != 0

    @property
    def frog(self) -> bool:
        return self.status_flags & FROG_FLAG != 0

    @property
    def petrified(self) -> bool:
        return self.status_flags & PETRIFY_FLAG != 0

    @property
    def charm(self) -> bool:
        return self.status_flags & CHARM_FLAG != 0

    @property
    def confusion(self) -> bool:
        return self.status_flags & CONFUSION_FLAG != 0

    @property
    def abandon(self) -> bool:
        return 'Abandon' in self.skills

    @property
    def parry(self) -> bool:
        return 'Parry' in self.skills

    @property
    def blade_grasp(self) -> bool:
        return 'Blade Grasp' in self.skills

    @property
    def arrow_guard(self) -> bool:
        return 'Arrow Guard' in self.skills

    @property
    def throw_item(self) -> bool:
        return 'Throw Item' in self.skills

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
    def has_offhand_weapon(self) -> bool:
        return self.offhand.weapon_type is not None

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
