import asyncio
import logging
import os
import re
import threading
from datetime import datetime
from typing import Optional

import pytz
from tenacity import retry, wait_exponential, after_log, stop_after_delay
from twitchio.ext import commands

import fftbg.twitch.incoming.messages as incoming_messages
import fftbg.twitch.outgoing.messages as outgoing_messages
import fftbg.twitch.parse as parse
from fftbg.twitch.incoming.pubsub import Publisher
from fftbg.twitch.outgoing.pubsub import Subscriber

LOG = logging.getLogger(__name__)


class IRCBot(commands.Bot):
    def __init__(self, irc_token, client_id, nick, prefix, fftbg_channel, publisher: Publisher, subscriber: Subscriber):
        super().__init__(
            irc_token=irc_token,
            client_id=client_id,
            nick=nick,
            prefix=prefix,
            initial_channels=[fftbg_channel])
        self.fftbg_channel = fftbg_channel
        self.waiting_for_odds = False
        self.betting_open_time = None
        self.publisher = publisher
        self.subscriber = subscriber
        self.outgoing = asyncio.Queue(loop=self.loop)
        self.time_between_messages_seconds = 5.0
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

    @retry(
        wait=wait_exponential(),
        stop=stop_after_delay(5),
        after=after_log(LOG, logging.ERROR))
    def _handle_incoming_messages_loop(self):
        msg: Optional[outgoing_messages.Message] = self.subscriber.get_message()
        if msg is None:
            return
        if msg.pot:
            asyncio.run_coroutine_threadsafe(self.send_pot_command(), loop=self.loop)
        elif msg.balance:
            asyncio.run_coroutine_threadsafe(self.send_balance_command(), loop=self.loop)
        elif msg.bet is not None:
            asyncio.run_coroutine_threadsafe(self.send_bet_command(msg.bet.color, msg.bet.amount), loop=self.loop)
        elif msg.say is not None:
            asyncio.run_coroutine_threadsafe(self.send_message(msg.say), loop=self.loop)

    async def _send_outgoing_messages(self):
        try:
            while True:
                await self._send_outgoing_messages_loop()
        except Exception as e:
            LOG.critical('Error in send outgoing messages loop', exc_info=e)
            os._exit(1)

    @retry(
        wait=wait_exponential(),
        stop=stop_after_delay(5),
        after=after_log(LOG, logging.ERROR))
    async def _send_outgoing_messages_loop(self):
        msg = await self.outgoing.get()
        await self._send_message_immediately(msg)
        await asyncio.sleep(self.time_between_messages_seconds)

    async def event_ready(self):
        LOG.info('Connected to twitch.tv')

    async def _send_message_immediately(self, text: str):
        try:
            channel = self.get_channel(self.fftbg_channel)
            await channel.send(text)
        except Exception as e:
            LOG.error(f'Error while saying: {text}', exc_info=e)

    async def send_message(self, text: str):
        await self.outgoing.put(text)

    async def send_balance_command(self):
        await self.send_message('!balance')

    async def send_pot_command(self):
        await self.send_message('!pot')

    async def send_bet_command(self, team, amount):
        await self.send_message(f'!bet {team} {amount}')

    async def event_message(self, message):
        current_time = datetime.now(tz=pytz.utc)

        if re.search(self.nick, message.content, re.IGNORECASE):
            LOG.info(f'Bot mentioned by {message.author.name}: {message.content}')

        bet_match = parse.BET_RE.findall(message.content)
        if bet_match:
            team, amount = bet_match[0]
            if amount.endswith('%'):
                percent = parse.parse_comma_int(amount.replace('%', ''))
                bet = incoming_messages.Bet(
                    user=message.author.name,
                    team=team,
                    percent=percent)
            else:
                amount = parse.parse_comma_int(amount)
                bet = incoming_messages.Bet(
                    user=message.author.name,
                    team=team,
                    amount=amount)
            msg = incoming_messages.Message(
                time=current_time,
                bet=bet)
            self.publisher.publish(msg)

        all_in_match = parse.ALL_IN_RE.findall(message.content)
        if all_in_match:
            team = all_in_match[0]
            msg = incoming_messages.Message(
                time=current_time,
                bet=incoming_messages.Bet(
                    user=message.author.name,
                    team=team,
                    all_in=True)
            )
            self.publisher.publish(msg)

        if message.author.channel.name != self.fftbg_channel:
            return

        if message.author.name != self.fftbg_channel:
            msg = incoming_messages.Message(
                time=current_time,
                say=incoming_messages.Say(
                    user=message.author.name,
                    text=message.content))
            self.publisher.publish(msg)
            return

        if parse.NEW_TOURNAMENT in message.content:
            msg = incoming_messages.Message(
                time=current_time,
                new_tournament=True)
            self.publisher.publish(msg)

        balance_match = parse.BALANCE_RE.findall(message.content)
        if balance_match:

            for (user, balance) in balance_match:
                amount = parse.parse_comma_int(balance)
                msg = incoming_messages.Message(
                    time=current_time,
                    balance=incoming_messages.Balance(user=user, amount=amount))
                self.publisher.publish(msg)

        betting_open = parse.BETTING_OPEN_RE.findall(message.content)
        if betting_open:
            (left, right) = betting_open[0]
            msg = incoming_messages.Message(
                time=current_time,
                betting_open=incoming_messages.BettingOpen(left_team=left, right_team=right))
            self.publisher.publish(msg)

        betting_closed = parse.BETTING_CLOSED_RE.findall(message.content)
        if betting_closed:
            for user in betting_closed:
                msg = incoming_messages.Message(
                    time=current_time,
                    betting_closed_sorry=incoming_messages.BettingClosedSorry(user))
                self.publisher.publish(msg)

        betting_close = parse.BETTING_CLOSE_RE.findall(message.content)
        if betting_close:
            left, left_bets, left_total, right, right_bets, right_total = betting_close[0]
            left_total_n = parse.parse_comma_int(left_total)
            right_total_n = parse.parse_comma_int(right_total)
            msg = incoming_messages.Message(
                time=current_time,
                betting_pool=incoming_messages.BettingPool(
                    final=True,
                    left_team=incoming_messages.TeamBets(
                        color=left,
                        bets=left_bets,
                        amount=left_total_n),
                    right_team=incoming_messages.TeamBets(
                        color=right,
                        bets=right_bets,
                        amount=right_total_n)))
            self.publisher.publish(msg)
            return

        odds_match = parse.ODDS_RE.findall(message.content)
        if odds_match:
            left, left_bets, left_total, right, right_bets, right_total = odds_match[0]
            left_total_n = parse.parse_comma_int(left_total)
            right_total_n = parse.parse_comma_int(right_total)
            msg = incoming_messages.Message(
                time=current_time,
                betting_pool=incoming_messages.BettingPool(
                    final=False,
                    left_team=incoming_messages.TeamBets(
                        color=left,
                        bets=left_bets,
                        amount=left_total_n),
                    right_team=incoming_messages.TeamBets(
                        color=right,
                        bets=right_bets,
                        amount=right_total_n)))
            self.publisher.publish(msg)
            return

        await self.handle_commands(message)

    async def event_command_error(self, ctx, error):
        LOG.error('Error while processing command', exc_info=error)

    async def event_error(self, error: Exception, data=None):
        LOG.error('Error while processing event', exc_info=error)
