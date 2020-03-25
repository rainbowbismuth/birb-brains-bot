from pathlib import Path

import fftbg.patch
import fftbg.tournament


def main():
    tourny = fftbg.tournament.parse_tournament(Path('data/tournaments/1584818551017.json'))
    patch = fftbg.patch.get_patch(tourny.modified)

    print(patch.to_json())
    print(tourny.match_ups[0].to_json())


if __name__ == '__main__':
    main()
