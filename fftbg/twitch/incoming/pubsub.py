from redis import Redis
import fftbg.twitch.incoming.messages as messages
import logging
from typing import Optional

LOG = logging.getLogger(__name__)
CHANNEL = 'twitch_incoming'


class Publisher:
    def __init__(self, redis: Redis):
        self.redis = redis

    def publish(self, message: messages.Message):
        msg_json = message.to_json()
        self.redis.publish(CHANNEL, msg_json)
        LOG.debug(f'Published {CHANNEL} <- {msg_json}')


class Subscriber:
    def __init__(self, redis: Redis):
        self.redis = redis
        self.sub = redis.pubsub()
        self.sub.subscribe(CHANNEL)

    def get_message(self, timeout: float = 60.0) -> Optional[messages.Message]:
        redis_msg = self.sub.get_message(ignore_subscribe_messages=True, timeout=timeout)
        if redis_msg is None:
            return None
        msg = messages.Message.from_json(redis_msg['data'])
        LOG.debug(f'Received {CHANNEL} -> {str(msg)}')
        return msg
