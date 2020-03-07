import logging
import os
import sys
from typing import cast

from walrus import Database

NAME = None


def set_name(name: str):
    global NAME
    assert NAME is None
    NAME = name


def get_name() -> str:
    assert NAME is not None
    return cast(str, NAME)


def get_redis() -> Database:
    host = os.environ.get('REDIS_HOST', 'localhost')
    port = int(os.environ.get('REDIS_PORT', '6379'))
    return Database(host=host, port=port, decode_responses=True)


def configure_logging(env_var: str):
    log_level = os.environ.get(env_var, 'INFO')
    logging.basicConfig(stream=sys.stdout, level=log_level)
