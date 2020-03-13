import fftbg.patch


def test_parse_combatant_stats():
    patch = fftbg.patch.get_test_patch()
    assert patch.get_base_stats('Floating Eye', 'Monster')
    assert patch.get_base_stats('Ghost', 'Monster').pa == 11
    assert patch.get_base_stats('Chocobo', 'Monster').skills == ('Choco Attack', 'Choco Cure', 'Choco Esuna',
                                                                 'Choco Ball*')
