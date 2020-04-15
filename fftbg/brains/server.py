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
from fftbg.brains.api import CURRENT_TOURNAMENT_KEY, CURRENT_MATCH_KEY, get_current_tournament_id, get_prediction_key, \
    get_prediction, get_importance_key, get_map_key
from fftbg.brains.baked_model import BakedModel
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


def set_prediction(db: Database, prediction: Predictions):
    data = prediction.to_json()
    db.set(get_prediction_key(prediction.tournament_id), data)


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


def post_prediction(db: Database, event_stream: EventStream, model: BakedModel, tournament: Tournament):
    prediction = get_prediction(db, tournament.id)
    if prediction is not None:
        LOG.info(f'Prediction already exists for {tournament.id}')
        return
    LOG.info(f'Computing prediction for {tournament.id}')
    prediction = model.predict(tournament)
    set_prediction(db, prediction)
    msg = {'type': NEW_PREDICTIONS,
           'key': get_prediction_key(prediction.tournament_id)}
    event_stream.publish(msg)
    LOG.info(f'Posted prediction for {tournament.id}')


def set_importance(db: Database, tournament_id, match_up: MatchUp, importance: List[dict]):
    key = get_importance_key(tournament_id, match_up.left.color, match_up.right.color)
    db.set(key, json.dumps(importance))


def set_map(db: Database, tournament_id, match_up: MatchUp):
    key = get_map_key(tournament_id, match_up.left.color, match_up.right.color)
    db.set(key, match_up.game_map)


def post_importance(db: Database, model: BakedModel, tournament_id, match_up: MatchUp, patch_time):
    LOG.info(f'Computing importance for {tournament_id}, {match_up.left.color} vs {match_up.right.color}')
    importance = fftbg.brains.importance.compute(model, match_up, patch_time)
    set_importance(db, tournament_id, match_up, importance)
    LOG.info(f'Posted importance for {tournament_id}, {match_up.left.color} vs {match_up.right.color}')


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='BRAINS_LOG_LEVEL')
    db = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(db)

    model = BakedModel()
    tournament = try_load_new(db, retry_until_new=False)
    LOG.info(f'Loaded initial tournament {tournament.id}')
    post_prediction(db, event_stream, model, tournament)

    while True:
        for (_, msg) in event_stream.read():
            if msg.get('type') == msg_types.CONNECTED_TO_TWITCH:
                tournament = try_load_new(db, retry_until_new=False)
                post_prediction(db, event_stream, model, tournament)

            elif msg.get('type') == msg_types.RECV_BETTING_OPEN:
                left_team = msg['left_team']
                right_team = msg['right_team']

                if left_team == 'red' and right_team == 'blue':
                    tournament = try_load_new(db, retry_until_new=True)
                    post_prediction(db, event_stream, model, tournament)
                idx = fftbg.tournament.look_up_prediction_index(left_team, right_team)
                match_up = tournament.match_ups[idx]

                set_map(db, tournament.id, match_up)
                post_importance(db, model, tournament.id, match_up, tournament.modified)
                set_current_match(db, left_team, right_team)


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Brains died', exc_info=e)
