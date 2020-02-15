import twitch_bot


def test_regular_expressions():
    assert twitch_bot.BALANCE_RE.findall('randomuser1, your balance is: 30,500G (Spendable: 30,368G).; random_user3, '
                                         'your balance is: 3,363G (Spendable: 3,239G).; anotherRANDOMuser, your balance is: '
                                         '5,700G (Spendable: 5,464G).')

    user, balance = twitch_bot.BALANCE_RE.findall('abc, your balance is: 10,400G')[0]
    assert user == 'abc'
    assert balance == '10,400'

    assert twitch_bot.BETTING_OPEN_RE.findall('Betting is open for black vs brown.')

    assert twitch_bot.BETTING_CLOSE_RE.findall(
        'Betting is closed: Final Bets: purple - 55 bets for 38,304G; brown - 42 bets '
        'for 31,592G... Good luck!')

    assert twitch_bot.ODDS_RE.findall(
        'Final Bets: purple - 72 bets for 88,285G (85.9%, x0.16); brown - 44 bets for 14,477G (14.1%, x6.10)')
