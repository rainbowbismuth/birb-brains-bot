from dataclasses import dataclass
from dataclasses_json import dataclass_json
import re
from typing import List
from pathlib import Path
import math


@dataclass
class StartingLocation:
    x: int
    y: int
    team: str
    unit: int
    layer: bool


@dataclass
class Tile:
    height: int


@dataclass_json
@dataclass
class Arena:
    width: int
    height: int
    lower: List[List[Tile]]
    upper: List[List[Tile]]
    surface_types: List[str]
    starting_locations: List[StartingLocation]

    @property
    def area(self):
        return self.width * self.height

    def starting_location_for(self, cid) -> StartingLocation:
        looking_for = 'Player 1'
        if cid >= 4:
            looking_for = 'Player 2'
            cid -= 4
        for start_loc in self.starting_locations:
            if start_loc.team == looking_for and start_loc.unit == cid:
                return start_loc
        raise Exception(f'starting location for {cid} not found')

    def distance(self, id1, id2) -> (float, float):
        loc1 = self.starting_location_for(id1)
        loc2 = self.starting_location_for(id2)
        dist = math.sqrt(abs(loc1.x - loc2.x) ** 2 + abs(loc1.y - loc2.y))
        elev_diff = self.lower[loc1.y][loc1.x].height - self.lower[loc2.y][loc2.x].height
        return dist, elev_diff


ARENA_MAP = {}


def get_arena(name: str) -> Arena:
    if name in ARENA_MAP:
        return ARENA_MAP[name]
    num = int(re.match(r'(\d+)', name)[0])
    arena = Arena.from_json(Path(f'data/arena/MAP{num:03}.json').read_text())
    ARENA_MAP[name] = arena
    return arena
