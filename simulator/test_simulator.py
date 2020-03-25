import json
import subprocess
from pathlib import Path

import tqdm

import fftbg.patch
import fftbg.tournament


def main():
    simulator = subprocess.Popen('simulator/target/release/simulator',
                                 text=True,
                                 bufsize=1,
                                 stdin=subprocess.PIPE,
                                 stdout=subprocess.PIPE)

    patch_texts = {}

    for path in tqdm.tqdm(list(Path('data/tournaments').glob('*.json'))):
        tourny = fftbg.tournament.parse_tournament(path)
        patch = fftbg.patch.get_patch(tourny.modified)
        if patch.time not in patch_texts:
            # Cut out some of the bulk that I don't use on the other side
            patch.ability.by_adds = None
            patch.ability.by_cancels = None

            patch_json = patch.to_json()
            patch_texts[patch.time] = patch_json
        patch_text = patch_texts[patch.time]
        for match_up in tourny.match_ups:
            match_text = match_up.to_json()
            simulator.stdin.writelines([patch_text, '\n', match_text, '\n'])

            output = json.loads(simulator.stdout.readline())
            if output['error']:
                raise Exception(output['error'])


if __name__ == '__main__':
    main()
