from ganesha.fft.map.gns import GNS
from ganesha.fft.map import Map, Tile, Terrain
from pathlib import Path
import json

SLOPE_TYPES = {
    0x00: 'Flat 0',
    0x85: 'Incline N',
    0x52: 'Incline E',
    0x25: 'Incline S',
    0x58: 'Incline W',
    0x41: 'Convex NE',
    0x11: 'Convex SE',
    0x14: 'Convex SW',
    0x44: 'Convex NW',
    0x96: 'Concave NE',
    0x66: 'Concave SE',
    0x69: 'Concave SW',
    0x99: 'Concave NW',
}

SURFACE_TYPES = {
    0x00: "Natural Surface",
    0x01: "Sand area",
    0x02: "Stalactite",
    0x03: "Grassland",
    0x04: "Thicket",
    0x05: "Snow",
    0x06: "Rocky cliff",
    0x07: "Gravel",
    0x08: "Wasteland",
    0x09: "Swamp",
    0x0A: "Marsh",
    0x0B: "Poisoned marsh",
    0x0C: "Lava rocks",
    0x0D: "Ice",
    0x0E: "Waterway",
    0x0F: "River",
    0x10: "Lake",
    0x11: "Sea",
    0x12: "Lava",
    0x13: "Road",
    0x14: "Wooden floor",
    0x15: "Stone floor",
    0x16: "Roof",
    0x17: "Stone wall",
    0x18: "Sky",
    0x19: "Darkness",
    0x1A: "Salt",
    0x1B: "Book",
    0x1C: "Obstacle",
    0x1D: "Rug",
    0x1E: "Tree",
    0x1F: "Box",
    0x20: "Brick",
    0x21: "Chimney",
    0x22: "Mud wall",
    0x23: "Bridge",
    0x24: "Water plant",
    0x25: "Stairs",
    0x26: "Furniture",
    0x27: "Ivy",
    0x28: "Deck",
    0x29: "Machine",
    0x2A: "Iron plate",
    0x2B: "Moss",
    0x2C: "Tombstone",
    0x2D: "Waterfall",
    0x2E: "Coffin",
    0x2F: "(blank)",
    0x30: "(blank)",
    0x3F: "Cross section"
}


def tile_to_dict(tile: Tile, x: int, y: int) -> dict:
    return {'x': x,
            'y': y,
            'no_cursor': tile.cant_cursor != 0,
            'no_walk': tile.cant_walk != 0,
            'depth': tile.depth,
            'height': tile.height,
            'slope_type': SLOPE_TYPES.get(tile.slope_type),
            'slope_type_numeric': tile.slope_type,
            'surface_type': SURFACE_TYPES[tile.surface_type],
            'surface_type_numeric': tile.surface_type,
            'slope_height': tile.slope_height}


def layer_to_dict(tiles: list, surface_types: set) -> list:
    out = []
    for y, tile_row in enumerate(tiles):
        out.append([])
        row = out[-1]
        for x, tile in enumerate(tile_row):
            surface_types.add(SURFACE_TYPES[tile.surface_type])
            row.append(tile_to_dict(tile, x, y))
    return out

ENT_SLOT_BYTES = 40
ENT_RECORD_BYTES = ENT_SLOT_BYTES * 16

UNIT_OFFSETS = [
    (0, 'Player 1', 0),
    (1, 'Player 2', 0),
    (2, 'Player 2', 1),
    (3, 'Player 1', 1),
    (5, 'Player 1', 2),
    (6, 'Player 2', 2),
    (7, 'Player 2', 3),
    (8, 'Player 1', 3)
]

UNIT_DIRECTIONS = {
    0: 'South',
    1: 'East',
    2: 'North',
    3: 'West',
}


def load_starting_locations(map_num: int) -> list:
    ent_record = 0x100 + map_num
    if ent_record >= 0x180:
        ent_offset = ent_record - 0x180
        ent_data = Path('data/ENTD/ENTD4.ENT').read_bytes()
    else:
        ent_offset = ent_record - 0x100
        ent_data = Path('data/ENTD/ENTD3.ENT').read_bytes()
    ent_start = ent_offset * ENT_RECORD_BYTES
    ent_end = ent_start + ENT_RECORD_BYTES
    ent_record = ent_data[ent_start: ent_end]
    units = []
    for (unit_idx, team, unit_num) in UNIT_OFFSETS:
        unit_offset = unit_idx * ENT_SLOT_BYTES
        pos_x = ent_record[unit_offset + 0x19]
        pos_y = ent_record[unit_offset + 0x1A]
        direction = ent_record[unit_offset + 0x1B] & 0b11
        units.append({
            'x': pos_x,
            'y': pos_y,
            'facing': UNIT_DIRECTIONS[direction],
            'team': team,
            'unit': unit_num
        })
    return units


def terrain_to_dict(terrain_data: Terrain, gns: str) -> dict:
    surface_types = set()
    lower_tiles = layer_to_dict(terrain_data.tiles[0], surface_types)
    upper_tiles = layer_to_dict(terrain_data.tiles[1], surface_types)
    width = len(lower_tiles[0])
    height = len(lower_tiles)
    map_num = int(gns[3:6])
    starting_locations = load_starting_locations(map_num)
    return {'gns': gns, 'num': map_num, 'lower': lower_tiles, 'upper': upper_tiles, 'width': width, 'height': height,
            'surface_types': sorted(surface_types), 'starting_locations': starting_locations}


def write_all_maps():
    for path in Path('data/MAP').glob('*.GNS'):
        try:
            gns_path = str(path)
            gns_name = path.name

            game_map = Map()
            game_map.gns = GNS()
            game_map.gns.read(gns_path)
            game_map.set_situation(0)
            game_map.read()
            terrain_data = game_map.get_terrain()

            out = terrain_to_dict(terrain_data, gns_name)
            txt = json.dumps(out, indent=4)
            out_path = Path(f'data/arena/{path.stem}.json')
            out_path.parent.mkdir(parents=True, exist_ok=True)
            out_path.write_text(txt)
        except:
            print(f'Error reading {path}')


if __name__ == '__main__':
    write_all_maps()
    # for k, v in SURFACE_TYPES.items():
    #     print(f'pub const SURFACE_{v.replace(" ","_").upper()}: u8 = {k};')
    # for k, v in SLOPE_TYPES.items():
    #     print(f'pub const SLOPE_{v.replace(" ","_").upper()}: u8 = {k};')
