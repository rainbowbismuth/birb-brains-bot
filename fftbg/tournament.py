import json
import logging
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Tuple

import pandas

import fftbg.patch
from fftbg.ability import SKILL_TAG
from fftbg.arena import get_arena
from fftbg.combatant import CATEGORICAL, combatant_to_dict, can_heal, \
    zodiac_compat, can_cause, can_cancel, lethality
from fftbg.config import TOURNAMENTS_ROOT
from fftbg.patch import Patch

LOG = logging.getLogger(__name__)

COLORS = ['red', 'blue', 'green', 'yellow', 'white', 'black', 'purple', 'brown', 'champion']
CAN_HEAL_TEAM = [f'Can-Heal-Team-{i}' for i in range(4)]
CAN_HURT_ENEMY = [f'Can-Hurt-Enemy-{i}' for i in range(4)]
ZODIAC_TEAM = [f'Zodiac-Team-{i}' for i in range(4)]
ZODIAC_ENEMY = [f'Zodiac-Enemy-{i}' for i in range(4)]
OFFENSIVE_STATUSES = [
    'Poison', 'Sleep', 'Frog', 'Silence', 'Confusion', 'Darkness', 'Undead', 'Petrify',
    'Oil', 'Don\'t Act', 'Don\'t Move', 'Death Sentence', 'Charm', 'Stop', 'Blood Suck',
    'Berserk', 'Death'
]
CAUSE_STATUS = [f'Can-{status}-Enemy-{j}' for status in OFFENSIVE_STATUSES for j in range(4)]
CANCEL_STATUS = [f'Can-Cancel-{status}-Team-{j}' for status in OFFENSIVE_STATUSES for j in range(4)]
NUMERIC = ['Map-Area', 'Map-Team-Split', 'Map-Height-Diff', 'Map-Choke-Point', 'Map-Team-Distance',
           'Map-Min-Dimension', 'Map-Max-Dimension', 'Map-Archer-Boon', 'Map-Meat-Grinder'] \
          + CAN_HEAL_TEAM + CAN_HURT_ENEMY + ZODIAC_TEAM + ZODIAC_ENEMY \
          + CAUSE_STATUS + CANCEL_STATUS


