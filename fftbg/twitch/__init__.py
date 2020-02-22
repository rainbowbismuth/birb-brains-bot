import logging

LOG = logging.getLogger(__name__)


def main():
    import sys
    import os
    import asyncio
    log_level = os.environ.get('TWITCH_LOG_LEVEL', 'INFO')
    logging.basicConfig(stream=sys.stdout, level=log_level)

    import fftbg.redis
    from fftbg.twitch.ircbot import IRCBot
    from fftbg.twitch.incoming.pubsub import Publisher
    from fftbg.twitch.outgoing.pubsub import Subscriber

    tmi_token = os.environ['TWITCH_TMI_TOKEN']
    client_id = os.environ['TWITCH_CLIENT_ID']
    bot_nick = os.environ['TWITCH_BOT_NICK']
    channel = os.environ['TWITCH_BOT_CHANNEL']
    prefix = '@' + bot_nick

    redis = fftbg.redis.get_redis()
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


if __name__ == '__main__':
    try:
        main()
    except Exception as e:
        LOG.critical('Bot died', exc_info=e)
