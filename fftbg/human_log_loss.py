import numpy as np

from fftbg.bird.memory import Memory

LIMIT = 85322
mem = Memory()
log = mem.get_balance_log(limit=LIMIT)

loss = 0.0
correct = 0

left_wins = []

for entry in log:
    if entry.left_total_final == 0 or entry.right_total_final == 0:
        continue
    left_wins_percent = entry.left_total_final / (entry.left_total_final + entry.right_total_final)
    left_wins.append(left_wins_percent)

    if entry.left_wins:
        loss += -np.log(left_wins_percent)
    else:
        loss += -np.log(1.0 - left_wins_percent)

    if left_wins_percent > 0.5 and entry.left_wins:
        correct += 1
    if left_wins_percent < 0.5 and not entry.left_wins:
        correct += 1

print(f'odds mean: {np.mean(left_wins)}')
print(f'odds std: {np.std(left_wins)}')

print(f'correct: {correct / LIMIT}')
print(f'log loss: {loss / LIMIT}')
