import logging
import os
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
    if 'DD_LOGS_INJECTION' in os.environ:
        import json_log_formatter
        formatter = json_log_formatter.JSONFormatter()
        handler = logging.StreamHandler()
        handler.setFormatter(formatter)

        logger = logging.getLogger()
        logger.handlers = []
        logger.setLevel(log_level)
        logger.addHandler(handler)
    else:
        import sys
        logging.basicConfig(stream=sys.stdout, level=log_level)
