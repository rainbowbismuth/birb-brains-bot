import numpy as np

from fftbg.bird.memory import Memory

LIMIT = 10000
mem = Memory()
log = mem.get_balance_log(limit=LIMIT)

loss = 0.0
correct = 0
for entry in log:
    # if entry.left_total_final == 0 or entry.right_total_final == 0:
    #     continue
    left_wins_percent = entry.left_total_final / (entry.left_total_final + entry.right_total_final)

    if entry.left_wins:
        loss += -np.log(left_wins_percent)
    else:
        loss += -np.log1p(left_wins_percent)

    if left_wins_percent > 0.5 and entry.left_wins:
        correct += 1
    if left_wins_percent < 0.5 and not entry.left_wins:
        correct += 1

print(f'correct: {correct / LIMIT}')
print(f'log loss: {loss / LIMIT}')
