from dataclasses import dataclass

import numpy as np


@dataclass
class Arena:
    name: str
    width: int
    height: int
    # 0 = next to each other
    team_split: float
    # 0 = no elevation difference
    height_diff: float
    # 0 = completely open, 1 = tiny choke point
    choke_point: float
    # 0 = both teams start on top of each other
    team_distance: float

    @property
    def area(self):
        return self.width * self.height


ARENAS = [
    Arena(
        name='1) Gate of Igros Castle',
        width=10,
        height=13,
        team_split=0.75,
        height_diff=0.2,
        choke_point=0.6,
        team_distance=0.4
    ),
    Arena(
        name='10) Igros Castle',
        width=11,
        height=12,
        team_split=0.0,
        height_diff=0.7,
        choke_point=0.8,
        team_distance=0.6
    ),
    Arena(
        name='100) Public Cemetary',
        width=15,
        height=15,
        team_split=0.2,  # unsure
        height_diff=0.1,
        choke_point=0.0,
        team_distance=0.7
    ),
    Arena(
        name='101) Tutorial',
        width=11,
        height=11,
        team_split=0.0,
        height_diff=0.0,
        choke_point=0.0,
        team_distance=0.4
    ),
    Arena(
        name='102) Tutorial Bridge',
        width=11,
        height=11,
        team_split=0.0,
        height_diff=0.7,
        choke_point=0.2,
        team_distance=0.6
    ),
    Arena(
        name='103) Fovoham Windmill',
        width=8,
        height=10,
        team_split=0.4,
        height_diff=0.3,
        choke_point=0.5,
        team_distance=0.4
    ),
    Arena(
        name='104) Beoulve Residence',
        width=9,
        height=8,
        team_split=0.0,
        height_diff=0.1,
        choke_point=0.1,
        team_distance=0.1
    ),
    Arena(
        name='105) TERMINATE',
        width=12,
        height=9,
        team_split=0.0,  # unsure
        height_diff=0.3,
        choke_point=0.1,
        team_distance=0.5
    ),
    Arena(
        name='106) DELTA',
        width=16,
        height=10,
        team_split=0.1,
        height_diff=0.3,
        choke_point=0.5,
        team_distance=0.8
    ),
    Arena(
        name='107) NOGIAS',
        width=10,
        height=10,
        team_split=1.0,
        height_diff=1.0,
        choke_point=0.7,
        team_distance=0.7
    ),
    Arena(
        name='108) VOYAGE',
        width=10,
        height=10,
        team_split=0.1,
        height_diff=0.2,
        choke_point=0.3,
        team_distance=0.6
    ),
    Arena(
        name='109) BRIDGE',
        width=15,
        height=9,
        team_split=0.2,
        height_diff=0.4,
        choke_point=0.4,
        team_distance=0.8
    ),
    Arena(
        name='11) Office of Igros Castle',
        width=9,
        height=7,
        team_split=0.1,
        height_diff=0.1,
        choke_point=0.1,
        team_distance=0.1
    ),
    Arena(
        name='110) VALKYRIES',
        width=11,
        height=16,
        team_split=0.0,
        height_diff=0.2,
        choke_point=0.6,
        team_distance=0.8
    ),
    Arena(
        name='111) MLAPAN',
        width=12,
        height=7,
        team_split=0.0,
        height_diff=0.9,
        choke_point=0.5,
        team_distance=0.5
    ),
    Arena(
        name='112) TIGER',
        width=11,
        height=10,
        team_split=0.0,
        height_diff=0.3,
        choke_point=0.7,
        team_distance=0.9
    ),
    Arena(
        name='113) HORROR',
        width=12,
        height=10,
        team_split=0.4,
        height_diff=0.4,
        choke_point=0.5,
        team_distance=0.7
    ),
    Arena(
        name='114) END',
        width=13,
        height=13,
        team_split=0.0,
        height_diff=0.8,
        choke_point=0.8,
        team_distance=0.8
    ),
    Arena(
        name='115) Banished Fort',
        width=8,
        height=11,
        team_split=0.1,
        height_diff=0.5,
        choke_point=0.4,
        team_distance=0.5
    ),
    Arena(
        name='116) Arena',
        width=11,
        height=11,
        team_split=0.5,
        height_diff=0.0,
        choke_point=0.0,
        team_distance=0.5
    ),
]

