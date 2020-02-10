import logging.config
from pathlib import Path

import toml

FFTBG_API_URL = 'https://fftbg.com'
DATA_PATH = Path('./data')
TOURNAMENTS_ROOT = DATA_PATH / 'tournaments'
CONFIG_PATH = Path('./config')
MODEL_PATH = DATA_PATH / 'model.h5'

SAVE_MODEL = False

LOGGING_CONFIG = toml.loads((CONFIG_PATH / 'logging.toml').read_text())
logging.config.dictConfig(LOGGING_CONFIG)

LOG = logging.getLogger(__name__)
LOG.info("Good morning")
