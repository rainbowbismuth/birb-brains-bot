import logging
import sched

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


class Server:
    def __init__(self, db: Database, event_stream: EventStream, bird: Bird, scheduler: sched.scheduler):
        self.db = db
        self.event_stream = event_stream
        self.bird = bird
        self.scheduler = scheduler
        self.waiting_for_odds = False

    def ask_for_odds(self):
        self.event_stream.publish({'type': msg_types.SEND_POT})
        self.waiting_for_odds = True
        LOG.info(f'Asking for odds')

    def ask_for_balance(self):
        self.event_stream.publish({'type': msg_types.SEND_BALANCE})
        LOG.info(f'Asking for balance')

    def publish_bet(self, color, amount):
        amount = int(amount)
        msg = {
            'type': msg_types.SEND_BET,
            'color': color,
            'amount': amount
        }
        self.event_stream.publish(msg)
        LOG.info(f'Published bet, {amount} on {color}')

    def check_messages(self):
        self.scheduler.enter(1, 1, self.check_messages)
        for (_, msg) in self.event_stream.read():
            if msg.get('type') == NEW_PREDICTIONS:
                self.bird.load_current_tournament()

            elif msg.get('type') == msg_types.RECV_TEAM_VICTORY:
                self.ask_for_balance()

            # TODO: stop hard-coding bot nick here
            elif msg.get('type') == msg_types.RECV_BALANCE and msg['user'].lower() == 'birbbrainsbot':
                new_balance = int(msg['amount'])
                self.bird.update_balance(new_balance)

            elif msg.get('type') == msg_types.RECV_BETTING_OPEN:
                left_team = msg['left_team']
                right_team = msg['right_team']
                LOG.info(f'Betting has opened for {left_team} vs {right_team}')
                self.scheduler.enter(30, 1, lambda: self.bird.log_prediction(left_team, right_team))
                self.scheduler.enter(30, 2, self.ask_for_odds)

            elif msg.get('type') == msg_types.RECV_BETTING_POOL:
                final = int(msg['final']) != 0
                left_total = int(msg['left_team_amount'])
                right_total = int(msg['right_team_amount'])
                if final:
                    self.bird.final_odds(left_total, right_total)
                elif self.waiting_for_odds:
                    color, wager = self.bird.make_bet(left_total, right_total)
                    self.publish_bet(color, wager)
                    self.waiting_for_odds = False


def run_server():
    fftbg.server.set_name(__package__)
    fftbg.server.configure_logging(env_var='BIRD_LOG_LEVEL')
    db = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(db)

    bird = Bird(db, event_stream)
    bird.load_current_tournament()
    scheduler = sched.scheduler()

    server = Server(db, event_stream, bird, scheduler)
    scheduler.enter(0, 1, server.check_messages)
    scheduler.run()


def main():
    try:
        run_server()
    except Exception as e:
        LOG.critical('Bird died', exc_info=e)
