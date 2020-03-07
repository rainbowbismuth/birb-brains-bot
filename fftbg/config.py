from pathlib import Path

FFTBG_API_URL = 'https://fftbg.com'
DATA_PATH = Path('./data')
TOURNAMENTS_ROOT = DATA_PATH / 'tournaments'

CONFIG_PATH = Path('./config')
MODEL_PATH = DATA_PATH / 'model.h5'
COLUMN_XFORM_PATH = DATA_PATH / 'column_xform.pickle'
FEATURE_SCALER_PATH = DATA_PATH / 'feature_scaler.pickle'
ALL_COLUMNS_PATH = DATA_PATH / 'all_columns.pickle'
BOT_MEMORY_PATH = DATA_PATH / 'bot_memory.db'

SAVE_MODEL = True

# FIXME: Wrecking old style logging
# LOGGING_CONFIG = toml.loads((CONFIG_PATH / 'logging.toml').read_text())
# logging.config.dictConfig(LOGGING_CONFIG)
#
# BOT_CONFIG = toml.loads((CONFIG_PATH / 'twitch.toml').read_text())
#
# LOG = logging.getLogger(__name__)
# LOG.info("Good morning")
