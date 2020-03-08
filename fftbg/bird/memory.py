import logging
import sqlite3
from dataclasses import dataclass

import fftbg.config as config

LOG = logging.getLogger(__name__)

SCHEMA_BALANCE_LOG = """
CREATE TABLE IF NOT EXISTS 'balance_log' (
    'id' INTEGER PRIMARY KEY AUTOINCREMENT,
    'time' TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    'tournament_id' INTEGER,
    'old_balance' INTEGER,
    'new_balance' INTEGER,
    'bet_on' TEXT,
    'wager' INTEGER,
    'left_team' TEXT,
    'left_prediction' REAL,
    'left_total_on_bet' INTEGER,
    'left_total_final' INTEGER,
    'right_team' TEXT,
    'right_prediction' REAL,
    'right_total_on_bet' INTEGER,
    'right_total_final' INTEGER,
    'left_wins' BOOL)
"""

SCHEMA_PLACED_BET = """ 
CREATE TABLE IF NOT EXISTS 'placed_bet' (
    'id' INTEGER PRIMARY KEY AUTOINCREMENT,
    'time' TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    'tournament_id' INTEGER,
    'bet_on' TEXT,
    'wager' INTEGER,
    'left_team' TEXT,
    'left_prediction' REAL,
    'right_team' TEXT,
    'right_prediction' REAL
)
"""

INSERT_BALANCE_LOG = """
INSERT INTO 'balance_log'(
    tournament_id, old_balance, new_balance, bet_on, wager,
    left_team, left_prediction, left_total_on_bet, left_total_final, 
    right_team, right_prediction, right_total_on_bet, right_total_final, 
    left_wins)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"""

GET_BALANCE_LOG = """
SELECT * FROM 'balance_log' 
ORDER BY id DESC
"""

INSERT_PLACED_BET = """
INSERT INTO 'placed_bet'(
    tournament_id, bet_on, wager, left_team, left_prediction, 
    right_team, right_prediction)
VALUES (?, ?, ?, ?, ?, ?, ?)
"""

GET_PLACED_BETS = """
SELECT * FROM 'placed_bet' 
ORDER BY id DESC
"""


@dataclass
class BalanceLogDTO:
    id: int
    time: str
    tournament: int
    old_balance: int
    new_balance: int
    bet_on: str
    wager: int
    left_team: str
    left_prediction: float
    left_total_on_bet: int
    left_total_final: int
    right_team: str
    right_prediction: float
    right_total_on_bet: int
    right_total_final: int
    left_wins: bool


@dataclass
class PlacedBetDTO:
    id: int
    time: str
    tournament: int
    bet_on: str
    wager: int
    left_team: str
    left_prediction: float
    right_team: str
    right_prediction: float


class Memory:
    def __init__(self, db_path=config.BOT_MEMORY_PATH):
        self.db_path = db_path
        LOG.debug(f'Opening up sqlite3 connection to {self.db_path}')
        self.connection = sqlite3.connect(self.db_path)
        with self.connection:
            self.connection.execute(SCHEMA_BALANCE_LOG)
            self.connection.execute(SCHEMA_PLACED_BET)

    def __del__(self):
        if self.connection is not None:
            self.connection.close()

    def log_balance(self, tournament_id, old_balance, new_balance, bet_on, wager,
                    left_team, left_prediction, left_total_on_bet, left_total_final,
                    right_team, right_prediction, right_total_on_bet, right_total_final,
                    left_wins):
        with self.connection:
            self.connection.execute(
                INSERT_BALANCE_LOG,
                (int(tournament_id), int(old_balance), int(new_balance), bet_on, int(wager),
                 left_team, float(left_prediction), int(left_total_on_bet), int(left_total_final),
                 right_team, float(right_prediction), int(right_total_on_bet), int(right_total_final),
                 bool(left_wins)))

    def get_balance_log(self, limit: int = 200):
        with self.connection:
            tuples = self.connection.execute(GET_BALANCE_LOG).fetchmany(limit)
            return [BalanceLogDTO(*(t[:-1] + (bool(t[-1]),))) for t in tuples]

    def placed_bet(self, tournament_id, bet_on, wager,
                   left_team, left_prediction,
                   right_team, right_prediction):
        with self.connection:
            self.connection.execute(
                INSERT_PLACED_BET,
                (int(tournament_id), bet_on, int(wager),
                 left_team, float(left_prediction),
                 right_team, float(right_prediction)))

    def get_placed_bet(self):
        with self.connection:
            return PlacedBetDTO(*self.connection.execute(GET_PLACED_BETS).fetchone())
