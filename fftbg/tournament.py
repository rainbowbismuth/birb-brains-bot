import json
import logging
import re
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Tuple

import numpy as np
import pandas
from dataclasses_json import dataclass_json

import fftbg.patch
from fftbg.arena import get_arena
from fftbg.combatant import CATEGORICAL, combatant_to_dict, can_heal, \
    zodiac_compat, can_cause, can_cancel, lethality, can_hurt
from fftbg.config import TOURNAMENTS_ROOT
from fftbg.patch import Patch
from fftbg.progress import progress_bar
import fftbg.load_map

LOG = logging.getLogger(__name__)

COLORS = ['red', 'blue', 'green', 'yellow', 'white', 'black', 'purple', 'brown', 'champion']
CAN_HEAL_TEAM = [f'Can-Heal-Team-{i}' for i in range(4)]
CAN_HURT_ENEMY = [f'Can-Hurt-Enemy-{i}' for i in range(4)]
CAN_LETHAL_ENEMY = [f'Can-Lethal-Enemy-{i}' for i in range(4)]
DISTANCE_ALLY = [f'Distance-Ally-{i}' for i in range(4)]
HEIGHT_DIFF_ALLY = [f'Height-Diff-Ally-{i}' for i in range(4)]
DISTANCE_ENEMY = [f'Distance-Enemy-{i}' for i in range(4)]
HEIGHT_DIFF_ENEMY = [f'Height-Diff-Enemy-{i}' for i in range(4)]
ZODIAC_TEAM = [f'Zodiac-Team-{i}' for i in range(4)]
ZODIAC_ENEMY = [f'Zodiac-Enemy-{i}' for i in range(4)]
OFFENSIVE_STATUSES = [
    'Poison', 'Sleep', 'Frog', 'Silence', 'Confusion', 'Darkness', 'Undead', 'Petrify',
    'Oil', 'Don\'t Act', 'Don\'t Move', 'Death Sentence', 'Charm', 'Stop', 'Blood Suck',
    'Berserk', 'Death'
]
CAUSE_STATUS = [f'Can-{status}-Enemy-{j}' for status in OFFENSIVE_STATUSES for j in range(4)]
CANCEL_STATUS = [f'Can-Cancel-{status}-Team-{j}' for status in OFFENSIVE_STATUSES for j in range(4)]

SURFACE_TYPES = ['Map-Surface-' + surface
                 for surface in fftbg.load_map.SURFACE_TYPES.values()
                 if surface not in ('(blank)', 'Ice')]
# TODO: Not ready to include this 'Sim-Win-Percent'
NUMERIC = ['Map-Area', 'Sim-Win-Percent', 'Sim-Win-Percent-Op'] + SURFACE_TYPES \
          + CAN_HEAL_TEAM + CAN_HURT_ENEMY + CAN_LETHAL_ENEMY + ZODIAC_TEAM + ZODIAC_ENEMY \
          + CAUSE_STATUS + CANCEL_STATUS + DISTANCE_ALLY + DISTANCE_ENEMY + HEIGHT_DIFF_ALLY + HEIGHT_DIFF_ENEMY


SKIP_ID_RANGES = [
    (1585741145317, 1585787081769)  # April fools, 2020.
]
BAD_TOURNAMENTS = {1594125397508, 1594172557423}


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


def look_up_prediction_index(left, right):
    for i, (team_1, team_2, _) in enumerate(HYPOTHETICAL_MATCHES):
        if left == team_1 and right == team_2:
            return i
    raise Exception(f'unable to find teams ({left}, {right})')


@dataclass
class Team:
    color: str
    combatants: List[dict]

    def to_combatants(self, patch: Patch) -> List[dict]:
        return [combatant_to_dict(combatant, patch) for combatant in self.combatants]


