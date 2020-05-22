import logging
import os

import fftbg.event_stream
import fftbg.server
import discord

LOG = logging.getLogger(__name__)


class MyClient(discord.Client):
    dev_channel: discord.TextChannel

    async def on_ready(self):
        print('Logged on as', self.user)
        for channel in self.get_all_channels():
            if channel.guild.name == "FFTBattleground" and channel.name == "development":
                self.dev_channel = channel
        # await self.dev_channel.send('Kweh!! (Thank\'s Nacho!!)')

    async def on_message(self, message):
        # don't respond to ourselves
        if message.author == self.user:
            return

        if message.content == 'ping':
            await message.channel.send('pong')


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='DISCORD_LOG_LEVEL')

    token = os.environ['DISCORD_TOKEN']
    client = MyClient()
    client.run(token)

    #redis = fftbg.server.get_redis()
    # event_stream = fftbg.event_stream.EventStream(redis)


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Discord Bot died', exc_info=e)
