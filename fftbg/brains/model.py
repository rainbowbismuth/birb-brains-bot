from datetime import datetime
from pathlib import Path
from typing import List

import fftbg.simulator
import numpy as np
import pandas
import config
import pickle

import fftbg.model as model
import fftbg.patch as patch
import fftbg.tournament as tournament
from fftbg.tournament import MatchUp
import logging

LOG = logging.getLogger(__name__)


class Model:
    def predict_match_up(self, match_up: MatchUp, patch_date: datetime) -> float:
        raise NotImplemented()


class BakedModel(Model):
    def __init__(self):
        self.model = model.read_model()
        self.column_transformer = model.read_column_transformer()
        self.feature_scalers = model.read_feature_scalers()

        with config.COLUMN_SET_PATH.open() as f:
            (all_columns, skill_columns, status_elemental_columns) = pickle.load(f)
            self.all_columns = all_columns
            self.skill_columns = skill_columns
            self.status_elemental_columns = status_elemental_columns

    def predict_match_up(self, match_up: MatchUp, patch_date: datetime, sim_left_wins: float = 0.5) -> float:
        p = patch.get_patch(patch_date)
        df = tournament.match_up_to_combatants(match_up, patch=p, sim_left_wins=sim_left_wins)
        res = self._predict(df)
        return res[0, 1]

    def predict_match_ups(self, match_ups: List[MatchUp], patch_date: datetime) -> np.ndarray:
        p = patch.get_patch(patch_date)
        df = tournament.match_ups_to_combatants(match_ups, p)
        return self._predict(df)

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


class SimulatorModel(Model):
    def __init__(self, baked: BakedModel, num_runs: int = 1000):
        self.num_runs = num_runs
        self.baked = baked

    def predict_match_up(self, match_up: MatchUp, patch_date: datetime) -> float:
        patch_json = patch.get_patch(patch_date).to_json()
        patch_obj = fftbg.simulator.Patch(patch_json)
        match_up_json = match_up.to_json()
        arena_json = Path(f'data/arena/MAP{match_up.game_map_num:03d}.json').read_text()
        arena_obj = fftbg.simulator.Arena(arena_json)
        left_wins = fftbg.simulator.run_simulation(patch_obj, arena_obj, match_up_json, self.num_runs)
        LOG.info(f'sim_left_wins = {left_wins:.4f}')
        return self.baked.predict_match_up(match_up, patch_date, left_wins)

