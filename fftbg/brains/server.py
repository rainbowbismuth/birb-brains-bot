import json
import logging
import time
from typing import List

from walrus import Database

import fftbg.brains.importance
import fftbg.download
import fftbg.event_stream
import fftbg.server
import fftbg.tournament
import fftbg.twitch.msg_types as msg_types
from fftbg.brains.api import CURRENT_TOURNAMENT_KEY, CURRENT_MATCH_KEY, get_current_tournament_id, get_predictions_key, \
    get_predictions, get_importance_key, get_map_key, get_sim_log_key, get_prediction_key, get_prediction
from fftbg.brains.model import BakedModel, SimulatorModel, Model
from fftbg.brains.msg_types import NEW_PREDICTIONS
from fftbg.brains.predictions import Predictions
from fftbg.event_stream import EventStream
from fftbg.tournament import Tournament, MatchUp

LOG = logging.getLogger(__name__)

SLEEP_TIME = 2.0


def set_current_tournament_id(db: Database, tournament_id):
    db.set(CURRENT_TOURNAMENT_KEY, tournament_id)


def set_current_match(db: Database, left_team, right_team):
    db.set(CURRENT_MATCH_KEY, f'{left_team} {right_team}')


def set_prediction(db: Database, tournament_id, left_team, right_team, left_wins: float):
    db.set(get_prediction_key(tournament_id, left_team, right_team), str(left_wins))


def try_load_new(db: Database, retry_until_new=True) -> Tournament:
    tournament = None
    current_id = '0'
    existing_id = int(get_current_tournament_id(db))

    while tournament is None or retry_until_new:
        text = fftbg.download.get_latest_tournament()
        tournament = fftbg.tournament.parse_hypothetical_tournament(json.loads(text))
        current_id = tournament.id
        if current_id != existing_id:
            break
        elif retry_until_new:
            time.sleep(SLEEP_TIME)

    set_current_tournament_id(db, current_id)
    LOG.info(f'Set tournament id to {current_id}')
    return tournament


def post_prediction(db: Database, event_stream: EventStream, model: Model, tournament: Tournament, left_team: str, right_team: str):
    prediction = get_prediction(db, tournament.id, left_team, right_team)
    if prediction is not None:
        LOG.info(f'Prediction already exists for {tournament.id}-{left_team}-{right_team}')
        return
    LOG.info(f'Computing prediction for {tournament.id}-{left_team}-{right_team}')
    match_up = tournament.find_match_up(left_team, right_team)
    left_wins = model.predict_match_up(match_up, tournament.modified)
    set_prediction(db, tournament.id, left_team, right_team, left_wins)
    msg = {'type': NEW_PREDICTIONS,
           'key': get_prediction_key(tournament.id, left_team, right_team)}
    event_stream.publish(msg)
    LOG.info(f'Posted prediction for {tournament.id}-{left_team}-{right_team}')


def set_importance(db: Database, tournament_id, match_up: MatchUp, importance: List[dict]):
    key = get_importance_key(tournament_id, match_up.left.color, match_up.right.color)
    db.set(key, json.dumps(importance))


def set_sim_log(db: Database, tournament_id, match_up: MatchUp, log: List[str]):
    key = get_sim_log_key(tournament_id, match_up.left.color, match_up.right.color)
    db.set(key, json.dumps(log))


def set_map(db: Database, tournament_id, match_up: MatchUp):
    key = get_map_key(tournament_id, match_up.left.color, match_up.right.color)
    db.set(key, match_up.game_map)


def post_importance(db: Database, model, tournament_id, match_up: MatchUp, patch_time):
    LOG.info(f'Computing importance for {tournament_id}, {match_up.left.color} vs {match_up.right.color}')
    importance = fftbg.brains.importance.compute(model, match_up, patch_time)
    set_importance(db, tournament_id, match_up, importance)
    LOG.info(f'Posted importance for {tournament_id}, {match_up.left.color} vs {match_up.right.color}')


def post_sim_log(db: Database, model: SimulatorModel, tournament_id, match_up: MatchUp, patch_time):
    LOG.info(f'Computing simulation log for {tournament_id}, {match_up.left.color} vs {match_up.right.color}')
    log = model.predict_sim_match(match_up, patch_time)
    set_sim_log(db, tournament_id, match_up, log)
    LOG.info(f'Posted simulation log for {tournament_id}, {match_up.left.color} vs {match_up.right.color}')


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='BRAINS_LOG_LEVEL')
    db = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(db)

    baked_model = BakedModel()
    sim_model = SimulatorModel(baked_model)

    tournament = try_load_new(db, retry_until_new=False)
    LOG.info(f'Loaded initial tournament {tournament.id}')

    while True:
        for (_, msg) in event_stream.read():
            if msg.get('type') == msg_types.CONNECTED_TO_TWITCH:
                tournament = try_load_new(db, retry_until_new=False)

            elif msg.get('type') == msg_types.RECV_BETTING_OPEN:
                left_team = msg['left_team']
                right_team = msg['right_team']

                if left_team == 'red' and right_team == 'blue':
                    tournament = try_load_new(db, retry_until_new=True)

                post_prediction(db, event_stream, sim_model, tournament, left_team, right_team)

                idx = fftbg.tournament.look_up_prediction_index(left_team, right_team)
                match_up = tournament.match_ups[idx]

                set_map(db, tournament.id, match_up)
                # post_sim_log(db, model, tournament.id, match_up, tournament.modified)
                post_importance(db, baked_model, tournament.id, match_up, tournament.modified)
                set_current_match(db, left_team, right_team)


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Brains died', exc_info=e)
