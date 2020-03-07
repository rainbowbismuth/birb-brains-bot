from dataclasses import dataclass
from datetime import datetime
from typing import Dict, List

import numpy as np
import pandas
from dataclasses_json import dataclass_json

import fftbg.data as data
import fftbg.model as model
import fftbg.patch as patch
import fftbg.tournament as tournament
from fftbg.tournament import MatchUp, Tournament


@dataclass_json()
@dataclass
class Predictions:
    tournament_id: int
    left_wins: Dict[str, float]
    right_wins: Dict[str, float]


class BakedModel:
    def __init__(self):
        self.model = model.read_model()
        self.column_transformer = model.read_column_transformer()
        self.feature_scalers = model.read_feature_scalers()
        all_combatants_df = data.read_combatants()
        self.all_columns = model.get_all_columns(all_combatants_df)
        self.skill_columns = model.get_skill_columns(all_combatants_df)

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
            left_wins[match] = float(predictions[i, 0])
            right_wins[match] = float(predictions[i, 1])

        return Predictions(tourny.id, left_wins, right_wins)

    def _predict(self, df: pandas.DataFrame) -> np.ndarray:
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

        return model.mc_predict(self.model, combatant_dfs)
