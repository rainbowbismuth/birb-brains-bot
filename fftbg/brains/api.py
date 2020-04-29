import json
from typing import Optional, List

from walrus import Database

from fftbg.brains.predictions import Predictions

CURRENT_TOURNAMENT_KEY = 'brains.tournament_id'
CURRENT_MATCH_KEY = 'brains.match_up_teams'


def get_current_tournament_id(db: Database):
    return db.get(CURRENT_TOURNAMENT_KEY)


def get_current_match(db: Database):
    data = db.get(CURRENT_MATCH_KEY)
    if data is not None:
        data = tuple(data.split(' '))
    return data


def get_prediction_key(tournament_id):
    return f'brains.predictions:{tournament_id}'


def get_prediction(db: Database, tournament_id) -> Optional[Predictions]:
    data = db.get(get_prediction_key(tournament_id))
    if data is not None:
        data = Predictions.from_json(data)
    return data


def get_importance_key(tournament_id, left_team, right_team):
    return f'brains.importance:{tournament_id}-{left_team}-{right_team}'


def get_importance(db: Database, tournament_id, left_team, right_team) -> List[dict]:
    key = get_importance_key(tournament_id, left_team, right_team)
    data = db.get(key)
    if data is not None:
        data = json.loads(data)
    return data


def get_map_key(tournament_id, left_team, right_team):
    return f'brains.map:{tournament_id}-{left_team}-{right_team}'


def get_map(db: Database, tournament_id, left_team, right_team) -> str:
    key = get_map_key(tournament_id, left_team, right_team)
    return db.get(key)


def get_sim_log_key(tournament_id, left_team, right_team):
    return f'brains.sim-log:{tournament_id}-{left_team}-{right_team}'


def get_sim_log(db: Database, tournament_id, left_team, right_team) -> List[str]:
    key = get_sim_log_key(tournament_id, left_team, right_team)
    data = db.get(key)
    if data is not None:
        data = json.loads(data)
    return data
