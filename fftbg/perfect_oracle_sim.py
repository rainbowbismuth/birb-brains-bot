import numpy.random
from random import random
import math

trials = 100_000
mean = 0.5


def run_sim(std):
    nums = numpy.random.normal(mean, std, trials)
    guessed_correct = 0
    log_loss = 0
    for i, left_wins in enumerate(nums):
        left_wins_odd = max(0, min(1.0, left_wins))
        correct_prop = max(left_wins_odd, 1.0 - left_wins_odd)
        if random() < correct_prop:
            guessed_correct += 1
            log_loss += -math.log(correct_prop)
        else:
            log_loss += -math.log(1.0 - correct_prop)

    print(f'odds stddev: {std}')
    print(f'accuracy:    {guessed_correct / trials}')
    print(f'log_loss:    {log_loss / trials}\n')


for std in [0.1, 0.15, 0.2, 0.25, 0.3]:
    run_sim(std)

run_sim(0.17308536519525078)
