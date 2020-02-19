import dataclasses
import json
import logging

import numpy as np
import pandas
from flask import Flask, render_template

from fftbg.bot.memory import BotMemory

LOG = logging.getLogger(__name__)
app = Flask(
    import_name=__name__,
    static_url_path='/static.1',
    static_folder='static.1',
    template_folder='templates')

WINDOW = 20
LIMIT = 200


@app.route('/')
def get_index():
    return render_template('/index.html')


@app.route('/balance-log')
def get_balance_log():
    memory = BotMemory()
    log = memory.get_balance_log(LIMIT)
    log_entries = [dataclasses.asdict(entry) for entry in log]
    return json.dumps(log_entries, sort_keys=True)


@app.route('/placed-bet')
def get_placed_bet():
    memory = BotMemory()
    bet = memory.get_placed_bet()
    return json.dumps(dataclasses.asdict(bet), sort_keys=True)


@app.route('/balance-log-stats')
def get_balance_log_stats():
    memory = BotMemory()
    log = memory.get_balance_log(LIMIT + WINDOW)
    log_entries = [dataclasses.asdict(entry) for entry in log]

    log_entries.sort(key=lambda u: u['id'])
    df = pandas.DataFrame(log_entries)

    correct_bets_left = (df['left_prediction'] > 0.5) * df['left_wins']
    correct_bets_right = (df['right_prediction'] > 0.5) * (1 - df['left_wins'])

    predictions_left = df['left_prediction'] * df['left_wins']
    predictions_right = df['right_prediction'] * (1 - df['left_wins'])

    df['log_loss'] = -np.log(predictions_left + predictions_right)
    df['winner_predicted'] = correct_bets_left + correct_bets_right

    df['rolling_accuracy'] = df['winner_predicted'].rolling(WINDOW).mean()
    df['rolling_log_loss'] = df['log_loss'].rolling(WINDOW).mean()

    return df.iloc[-LIMIT:].to_json(orient='records')
