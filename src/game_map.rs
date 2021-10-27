use std::collections::HashMap;

use crate::tile::Tile;

pub struct GameMap<'a> {
    pub map_tiles: HashMap<(u32, u32), Tile<'a>>,

    pub possible_moves: Vec<(u32, u32)>,
	pub possible_attacks: Vec<(u32, u32)>,
	pub actual_attacks: Vec<(u32, u32)>,
}

impl GameMap<'_> {
    pub fn new<'a>() -> GameMap<'a> {
        GameMap {
            map_tiles: HashMap::new(),
            possible_moves: Vec::new(),
	        possible_attacks: Vec::new(),
	        actual_attacks: Vec::new(),
        }
    }
}