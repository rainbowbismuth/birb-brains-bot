import os

from redis import Redis


def get_redis() -> Redis:
    host = os.environ.get('REDIS_HOST', 'localhost')
    port = int(os.environ.get('REDIS_PORT', '6379'))
    return Redis(host=host, port=port)
