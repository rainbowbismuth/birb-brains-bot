def optimal_bet(win_percent, our_side, their_side):
    top = win_percent * (our_side + their_side) - our_side
    bottom = 2 - win_percent * 2
    return top / bottom


def expected_payoff(win_percent, bet, our_side, their_side):
    if_win = win_percent * (bet / (bet + our_side)) * their_side
    if_lose = (1 - win_percent) * (-bet)
    return if_win + if_lose
