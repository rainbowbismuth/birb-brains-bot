import json
from copy import deepcopy
from pathlib import Path

import fftbg.brains.baked_model
import fftbg.download
import fftbg.tournament

KEYS = ['ReactionSkill', 'SupportSkill', 'MoveSkill', 'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']


def main():
    baked = fftbg.brains.baked_model.BakedModel()
    latest_json = json.loads(Path('data/tournaments/1582860952351.json').read_text())
    # latest_json = json.loads(fftbg.download.get_latest_tournament())
    latest = fftbg.tournament.parse_hypothetical_tournament(latest_json)
    match_up_root = latest.match_ups[1]  # just taking the first match up
    # match_up_root.right.combatants[3]['Mainhand'] = 'Main Gauche'
    match_ups = []
    base_line = baked.predict_match_ups([match_up_root], latest.modified)
    for i, combatant in enumerate(match_up_root.left.combatants):
        for k in KEYS:
            if match_up_root.left.combatants[i][k] == '':
                continue
            copied = deepcopy(match_up_root)
            copied.left.combatants[i][k] = ''
            match_ups.append((i, combatant[k], copied))
        for k in combatant['ClassSkills']:
            copied = deepcopy(match_up_root)
            copied.left.combatants[i]['ClassSkills'].remove(k)
            match_ups.append((i, k, copied))
        for k in combatant['ExtraSkills']:
            copied = deepcopy(match_up_root)
            copied.left.combatants[i]['ExtraSkills'].remove(k)
            match_ups.append((i, k, copied))
    for i, combatant in enumerate(match_up_root.right.combatants):
        for k in KEYS:
            if match_up_root.right.combatants[i][k] == '':
                continue
            copied = deepcopy(match_up_root)
            copied.right.combatants[i][k] = ''
            match_ups.append((i + 4, combatant[k], copied))
        for k in combatant['ClassSkills']:
            copied = deepcopy(match_up_root)
            copied.right.combatants[i]['ClassSkills'].remove(k)
            match_ups.append((i + 4, k, copied))
        for k in combatant['ExtraSkills']:
            copied = deepcopy(match_up_root)
            copied.right.combatants[i]['ExtraSkills'].remove(k)
            match_ups.append((i + 4, k, copied))
    predictions = baked.predict_match_ups([x[2] for x in match_ups], latest.modified)

    diffs = []
    for x, (i, k, match_up) in enumerate(match_ups):
        if i < 4:
            d = base_line[0][1] - predictions[x][1]
        else:
            d = base_line[0][0] - predictions[x][0]
        diffs.append((d, i, k))

    diffs = sorted(diffs, reverse=True)
    diffs = sorted(diffs, key=lambda x: x[1])
    for diff in diffs:
        print(diff)


if __name__ == '__main__':
    main()
