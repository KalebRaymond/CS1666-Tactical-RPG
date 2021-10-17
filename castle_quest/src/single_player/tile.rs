use sdl2::render::Texture;
use std::fmt;

pub struct Tile<'a> {
    pub x: u32,
    pub y: u32,
    pub is_traversable: bool,
    pub contains_unit: bool,
    pub texture: &'a Texture<'a>,
}

impl Tile <'_>{
    pub fn new<'a> (x:u32, y:u32, is_traversable: bool, contains_unit: bool, texture: &'a Texture) -> Tile<'a> {
        Tile {
            x,
            y,
            is_traversable,
            contains_unit,
            texture,
        }
    }
}

impl fmt::Display for Tile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tile(x:{}, y:{}, is_traversable:{}, contains_unit:{})", self.x, self.y, self.is_traversable, self.contains_unit)
    }
}