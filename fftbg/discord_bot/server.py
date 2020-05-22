import logging
import os
import traceback

from discord.ext import commands

import fftbg.event_stream
import fftbg.server
import fftbg.twitch.msg_types as msg_types

import asyncio

LOG = logging.getLogger(__name__)

DIV_BY_ZERO_EMOTE = '<:fftbgDivideByZero:701439212246794291>'
SAD_BIRD_EMOTE = '<:fftbgSadBirb:669566649434767360>'
MAGIC_BOTTLE = 668345361420517377


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='DISCORD_LOG_LEVEL')

    token = os.environ['DISCORD_TOKEN']
    redis = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(redis)
    loop = fftbg.server.get_loop()
    bot = commands.Bot(command_prefix='!bird-', loop=loop)

    async def listen_loop():
        while True:
            await asyncio.sleep(1)
            for (_, msg) in event_stream.read():
                if msg.get('type') == msg_types.RECV_NEW_TOURNAMENT and msg.get('skill_drop'):
                    await bot.dev_channel.send(f'Kweh! (The new skill drop is {msg["skill_drop"]}!)')

    loop.create_task(listen_loop())

    @bot.command()
    async def badday(ctx, num: int):
        assert num == 5

    @bot.event
    async def on_command_error(ctx, error):
        if hasattr(error, 'original'):
            ex = error.original
            exc_str = ''.join(traceback.format_exception(etype=type(ex), value=ex, tb=ex.__traceback__))
            user = bot.get_user(MAGIC_BOTTLE)
            await user.send(f'{DIV_BY_ZERO_EMOTE} Wark! Someone is having an issue with me! '
                            f'\n```\n{ctx.author}: {ctx.message.content}\n\n{exc_str}\n```')
            await ctx.send(f'{DIV_BY_ZERO_EMOTE} Wark! (Something bad happened while running your command! I messaged '
                           f'MagicBottle about it don\'t worry.)')
        else:
            await ctx.send(f'{SAD_BIRD_EMOTE} Kweh.. ({str(error)})')

    @bot.event
    async def on_ready():
        print('Logged on as', bot.user)
        for channel in bot.get_all_channels():
            if channel.guild.name == "FFTBattleground" and channel.name == "development":
                bot.dev_channel = channel

    async def run_bot():
        try:
            await bot.start(token)
        finally:
            await bot.close()

    loop.create_task(run_bot())
    loop.run_forever()


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Discord Bot died', exc_info=e)
