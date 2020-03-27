import asyncio
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


def get_loop() -> asyncio.AbstractEventLoop:
    LOG = logging.getLogger(__name__)

    def handle_exception(_loop, context):
        if 'exception' in context:
            LOG.critical('uncaught exception', exc_info=context['exception'])
        else:
            LOG.critical(f'exception {context["message"]}')
        import os
        os._exit(1)

    loop = asyncio.get_event_loop()
    loop.set_exception_handler(handle_exception)
    return loop
