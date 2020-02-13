import base_stats


def test_parse_combatant_stats():
    base_stats.parse_base_stats()
    assert base_stats.get_base_stats('Floating Eye', 'Monster')
    assert base_stats.get_base_stats('Ghost', 'Monster').pa == 11
