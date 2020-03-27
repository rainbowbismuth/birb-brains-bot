import asyncio
import logging

from walrus import Database

import fftbg.download
import fftbg.event_stream
import fftbg.server
import fftbg.tournament
import fftbg.twitch.msg_types as msg_types
from fftbg.bird.bird import Bird
from fftbg.brains.msg_types import NEW_PREDICTIONS
from fftbg.event_stream import EventStream

LOG = logging.getLogger(__name__)

REMINDER_MIN = 90
MIN_BET = 250000
MAX_BET = 300000


class Server:
    def __init__(self, db: Database, event_stream: EventStream, bird: Bird, loop: asyncio.AbstractEventLoop):
        self.db = db
        self.event_stream = event_stream
        self.bird = bird
        self.loop = loop
        self.waiting_for_odds = False
        self.go_all_in = False
        self.predictions_ready = asyncio.Event()
        self.predictions_ready.set()

    def ask_for_odds(self):
        self.event_stream.publish({'type': msg_types.SEND_POT})
        self.waiting_for_odds = True
        LOG.info(f'Asking for odds')

    def ask_for_balance(self):
        self.event_stream.publish({'type': msg_types.SEND_BALANCE})
        LOG.info(f'Asking for balance')

    def publish_bet(self, color, amount):
        if self.go_all_in:
            self.say_message(f'!allin {color}')
            self.say_message(f'Kweh! (I\'m so nervous! kwehLurk )')
            return

        amount = int(amount)
        msg = {
            'type': msg_types.SEND_BET,
            'color': color,
            'amount': amount
        }
        self.event_stream.publish(msg)
        LOG.info(f'Published bet, {amount} on {color}')

    def say_message(self, text):
        msg = {
            'type': msg_types.SEND_MESSAGE,
            'say': text
        }
        self.event_stream.publish(msg)
        LOG.info(f'Sending message: {text}')

    async def all_in_ready(self):
        await asyncio.sleep(60 * 10)
        while True:
            cur_bal = self.bird.balance
            if cur_bal == 0 or cur_bal < MIN_BET or self.go_all_in:
                return
            number = int(MAX_BET - cur_bal)
            if number <= 0:
                return
            self.say_message(
                f'Kweh-kweh!! (I\'m {number:,d} G away from {MAX_BET:,d} G! I can\'t wait to all-in! kwehWink )')
            await asyncio.sleep(60 * REMINDER_MIN)

    def update_balance(self, new_balance):
        if new_balance < MIN_BET and self.go_all_in:
            self.go_all_in = False
            self.say_message('Kweh... (Oh no... I really messed up didn\'t I?)')
            self.say_message('kwehQQ')
            self.say_message('*sniffle* (Going to have to start from scratch now..)')
            self.say_message('Wark!! (I know I can do it though ;)! You believe in me, right? kwehLove )')
        elif new_balance >= MAX_BET and self.go_all_in:
            self.say_message('Kweh?? (Did... did I win? >.<)')
            self.say_message('kwehWut')
            self.say_message(f'Wark.. (What am I going to do with {new_balance:,d} G?)')
            self.say_message('Wark-wark!! (Guess I\'m going to all-in again!! kwehSwag )')

        self.bird.update_balance(new_balance)
        if not self.go_all_in and new_balance >= MAX_BET:
            self.go_all_in = True
            self.say_message(f'Wark!!! (I made it to {new_balance:,d} G!! I\'m going all in!!! kwehSpook )')

    async def prepare_to_bet(self, betting_delay, left_team, right_team):
        await asyncio.sleep(betting_delay)
        await self.predictions_ready.wait()
        self.bird.log_prediction(left_team, right_team)
        self.ask_for_odds()

    async def check_messages(self):
        while True:
            await asyncio.sleep(1)
            for (_, msg) in self.event_stream.read():
                if msg.get('type') == msg_types.RECV_NEW_TOURNAMENT:
                    self.predictions_ready.clear()
                    LOG.info('predictions_ready cleared')

                elif msg.get('type') == NEW_PREDICTIONS:
                    self.bird.load_current_tournament()
                    self.predictions_ready.set()
                    LOG.info('predictions_ready set')

                elif msg.get('type') == msg_types.RECV_TEAM_VICTORY:
                    self.ask_for_balance()

                # TODO: stop hard-coding bot nick here
                elif msg.get('type') == msg_types.RECV_BALANCE and msg['user'].lower() == 'birbbrainsbot':
                    new_balance = int(msg['amount'])
                    self.update_balance(new_balance)

                elif msg.get('type') == msg_types.RECV_BETTING_OPEN:
                    left_team = msg['left_team']
                    right_team = msg['right_team']
                    LOG.info(f'Betting has opened for {left_team} vs {right_team}')

                    betting_time = 30.0
                    if self.go_all_in:
                        betting_time = 1.0

                    self.loop.create_task(self.prepare_to_bet(betting_time, left_team, right_team))

                elif msg.get('type') == msg_types.RECV_BETTING_POOL:
                    final = int(msg['final']) != 0
                    left_total = int(msg['left_team_amount'])
                    right_total = int(msg['right_team_amount'])
                    if final:
                        self.bird.final_odds(left_total, right_total, self.go_all_in)
                    elif self.waiting_for_odds:
                        color, wager = self.bird.make_bet(left_total, right_total, self.go_all_in)
                        self.publish_bet(color, wager)
                        self.waiting_for_odds = False


def handle_exception(loop, context):
    if 'exception' in context:
        LOG.critical('uncaught exception', exc_info=context['exception'])
    else:
        LOG.critical(f'exception {context["message"]}')
    import os
    os._exit(1)


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='BIRD_LOG_LEVEL')
    db = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(db)

    bird = Bird(db, event_stream)
    bird.load_current_tournament()
    loop = asyncio.get_event_loop()
    loop.set_exception_handler(handle_exception)

    server = Server(db, event_stream, bird, loop)

    loop.create_task(server.check_messages())
    loop.create_task(server.all_in_ready())

    loop.run_forever()


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Bird died', exc_info=e)
