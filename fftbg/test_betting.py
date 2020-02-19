import fftbg.betting as betting


def test_optimal_is_optimal():
    p, c, e = 0.15, 200, 1000
    bet = betting.optimal_bet(p, c, e)

    payoff = betting.expected_payoff(p, bet, c, e)
    payoff_bet_low = betting.expected_payoff(p, bet - 10, c, e)
    payoff_bet_high = betting.expected_payoff(p, bet + 10, c, e)
    assert payoff_bet_low < payoff and payoff_bet_high < payoff


def test_optimal_bet_is_negative_if_losing_side():
    p, c, e = 0.20, 800, 1000
    bet = betting.optimal_bet(p, c, e)
    assert bet < 0
