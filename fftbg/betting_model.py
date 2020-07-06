import fftbg.server
import fftbg.bird.memory
import logging
import pandas

LOG = logging.getLogger(__name__)


def main():
    fftbg.server.configure_logging('MODEL_LOG_LEVEL')
    LOG.info('Going to compute betting model')
    memory = fftbg.bird.memory.Memory()
    balance_log = memory.get_balance_log(limit=1_000_000)

    df = pandas.DataFrame(balance_log)


if __name__ == '__main__':
    main()