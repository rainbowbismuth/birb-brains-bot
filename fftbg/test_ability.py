import fftbg.ability as ability


def test_abilities_with_multipliers():
    assert ability.get_ability('Chakra').multiplier == ability.MULT_PA
    assert ability.get_ability('Turn Punch').multiplier == ability.MULT_PA_PA_BANG
    assert ability.get_ability('Raise 2').multiplier == ability.MULT_FAITH_MA
    assert ability.get_ability('Fire 4').multiplier == ability.MULT_FAITH_MA
    assert ability.get_ability('Void Storage').element == 'Dark'
    assert ability.get_ability('Justice Sword').range == 2
    assert ability.get_ability('Cure 3').heals
    assert ability.get_ability('Meteor').damage
