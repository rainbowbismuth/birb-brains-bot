import logging
import sys
import time

import fftbg.server
import fftbg.twitch.outgoing.messages as outgoing_messages
from fftbg.twitch.incoming.pubsub import Subscriber
from fftbg.twitch.outgoing.pubsub import Publisher


def main():
    logging.basicConfig(stream=sys.stdout, level='INFO')

    redis = fftbg.server.get_redis()
    sub = Subscriber(redis)
    pub = Publisher(redis)
    while True:
        msg = sub.get_message()
        if msg is None or msg.betting_open is None:
            continue

        if msg.betting_open.right_team != 'champion':
            continue

        print(str(msg.time))
        print(msg)
        time.sleep(120)
        pub.publish(outgoing_messages.Message(say='!buyskill'))
        time.sleep(3)
        pub.publish(outgoing_messages.Message(say='!giftskill'))
        time.sleep(3)
        pub.publish(outgoing_messages.Message(say='Kweh! (Enjoy the skill!)'))


if __name__ == '__main__':
    main()
