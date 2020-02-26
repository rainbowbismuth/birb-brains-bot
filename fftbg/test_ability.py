import fftbg.ability as ability
import fftbg.patch


def test_abilities_with_multipliers():
    patch = fftbg.patch.get_test_patch()
    assert patch.get_ability('Chakra').multiplier == ability.MULT_PA
    assert patch.get_ability('Turn Punch').multiplier == ability.MULT_PA_PA_BANG
    assert patch.get_ability('Raise 2').multiplier == ability.MULT_FAITH_MA
    assert patch.get_ability('Fire 4').multiplier == ability.MULT_FAITH_MA
    assert patch.get_ability('Void Storage').element == 'Dark'
    assert patch.get_ability('Justice Sword').range == 2
    assert patch.get_ability('Cure 3').heals
    assert patch.get_ability('Meteor').damage
