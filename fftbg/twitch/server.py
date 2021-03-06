import logging
import os

import fftbg.event_stream
import fftbg.server
from fftbg.twitch.ircbot import IRCBot

LOG = logging.getLogger(__name__)


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='TWITCH_LOG_LEVEL')

    tmi_token = os.environ['TWITCH_TMI_TOKEN']
    client_id = os.environ['TWITCH_CLIENT_ID']
    bot_nick = os.environ['TWITCH_BOT_NICK']
    channel = os.environ['TWITCH_BOT_CHANNEL']
    mute = bool(int(os.environ['TWITCH_BOT_MUTE']))
    prefix = '@' + bot_nick

    redis = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(redis)
    loop = fftbg.server.get_loop()
    bot = IRCBot(
        loop=loop,
        irc_token=tmi_token,
        client_id=client_id,
        prefix=prefix,
        nick=bot_nick,
        fftbg_channel=channel,
        event_stream=event_stream,
        mute=mute)
    bot.run()


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Twitch IRC Bot died', exc_info=e)
