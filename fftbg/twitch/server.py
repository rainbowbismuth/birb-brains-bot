import logging
import sys
import os
import fftbg.server
from fftbg.twitch.ircbot import IRCBot
from fftbg.twitch.incoming.pubsub import Publisher
from fftbg.twitch.outgoing.pubsub import Subscriber

LOG = logging.getLogger(__name__)


def run_server():
    fftbg.server.configure_logging(env_var='TWITCH_LOG_LEVEL')

    tmi_token = os.environ['TWITCH_TMI_TOKEN']
    client_id = os.environ['TWITCH_CLIENT_ID']
    bot_nick = os.environ['TWITCH_BOT_NICK']
    channel = os.environ['TWITCH_BOT_CHANNEL']
    prefix = '@' + bot_nick

    redis = fftbg.server.get_redis()
    publisher = Publisher(redis)
    subscriber = Subscriber(redis)
    bot = IRCBot(
        irc_token=tmi_token,
        client_id=client_id,
        prefix=prefix,
        nick=bot_nick,
        fftbg_channel=channel,
        publisher=publisher,
        subscriber=subscriber)
    bot.run()


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Twitch IRC Bot died', exc_info=e)
