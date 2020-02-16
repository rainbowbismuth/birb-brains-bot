import asyncio
import json
import logging

import config
import data
import download
import model
from tournament import \
    Tournament, tournament_to_combatants, parse_hypothetical_tournament, HYPOTHETICAL_MATCHES

LOG = logging.getLogger(__name__)


# TODO: What I really want is a Mutex around tournament essentially..

def look_up_prediction_index(left, right):
    for i, (team_1, team_2, _) in enumerate(HYPOTHETICAL_MATCHES):
        if left == team_1 and right == team_2:
            return i
    raise Exception(f'unable to find teams ({left}, {right})')


class BakedModel:
    def __init__(self):
        self.model = model.read_model()
        self.column_transformer = model.read_column_transformer()
        self.feature_scalers = model.read_feature_scalers()
        all_combatants_df = data.read_combatants()
        self.all_columns = model.get_all_columns(all_combatants_df)
        self.skill_columns = model.get_skill_columns(all_combatants_df)

    def predict(self, tournament: Tournament):
        df = tournament_to_combatants([tournament])

        for column in self.skill_columns:
            if column not in df:
                df[column] = False

        combatant_dfs = [df[df['UIDX'] == i][self.all_columns]
                         for i in range(8)]

        combatant_dfs = [combat_df.sort_index(axis=1) for combat_df in combatant_dfs]

        combatant_dfs = [self.column_transformer.transform(combatant_df).astype('float32')
                         for combatant_df in combatant_dfs]

        combatant_dfs = [scaler.transform(combatant_df)
                         for (scaler, combatant_df) in zip(self.feature_scalers, combatant_dfs)]

        predictions = model.mc_predict(self.model, combatant_dfs)
        return predictions


class BotBrains:
    def __init__(self):
        LOG.info('Starting up BotBrains')
        self.model = BakedModel()
        self.balance = 0
        self.refreshing_tournament = asyncio.Event()
        self.tournament_ready = asyncio.Event()
        self.tournament = None
        self.predictions = None
        self.betting_on = None
        self.left_team = None
        self.right_team = None
        self.prediction = None

    def update_balance(self, balance):
        if self.balance == 0:
            self.balance = balance
            LOG.info(f'Balance is {balance} G')
            return
        difference = balance - self.balance
        if difference == 0:
            return
        self.balance = balance
        if difference > 0:
            LOG.info(f'Won {difference} G betting on {self.betting_on}, new balance {self.balance} G')
        else:
            LOG.info(f'Lost {abs(difference)} G betting on {self.betting_on}, new balance {self.balance} G')

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
                sleep_seconds = 10
                LOG.info(f'Waiting for newer tournament, sleeping for {sleep_seconds} seconds')
                await asyncio.sleep(sleep_seconds)
        finally:
            self.refreshing_tournament.clear()

    async def log_prediction(self, left, right):
        if not self.tournament_ready.is_set():
            LOG.info('Skipping log_prediction() because tournament is not ready')
        i = look_up_prediction_index(left, right)
        prediction = self.predictions[i, :]
        LOG.info(f'Prediction is {left} {prediction[1]:.1%} vs {right} {prediction[0]:.1%}')
        self.left_team = left
        self.right_team = right
        self.prediction = prediction

    async def make_bet(self, left_total, right_total):
        if not self.tournament_ready.is_set():
            LOG.info('Skipping make_bet() because tournament is not ready')
        pool_total = left_total + right_total

        left_wins_percent = self.prediction[1]
        right_wins_percent = self.prediction[0]

        LOG.info(f'Adjusted predictions to {self.left_team} {left_wins_percent:.1%} vs'
                 f' {self.right_team} {right_wins_percent:.1%}')

        left_wins = left_wins_percent * right_total
        right_wins = right_wins_percent * left_total
        if left_wins > right_wins:
            self.betting_on = self.left_team
            return self.left_team, self._how_much_to_bet(left_wins_percent, pool_total)
        else:
            self.betting_on = self.right_team
            return self.right_team, self._how_much_to_bet(right_wins_percent, pool_total)

    def _how_much_to_bet(self, confidence, pool_total):
        amount = max(200, self.balance * (confidence / 10.0))
        betting_cap = pool_total // 10
        if amount > betting_cap:
            LOG.info(f'Capping bet at {betting_cap}')
            return betting_cap
        return amount


def main():
    import json
    baked = BakedModel()
    fp = config.TOURNAMENTS_ROOT / '1580897292273.json'
    js = json.loads(fp.read_text())
    hypothetical = parse_hypothetical_tournament(js)
    _predictions = baked.predict(hypothetical)


if __name__ == '__main__':
    main()
