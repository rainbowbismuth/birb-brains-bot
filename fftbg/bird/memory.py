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

SCHEMA_NOTIFY_SKILL_DROP = """
CREATE TABLE IF NOT EXISTS 'notify_skill_drop' (
    'id' INTEGER PRIMARY KEY AUTOINCREMENT,
    'time' TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    'user_id' INTEGER,
    'user_name' TEXT,
    'skill_drop' TEXT
)
"""

SCHEMA_DISCORD_TWITCH_LINK = """
CREATE TABLE IF NOT EXISTS 'twitch_link' (
    'user_id' INTEGER PRIMARY KEY,
    'twitch_user_name' TEXT,
    'time' TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)
"""

SCHEMA_NOTIFY_SKILL_DROP_INDEX = """
CREATE UNIQUE INDEX IF NOT EXISTS 'notify_skill_drop_index' on 'notify_skill_drop' (
    'user_id', 'skill_drop' )
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

INSERT_NOTIFY_SKILL_DROP_REQUEST = """
INSERT OR IGNORE INTO 'notify_skill_drop'(
    user_id, user_name, skill_drop)
VALUES (?, ?, ?)
"""

DELETE_NOTIFY_SKILL_DROP_REQUEST = """
DELETE FROM 'notify_skill_drop'
WHERE user_id = ? AND skill_drop = ?
"""

CLEAR_ALL_NOTIFY_SKILL_DROP_REQUESTS = """
DELETE FROM 'notify_skill_drop'
WHERE user_id = ?
"""

GET_SKILL_DROP_NOTIFICATION_REQUESTS = """
SELECT skill_drop
FROM 'notify_skill_drop'
WHERE user_id = ?
"""

GET_USERS_TO_SKILL_DROP_NOTIFY = """
SELECT user_id, user_name
FROM 'notify_skill_drop'
WHERE skill_drop = ?
"""

REPLACE_INTO_DISCORD_TWITCH_LINK = """
REPLACE INTO 'twitch_link'(user_id, twitch_user_name)
VALUES (?, ?)
"""

UNLINK_TWITCH = """
DELETE FROM 'twitch_link'
WHERE user_id = ?
"""

FIND_TWITCH_USER_NAME = """
SELECT twitch_user_name
FROM 'twitch_link'
WHERE user_id = ?
"""

FIND_DISCORD_FROM_TWITCH = """
SELECT user_id
FROM 'twitch_link'
WHERE twitch_user_name = ? COLLATE NOCASE
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
    def __init__(self, schema_check=True, db_path=config.BOT_MEMORY_PATH):
        self.db_path = db_path
        LOG.debug(f'Opening up sqlite3 connection to {self.db_path}')
        self.connection = sqlite3.connect(self.db_path)
        if schema_check:
            with self.connection:
                self.connection.execute(SCHEMA_BALANCE_LOG)
                self.connection.execute(SCHEMA_PLACED_BET)
                self.connection.execute(SCHEMA_NOTIFY_SKILL_DROP)
                self.connection.execute(SCHEMA_NOTIFY_SKILL_DROP_INDEX)
                self.connection.execute(SCHEMA_DISCORD_TWITCH_LINK)

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

    def add_notify_skill_drop_requests(self, user_id, user_name, skill_drop_list):
        with self.connection:
            for skill_drop in skill_drop_list:
                self.connection.execute(
                    INSERT_NOTIFY_SKILL_DROP_REQUEST,
                    (int(user_id), user_name, skill_drop))

    def get_skill_drop_notify_requests(self, user_id, limit: int = 300):
        with self.connection:
            tuples = self.connection.execute(
                GET_SKILL_DROP_NOTIFICATION_REQUESTS, (int(user_id),)).fetchmany(limit)
            return [str(item[0]) for item in tuples]

    def remove_notify_skill_drop_requests(self, user_id, skill_drop_list):
        with self.connection:
            for skill_drop in skill_drop_list:
                self.connection.execute(
                    DELETE_NOTIFY_SKILL_DROP_REQUEST,
                    (int(user_id), skill_drop))

    def clear_notify_skill_drop_requests(self, user_id):
        with self.connection:
            self.connection.execute(
                CLEAR_ALL_NOTIFY_SKILL_DROP_REQUESTS,
                (int(user_id),))

    def get_users_to_skill_drop_notify(self, skill_drop):
        with self.connection:
            return self.connection.execute(
                GET_USERS_TO_SKILL_DROP_NOTIFY,
                (skill_drop,)).fetchall()

    def set_discord_twitch_link(self, user_id, twitch_user_name):
        with self.connection:
            self.connection.execute(REPLACE_INTO_DISCORD_TWITCH_LINK, (int(user_id), twitch_user_name))

    def find_twitch_user_name(self, user_id):
        with self.connection:
            res = self.connection.execute(FIND_TWITCH_USER_NAME, (int(user_id),)).fetchone()
            if not res:
                return None
            return res[0]

    def find_discord_id_from_twitch(self, twitch_user_name):
        with self.connection:
            res = self.connection.execute(FIND_DISCORD_FROM_TWITCH, (twitch_user_name,)).fetchone()
            if not res:
                return None
            return res[0]

    def unlink_twitch_account(self, user_id):
        with self.connection:
            self.connection.execute(UNLINK_TWITCH, (int(user_id),))