#  '117) Checkerboard Land',
#  '12) Gate of Lionel Castle',
#  '125) Checkerboard Stairs',
#  '13) Lionel Castle',
#  '14) Office of Lionel Castle',
#  '15) Gate of Limberry Castle',
#  '16) Limberry Castle',
#  '17) Underground Cemetary of Limberry Castle',
#  '18) Office of Limberry Castle',
#  '19) Back Gate of Limberry Castle',
#  '2) Back Gate of Lesalia Castle',
#  '20) Zeltennia Castle',
#  '21) Office of Zeltennia Castle',
#  '22) Magic City Gariland',
#  '23) Beoulve Estate',
#  '24) Military Academy Auditorium',
#  '25) Yardow Fort City',
#  '26) Weapon Storage of Yardow',
#  '27) Goland Coal City',
#  '28) Colliery Underground First Floor',
#  '29) Colliery Underground Second Floor',
#  '3) Hall of St. Murond Temple',
#  '30) Colliery Underground Third Floor',
#  '31) Dorter Trade City',
#  '32) Slums in Dorter',
#  '33) Hospital in Slums',
#  '34) Cellar of Sand Mouse',
#  '35) Zaland Fort City',
#  '36) Church Outside of Town',
#  '37) Ruins Outside Zaland',
#  '38) Goug Machine City',
#  '39) Underground Passage in Goland',
#  '4) Lesalia Castle',
#  '40) Slums in Goug',
#  "41) Besrodio's House",
#  '42) Warjilis Trade City',
#  '43) Port of Warjilis',
#  '44) Bervenia Free City',
#  "45) Ruins of Zeltennia Castle's Church",
#  '46) Cemetary of Heavenly Knight, Balbanes',
#  '47) Zarghidas Trade City',
#  '48) Slums of Zarghidas',
#  '49) Fort Zeakden',
#  '5) Roof of Riovanes Castle',
#  '50) St. Murond Temple',
#  '51) Office of St. Murond Temple',
#  '52) Chapel of St. Murond Temple',
#  '53) Entrance to Death City',
#  '54) Lost Sacred Precincts',
#  '55) Graveyard of Airships',
#  '56) Orbonne Monastery',
#  '57) Underground Book Storage First Floor',
#  '58) Underground Book Storage Second Floor',
#  '59) Underground Book Storage Third Floor',
#  '6) Gate of Riovanes Castle',
#  '60) Underground Book Storage Fourth Floor',
#  '61) Underground Book Storage Fifth Floor',
#  '62) Chapel of Orbonne Monastery',
#  '63) Golgorand Execution Site',
#  '64) Sluice of Bethla Garrison',
#  '65) Granary of Bethla Garrison',
#  '66) South Wall of Bethla Garrison',
#  '67) North Wall of Bethla Garrison',
#  '68) Bethla Garrison',
#  '69) Murond Death City',
#  '7) Riovanes Castle',
#  '70) Nelveska Temple',
#  '71) Dolbodar Swamp',
#  '72) Fovoham Plains',
#  '73) Windmill Shed',
#  '74) Sweegy Woods',
#  '75) Bervenia Volcano',
#  '76) Zeklaus Desert',
#  '77) Lenalia Plateau',
#  '78) Zigolis Swamp',
#  '79) Yuguo Woods',
#  '8) Office of Riovanes Castle',
#  '80) Araguay Woods',
#  '81) Grog Hill',
#  '82) Bed Desert',
#  '83) Zirekile Falls',
#  '84) Bariaus Hill',
#  '85) Mandalia Plains',
#  '86) Doguola Pass',
#  '87) Bariaus Valley',
#  '88) Finath River',
#  '89) Poeskas Lake',
#  '9) Citadel of Igros Castle',
#  '90) Germinas Peak',
#  '91) Thieves Fort',
#  '92) Igros Residence',
#  '93) Wooden Shed',
#  '94) Stone Shed',
#  '95) Church',
#  '96) Pub',
#  '97) Lesalia Imperial Capital',
#  '98) Gate of Lesalia Castle',
#  '99) Main Street of Lesalia'

# noinspection PyTypeChecker
DEFAULT_ARENA = Arena(
    name='',
    width=np.median([arena.width for arena in ARENAS]),
    height=np.median([arena.height for arena in ARENAS]),
    team_split=np.median([arena.team_split for arena in ARENAS]),
    height_diff=np.median([arena.height_diff for arena in ARENAS]),
    choke_point=np.median([arena.choke_point for arena in ARENAS]),
    team_distance=np.median([arena.team_distance for arena in ARENAS])
)

ARENA_MAP = dict([(arena.name, arena) for arena in ARENAS])


def get_arena(name: str) -> Arena:
    return ARENA_MAP.get(name, DEFAULT_ARENA)
