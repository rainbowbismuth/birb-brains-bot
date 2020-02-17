from bot_memory import BotMemory


def test_creation_and_logging():
    b = BotMemory(db_path=':memory:')
    arg = 1, 2, 3, 'red', 100, 'red', 0.456, 4, 5, 'blue', 0.678, 5, 6, True
    b.log_balance(*arg)
    log = b.get_balance_log()
    assert len(log) == 1
    assert log[0][2:] == arg
