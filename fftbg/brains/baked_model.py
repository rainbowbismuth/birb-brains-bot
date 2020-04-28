from datetime import datetime
from typing import List

import numpy as np
import pandas

import fftbg.data as data
import fftbg.model as model
import fftbg.patch as patch
import fftbg.tournament as tournament
from fftbg.brains.predictions import Predictions
from fftbg.tournament import MatchUp, Tournament

import fftbg.simulator
from pathlib import Path

class SimulatorModel:
    def __init__(self):
        pass

    def predict_match_ups(self, match_ups: List[MatchUp], patch_date: datetime, n=20) -> np.ndarray:
        patch_json = patch.get_patch(patch_date).to_json()
        patch_obj = fftbg.simulator.Patch(patch_json)
        results = []
        for match_up in match_ups:
            match_up_json = match_up.to_json()
            arena_json = Path(f'data/arena/MAP{match_up.game_map_num:03d}.json').read_text()
            arena_obj = fftbg.simulator.Arena(arena_json)

            left_wins = fftbg.simulator.run_simulation(patch_obj, arena_obj, match_up_json, n)
            results.append([1.0-left_wins, left_wins])
        return np.array(results)

    def predict(self, tourny: Tournament) -> Predictions:
        predictions = self.predict_match_ups(tourny.match_ups, tourny.modified, n=100)

        left_wins = {}
        right_wins = {}
        for i, (team_1, team_2, _) in enumerate(tournament.HYPOTHETICAL_MATCHES):
            match = f'{team_1} {team_2}'
            left_wins[match] = float(predictions[i, 1])
            right_wins[match] = float(predictions[i, 0])

        return Predictions(tourny.id, left_wins, right_wins)


class BakedModel:
    def __init__(self):
        self.model = model.read_model()
        self.column_transformer = model.read_column_transformer()
        self.feature_scalers = model.read_feature_scalers()
        all_combatants_df = data.read_combatants()
        self.all_columns = model.get_all_columns(all_combatants_df)
        self.skill_columns = model.get_skill_columns(all_combatants_df)
        self.status_elemental_columns = model.get_status_elemental_columns(all_combatants_df)

    def predict_match_ups(self, match_ups: List[MatchUp], patch_date: datetime) -> np.ndarray:
        p = patch.get_patch(patch_date)
        df = tournament.match_ups_to_combatants(match_ups, patch=p)
        return self._predict(df)

    def predict(self, tourny: Tournament) -> Predictions:
        df = tournament.tournament_to_combatants([tourny])
        predictions = self._predict(df)

        left_wins = {}
        right_wins = {}
        for i, (team_1, team_2, _) in enumerate(tournament.HYPOTHETICAL_MATCHES):
            match = f'{team_1} {team_2}'
            left_wins[match] = float(predictions[i, 1])
            right_wins[match] = float(predictions[i, 0])

        return Predictions(tourny.id, left_wins, right_wins)

    def _predict(self, df: pandas.DataFrame) -> np.ndarray:
        for column in self.skill_columns:
            if column not in df:
                df[column] = False
        for column in self.status_elemental_columns:
            if column not in df:
                df[column] = 0.0

        combatant_dfs = [df[df['UIDX'] == i][self.all_columns]
                         for i in range(8)]

        combatant_dfs = [combat_df.sort_index(axis=1) for combat_df in combatant_dfs]

        combatant_dfs = [self.column_transformer.transform(combatant_df).astype('float32')
                         for combatant_df in combatant_dfs]

        combatant_dfs = [scaler.transform(combatant_df)
                         for (scaler, combatant_df) in zip(self.feature_scalers, combatant_dfs)]

        return model.mc_predict(self.model, combatant_dfs)
