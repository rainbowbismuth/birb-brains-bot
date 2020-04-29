from fftbg.betting import expected_payoff
from fftbg.bird.memory import Memory

LIMIT = 5000
mem = Memory()
log = mem.get_balance_log(limit=LIMIT)

count = 0
payout_bet_sum = 0
payout_sum = 0
neg_sum = 0
for entry in log:
    if entry.bet_on == entry.left_team:
        payoff_on_bet = expected_payoff(entry.left_prediction, entry.wager,
                                        entry.left_total_on_bet, entry.right_total_on_bet)
    else:
        payoff_on_bet = expected_payoff(entry.right_prediction, entry.wager,
                                        entry.right_total_on_bet, entry.left_total_on_bet)

    if entry.bet_on == entry.left_team:
        payoff_on_final = expected_payoff(entry.left_prediction, entry.wager,
                                          entry.left_total_final, entry.right_total_final)
    else:
        payoff_on_final = expected_payoff(entry.right_prediction, entry.wager,
                                          entry.right_total_final, entry.left_total_final)

    # print(f'payoff bet: {int(payoff_on_bet):d}, final: {int(payoff_on_final):d}')
    if payoff_on_final < -100:
        neg_sum += payoff_on_final
        count += 1
    payout_bet_sum += payoff_on_bet
    payout_sum += payoff_on_final

print(f'{count}/{LIMIT} = {count / LIMIT}')
print(f'bet {payout_bet_sum}/{LIMIT} = {payout_bet_sum / LIMIT}')
print(f'final {payout_sum}/{LIMIT} = {payout_sum / LIMIT}')

print(f'avg neg {neg_sum}/{count} = {neg_sum / count}')
