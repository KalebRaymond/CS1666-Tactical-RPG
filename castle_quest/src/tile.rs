use sdl2::render::Texture;
use std::fmt;
use crate::unit::{Unit};

pub struct Tile<'a> {
    pub x: u32,
    pub y: u32,
    pub is_traversable: bool,
    pub contained_unit: Option<&'a Unit<'a>>,
    pub texture: &'a Texture<'a>,
}

impl Tile <'_>{
    pub fn new<'a> (x:u32, y:u32, is_traversable: bool, contained_unit: Option<&'a Unit>, texture: &'a Texture) -> Tile<'a> {
        Tile {
            x,
            y,
            is_traversable,
            contained_unit,
            texture,
        }
    }
}

impl fmt::Display for Tile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tile(x:{}, y:{}, is_traversable:{})", self.x, self.y, self.is_traversable)
    }
}