import fftbg.event_stream
import fftbg.server
import fftbg.twitch.msg_types as msg_types


def main():
    fftbg.server.set_name('fftbg.twitch.listen')
    fftbg.server.configure_logging('TWITCH_LOG_LEVEL')

    redis = fftbg.server.get_redis()
    event_stream = fftbg.event_stream.EventStream(redis)
    while True:
        for (_, msg) in event_stream.read():
            if msg.get('type') == msg_types.RECV_SAY:
                print(f'{msg["user"]}: {msg["text"]}')


if __name__ == '__main__':
    main()
