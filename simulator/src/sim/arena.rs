use crate::dto::rust;
use crate::dto::rust::{StartingLocation, Tile};
use crate::sim::{Location, Panel};

#[derive(Clone)]
pub struct Arena {
    pub lower: Vec<Tile>,
    pub upper: Vec<Tile>,
    pub width: u8,
    pub height: u8,
    pub starting_locations: Vec<StartingLocation>,
}

impl Arena {
    pub fn from_dto(arena: rust::Arena) -> Arena {
        Arena {
            lower: arena.lower,
            upper: arena.upper,
            width: arena.width,
            height: arena.height,
            starting_locations: arena.starting_locations,
        }
    }

    pub fn to_index(&self, x: usize, y: usize) -> usize {
        (self.width as usize) * y + x
    }

    pub fn panel_to_index(&self, panel: Panel) -> Option<usize> {
        if panel.x() >= self.width || panel.y() >= self.height {
            return None;
        }
        Some(self.to_index(panel.x() as usize, panel.y() as usize))
    }

    pub fn location_to_index(&self, location: Location) -> Option<usize> {
        if location.x < 0
            || location.y < 0
            || location.x >= self.width as i16
            || location.y >= self.height as i16
        {
            return None;
        }
        Some(self.to_index(location.x as usize, location.y as usize))
    }

    pub fn tile(&self, panel: Panel) -> Tile {
        let idx = self.panel_to_index(panel).expect("panel out of bounds");
        if panel.layer() {
            self.upper[idx]
        } else {
            self.lower[idx]
        }
    }
}
