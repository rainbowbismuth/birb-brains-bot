from dataclasses import dataclass
from dataclasses_json import dataclass_json
from typing import Optional
from datetime import datetime


@dataclass_json
@dataclass
class Say:
    user: str
    text: str


@dataclass_json
@dataclass
class Balance:
    user: str
    amount: int


@dataclass_json
@dataclass
class Bet:
    user: str
    team: str
    all_in: bool = False
    amount: Optional[int] = None
    percent: Optional[float] = None


@dataclass_json
@dataclass
class BettingOpen:
    left_team: str
    right_team: int


@dataclass_json
@dataclass
class BettingClosedSorry:
    user: str


@dataclass_json
@dataclass
class TeamBets:
    color: str
    bets: int
    amount: int


@dataclass_json
@dataclass
class BettingPool:
    final: bool
    left_team: TeamBets
    right_team: TeamBets


@dataclass_json
@dataclass
class Message:
    time: datetime
    new_tournament: bool = False
    say: Optional[Say] = None
    bet: Optional[Bet] = None
    balance: Optional[Balance] = None
    betting_open: Optional[BettingOpen] = None
    betting_closed_sorry: Optional[BettingClosedSorry] = None
    betting_pool: Optional[BettingPool] = None
