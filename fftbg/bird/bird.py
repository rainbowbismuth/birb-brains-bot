import logging
from typing import Optional

from walrus import Database

import fftbg.betting as betting
from fftbg.bird.memory import Memory
from fftbg.brains.api import get_current_tournament_id, get_prediction
from fftbg.brains.predictions import Predictions
from fftbg.event_stream import EventStream

LOG = logging.getLogger(__name__)


class Bird:
    def __init__(self, db: Database, event_stream: EventStream):
        self.db = db
        self.event_stream = event_stream
        self.current_tournament_id = None
        self.predictions: Optional[Predictions] = None
        self.memory = Memory()
        self.balance = 0

        self.left_team = None
        self.right_team = None
        self.prediction = None

        self.moving_increase = 2.0

        # Per Bet information we need to log when victor is confirmed
        self.tournament_id = None
        self.betting_on = None
        self.wager = 0.0
        self.left_team_bet = None
        self.left_prediction = None
        self.left_total_on_bet = None
        self.left_total_final = None
        self.right_team_bet = None
        self.right_prediction = None
        self.right_total_on_bet = None
        self.right_total_final = None

    def load_current_tournament(self):
        self.current_tournament_id = get_current_tournament_id(self.db)
        self.predictions = get_prediction(self.db, self.current_tournament_id)
        LOG.info(f'Loaded predictions for {self.current_tournament_id}')

    def update_balance(self, balance):
        old_balance = self.balance
        if self.balance == 0:
            self.balance = balance
            LOG.info(f'Balance is {balance} G')
            return
        difference = balance - self.balance
        if difference == 0:
            return
        self.balance = balance
        if difference > 0:
            left_wins = self.left_team_bet == self.betting_on
            LOG.info(f'Won {difference} G betting on {self.betting_on}, new balance {self.balance} G')
        else:
            left_wins = self.left_team_bet != self.betting_on
            LOG.info(f'Lost {abs(difference)} G betting on {self.betting_on}, new balance {self.balance} G')
        if self.left_team_bet is None or self.right_team_bet is None:
            return  # skip if we restarted the bot, essentially.
        self.memory.log_balance(
            tournament_id=self.tournament_id,  # we can't use self.current_tournament_id, it may have changed
            old_balance=old_balance,
            new_balance=self.balance,
            bet_on=self.betting_on,
            wager=self.wager,
            left_team=self.left_team_bet,
            left_prediction=self.left_prediction,
            left_total_on_bet=self.left_total_on_bet,
            left_total_final=self.left_total_final,
            right_team=self.right_team_bet,
            right_prediction=self.right_prediction,
            right_total_on_bet=self.right_total_on_bet,
            right_total_final=self.right_total_final,
            left_wins=left_wins)

    def final_odds(self, left_total, right_total):
        self.left_total_final = left_total
        self.right_total_final = right_total

        if not self.betting_on:
            return

        old_increase = self.moving_increase
        left_increase = self.left_total_final / self.left_total_on_bet
        right_increase = self.right_total_final / self.right_total_on_bet
        average = (left_increase + right_increase) / 2
        LOG.info(f'Average pool increase was {average:.4}')
        self.moving_increase = old_increase * 0.80 + min(old_increase * 3, average) * 0.20
        self.moving_increase = max(1, self.moving_increase)
        LOG.info(f'Moving increase changed from {old_increase:.4} to {self.moving_increase:.4}')

    def log_prediction(self, left, right):
        pred_key = f'{left} {right}'
        left_wins = self.predictions.left_wins[pred_key]
        right_wins = self.predictions.right_wins[pred_key]
        prediction = [right_wins, left_wins]
        LOG.info(f'Prediction is {left} {prediction[1]:.1%} vs {right} {prediction[0]:.1%}')
        self.left_team = left
        self.right_team = right
        self.prediction = prediction

    def make_bet(self, left_total, right_total):
        pool_total_est = (left_total + right_total) * self.moving_increase

        left_wins_percent = self.prediction[1]
        right_wins_percent = self.prediction[0]

        self.tournament_id = self.current_tournament_id
        self.left_team_bet = self.left_team
        self.right_team_bet = self.right_team
        self.left_prediction = left_wins_percent
        self.right_prediction = right_wins_percent
        self.left_total_on_bet = left_total
        self.right_total_on_bet = right_total

        new_left_total = left_total * self.moving_increase
        new_right_total = right_total * self.moving_increase

        left_bet = betting.optimal_bet(left_wins_percent, new_left_total, new_right_total)
        right_bet = betting.optimal_bet(right_wins_percent, new_right_total, new_left_total)

        LOG.info(f'Optimal bet: {int(left_bet)} vs {int(right_bet)}')
        assert not (left_bet > 0 and right_bet > 0)

        if left_bet > right_bet:
            self.betting_on = self.left_team
            self.wager = left_bet
        else:
            self.betting_on = self.right_team
            self.wager = right_bet
        self.wager = min(self.wager, 1000 + pool_total_est * 0.02)
        if left_bet > right_bet:
            self.wager *= left_wins_percent
        else:
            self.wager *= right_wins_percent
        if self.balance <= 200:
            self.wager = 200
        else:
            self.wager = int(max(100, self.wager))
        self.wager = min(self.wager, self.balance, 1000)

        self.memory.placed_bet(
            self.tournament_id, self.betting_on, self.wager,
            self.left_team_bet, self.left_prediction,
            self.right_team_bet, self.right_prediction
        )
        return self.betting_on, self.wager
