from dataclasses import dataclass
from typing import Dict

from dataclasses_json import dataclass_json


@dataclass_json()
@dataclass
class Predictions:
    tournament_id: int
    left_wins: Dict[str, float]
    right_wins: Dict[str, float]
