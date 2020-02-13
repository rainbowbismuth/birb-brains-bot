import config
import tournament


def test_to_units():
    t = tournament.parse_tournament(config.TOURNAMENTS_ROOT / '1580897292273.json')
    assert len(t.to_units()) == 4 * 2 * 8


def test_tournament_conversions():
    tournaments = tournament.parse_tournaments()[:10]
    units = tournament.tournaments_to_units(tournaments)
    assert len(units) >= 70 * 8
