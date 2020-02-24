from fftbg.combatant import zodiac_compat


def test_zodiac_compat():
    gemini_male = {'Sign': 'Gemini', 'Gender': 'Male'}
    gemini_female = {'Sign': 'Gemini', 'Gender': 'Female'}

    sagittarius_male = {'Sign': 'Sagittarius', 'Gender': 'Male'}
    sagittarius_female = {'Sign': 'Sagittarius', 'Gender': 'Female'}
    assert zodiac_compat(gemini_male, gemini_female) == 1
    assert zodiac_compat(gemini_male, sagittarius_male) == 0.5
    assert zodiac_compat(gemini_male, sagittarius_female) == 1.5
    assert zodiac_compat(sagittarius_male, gemini_male) == 0.5
    assert zodiac_compat(sagittarius_female, gemini_male) == 1.5
