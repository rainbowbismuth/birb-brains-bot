use crate::sim::actions::Ability;

use crate::sim::{
    Combatant, CombatantId, Condition, Element, Simulation, Source, CAN_BE_CALCULATED,
    CAN_BE_REFLECTED, NOT_ALIVE_OK, SILENCEABLE,
};

pub const ELEMENTAL_ABILITIES: &[Ability] = &[
    // Pitfall: 5 range, 1 AoE. Terrain: Soil, Wasteland, Road. Element: Dark. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Don't Move.
    // Water Ball: 5 range, 1 AoE. Terrain: Canal, River, Lake, Ocean, Waterfall. Element: Water. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Frog.
    // Hell Ivy: 5 range, 1 AoE. Terrain: Grassland, Underbrush, Vines. Element: Earth. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Stop.
    // Hallowed Ground: 5 range, 1 AoE. Terrain: Gravel, Flagstone, Wall, Gravestone. Element: Holy. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Petrify, Oil (Random).
    // Local Quake: 5 range, 1 AoE. Terrain: Stone, Basalt. Element: Earth. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Confusion.
    // Static Shock: 5 range, 1 AoE. Terrain: Book, Tree, Bridge, Furnishing, Iron, Moss, Coffin. Element: Lightning. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Don't Act.
    // Will-O-Wisp: 5 range, 1 AoE. Terrain: Wood Floor, Carpet, Coffer, Stairs, Deck. Element: Fire. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Sleep.
    // Quicksand: 5 range, 1 AoE. Terrain: Marsh, Swamp, Poisonous Fen. Element: Water. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Death Sentence.
    // Sand Storm: 5 range, 1 AoE. Terrain: Sand, Stalactite, Salt. Element: Wind. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Darkness.
    // Blizzard: 5 range, 1 AoE. Terrain: Snow, Ice. Element: Ice. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Silence.
    // Gusty Wind: 5 range, 1 AoE. Terrain: Roof, Chimney. Element: Wind. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Slow.
    // Lava Ball: 5 range, 1 AoE. Terrain: Lava, Machinery. Element: Fire. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Death.
];

struct ElementalImpl {
    range: u8,
    element: Element,
    terrain: &'static [u8],
    add_conditions: &'static [Condition],
}
