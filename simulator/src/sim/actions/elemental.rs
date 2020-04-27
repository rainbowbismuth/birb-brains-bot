use crate::sim::actions::common::mod_5_formula_xa;
use crate::sim::actions::{Ability, AbilityImpl, Action};
use crate::sim::constants::*;

use crate::sim::{
    AoE, Combatant, CombatantId, Condition, Element, Simulation, Source, ALLY_OK,
    CAN_BE_CALCULATED, CAN_BE_REFLECTED, FOE_OK, NOT_ALIVE_OK, SILENCEABLE,
};

pub const ELEMENTAL_ABILITIES: &[Ability] = &[
    // Pitfall: 5 range, 1 AoE. Terrain: Soil, Wasteland, Road. Element: Dark. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Don't Move.
    Ability {
        name: "Pitfall",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Dark,
            terrain: &[SURFACE_NATURAL_SURFACE, SURFACE_WASTELAND],
            add_conditions: &[Condition::DontMove],
        },
    },
    // Water Ball: 5 range, 1 AoE. Terrain: Canal, River, Lake, Ocean, Waterfall. Element: Water. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Frog.
    Ability {
        name: "Water Ball",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Water,
            terrain: &[SURFACE_WATERWAY, SURFACE_RIVER, SURFACE_LAKE, SURFACE_SEA],
            add_conditions: &[Condition::Frog],
        },
    },
    // Hell Ivy: 5 range, 1 AoE. Terrain: Grassland, Underbrush, Vines. Element: Earth. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Stop.
    Ability {
        name: "Hell Ivy",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Earth,
            terrain: &[SURFACE_GRASSLAND, SURFACE_IVY, SURFACE_MOSS],
            add_conditions: &[Condition::Stop],
        },
    },
    // Hallowed Ground: 5 range, 1 AoE. Terrain: Gravel, Flagstone, Wall, Gravestone. Element: Holy. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Petrify, Oil (Random).
    Ability {
        name: "Hallowed Ground",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Holy,
            terrain: &[SURFACE_GRAVEL, SURFACE_STONE_FLOOR, SURFACE_STONE_WALL],
            add_conditions: &[Condition::Petrify, Condition::Oil],
        },
    },
    // Local Quake: 5 range, 1 AoE. Terrain: Stone, Basalt. Element: Earth. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Confusion.
    Ability {
        name: "Local Quake",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Earth,
            // TODO: Not sure about this...
            terrain: &[SURFACE_ROAD, SURFACE_BRICK],
            add_conditions: &[Condition::Confusion],
        },
    },
    // Static Shock: 5 range, 1 AoE. Terrain: Book, Tree, Bridge, Furnishing, Iron, Moss, Coffin. Element: Lightning. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Don't Act.
    Ability {
        name: "Static Shock",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Lightning,
            terrain: &[
                SURFACE_BOOK,
                SURFACE_TREE,
                SURFACE_BRIDGE,
                SURFACE_FURNITURE,
                SURFACE_IRON_PLATE,
                SURFACE_MOSS,
                SURFACE_COFFIN,
            ],
            add_conditions: &[Condition::DontAct],
        },
    },
    // Will-O-Wisp: 5 range, 1 AoE. Terrain: Wood Floor, Carpet, Coffer, Stairs, Deck. Element: Fire. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Sleep.
    Ability {
        name: "Will-O-Wisp",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Fire,
            terrain: &[
                SURFACE_WOODEN_FLOOR,
                SURFACE_RUG,
                SURFACE_COFFIN,
                SURFACE_STAIRS,
                SURFACE_DECK,
            ],
            add_conditions: &[Condition::Sleep],
        },
    },
    // Quicksand: 5 range, 1 AoE. Terrain: Marsh, Swamp, Poisonous Fen. Element: Water. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Death Sentence.
    Ability {
        name: "Quicksand",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Water,
            terrain: &[SURFACE_MARSH, SURFACE_SWAMP, SURFACE_POISONED_MARSH],
            add_conditions: &[Condition::DeathSentence],
        },
    },
    // Sand Storm: 5 range, 1 AoE. Terrain: Sand, Stalactite, Salt. Element: Wind. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Darkness.
    Ability {
        name: "Sand Storm",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Wind,
            terrain: &[SURFACE_SAND_AREA, SURFACE_STALACTITE, SURFACE_SALT],
            add_conditions: &[Condition::Darkness],
        },
    },
    // Blizzard: 5 range, 1 AoE. Terrain: Snow, Ice. Element: Ice. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Silence.
    Ability {
        name: "Blizzard",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Ice,
            terrain: &[SURFACE_SNOW, SURFACE_ICE],
            add_conditions: &[Condition::Silence],
        },
    },
    // Gusty Wind: 5 range, 1 AoE. Terrain: Roof, Chimney. Element: Wind. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Slow.
    Ability {
        name: "Gusty Wind",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Wind,
            terrain: &[SURFACE_ROOF, SURFACE_CHIMNEY],
            add_conditions: &[Condition::Slow],
        },
    },
    // Lava Ball: 5 range, 1 AoE. Terrain: Lava, Machinery. Element: Fire. Effect: Damage ((PA + 2) / 2 * MA); Chance to Add Death.
    Ability {
        name: "Lava Ball",
        flags: ALLY_OK | FOE_OK,
        mp_cost: 0,
        aoe: AoE::Diamond(1),
        implementation: &ElementalImpl {
            range: 5,
            element: Element::Fire,
            terrain: &[SURFACE_LAVA, SURFACE_LAVA_ROCKS, SURFACE_MACHINE],
            add_conditions: &[Condition::Death],
        },
    },
];

struct ElementalImpl {
    range: u8,
    element: Element,
    terrain: &'static [u8],
    add_conditions: &'static [Condition],
}

impl AbilityImpl for ElementalImpl {
    fn consider<'a>(
        &self,
        actions: &mut Vec<Action<'a>>,
        ability: &'a Ability<'a>,
        sim: &Simulation<'a>,
        user: &Combatant<'a>,
        target: &Combatant<'a>,
    ) {
        let tile = sim.tile(user.panel);
        if !self.terrain.contains(&tile.surface_type) {
            return;
        }
        actions.push(Action::new(ability, self.range, None, target.id()));
    }

    fn perform<'a>(&self, sim: &mut Simulation<'a>, user_id: CombatantId, target_id: CombatantId) {
        let user = sim.combatant(user_id);
        let target = sim.combatant(target_id);
        let tile = sim.tile(user.panel);
        if !self.terrain.contains(&tile.surface_type) {
            return;
        }

        let xa = mod_5_formula_xa(user.pa(), user, target, self.element, false);
        let damage = ((xa + 2) / 2) * user.ma();
        sim.change_target_hp(target_id, damage, Source::Ability);

        if sim.roll_auto_fail() <= 0.225 {
            return;
        }

        let index = sim.roll_inclusive(1, self.add_conditions.len() as i16) - 1;
        let condition = self.add_conditions[index as usize];
        sim.add_condition(target_id, condition, Source::Ability);
    }
}
