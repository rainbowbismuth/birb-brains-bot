import dataclasses
import json
import logging

from flask import Flask, render_template

from fftbg.bot_memory import BotMemory

LOG = logging.getLogger(__name__)
app = Flask(
    import_name=__name__,
    static_url_path='/static.1',
    static_folder='static.1',
    template_folder='templates')

LIMIT = 100


@app.route('/')
def get_index():
    return render_template('/index.html')


@app.route('/balance-log')
def get_balance_log():
    memory = BotMemory()
    log = memory.get_balance_log(LIMIT)
    return json.dumps([dataclasses.asdict(entry) for entry in log], sort_keys=True)


@app.route('/placed-bet')
def get_placed_bet():
    memory = BotMemory()
    bet = memory.get_placed_bet()
    return json.dumps(dataclasses.asdict(bet), sort_keys=True)
