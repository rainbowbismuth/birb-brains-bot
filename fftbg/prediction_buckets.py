from fftbg.betting import expected_payoff
from fftbg.bird.memory import Memory

LIMIT = 5_000
mem = Memory()
log = mem.get_balance_log(limit=LIMIT)

n_buckets = 5
by_n = int(100 / n_buckets)
bucket_total = [0] * n_buckets
bucket_wins = [0] * n_buckets

for entry in log:
    i = int((entry.left_prediction * 100) / by_n)
    bucket_total[i] += 1
    if entry.left_wins:
        bucket_wins[i] += 1

    i = int((entry.right_prediction * 100) / by_n)
    bucket_total[i] += 1
    if not entry.left_wins:
        bucket_wins[i] += 1


for i in range(0, n_buckets):
    t = bucket_total[i]
    w = bucket_wins[i]
    if t == 0:
        t = 1
    diff = ((w*100/t)-(i+0.5)*by_n)
    print(f'{i * by_n:>3}%-{(i + 1) * by_n:>3}%: {w:>4}/{t:>4} or {w * 100 / t:>5.1f}%; diff {diff:>4.1f}%')
