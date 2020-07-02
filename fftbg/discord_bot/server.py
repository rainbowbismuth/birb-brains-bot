import logging
import os
import traceback

import discord
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
BEHE_CHAMP_EMOTE = '<:fftbgBeheChamp:680376858705133622>'
CHARM_EMOTE = '<:fftbgCharm:693187279308456007>'
SONG_OF_MY_PEOPLE_EMOTE = '<:fftbgSongOfMyPeople:670788872363311104>'
CHIP_EMOTE = '<:fftbgChip:693187102350770216>'
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

    cmd_prefix_help = command_prefix
    if command_prefix == '!bird ':
        command_prefix = ('!bird ', '!birb ')
    if command_prefix == '!bird-dev ':
        command_prefix = ('!bird-dev ', '!birb-dev ')

    bot = commands.Bot(command_prefix=command_prefix, loop=loop)
    bot.help_command = None

    all_skill_drops = set([skill.strip() for skill in Path('data/userskills.txt').read_text().split()])

    skill_drop_case = {}
    for skill in all_skill_drops:
        skill_drop_case[skill.lower()] = skill

    async def send(ctx, msg):
        channel = getattr(ctx, 'channel')
        if channel and isinstance(channel, discord.TextChannel):
            await bot.bot_spam_channel.send(msg)
        else:
            await ctx.send(msg)

    async def skill_drop_notify(skill):
        tuples = memory.get_users_to_skill_drop_notify(skill)
        LOG.info(f'Sending notifications for {skill}, {len(tuples)} recipients')
        for (user_id, user_name) in tuples:
            try:
                user = bot.get_user(user_id)
                await user.send(f'Hiii, {user.display_name}, *{skill}* is the new skill drop on FFTBG! Wark!!')
            except Exception as exc:
                LOG.error(f'Error sending skill drop notification to {user_name} ({user_id})', exc_info=exc)

    async def notify_skill_obtained(twitch_user_name, skill, verb: str, gifter=None):
        user_id = memory.find_discord_id_from_twitch(twitch_user_name)
        if not user_id:
            return
        skills = memory.get_skill_drop_notify_requests(user_id)
        if skill not in skills:
            return
        memory.remove_notify_skill_drop_requests(user_id, [skill])
        user = bot.get_user(user_id)
        if verb == 'bought':
            emote = BEHE_CHAMP_EMOTE
        else:
            emote = SONG_OF_MY_PEOPLE_EMOTE
        await user.send(f'{emote} Looks like you just {verb} {skill}, sweet! I removed it from your list.')
        if gifter:
            await user.send(f'Oh, and don\'t forget to thank {gifter} for the gift! {CHARM_EMOTE}')

    async def notify_new_tournament():
        notifications = memory.find_triggered_event_notifications('tournament')
        LOG.info(f'Sending out {len(notifications)} new tournament notifications')
        for (user_id, minutes_left) in notifications:
            user = bot.get_user(user_id)
            await user.send(f'{CHIP_EMOTE} A new tournament has started!'
                            f' ({int(minutes_left)} minutes left on this alert)')

    async def notify_new_match(left_team, right_team):
        notifications = memory.find_triggered_event_notifications('new_match')
        LOG.info(f'Sending out {len(notifications)} new match notifications')
        for (user_id, minutes_left) in notifications:
            user = bot.get_user(user_id)
            await user.send(f'{CHIP_EMOTE} The {left_team} vs {right_team} match is about to begin!'
                            f' ({int(minutes_left)} minutes left on this alert)')

    async def notify_match_ended(victory_team):
        notifications = memory.find_triggered_event_notifications('match_ended')
        LOG.info(f'Sending out {len(notifications)} match ended notifications')
        for (user_id, minutes_left) in notifications:
            user = bot.get_user(user_id)
            await user.send(f'{CHIP_EMOTE} The {victory_team} team is victorious!'
                            f' ({int(minutes_left)} minutes left on this alert)')

    async def listen_loop():
        while True:
            await asyncio.sleep(1)
            for (_, msg) in event_stream.read(block=1):
                if msg.get('type') == msg_types.RECV_NEW_TOURNAMENT:
                    await notify_new_tournament()
                if msg.get('type') == msg_types.RECV_NEW_TOURNAMENT and msg.get('skill_drop'):
                    await skill_drop_notify(msg["skill_drop"])
                elif msg.get('type') == msg_types.RECV_SKILL_PURCHASE:
                    await notify_skill_obtained(msg['user'], msg['skill'], verb='bought')
                elif msg.get('type') == msg_types.RECV_SKILL_LEARN:
                    await notify_skill_obtained(msg['user'], msg['skill'], verb='learned')
                elif msg.get('type') == msg_types.RECV_SKILL_GIFT:
                    await notify_skill_obtained(msg['user'], msg['skill'], verb='received', gifter=msg['gifter'])
                elif msg.get('type') == msg_types.RECV_BETTING_OPEN:
                    await notify_new_match(msg['left_team'], msg['right_team'])
                elif msg.get('type') == msg_types.RECV_TEAM_VICTORY:
                    await notify_match_ended(msg['team'])

    loop.create_task(listen_loop())

    @bot.command()
    async def help(ctx):
        await send(ctx, f"""
> {cmd_prefix_help}twitch
>    - Display your currently linked twitch username

> {cmd_prefix_help}twitch link __username__
>    - Link your twitch username, so I can remove purchased skills for you

> {cmd_prefix_help}twitch unlink
>    - Unlink your twitch username

> {cmd_prefix_help}skills 
>    - List all skill drop notification requests

> {cmd_prefix_help}skills add __skill 1__ ... __skill n__
>    - Add skill drops to your notification list

> {cmd_prefix_help}skills remove __skill 1__ ... __skill n__
>    - Remove skill drops from your notification list

> {cmd_prefix_help}skills clear
>    - Remove all skill drops from your notification list

> {cmd_prefix_help}alert tournament __hours__
>    - Alert me when a new tournament starts. If __hours__ equals zero, disable alert

> {cmd_prefix_help}alert match __hours__
>    - Alert me when betting for a match opens

> {cmd_prefix_help}alert victory __hours__
>    - Alert me when a match ends

> {cmd_prefix_help}alert off
>    - Turn off any tournament or match alerts you had turned on
        """)

    @bot.group(invoke_without_command=True)
    async def alert(ctx):
        alerts = memory.find_users_event_notifications(ctx.author.id)
        alerts.sort()
        if not alerts:
            await send(ctx, f'{ctx.author.display_name}, you don\'t have any tournament or match alerts enabled.')
            return
        msg = [f'{alert_name}, {int(minutes)} minutes remaining.' for (alert_name, minutes) in alerts]
        await send(ctx, f'{ctx.author.display_name}, you have the following alerts enabled:')
        await send(ctx, '\n'.join(msg))

    async def refresh_event(ctx, user_id, user_name, event, hours, msg):
        hours = max(0, hours)
        memory.refresh_event_notification(user_id, user_name, event, hours)
        if hours == 0:
            await send(ctx, f'{user_name}, I\'ve disabled that alert for you.')
        else:
            await send(ctx, f'{user_name}, I\'ll notify you when {msg} for the next {hours} hours.')

    @alert.command()
    async def tournament(ctx, hours: int):
        await refresh_event(ctx, ctx.author.id, ctx.author.display_name, 'tournament', hours, 'a new tournament starts')

    @alert.command()
    async def match(ctx, hours: int):
        await refresh_event(ctx, ctx.author.id, ctx.author.display_name, 'new_match', hours, 'a new match starts')

    @alert.command()
    async def victory(ctx, hours: int):
        await refresh_event(ctx, ctx.author.id, ctx.author.display_name, 'match_ended', hours, 'a match ends')

    @alert.command()
    async def off(ctx):
        memory.turn_off_event_notifications(ctx.author.id)
        await send(ctx, f'{ctx.author.display_name}, I turned off your tournament & match notifications.')

    @bot.group(invoke_without_command=True)
    async def twitch(ctx):
        user_name = memory.find_twitch_user_name(ctx.author.id)
        if not user_name:
            await send(ctx, f'{ctx.author.display_name}, you don\'t have a twitch account linked!')
            return
        await send(ctx, f'{ctx.author.display_name}, I have your username down as {user_name}')

    @twitch.command()
    async def link(ctx, username: str):
        memory.set_discord_twitch_link(ctx.author.id, username)
        await send(ctx, f'{ctx.author.display_name}, done! I have your twitch username down as {username}')

    @twitch.command()
    async def unlink(ctx):
        memory.unlink_twitch_account(ctx.author.id)
        await send(ctx, f'{ctx.author.display_name}, done!')

    @bot.group(invoke_without_command=True)
    async def skills(ctx):
        requests = memory.get_skill_drop_notify_requests(ctx.author.id)
        if not requests:
            await send(ctx, f'{ctx.author.display_name}, you don\'t have any notifications set up!')
            return
        requests.sort()
        msg = f'{ctx.author.display_name}, I\'ll notify you when these skills drop: {", ".join(requests)}.'
        if len(msg) > 500:
            msg = msg[:500] + '*... (that\'s too many skills to say!)*'
        await send(ctx, msg)

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
            await send(ctx, f'{display_name}, you need to list some skill drops with this command!')
            return

        if bad_skills:
            await send(ctx, f'{display_name}, these aren\'t skill drops: {", ".join(bad_skills)}')
            return

        memory.add_notify_skill_drop_requests(user_id, display_name, good_skills)
        count = len(memory.get_skill_drop_notify_requests(user_id))
        await send(ctx, f'{display_name}, you got it! You are subscribed to {count} skill drops now.')

    @skills.command()
    async def remove(ctx, *skills):
        user_id = ctx.author.id
        display_name = ctx.author.display_name

        (bad_skills, good_skills) = massage_skills(skills)
        if not (bad_skills or good_skills):
            await send(ctx, f'{display_name}, you need to list some skill drops with this command!')
            return

        if bad_skills:
            await send(ctx, f'{display_name}, these aren\'t skill drops: {", ".join(bad_skills)}')
            return

        memory.remove_notify_skill_drop_requests(user_id, good_skills)
        count = len(memory.get_skill_drop_notify_requests(user_id))
        await send(ctx, f'{display_name}, you got it! You\'re subscribed to {count} skill drops now.')

    @skills.command()
    async def clear(ctx):
        memory.clear_notify_skill_drop_requests(ctx.author.id)
        await send(ctx, f'{ctx.author.display_name}, cleared em!')

    @bot.command()
    async def test_skill_drop(ctx, skill: str):
        if ctx.author.id != MAGIC_BOTTLE:
            return
        await skill_drop_notify(skill)

    @bot.command()
    async def test_skill_buy(ctx, username: str, skill: str, gifter=None):
        if ctx.author.id != MAGIC_BOTTLE:
            return
        await notify_skill_obtained(username, skill, verb='stole', gifter=gifter)

    @bot.command()
    async def test_event(ctx, event: str):
        if ctx.author.id != MAGIC_BOTTLE:
            return
        if event == 'tournament':
            await notify_new_tournament()
        elif event == 'new_match':
            await notify_new_match('red', 'blue')
        elif event == 'match_ended':
            await notify_match_ended('red')

    @bot.command()
    async def test_echo(ctx, msg: str):
        if ctx.author.id != MAGIC_BOTTLE:
            return
        await send(ctx, msg)

    @bot.event
    async def on_command_error(ctx, error):
        if hasattr(error, 'original'):
            ex = error.original
            LOG.error(msg='Command exception', exc_info=ex)
            exc_str = ''.join(traceback.format_exception(etype=type(ex), value=ex, tb=ex.__traceback__))
            user = bot.get_user(MAGIC_BOTTLE)
            await user.send(f'{DIV_BY_ZERO_EMOTE} Wark! Someone is having an issue with me! '
                            f'\n```\n{ctx.author}: {ctx.message.content}\n\n{exc_str}\n```')
            await send(ctx, f'{DIV_BY_ZERO_EMOTE} Wark! (Something bad happened while running your command! I messaged '
                            f'MagicBottle about it, don\'t worry.)')
        else:
            await send(ctx, f'{SAD_BIRD_EMOTE} Kweh.. ({str(error)})')

    @bot.event
    async def on_ready():
        print('Logged on as', bot.user)
        for channel in bot.get_all_channels():
            if channel.guild.name == "FFTBattleground" and channel.name == "development":
                bot.dev_channel = channel
            if channel.guild.name == "FFTBattleground" and channel.name == "bot-spam":
                bot.bot_spam_channel = channel

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
