import os

from redis import Redis
import logging
import sys


def get_redis() -> Redis:
    host = os.environ.get('REDIS_HOST', 'localhost')
    port = int(os.environ.get('REDIS_PORT', '6379'))
    return Redis(host=host, port=port)


def configure_logging(env_var: str):
    log_level = os.environ.get(level_var, 'INFO')
    logging.basicConfig(stream=sys.stdout, level=log_level)
