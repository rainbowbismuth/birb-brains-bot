import config
import tournament


def test_to_combatants():
    t = tournament.parse_tournament(config.TOURNAMENTS_ROOT / '1580897292273.json')
    assert len(t.to_combatants()) == 4 * 2 * 8


def test_tournament_conversions():
    tournaments = tournament.parse_tournaments()[:10]
    combatants = tournament.tournament_to_combatants(tournaments)
    assert len(combatants) >= 70 * 8
