import tournament
from pathlib import Path


def test_to_units():
    t = tournament.parse_tournament(Path('tournaments/1580897292273.json'))
    assert len(t.to_units()) == 4 * 2 * 8


def test_parse_tournaments():
    assert len(tournament.parse_tournaments()) > 100


def test_parse_all_units():
    assert len(tournament.parse_all_units()) > 1000
