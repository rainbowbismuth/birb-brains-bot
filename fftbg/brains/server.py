import logging
import sys
import os
import fftbg.server
from redis import Redis
from fftbg.brains.baked_model import BakedModel, Predictions
from fftbg.twitch.incoming.pubsub import Subscriber
from typing import Optional

LOG = logging.getLogger(__name__)

EXPIRE_SECONDS = 60 * 60


class BrainsAPI:
    def __init__(self, redis: Redis):
        self.redis = redis

    def prediction_key(self, tournament_id: int) -> str:
        return f'tournament_prediction:{tournament_id}'

    def get_predictions(self, tournament_id: int) -> Optional[Predictions]:
        predictions = self.redis.get(self.prediction_key(tournament_id))
        if not predictions:
            return None
        return Predictions.from_json(predictions)

    def set_predictions(self, tournament_id: int, predictions: Predictions):
        self.redis.set(self.prediction_key(tournament_id), predictions, ex=EXPIRE_SECONDS)


def run_server():
    fftbg.server.configure_logging(env_var='BRAINS_LOG_LEVEL')
    redis = fftbg.server.get_redis()
    subscriber = Subscriber(redis)
    api = BrainsAPI(redis)
    model = BakedModel()


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Twitch IRC Bot died', exc_info=e)
