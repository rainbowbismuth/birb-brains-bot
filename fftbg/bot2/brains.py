import asyncio
import json
import logging

from redis import Redis
import fftbg.betting as betting
import fftbg.data as data
import fftbg.download as download
import fftbg.model as model
from fftbg.bot.memory import BotMemory
from fftbg.tournament import Tournament, tournament_to_combatants, parse_hypothetical_tournament
from fftbg.bot2.baked_model import BakedModel

LOG = logging.getLogger(__name__)


class BotBrains:
    def __init__(self):
        LOG.info('Starting up BotBrains')
        self.model = BakedModel()
        self.memory = BotMemory()
        self.balance = 0
        self.refreshing_tournament = asyncio.Event()
        self.tournament_ready = asyncio.Event()
        self.tournament = None
        self.predictions = None
        self.left_team = None
        self.right_team = None
        self.prediction = None

        self.moving_increase = 2.0

        # Per Bet information we need to log when victor is confirmed
        self.tournament_id = None
        self.betting_on = None
        self.wager = None
        self.left_team_bet = None
        self.left_prediction = None
        self.left_total_on_bet = None
        self.left_total_final = None
        self.right_team_bet = None
        self.right_prediction = None
        self.right_total_on_bet = None
        self.right_total_final = None

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
            tournament_id=self.tournament_id,  # we can't use self.tournament, it may have changed
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

    @property
    def tournament_id_or_none(self):
        if self.tournament:
            return self.tournament.id
        return None

    def new_tournament(self):
        LOG.info('New tournament starting')
        self.tournament_ready.clear()

    async def refresh_tournament(self):
        if self.refreshing_tournament.is_set():
            LOG.info(f'Already refreshing tournament, skipping')
            return
        try:
            self.refreshing_tournament.set()
            self.tournament_ready.clear()
            while True:
                text = download.get_latest_tournament()
                tournament = parse_hypothetical_tournament(json.loads(text))
                if self.tournament_id_or_none != tournament.id:
                    LOG.info(f'Refreshed tournament new: {tournament.id}, old: {self.tournament_id_or_none}')
                    self.tournament = tournament
                    LOG.info(f'Running predictions')
                    self.predictions = self.model.predict(self.tournament)
                    LOG.info(f'Tournament ready')
                    self.tournament_ready.set()
                    return
                sleep_seconds = 2.5
                LOG.info(f'Waiting for newer tournament, sleeping for {sleep_seconds} seconds')
                await asyncio.sleep(sleep_seconds)
        except Exception as e:
            LOG.error(f'Error refreshing tournament', exc_info=e)
        finally:
            self.refreshing_tournament.clear()
            LOG.info(f'tournament_ready.is_set() = {self.tournament_ready.is_set()}')

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
        self.moving_increase = old_increase * 0.85 + min(old_increase * 3, average) * 0.15
        self.moving_increase = max(1, self.moving_increase)
        LOG.info(f'Moving increase changed from {old_increase:.4} to {self.moving_increase:.4}')

    def log_prediction(self, left, right):
        if not self.tournament_ready.is_set():
            LOG.info('Skipping log_prediction() because tournament is not ready')
            return
        i = look_up_prediction_index(left, right)
        prediction = self.predictions[i, :]
        LOG.info(f'Prediction is {left} {prediction[1]:.1%} vs {right} {prediction[0]:.1%}')
        self.left_team = left
        self.right_team = right
        self.prediction = prediction

    def make_bet(self, left_total, right_total):
        if not self.tournament_ready.is_set():
            LOG.info('Skipping make_bet() because tournament is not ready')
            return
        pool_total_est = (left_total + right_total) * self.moving_increase

        left_wins_percent = self.prediction[1]
        right_wins_percent = self.prediction[0]

        self.tournament_id = self.tournament.id
        self.left_team_bet = self.left_team
        self.right_team_bet = self.right_team
        self.left_prediction = left_wins_percent
        self.right_prediction = right_wins_percent
        self.left_total_on_bet = left_total
        self.right_total_on_bet = right_total

        MIN_BET = 200
        MAX_BET_PERCENT = 0.10

        # if self.left_prediction > self.right_prediction:
        #     self.betting_on = self.left_team
        #     self.wager = int(
        #         max(MIN_BET, min(self.balance * MAX_BET_PERCENT * self.left_prediction, pool_total_est / 10.0)))
        # else:
        #     self.betting_on = self.right_team
        #     self.wager = int(
        #         max(MIN_BET, min(self.balance * MAX_BET_PERCENT * self.right_prediction, pool_total_est / 10.0)))

        new_left_total = min(left_total, right_total * 3)
        new_right_total = min(left_total * 3, right_total)
        LOG.info(f'Capping left total {left_total} -> {new_left_total}')
        LOG.info(f'Capping right total {right_total} -> {new_right_total}')

        optimistic_left_bet = betting.optimal_bet(left_wins_percent, new_left_total, new_right_total)
        left_optimal_bet = betting.optimal_bet(
            left_wins_percent, new_left_total * self.moving_increase, new_right_total * self.moving_increase)

        optimistic_right_bet = betting.optimal_bet(right_wins_percent, new_right_total, new_left_total)
        right_optimal_bet = betting.optimal_bet(
            right_wins_percent, new_right_total * self.moving_increase, new_left_total * self.moving_increase)

        LOG.info(f'Optimistic optimal bet: {int(optimistic_left_bet)} vs {int(optimistic_right_bet)}')
        LOG.info(f'Pessimistic optimal bet: {int(left_optimal_bet)} vs {int(right_optimal_bet)} ')

        assert not (left_optimal_bet > 0 and right_optimal_bet > 0)
        if left_optimal_bet > right_optimal_bet:
            self.betting_on = self.left_team
            self.wager = int(max(MIN_BET, min(left_optimal_bet, self.balance * MAX_BET_PERCENT)))
        else:
            self.betting_on = self.right_team
            self.wager = int(max(MIN_BET, min(right_optimal_bet, self.balance * MAX_BET_PERCENT)))

        self.memory.placed_bet(
            self.tournament_id, self.betting_on, self.wager,
            self.left_team_bet, self.left_prediction,
            self.right_team_bet, self.right_prediction
        )
        return self.betting_on, self.wager