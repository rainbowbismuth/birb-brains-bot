import fftbg.patch


def test_parse_equipment():
    patch = fftbg.patch.get_test_patch()
    assert patch.get_equipment('Dagger').wp == 4
    assert patch.get_equipment('Dagger').speed_bonus == 1
    assert patch.get_equipment('Thief Hat').speed_bonus == 2
    assert patch.get_equipment('Battle Folio').pa_bonus == 1
    assert patch.get_equipment('Bestiary').ma_bonus == 1
    assert patch.get_equipment('Small Mantle').phys_ev == 10
    assert patch.get_equipment('Small Mantle').speed_bonus == 1
    assert patch.get_equipment('Bracer').pa_bonus == 3
    assert patch.get_equipment('Kunai').move_bonus == 1
    assert patch.get_equipment('Ice Rod').weapon_element == 'Ice'
    assert patch.get_equipment('Gold Shield').phys_ev != 0
    assert patch.get_equipment('Gold Shield').magic_ev != 0
    assert 'Ice' in patch.get_equipment('Ice Rod').strengthens
    assert 'Holy' in patch.get_equipment('108 Gems').strengthens
    assert 'Earth' in patch.get_equipment('Defense Armlet').halves
    assert 'Ice' in patch.get_equipment('Ice Shield').absorbs
    assert 'Slow' in patch.get_equipment('Stone Gun').initial
