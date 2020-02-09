from dataclasses import dataclass
from datetime import datetime
from typing import List, Optional
from pathlib import Path
import logging
import pandas
import json

from config import TOURNAMENTS_ROOT

LOG = logging.getLogger(__name__)

COLORS = ['red', 'blue', 'green', 'yellow', 'white', 'black', 'purple', 'brown', 'champion']
CATEGORICAL = ['Gender', 'Sign', 'Class', 'ActionSkill', 'ReactionSkill', 'SupportSkill', 'MoveSkill', 'Mainhand',
               'Offhand', 'Head', 'Armor', 'Accessory', 'Color', 'Side', 'Map']
SKILL_TAG = 'Skill - '


@dataclass
class Team:
    color: str
    units: List[dict]

    def to_units(self):
        units = []
        for unit in self.units:
            skills = {}
            for skill in unit['ClassSkills']:
                skills[SKILL_TAG + skill] = True
            for skill in unit['ExtraSkills']:
                skills[SKILL_TAG + skill] = True
            new_unit = {**unit, **skills, 'Color': self.color}
            del new_unit['ClassSkills']
            del new_unit['ExtraSkills']
            units.append(new_unit)
        return units


@dataclass
class MatchUp:
    left: Team
    right: Team
    left_wins: bool
    game_map: str

    def to_units(self):
        left = {'Side': 'Left', 'Winner': self.left_wins, 'Map': self.game_map}
        right = {'Side': 'Right', 'Winner': not self.left_wins, 'Map': self.game_map}
        out = []
        for unit in self.left.to_units():
            unit.update(left)
            out.append(unit)
        for unit in self.right.to_units():
            unit.update(right)
            out.append(unit)
        return out


@dataclass
class Tournament:
    id: int
    modified: datetime
    teams: {str: Team}
    match_ups: List[MatchUp]

    def to_units(self):
        tournament = {'TID': self.id, 'Modified': self.modified}
        out = []
        for i, match_up in enumerate(self.match_ups):
            for unit in match_up.to_units():
                unit.update(tournament)
                unit['MatchUp'] = i
                out.append(unit)
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


def parse_all_units() -> pandas.DataFrame:
    LOG.info('Parsing all units')
    data = []
    for tournament in parse_tournaments():
        data.extend(tournament.to_units())
    df = pandas.DataFrame(data)
    df['Name'] = df['Name'].astype('string')
    for category in CATEGORICAL:
        df[category] = df[category].astype('category')
    for column in df.keys():
        if not column.startswith(SKILL_TAG):
            continue
        df[column].fillna(False, inplace=True)
        df[column] = df[column].astype(bool)
    return df
