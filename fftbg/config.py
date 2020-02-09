from pathlib import Path
import logging.config
import toml

FFTBG_API_URL = 'https://fftbg.com'
DATA_PATH = Path('./data')
TOURNAMENTS_ROOT = DATA_PATH / 'tournaments'
CONFIG_PATH = Path('./config')

LOGGING_CONFIG = toml.loads((CONFIG_PATH / 'logging.toml').read_text())
logging.config.dictConfig(LOGGING_CONFIG)