def _calculate_hypothetical_match_ups():
    matches = []
    for i in range(0, len(COLORS) - 1, 2):
        left = COLORS[i]
        right = COLORS[i + 1]
        matches.append((left, right, i // 2))
    for left in COLORS[0:2]:
        for right in COLORS[2:4]:
            matches.append((left, right, 4))
    for left in COLORS[4:6]:
        for right in COLORS[6:8]:
            matches.append((left, right, 5))
    for left in COLORS[0:4]:
        for right in COLORS[4:8]:
            matches.append((left, right, 6))
    for left in COLORS[:-1]:
        matches.append((left, 'champion', 7))
    return matches


HYPOTHETICAL_MATCHES: List[Tuple[str, str, int]] = _calculate_hypothetical_match_ups()


@dataclass
class Team:
    color: str
    combatants: List[dict]

    def to_combatants(self, patch: Patch) -> List[dict]:
        return [combatant_to_dict(combatant, patch) for combatant in self.combatants]


@dataclass
class MatchUp:
    left: Team
    right: Team
    left_wins: Optional[bool]
    game_map: str

    def to_combatants(self, patch: Patch) -> List[dict]:
        arena = get_arena(self.game_map)
        arena_map = {
            'Map': self.game_map,
            'Map-Area': arena.area,
            'Map-Team-Split': arena.team_split,
            'Map-Height-Diff': arena.height_diff,
            'Map-Choke-Point': arena.choke_point,
            'Map-Team-Distance': arena.team_distance,
            'Map-Min-Dimension': arena.min_dimension,
            'Map-Max-Dimension': arena.max_dimension,
            'Map-Archer-Boon': arena.archer_boon,
            'Map-Meat-Grinder': arena.meat_grinder
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
        left_combatants = self.left.to_combatants(patch)
        right_combatants = self.right.to_combatants(patch)
        for i, combatant in enumerate(left_combatants):
            combatant.update(left)
            combatant['UIDX'] = i

            for j, ally in enumerate(left_combatants):
                combatant[f'Can-Heal-Team-{j}'] = can_heal(combatant, ally, patch)
                combatant[f'Zodiac-Team-{j}'] = zodiac_compat(combatant, ally)
                for status in OFFENSIVE_STATUSES:
                    combatant[f'Can-Cancel-{status}-Team-{j}'] = can_cancel(combatant, ally, status, patch)

            for j, victim in enumerate(right_combatants):
                combatant[f'Can-Hurt-Enemy-{j}'] = lethality(combatant, victim, patch)
                combatant[f'Zodiac-Enemy-{j}'] = zodiac_compat(combatant, victim)
                for status in OFFENSIVE_STATUSES:
                    combatant[f'Can-{status}-Enemy-{j}'] = can_cause(combatant, victim, status, patch)

            out.append(combatant)

        for i, combatant in enumerate(right_combatants):
            combatant.update(right)
            combatant['UIDX'] = i + 4

            for j, ally in enumerate(right_combatants):
                combatant[f'Can-Heal-Team-{j}'] = can_heal(combatant, ally, patch)
                combatant[f'Zodiac-Team-{j}'] = zodiac_compat(combatant, ally)
                for status in OFFENSIVE_STATUSES:
                    combatant[f'Can-Cancel-{status}-Team-{j}'] = can_cancel(combatant, ally, status, patch)

            for j, victim in enumerate(left_combatants):
                combatant[f'Can-Hurt-Enemy-{j}'] = lethality(combatant, victim, patch)
                combatant[f'Zodiac-Enemy-{j}'] = zodiac_compat(combatant, victim)
                for status in OFFENSIVE_STATUSES:
                    combatant[f'Can-{status}-Enemy-{j}'] = can_cause(combatant, victim, status, patch)

            out.append(combatant)

        return out


@dataclass
class Tournament:
    id: int
    modified: datetime
    teams: {str: Team}
    match_ups: List[MatchUp]

    def to_combatants(self, patch: Patch) -> List[dict]:
        tournament = {'TID': self.id, 'Modified': self.modified}
        out = []
        for i, match_up in enumerate(self.match_ups):
            for combatant in match_up.to_combatants(patch):
                combatant.update(tournament)
                combatant['MatchUp'] = i
                out.append(combatant)
        return out


def parse_hypothetical_tournament(tournament: dict) -> Tournament:
    modified, teams, tid = parse_teams(tournament)
    maps = tournament['Maps']

    match_ups = []
    for (left_color, right_color, map_index) in HYPOTHETICAL_MATCHES:
        left = teams[left_color]
        right = teams[right_color]
        match_ups.append(MatchUp(left, right, None, maps[map_index]))

    return Tournament(tid, modified, teams, match_ups)


def parse_tournament(path: Path) -> Tournament:
    tournament = json.loads(path.read_text())
    modified, teams, tid = parse_teams(tournament)

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


def parse_teams(tournament):
    tid = tournament['ID']
    modified = datetime.fromisoformat(tournament['LastMod'])
    teams = {}
    for color, team in tournament['Teams'].items():
        assert color in COLORS
        teams[color] = Team(color, team['Units'])
    return modified, teams, tid


def parse_tournaments() -> List[Tournament]:
    return [parse_tournament(p) for p in TOURNAMENTS_ROOT.glob('*.json')]


def tournament_to_combatants(tournaments: List[Tournament]) -> pandas.DataFrame:
    LOG.info('Converting tournaments to by-combatant DataFrame')
    data = []
    for tournament in tournaments:
        patch = fftbg.patch.get_patch(tournament.modified)
        data.extend(tournament.to_combatants(patch))

    _add_composite_id(data, 'UID', lambda c: f"{c['TID']}{c['Color']}{c['Name']}")
    _add_composite_id(data, 'MID', lambda c: f"{c['TID']}{c['MatchUp']}")

    df = _to_dataframe(data)
    df.set_index('MID')
    return df


def _to_dataframe(data) -> pandas.DataFrame:
    df = pandas.DataFrame(data)
    for category in CATEGORICAL:
        df[category].replace('', None, inplace=True)
        df[category] = df[category].astype('category')
    for column in df.keys():
        if not column.startswith(SKILL_TAG):
            continue
        df[column].fillna(0, inplace=True)
    return df


def match_ups_to_combatants(match_ups: List[MatchUp], patch: Patch) -> pandas.DataFrame:
    LOG.info('Converting match ups to by-combatant DataFrame')
    data = []
    for match_up in match_ups:
        data.extend(match_up.to_combatants(patch))
    return _to_dataframe(data)


def _add_composite_id(data, name, f):
    combatant_id = 0
    combatant_ids = {}
    for combatant in data:
        composite_id = f(combatant)
        if composite_id not in combatant_ids:
            combatant_ids[composite_id] = combatant_id
            combatant_id += 1
        combatant[name] = combatant_ids[composite_id]
