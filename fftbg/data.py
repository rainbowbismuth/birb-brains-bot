import logging

import pandas

from config import DATA_PATH

LOG = logging.getLogger(__name__)
COMBATANT_FILE = DATA_PATH / 'combatants.feather'


def write_combatants(df: pandas.DataFrame):
    LOG.info(f'Writing combatants to {COMBATANT_FILE}')
    DATA_PATH.mkdir(parents=True, exist_ok=True)
    df.to_feather(COMBATANT_FILE)


def read_combatants() -> pandas.DataFrame:
    LOG.info(f'Reading combatants from {COMBATANT_FILE}')
    return pandas.read_feather(COMBATANT_FILE)
