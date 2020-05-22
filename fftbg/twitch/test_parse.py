import fftbg.twitch.parse as parse


def test_regular_expressions():
    assert parse.BALANCE_RE.findall('randomuser1, your balance is: 30,500G (Spendable: 30,368G).; random_user3, '
                                    'your balance is: 3,363G (Spendable: 3,239G).; anotherRANDOMuser, your balance'
                                    'is: 5,700G (Spendable: 5,464G).')

    user, balance = parse.BALANCE_RE.findall('abc, your balance is: 10,400G')[0]
    assert user == 'abc'
    assert balance == '10,400'

    assert parse.BETTING_OPEN_RE.findall('Betting is open for black vs brown.')

    assert parse.BETTING_CLOSE_RE.findall(
        'Betting is closed: Final Bets: purple - 55 bets for 38,304G; brown - 42 bets '
        'for 31,592G... Good luck!')

    assert parse.ODDS_RE.findall(
        'Final Bets: purple - 72 bets for 88,285G (85.9%, x0.16); brown - 44 bets for 14,477G (14.1%, x6.10)')

    assert parse.BOTS_CANNOT_BET_RE.findall('BirbBrainsBot, bots cannot bet in the final 20 seconds!')
    assert parse.TEAM_VICTORY.findall('The blue team was victorious!')
    assert parse.NEW_TOURNAMENT_SKILL_DROP.findall('This tournament\'s Skill Drop is: PreferredArms.')


def test_bet_regular_expressions():
    assert parse.BET_RE.findall('!bet red 100')
    assert parse.BET_RE.findall('!bet random 51%')
    assert parse.ALL_IN_RE.findall('!allin yelow')


def test_parse_comma_int():
    assert parse.parse_comma_int('100') == 100
    assert parse.parse_comma_int('1,000') == 1000
