from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List

from dataclasses_json import dataclass_json
from pytz import timezone

import fftbg.ability as ability
import fftbg.base_stats as base_stats
import fftbg.equipment as equipment
from fftbg.ability import Ability, AbilityData
from fftbg.base_stats import BaseStats, BaseStatsData
from fftbg.config import DATA_PATH
from fftbg.equipment import Equipment, EquipmentData


@dataclass_json
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
EASTERN_TZ = timezone('US/Eastern')


def get_test_patch():
    return get_patch(datetime(year=2020, month=2, day=1, tzinfo=EASTERN_TZ))


def get_patch(when: datetime):
    root = DATA_PATH / 'static'
    for patch_dir in sorted(root.iterdir(), reverse=True):
        year, month, day = [int(x) for x in patch_dir.name.split('-')]
        time = datetime(year=year, month=month, day=day, tzinfo=EASTERN_TZ)
        if when > time:
            return get_patch_from_file(patch_dir)
    raise Exception(f'unable to find patch directory for {str(when)}')


def get_patch_from_file(patch_dir: Path):
    if patch_dir.name in PATCH_MAP:
        return PATCH_MAP[patch_dir.name]
    base_stat_data = base_stats.parse_base_stats(patch_dir / 'classhelp.txt', patch_dir / 'MonsterSkills.txt')
    ability_data = ability.parse_abilities(patch_dir / 'infoability.txt')
    equipment_data = equipment.parse_equipment(patch_dir / 'infoitem.txt')
    PATCH_MAP[patch_dir.name] = Patch(ability=ability_data, equipment=equipment_data, base_stats=base_stat_data)
    return PATCH_MAP[patch_dir.name]
