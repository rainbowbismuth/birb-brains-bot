import asyncio
import logging
import os
import threading

from twitchio.ext import commands

import fftbg.twitch.msg_types as msg_types
import fftbg.twitch.parse as parse
from fftbg.event_stream import EventStream

LOG = logging.getLogger(__name__)


class IRCBot(commands.Bot):
    def __init__(self, irc_token, client_id, nick, prefix, fftbg_channel, event_stream: EventStream):
        super().__init__(
            irc_token=irc_token,
            client_id=client_id,
            nick=nick,
            prefix=prefix,
            initial_channels=[fftbg_channel])
        self.fftbg_channel = fftbg_channel
        self.event_stream = event_stream
        self.outgoing_bot_say = asyncio.Queue(loop=self.loop)
        self.time_between_messages_seconds = 3.0
        self.thread = threading.Thread(name='Outgoing messages subscriber', target=self._handle_incoming_messages,
                                       daemon=True)
        self.thread.start()
        self.loop.create_task(self._send_outgoing_messages())

    def _handle_incoming_messages(self):
        try:
            while True:
                self._handle_incoming_messages_loop()
        except Exception as e:
            LOG.critical('Error in handling incoming messages loop', exc_info=e)
            os._exit(1)

    def _handle_incoming_messages_loop(self):
        while True:
            for (_, msg) in self.event_stream.read():
                if msg.get('type') == msg_types.SEND_POT:
                    asyncio.run_coroutine_threadsafe(
                        self.send_pot_command(), loop=self.loop)
                elif msg.get('type') == msg_types.SEND_BALANCE:
                    asyncio.run_coroutine_threadsafe(
                        self.send_balance_command(), loop=self.loop)
                elif msg.get('type') == msg_types.SEND_BET:
                    color = msg['color']
                    amount = int(msg['amount'])
                    asyncio.run_coroutine_threadsafe(
                        self.send_bet_command(color, amount), loop=self.loop)
                elif msg.get('type') == msg_types.SEND_MESSAGE:
                    say = msg['say']
                    asyncio.run_coroutine_threadsafe(
                        self.send_message(say), loop=self.loop)

    async def _send_outgoing_messages(self):
        try:
            while True:
                await self._send_outgoing_messages_loop()
        except Exception as e:
            LOG.critical('Error in send outgoing messages loop', exc_info=e)
            os._exit(1)

    async def _send_outgoing_messages_loop(self):
        msg = await self.outgoing_bot_say.get()
        await self._send_message_immediately(msg)
        await asyncio.sleep(self.time_between_messages_seconds)

    async def event_ready(self):
        LOG.info('Connected to twitch.tv')
        self.event_stream.publish({'type': msg_types.CONNECTED_TO_TWITCH})

    async def _send_message_immediately(self, text: str):
        try:
            channel = self.get_channel(self.fftbg_channel)
            await channel.send(text)
        except Exception as e:
            LOG.error(f'Error while saying: {text}', exc_info=e)

    async def send_message(self, text: str):
        await self.outgoing_bot_say.put(text)

    async def send_balance_command(self):
        await self.send_message('!balance')

    async def send_pot_command(self):
        await self.send_message('!pot')

    async def send_bet_command(self, team, amount):
        await self.send_message(f'!bet {team} {amount}')

    async def event_message(self, message):
        bet_match = parse.BET_RE.findall(message.content)
        if bet_match:
            team, amount = bet_match[0]
            bet = {'type': msg_types.RECV_BET,
                   'user': message.author.name,
                   'team': team}
            if amount.endswith('%'):
                percent = parse.parse_comma_int(amount.replace('%', ''))
                bet['percent'] = percent
            else:
                amount = parse.parse_comma_int(amount)
                bet['amount'] = amount
            self.event_stream.publish(bet)

        all_in_match = parse.ALL_IN_RE.findall(message.content)
        if all_in_match:
            team = all_in_match[0]
            bet = {'type': msg_types.RECV_BET,
                   'user': message.author.name,
                   'team': team,
                   'all_in': 1}
            self.event_stream.publish(bet)

        if message.author.channel.name != self.fftbg_channel:
            return

        if message.author.name != self.fftbg_channel and not message.content.startswith('!'):
            msg = {'type': msg_types.RECV_SAY,
                   'user': message.author.name,
                   'text': message.content}
            self.event_stream.publish(msg)
            return

        if parse.NEW_TOURNAMENT in message.content:
            msg = {'type': msg_types.RECV_NEW_TOURNAMENT}
            self.event_stream.publish(msg)

        balance_match = parse.BALANCE_RE.findall(message.content)
        if balance_match:
            for (user, balance) in balance_match:
                amount = parse.parse_comma_int(balance)
                msg = {'type': msg_types.RECV_BALANCE,
                       'user': user,
                       'amount': amount}
                self.event_stream.publish(msg)

        team_victory_match = parse.TEAM_VICTORY.findall(message.content)
        if team_victory_match:
            team = team_victory_match[0]
            msg = {'type': msg_types.RECV_TEAM_VICTORY,
                   'team': team}
            self.event_stream.publish(msg)

        betting_open = parse.BETTING_OPEN_RE.findall(message.content)
        if betting_open:
            (left, right) = betting_open[0]
            msg = {'type': msg_types.RECV_BETTING_OPEN,
                   'left_team': left,
                   'right_team': right}
            self.event_stream.publish(msg)

        betting_closed = parse.BETTING_CLOSED_RE.findall(message.content)
        if betting_closed:
            for user in betting_closed:
                msg = {'type': msg_types.RECV_BETTING_CLOSED_SORRY,
                       'user': user}
                self.event_stream.publish(msg)

        betting_close = parse.BETTING_CLOSE_RE.findall(message.content)
        if betting_close:
            left, left_bets, left_total, right, right_bets, right_total = betting_close[0]
            left_total_n = parse.parse_comma_int(left_total)
            right_total_n = parse.parse_comma_int(right_total)
            msg = {'type': msg_types.RECV_BETTING_POOL,
                   'final': 1,
                   'left_team': left,
                   'left_team_bets': left_bets,
                   'left_team_amount': left_total_n,
                   'right_team': right,
                   'right_team_bets': right_bets,
                   'right_team_amount': right_total_n}
            self.event_stream.publish(msg)
            return

        odds_match = parse.ODDS_RE.findall(message.content)
        if odds_match:
            left, left_bets, left_total, right, right_bets, right_total = odds_match[0]
            left_total_n = parse.parse_comma_int(left_total)
            right_total_n = parse.parse_comma_int(right_total)
            msg = {'type': msg_types.RECV_BETTING_POOL,
                   'final': 0,
                   'left_team': left,
                   'left_team_bets': left_bets,
                   'left_team_amount': left_total_n,
                   'right_team': right,
                   'right_team_bets': right_bets,
                   'right_team_amount': right_total_n}
            self.event_stream.publish(msg)
            return

        await self.handle_commands(message)

    async def event_command_error(self, ctx, error):
        LOG.error('Error while processing command', exc_info=error)

    async def event_error(self, error: Exception, data=None):
        LOG.error('Error while processing event', exc_info=error)
