import logging
import sys

import fftbg.redis
from fftbg.twitch.incoming.pubsub import Subscriber


def main():
    logging.basicConfig(stream=sys.stdout, level='INFO')

    redis = fftbg.redis.get_redis()
    sub = Subscriber(redis)
    while True:
        msg = sub.get_message()
        if msg is None:
            continue
        print(str(msg.time))
        print(msg)


if __name__ == '__main__':
    main()
