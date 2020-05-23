import logging
import os
import traceback

from discord.ext import commands

import fftbg.event_stream
import fftbg.server
import fftbg.twitch.msg_types as msg_types
import fftbg.bird.memory
from pathlib import Path

import asyncio

LOG = logging.getLogger(__name__)

DIV_BY_ZERO_EMOTE = '<:fftbgDivideByZero:701439212246794291>'
SAD_BIRD_EMOTE = '<:fftbgSadBirb:669566649434767360>'
MAGIC_BOTTLE = 668345361420517377


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='DISCORD_LOG_LEVEL')

    token = os.environ['DISCORD_TOKEN']
    command_prefix = os.environ.get('DISCORD_COMMAND_PREFIX', '!bird-dev ')
    # mute = bool(int(os.environ['DISCORD_MUTE']))
    redis = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(redis)
    memory = fftbg.bird.memory.Memory()
    loop = fftbg.server.get_loop()
    bot = commands.Bot(command_prefix=command_prefix, loop=loop)
    bot.help_command = None

    all_skill_drops = set([skill.strip() for skill in Path('data/userskills.txt').read_text().split()])

    skill_drop_case = {}
    for skill in all_skill_drops:
        skill_drop_case[skill.lower()] = skill

    async def skill_drop_notify(skill):
        for (user_id, user_name) in memory.get_users_to_skill_drop_notify(skill):
            user = bot.get_user(user_id)
            await user.send(f'Hiii, {user_name}, *{skill}* is the new skill drop on FFTBG! Wark!!')

    async def listen_loop():
        while True:
            await asyncio.sleep(1)
            for (_, msg) in event_stream.read(block=1):
                if msg.get('type') == msg_types.RECV_NEW_TOURNAMENT and msg.get('skill_drop'):
                    await skill_drop_notify(msg["skill_drop"])

    loop.create_task(listen_loop())

    @bot.command()
    async def help(ctx):
        await ctx.send(f"""```
{command_prefix}skills 
    - List all skill drop notification requests

{command_prefix}skills add skill_1 ... skill_n
    - Add skill drops to your notification list

{command_prefix}skills remove skill_1 ... skill_n
    - Remove skill drops from your notification list

{command_prefix}skills clear
    - Remove all skill drops from your notification list
        ```""")

    @bot.group(invoke_without_command=True)
    async def skills(ctx):
        requests = memory.get_skill_drop_notify_requests(ctx.author.id)
        if not requests:
            await ctx.send(f'{ctx.author.display_name}, you don\'t have any notifications set up!')
            return
        requests.sort()
        msg = f'{ctx.author.display_name}, I\'ll notify you when these skills drop: {", ".join(requests)}'
        if len(msg) > 500:
            msg = msg[:500] + '*... (that\'s too many skills to say!)*'
        await ctx.send(msg)

    def massage_skills(skills):
        bad_skills = []
        good_skills = []
        for skill in skills:
            sanitized_skill = str(skill).replace(',', '').strip().lower()
            if sanitized_skill in skill_drop_case:
                good_skills.append(skill_drop_case[sanitized_skill])
            else:
                bad_skills.append(skill.replace(',', '').strip())
        return bad_skills, good_skills

    @skills.command()
    async def add(ctx, *skills):
        user_id = ctx.author.id
        display_name = ctx.author.display_name

        (bad_skills, good_skills) = massage_skills(skills)
        if not (bad_skills or good_skills):
            await ctx.send(f'{display_name}, you need to list some skill drops with this command!')
            return

        if bad_skills:
            await ctx.send(f'{display_name}, these aren\'t skill drops: {", ".join(bad_skills)}')
            return

        memory.add_notify_skill_drop_requests(user_id, display_name, good_skills)
        count = len(memory.get_skill_drop_notify_requests(user_id))
        await ctx.send(f'{display_name}, you got it! You are subscribed to {count} skill drops now.')

    @skills.command()
    async def remove(ctx, *skills):
        user_id = ctx.author.id
        display_name = ctx.author.display_name

        (bad_skills, good_skills) = massage_skills(skills)
        if not (bad_skills or good_skills):
            await ctx.send(f'{display_name}, you need to list some skill drops with this command!')
            return

        if bad_skills:
            await ctx.send(f'{display_name}, these aren\'t skill drops: {", ".join(bad_skills)}')
            return

        memory.remove_notify_skill_drop_requests(user_id, good_skills)
        count = len(memory.get_skill_drop_notify_requests(user_id))
        await ctx.send(f'{display_name}, you got it! You\'re subscribed to {count} skill drops now.')

    @skills.command()
    async def clear(ctx):
        memory.clear_notify_skill_drop_requests(ctx.author.id)
        await ctx.send(f'{ctx.author.display_name}, cleared em!')

    @skills.command()
    async def test_skill_drop(ctx, skill: str):
        if ctx.author.id != MAGIC_BOTTLE:
            return
        await skill_drop_notify(skill)

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
