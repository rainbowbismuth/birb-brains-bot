from dataclasses import dataclass
from datetime import datetime
from typing import List

import pytz

import fftbg.ability as ability
import fftbg.base_stats as base_stats
import fftbg.equipment as equipment
from fftbg.ability import Ability, AbilityData
from fftbg.base_stats import BaseStats, BaseStatsData
from fftbg.config import DATA_PATH
from fftbg.equipment import Equipment, EquipmentData


@dataclass
class Patch:
    ability: AbilityData
    equipment: EquipmentData
    base_stats: BaseStatsData

    def get_ability(self, name: str) -> Ability:
        return self.ability.get_ability(name)

    def get_ability_by_adds(self, status: str) -> List[Ability]:
        return self.ability.get_ability_by_adds(status)

    def get_ability_by_cancels(self, status: str) -> List[Ability]:
        return self.ability.get_ability_by_cancels(status)

    def get_equipment(self, name: str) -> Equipment:
        return self.equipment.get_equipment(name)

    def get_base_stats(self, job: str, gender: str) -> BaseStats:
        return self.base_stats.get_base_stats(job, gender)


PATCH_MAP: {str: Patch} = {}


def get_test_patch():
    return get_patch_from_file('initial')


def get_patch(when: datetime):
    # TODO: hard coded for now
    if when > datetime(year=2020, month=2, day=26, tzinfo=pytz.utc):
        return get_patch_from_file('2020-02-26')
    if when > datetime(year=2020, month=2, day=25, tzinfo=pytz.utc):
        return get_patch_from_file('2020-02-25')
    else:
        return get_patch_from_file('initial')


def get_patch_from_file(filename: str):
    if filename in PATCH_MAP:
        return PATCH_MAP[filename]
    root = DATA_PATH / 'static' / filename
    base_stat_data = base_stats.parse_base_stats(root / 'classhelp.txt')
    ability_data = ability.parse_abilities(root / 'infoability.txt')
    equipment_data = equipment.parse_equipment(root / 'infoitem.txt')
    PATCH_MAP[filename] = Patch(ability=ability_data, equipment=equipment_data, base_stats=base_stat_data)
    return PATCH_MAP[filename]