@dataclass_json
@dataclass
class MatchUp:
    tournament_id: int
    modified: datetime
    left: Team
    right: Team
    left_wins: Optional[bool]
    game_map: str
    game_map_num: int

    def to_combatants(self, patch: Patch) -> List[dict]:
        arena = get_arena(self.game_map)
        arena_map = {
            'Map': self.game_map,
            'Map-Area': arena.area,
        }
        for key in SURFACE_TYPES:
            arena_map[key] = 0.0
        for surface_type in arena.surface_types:
            arena_map['Map-Surface-'+surface_type] = 1.0

        left = {
            'Side': 'Left',
            'Side-N': 0,
            'Color': self.left.color,
            'LeftWins': self.left_wins,
            'Winner': self.left_wins,
            **arena_map
        }
        right = {
            'Side': 'Right',
            'Side-N': 1,
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

                (dist, height) = arena.distance(i, j)
                combatant[f'Distance-Ally-{j}'] = dist
                combatant[f'Height-Diff-Ally-{j}'] = height

            for j, victim in enumerate(right_combatants):
                combatant[f'Can-Hurt-Enemy-{j}'] = can_hurt(combatant, victim, patch)
                combatant[f'Can-Lethal-Enemy-{j}'] = lethality(combatant, victim, patch)
                combatant[f'Zodiac-Enemy-{j}'] = zodiac_compat(combatant, victim)
                for status in OFFENSIVE_STATUSES:
                    combatant[f'Can-{status}-Enemy-{j}'] = can_cause(combatant, victim, status, patch)

                (dist, height) = arena.distance(i, j+4)
                combatant[f'Distance-Enemy-{j}'] = dist
                combatant[f'Height-Diff-Enemy-{j}'] = height

            out.append(combatant)

        for i, combatant in enumerate(right_combatants):
            combatant.update(right)
            combatant['UIDX'] = i + 4

            for j, ally in enumerate(right_combatants):
                combatant[f'Can-Heal-Team-{j}'] = can_heal(combatant, ally, patch)
                combatant[f'Zodiac-Team-{j}'] = zodiac_compat(combatant, ally)
                for status in OFFENSIVE_STATUSES:
                    combatant[f'Can-Cancel-{status}-Team-{j}'] = can_cancel(combatant, ally, status, patch)

                (dist, height) = arena.distance(i+4, j+4)
                combatant[f'Distance-Ally-{j}'] = dist
                combatant[f'Height-Diff-Ally-{j}'] = height

            for j, victim in enumerate(left_combatants):
                combatant[f'Can-Hurt-Enemy-{j}'] = can_hurt(combatant, victim, patch)
                combatant[f'Can-Lethal-Enemy-{j}'] = lethality(combatant, victim, patch)
                combatant[f'Zodiac-Enemy-{j}'] = zodiac_compat(combatant, victim)
                for status in OFFENSIVE_STATUSES:
                    combatant[f'Can-{status}-Enemy-{j}'] = can_cause(combatant, victim, status, patch)

                (dist, height) = arena.distance(i+4, j)
                combatant[f'Distance-Enemy-{j}'] = dist
                combatant[f'Height-Diff-Enemy-{j}'] = height

            out.append(combatant)

        return out


@dataclass_json
@dataclass
class Tournament:
    id: int
    modified: datetime
    teams: {str: Team}
    match_ups: List[MatchUp]

    def to_combatants(self, patch: Patch, sim_data: dict = None) -> List[dict]:
        if not sim_data:
            sim_data = {}

        tournament = {'TID': self.id, 'Modified': self.modified}
        out = []

        tourny_sim_data = sim_data.get(str(self.id), {})

        for i, match_up in enumerate(self.match_ups):
            match_sim_data = tourny_sim_data.get(f'{match_up.left.color},{match_up.right.color}', 0.5)

            for j, combatant in enumerate(match_up.to_combatants(patch)):
                combatant.update(tournament)
                combatant['MatchUp'] = i

                if j < 4:
                    combatant['Sim-Win-Percent'] = match_sim_data
                    combatant['Sim-Win-Percent-Op'] = 1 - match_sim_data
                else:
                    combatant['Sim-Win-Percent'] = 1 - match_sim_data
                    combatant['Sim-Win-Percent-Op'] = match_sim_data

                out.append(combatant)
        return out

    def find_match_up(self, left_team: str, right_team: str) -> MatchUp:
        for match_up in self.match_ups:
            if match_up.left.color == left_team and match_up.right.color == right_team:
                return match_up


def parse_hypothetical_tournament(tournament: dict) -> Tournament:
    modified, teams, tid = parse_teams(tournament)
    maps = tournament['Maps']

    match_ups = []
    for (left_color, right_color, map_index) in HYPOTHETICAL_MATCHES:
        left = teams[left_color]
        right = teams[right_color]
        game_map = maps[map_index]
        map_num = int(re.match(r'(\d+)', game_map)[0])
        match_ups.append(MatchUp(tid, modified, left, right, None, game_map, map_num))

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
            if tid not in BAD_TOURNAMENTS:
                assert winner == left.color or winner == right.color
            left_wins = winner == left.color
            map_num = int(re.match(r'(\d+)', game_map)[0])
            match_up = MatchUp(tid, modified, left, right, left_wins, game_map, map_num)
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
    map_num = int(re.match(r'(\d+)', game_map)[0])
    match_up = MatchUp(tid, modified, left, right, left_wins, game_map, map_num)
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
    out = []
    for p in TOURNAMENTS_ROOT.glob('*.json'):
        tournament = parse_tournament(p)
        if tournament.id in BAD_TOURNAMENTS:
            continue
        skip = False
        for (start, end) in SKIP_ID_RANGES:
            # FIXME: Hack to cut out everthing before April 1st
            if tournament.id <= end:
            # if start <= tournament.id <= end:
                skip = True
                break
        if skip:
            continue
        out.append(tournament)
    return out


def load_sim_json_if_exists():
    path = Path('data/sim.json')
    if path.exists():
        return json.loads(path.read_text())
    return {}


def tournament_to_combatants(tournaments: List[Tournament]) -> pandas.DataFrame:
    LOG.debug('Converting tournaments to by-combatant DataFrame')
    data = []
    sim_data = load_sim_json_if_exists()
    for tournament in progress_bar(tournaments):
        patch = fftbg.patch.get_patch(tournament.modified)
        data.extend(tournament.to_combatants(patch, sim_data))

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
    for column in progress_bar(list(df.keys())):
        if df[column].dtype.name == 'category':
            continue
        df[column].fillna(0.0, inplace=True)

    map_mean = df.groupby('Map')['LeftWins'].mean()
    df['Map-Wins-Mean'] = np.abs(df['Side-N'].values - map_mean[df['Map']].values)
    return df


def match_ups_to_combatants(match_ups: List[MatchUp], patch: Patch) -> pandas.DataFrame:
    LOG.debug('Converting match ups to by-combatant DataFrame')
    data = []
    for match_up in match_ups:
        data.extend(match_up.to_combatants(patch))

    for combatant in data:
        combatant['Sim-Win-Percent'] = 0.5
        combatant['Sim-Win-Percent-Op'] = 0.5

    return _to_dataframe(data)


def match_up_to_combatants(match_up: MatchUp, patch: Patch, sim_left_wins: float) -> pandas.DataFrame:
    combatants = match_up.to_combatants(patch)

    for j, combatant in enumerate(combatants):
        if j < 4:
            combatant['Sim-Win-Percent'] = sim_left_wins
            combatant['Sim-Win-Percent-Op'] = 1 - sim_left_wins
        else:
            combatant['Sim-Win-Percent'] = 1 - sim_left_wins
            combatant['Sim-Win-Percent-Op'] = sim_left_wins

    return _to_dataframe(combatants)


def _add_composite_id(data, name, f):
    combatant_id = 0
    combatant_ids = {}
    for combatant in data:
        composite_id = f(combatant)
        if composite_id not in combatant_ids:
            combatant_ids[composite_id] = combatant_id
            combatant_id += 1
        combatant[name] = combatant_ids[composite_id]
