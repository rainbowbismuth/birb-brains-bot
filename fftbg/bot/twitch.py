import asyncio
import logging
import re
import time

from twitchio.ext import commands

from fftbg.bot.brains import BotBrains
from fftbg.config import BOT_CONFIG

LOG = logging.getLogger(__name__)

TWITCH = BOT_CONFIG['twitch']
BOT_TMI_TOKEN = TWITCH['tmi_token']
BOT_CLIENT_ID = TWITCH['client_id']
BOT_NICK = TWITCH['bot_nick']
BOT_CHANNEL = TWITCH['channel']
BOT_PREFIX = '!!birbbrainsbot'

MATCH_BETTING_LENGTH = 60.0
MATCH_ODDS_TIME_REMAINING = 31.0

NEW_TOURNAMENT = 'You may now !fight to enter the tournament!'
BALANCE_RE = re.compile(r'(\w+), your balance is: ([\d,]+)G')
BETTING_OPEN_RE = re.compile(r'Betting is open for (\w+) vs (\w+).')
BETTING_CLOSE_RE = re.compile(r'Betting is closed: Final Bets: (\w+) - (\d+) bets for ([\d,]+)G(?:.*?); (\w+) - (\d+) '
                              r'bets for ([\d,]+)G')
ODDS_RE = re.compile(r'(\w+) - (\d+) bets for ([\d,]+)G(?:.*?); (\w+) - (\d+) bets for ([\d,]+)G')
BETTING_CLOSED_RE = re.compile(r'(\w+), betting has closed, sorry!')


# TODO:
#   - Log betting totals and predictions with tournament ID, in a file I guess? plain text? IDK.
#   - Make a better, simpler, more robust model.
#   - Add the betting logic and stuff.
#   - Need to loop refresh for tournament, mark that a new tournament has happened and not bet on old data


def parse_comma_int(s):
    return int(s.replace(',', ''))


class Bot(commands.Bot):
    def __init__(self):
        self.brains = BotBrains()
        LOG.info('Starting up the rest of the Bot')
        super().__init__(
            irc_token=BOT_TMI_TOKEN,
            client_id=BOT_CLIENT_ID,
            nick=BOT_NICK,
            prefix=BOT_PREFIX,
            initial_channels=[BOT_CHANNEL])
        self.waiting_for_odds = False
        self.betting_open_time = None

    # Events don't need decorators when subclassed
    async def event_ready(self):
        LOG.info('Connected to twitch.tv')
        await self.brains.refresh_tournament()
        await self.send_balance_command()

    async def send_balance_command(self):
        try:
            LOG.info('Asking for balance')
            channel = self.get_channel(BOT_CHANNEL)
            await channel.send("!balance")
        except Exception as e:
            LOG.error('Error asking for balance!', exc_info=e)

    async def send_pot_command(self):
        try:
            LOG.info('Asking for betting odds')
            self.waiting_for_odds = True
            channel = self.get_channel(BOT_CHANNEL)
            await channel.send("!pot")
        except Exception as e:
            LOG.error('Error asking for betting odds!', exc_info=e)

    async def send_bet_command(self, team, amount):
        try:
            amount = int(amount)
            LOG.info(f'Betting {amount} G on {team} team')
            channel = self.get_channel(BOT_CHANNEL)
            await channel.send(f'!bet {amount} {team}')
        except Exception as e:
            LOG.error('Error trying to bet!', exc_info=e)

    async def event_message(self, message):
        if re.search(BOT_NICK, message.content, re.IGNORECASE):
            LOG.info(f'Bot mentioned by {message.author.name}: {message.content}')

        if message.author.channel.name != BOT_CHANNEL:
            return

        if message.author.name != BOT_CHANNEL:
            return

        if NEW_TOURNAMENT in message.content:
            self.brains.new_tournament()
            await self.send_balance_command()

        balance_match = BALANCE_RE.findall(message.content)
        if balance_match:
            for (user, balance) in balance_match:
                if user != BOT_NICK:
                    continue
                self.brains.update_balance(parse_comma_int(balance))

        betting_open = BETTING_OPEN_RE.findall(message.content)
        if betting_open:
            (left, right) = betting_open[0]
            self.betting_open_time = time.time()
            await self.send_balance_command()
            if left == 'red' and right == 'blue':
                await self.brains.refresh_tournament()
            self.brains.log_prediction(left, right)

            time_diff = time.time() - self.betting_open_time
            time_remaining = MATCH_BETTING_LENGTH - time_diff
            if time_remaining > MATCH_ODDS_TIME_REMAINING:
                sleep_seconds = int(time_remaining - MATCH_ODDS_TIME_REMAINING)
                LOG.info(f'Sleeping for {sleep_seconds} seconds before asking for odds')
                await asyncio.sleep(sleep_seconds)
            await self.send_pot_command()

        betting_closed = BETTING_CLOSED_RE.findall(message.content)
        if betting_closed and any([BOT_NICK in msg for msg in betting_closed]):
            LOG.info(f'Missed the betting window!')
            self.waiting_for_odds = False

        betting_close = BETTING_CLOSE_RE.findall(message.content)
        if betting_close:
            self.waiting_for_odds = False
            left, left_bets, left_total, right, right_bets, right_total = betting_close[0]
            LOG.info(f'Final betting totals: {left}/{left_bets} {left_total} G; {right}/{right_bets} {right_total} G')
            left_total_n = parse_comma_int(left_total)
            right_total_n = parse_comma_int(right_total)
            self.brains.final_odds(left_total_n, right_total_n)
            return

        odds_match = ODDS_RE.findall(message.content)
        if odds_match and self.waiting_for_odds:
            self.waiting_for_odds = False
            left, left_bets, left_total, right, right_bets, right_total = odds_match[0]
            left_total_n = parse_comma_int(left_total)
            right_total_n = parse_comma_int(right_total)
            LOG.info(f'Betting totals: {left}/{left_bets} {left_total} G; {right}/{right_bets} {right_total} G')
            team, amount = self.brains.make_bet(left_total_n, right_total_n)
            await self.send_bet_command(team, amount)

        await self.handle_commands(message)

    async def event_command_error(self, ctx, error):
        LOG.error('Error while processing command', exc_info=error)

    async def event_error(self, error: Exception, data=None):
        LOG.error('Error while processing event', exc_info=error)


def main():
    try:
        bot = Bot()
        bot.run()
    except:
        LOG.critical('Bot died', exc_info=True)


if __name__ == '__main__':
    main()
