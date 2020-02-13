import json
import logging
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List

import pandas

from arena import get_arena
from combatant import CATEGORICAL, SKILL_TAG, combatant_to_dict
from config import TOURNAMENTS_ROOT

LOG = logging.getLogger(__name__)

COLORS = ['red', 'blue', 'green', 'yellow', 'white', 'black', 'purple', 'brown', 'champion']
NUMERIC = ['Map-Area', 'Map-Team-Split', 'Map-Height-Diff', 'Map-Choke-Point', 'Map-Team-Distance']


@dataclass
class Team:
    color: str
    combatants: List[dict]

    def to_combatants(self):
        return [combatant_to_dict(combatant) for combatant in self.combatants]


@dataclass
class MatchUp:
    left: Team
    right: Team
    left_wins: bool
    game_map: str

    def to_combatants(self):
        arena = get_arena(self.game_map)
        arena_map = {
            'Map': self.game_map,
            'Map-Area': arena.area,
            'Map-Team-Split': arena.team_split,
            'Map-Height-Diff': arena.height_diff,
            'Map-Choke-Point': arena.choke_point,
            'Map-Team-Distance': arena.team_distance
        }
        left = {
            'Side': 'Left',
            'Color': self.left.color,
            'LeftWins': self.left_wins,
            'Winner': self.left_wins,
            **arena_map
        }
        right = {
            'Side': 'Right',
            'Color': self.right.color,
            'LeftWins': self.left_wins,
            'Winner': not self.left_wins,
            **arena_map
        }
        out = []
        left_combatants = self.left.to_combatants()
        right_combatants = self.right.to_combatants()
        for i, combatant in enumerate(left_combatants):
            combatant.update(left)
            combatant['UIDX'] = i
            out.append(combatant)

        for i, combatant in enumerate(right_combatants):
            combatant.update(right)
            combatant['UIDX'] = i + 4
            out.append(combatant)
        return out


@dataclass
class Tournament:
    id: int
    modified: datetime
    teams: {str: Team}
    match_ups: List[MatchUp]

    def to_combatants(self):
        tournament = {'TID': self.id, 'Modified': self.modified}
        out = []
        for i, match_up in enumerate(self.match_ups):
            for combatant in match_up.to_combatants():
                combatant.update(tournament)
                combatant['MatchUp'] = i
                out.append(combatant)
        return out


def parse_tournament(path: Path) -> Tournament:
    tournament = json.loads(path.read_text())
    tid = tournament['ID']
    modified = datetime.fromisoformat(tournament['LastMod'])
    teams = {}
    for color, team in tournament['Teams'].items():
        assert color in COLORS
        teams[color] = Team(color, team['Units'])

    match_n = 0
    bracket = COLORS[:-1]
    match_ups = []
    winners = tournament['Winners']

    if len(winners) != 8:
        return Tournament(tid, modified, teams, [])

    maps = tournament['Maps']
    while len(bracket) > 1:
        new_bracket = []
        for i in range(len(bracket) // 2):
            winner = winners[match_n]
            game_map = maps[match_n]
            match_n += 1
            left = teams[bracket[i * 2]]
            right = teams[bracket[i * 2 + 1]]
            assert winner == left.color or winner == right.color
            left_wins = winner == left.color
            match_up = MatchUp(left, right, left_wins, game_map)
            match_ups.append(match_up)
            if left_wins:
                new_bracket.append(left.color)
            else:
                new_bracket.append(right.color)
        bracket = new_bracket
    winner = winners[match_n]
    game_map = maps[match_n]
    left = teams[bracket[0]]
    right = teams['champion']
    assert winner == left.color or winner == right.color
    left_wins = winner == left.color
    match_up = MatchUp(left, right, left_wins, game_map)
    match_ups.append(match_up)
    return Tournament(tid, modified, teams, match_ups)


def parse_tournaments() -> List[Tournament]:
    return [parse_tournament(p) for p in TOURNAMENTS_ROOT.glob('*.json')]


def tournament_to_combatants(tournaments: List[Tournament]) -> pandas.DataFrame:
    LOG.info('Converting tournaments to by-combatant DataFrame')
    data = []
    for tournament in tournaments:
        data.extend(tournament.to_combatants())

    _add_composite_id(data, 'UID', lambda c: f"{c['TID']}{c['Color']}{c['Name']}")
    _add_composite_id(data, 'MID', lambda c: f"{c['TID']}{c['MatchUp']}")

    df = pandas.DataFrame(data)
    for category in CATEGORICAL:
        df[category].replace('', None, inplace=True)
        df[category] = df[category].astype('category')
    for column in df.keys():
        if not column.startswith(SKILL_TAG):
            continue
        df[column].fillna(False, inplace=True)
        df[column] = df[column].astype(bool)
    df.set_index('MID')
    return df


def _add_composite_id(data, name, f):
    combatant_id = 0
    combatant_ids = {}
    for combatant in data:
        composite_id = f(combatant)
        if composite_id not in combatant_ids:
            combatant_ids[composite_id] = combatant_id
            combatant_id += 1
        combatant[name] = combatant_ids[composite_id]
