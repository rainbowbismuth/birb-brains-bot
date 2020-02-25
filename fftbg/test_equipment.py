import fftbg.equipment as equipment


def test_parse_equipment():
    equipment.parse_equipment()
    assert equipment.get_equipment('Dagger').wp == 4
    assert equipment.get_equipment('Dagger').speed_bonus == 1
    assert equipment.get_equipment('Thief Hat').speed_bonus == 2
    assert equipment.get_equipment('Battle Folio').pa_bonus == 1
    assert equipment.get_equipment('Bestiary').ma_bonus == 1
    assert equipment.get_equipment('Small Mantle').phys_ev == 10
    assert equipment.get_equipment('Small Mantle').speed_bonus == 1
    assert equipment.get_equipment('Bracer').pa_bonus == 3
    assert equipment.get_equipment('Kunai').move_bonus == 1
    assert equipment.get_equipment('Ice Rod').weapon_element == 'Ice'
    assert equipment.get_equipment('Gold Shield').phys_ev != 0
    assert equipment.get_equipment('Gold Shield').magic_ev != 0
    assert 'Ice' in equipment.get_equipment('Ice Rod').strengthens
    assert 'Holy' in equipment.get_equipment('108 Gems').strengthens
    assert 'Earth' in equipment.get_equipment('Defense Armlet').halves
    assert 'Ice' in equipment.get_equipment('Ice Shield').absorbs
