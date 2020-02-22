from dataclasses import dataclass
from typing import Optional

from dataclasses_json import dataclass_json


@dataclass_json
@dataclass
class Bet:
    color: str
    amount: int


@dataclass_json
@dataclass
class Message:
    bet: Optional[Bet] = None
    say: Optional[str] = None
    balance: bool = False
    pot: bool = False
