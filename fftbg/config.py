import logging.config
from pathlib import Path

import toml

FFTBG_API_URL = 'https://fftbg.com'
DATA_PATH = Path('./data')
TOURNAMENTS_ROOT = DATA_PATH / 'tournaments'
INFO_ITEM_PATH = DATA_PATH / 'static' / 'infoitem.txt'
CLASS_HELP_PATH = DATA_PATH / 'static' / 'classhelp.txt'
ABILITY_HELP_PATH = DATA_PATH / 'static' / 'infoability.txt'

CONFIG_PATH = Path('./config')
MODEL_PATH = DATA_PATH / 'model.h5'
COLUMN_XFORM_PATH = DATA_PATH / 'column_xform.pickle'
FEATURE_SCALER_PATH = DATA_PATH / 'feature_scaler.pickle'
ALL_COLUMNS_PATH = DATA_PATH / 'all_columns.pickle'
EVENT_LOG_PATH = DATA_PATH / 'event_log.json'

SAVE_MODEL = False

LOGGING_CONFIG = toml.loads((CONFIG_PATH / 'logging.toml').read_text())
logging.config.dictConfig(LOGGING_CONFIG)

BOT_CONFIG = toml.loads((CONFIG_PATH / 'twitch.toml').read_text())

LOG = logging.getLogger(__name__)
LOG.info("Good morning")
