from copy import deepcopy
from typing import List

from fftbg.brains.baked_model import BakedModel
from fftbg.tournament import MatchUp

KEYS = ['ReactionSkill', 'SupportSkill', 'MoveSkill', 'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']


def compute(model: BakedModel, match_up_root: MatchUp, patch_time) -> List[dict]:
    match_ups = []
    base_line = model.predict_match_ups([match_up_root], patch_time)
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
    predictions = model.predict_match_ups([x[2] for x in match_ups], patch_time)

    diffs = []
    for x, (i, k, match_up) in enumerate(match_ups):
        if i < 4:
            d = base_line[0][1] - predictions[x][1]
        else:
            d = base_line[0][0] - predictions[x][0]
        diffs.append((d, i, k))

    output = [{}, {}, {}, {}, {}, {}, {}, {}]
    for i, combatant in enumerate(match_up_root.left.combatants + match_up_root.right.combatants):
        output[i]['name'] = combatant['Name']
        output[i]['gender'] = combatant['Gender']
        output[i]['job'] = combatant['Class']
        output[i]['sign'] = combatant['Sign']
        output[i]['brave'] = combatant['Brave']
        output[i]['faith'] = combatant['Faith']
        output[i]['plus'] = []
        output[i]['minus'] = []

    for (d, i, k) in diffs:
        d = float(d)
        if d > 0.0:
            output[i]['plus'].append((k, d))
        else:
            output[i]['minus'].append((k, d))

    for data in output:
        data['plus'].sort(key=lambda x: x[1], reverse=True)
        data['minus'].sort(key=lambda x: x[1])

    return output
