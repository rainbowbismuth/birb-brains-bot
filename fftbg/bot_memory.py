import logging
import sqlite3

import config

LOG = logging.getLogger(__name__)

SCHEMA = """
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
"""


class BotMemory:
    def __init__(self, db_path=config.BOT_MEMORY_PATH):
        self.db_path = db_path
        LOG.info(f'Opening up sqlite3 connection to {self.db_path}')
        self.connection = sqlite3.connect(self.db_path)
        with self.connection:
            self.connection.execute(SCHEMA)

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

    def get_balance_log(self):
        with self.connection:
            return self.connection.execute(GET_BALANCE_LOG).fetchall()
