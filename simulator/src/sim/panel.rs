use crate::sim::{Distance, Facing, Location};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Panel {
    x_var: u8,
    y_var: u8,
}

impl Panel {
    pub fn new(location: Location, layer: bool) -> Panel {
        if location.x < 0 || location.y < 0 || location.y >= 0x80 {
            return Self::out_of_bounds();
        }
        let layer_bit = (layer as u8) << 7;
        Panel {
            x_var: location.x as u8,
            y_var: location.y as u8 | layer_bit,
        }
    }

    pub fn coords(x: u8, y: u8, layer: bool) -> Panel {
        if y >= 0x80 {
            return Self::out_of_bounds();
        }
        let layer_bit = (layer as u8) << 7;
        Panel {
            x_var: x,
            y_var: y | layer_bit,
        }
    }

    pub const fn out_of_bounds() -> Panel {
        Panel {
            x_var: 0xFF,
            y_var: 0xFF,
        }
    }

    pub fn plus(self, location: Location) -> Panel {
        let new_location = self.location() + location;
        self.on_same_layer(new_location)
    }

    pub fn x(self) -> u8 {
        self.x_var
    }

    pub fn y(self) -> u8 {
        self.y_var & 0x7F
    }

    pub fn layer(self) -> bool {
        self.y_var & 0x80 != 0
    }

    pub fn location(self) -> Location {
        Location::new(self.x() as i16, self.y() as i16)
    }

    pub fn on_same_layer(self, location: Location) -> Panel {
        Panel::new(location, self.layer())
    }

    pub fn other_layer(self) -> Panel {
        Panel {
            x_var: self.x_var,
            y_var: self.y_var ^ 0x80,
        }
    }

    pub fn lower(self) -> Panel {
        Panel {
            x_var: self.x_var,
            y_var: self.y_var & 0x7F,
        }
    }

    pub fn upper(self) -> Panel {
        Panel {
            x_var: self.x_var,
            y_var: self.y_var | 0x80,
        }
    }

    pub fn facing_towards(self, other: Panel) -> Facing {
        Facing::towards(self.location(), other.location())
    }

    pub fn distance(self, other: Panel) -> Distance {
        (self.x() as i16 - other.x() as i16).abs() + (self.y() as i16 - other.y() as i16).abs()
    }

    pub fn lined_up(self, other: Panel) -> bool {
        self.x() == other.x() || self.y() == other.y()
    }

    // TODO: This isn't great, but I want to get to the point of compiling
    pub fn diamond(self, size: u8) -> impl Iterator<Item = Panel> {
        self.location()
            .diamond(size)
            .map(move |location| self.on_same_layer(location))
    }

    // TODO: This isn't great, but I want to get to the point of compiling
    pub fn line(self, other: Panel) -> impl Iterator<Item = Panel> {
        self.location()
            .line(other.location())
            .map(move |location| self.on_same_layer(location))
    }
}
