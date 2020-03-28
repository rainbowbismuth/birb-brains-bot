import json

import fftbg.config as config
import fftbg.patch
import fftbg.tournament as tournament


def test_to_combatants():
    patch = fftbg.patch.get_test_patch()
    t = tournament.parse_tournament(config.TOURNAMENTS_ROOT / '1580897292273.json')
    assert len(t.to_combatants(patch)) == 4 * 2 * 8


def test_hypothetical_tournament():
    fp = config.TOURNAMENTS_ROOT / '1580897292273.json'
    js = json.loads(fp.read_text())
    hypothetical = tournament.parse_hypothetical_tournament(js)
    total_matches = 4 + 8 + 16 + 8
    assert len(hypothetical.match_ups) == total_matches

    to_df = tournament.tournament_to_combatants([hypothetical])
    assert len(to_df) == total_matches * 8


def test_hypothetical_match_up_list():
    assert len([c for (c, _, _) in tournament.HYPOTHETICAL_MATCHES if c == 'champion']) == 0
    assert len([c for (_, c, _) in tournament.HYPOTHETICAL_MATCHES if c == 'champion']) == 8
