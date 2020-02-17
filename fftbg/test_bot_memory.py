from fftbg.bot_memory import BotMemory


def test_creation_and_logging():
    b = BotMemory(db_path=':memory:')
    arg = 1, 2, 3, 'red', 100, 'red', 0.456, 4, 5, 'blue', 0.678, 5, 6, True
    b.log_balance(*arg)
    log = b.get_balance_log(1)
    assert len(log) == 1
    assert tuple(log[0].__dict__.values())[2:] == arg


def test_placed_bet():
    b = BotMemory(db_path=':memory:')
    arg = 1, 'red', 100, 'red', 0.45, 'blue', 0.65
    b.placed_bet(*arg)
    bet = b.get_placed_bet()
    assert tuple(bet.__dict__.values())[2:] == arg
