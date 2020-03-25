import subprocess
from pathlib import Path

import fftbg.patch
import fftbg.tournament


def main():
    tourny = fftbg.tournament.parse_tournament(Path('data/tournaments/1584818551017.json'))
    patch = fftbg.patch.get_patch(tourny.modified)

    patch_json = patch.to_json()
    matchup_json = tourny.match_ups[0].to_json()
    combined = patch_json + '\n' + matchup_json

    completed = subprocess.run('simulator/target/release/simulator',
                               input=combined,
                               text=True,
                               capture_output=True)
    if completed.stderr:
        raise Exception(completed.stderr)
    print(completed.stdout)


if __name__ == '__main__':
    main()
