import logging

import pandas

from config import DATA_PATH

LOG = logging.getLogger(__name__)
UNIT_FILE = DATA_PATH / 'units.feather'
MATCH_FILE = DATA_PATH / 'matches.feather'


def write_units(df: pandas.DataFrame):
    LOG.info(f'Writing units to {UNIT_FILE}')
    DATA_PATH.mkdir(parents=True, exist_ok=True)
    df.to_feather(UNIT_FILE)


def read_units() -> pandas.DataFrame:
    LOG.info(f'Reading units from {UNIT_FILE}')
    return pandas.read_feather(UNIT_FILE)


def write_matches(df: pandas.DataFrame):
    LOG.info(f'Writing matches to {MATCH_FILE}')
    DATA_PATH.mkdir(parents=True, exist_ok=True)
    df.to_feather(MATCH_FILE)


def read_matches() -> pandas.DataFrame:
    LOG.info(f'Reading matches from {MATCH_FILE}')
    return pandas.read_feather(MATCH_FILE)
