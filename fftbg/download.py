import json
import logging
from datetime import datetime

import requests

from config import FFTBG_API_URL, TOURNAMENTS_ROOT

LOG = logging.getLogger(__name__)


def get_tournament_list():
    j = requests.get(f'{FFTBG_API_URL}/api/tournaments').json()
    return [(t['ID'], datetime.fromisoformat(t['LastMod'])) for t in j]


def get_tournament(tid):
    return requests.get(f'{FFTBG_API_URL}/tournament/{tid}/json').text


def tournament_sync():
    LOG.info('Beginning tournament sync')
    TOURNAMENTS_ROOT.mkdir(exist_ok=True)
    changed = False

    for (tid, last_mod) in get_tournament_list():
        t_path = TOURNAMENTS_ROOT / f'{tid}.json'
        if t_path.exists():
            text = t_path.read_text()
            tournament_json = json.loads(text)
            modified = datetime.fromisoformat(tournament_json['LastMod'])
            if last_mod <= modified:
                continue
        LOG.info(f'Downloading tournament {tid} modified {last_mod.isoformat()}')
        t_path.write_text(get_tournament(tid))
        changed = True

    return changed
