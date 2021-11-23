use sdl2::render::Texture;

use std::collections::HashMap;

use crate::damage_indicator::DamageIndicator;
use crate::tile::Tile;

pub struct GameMap<'a> {
    pub map_tiles: HashMap<(u32, u32), Tile>,
    pub map_textures: HashMap<(u32, u32), &'a Texture<'a>>,

    pub possible_moves: Vec<(u32, u32)>,
	pub possible_attacks: Vec<(u32, u32)>,
	pub actual_attacks: Vec<(u32, u32)>,

    //Holds all damage indicators (the numbers that appear above a unit when attacked) that are visible
	pub damage_indicators: Vec<DamageIndicator<'a>>,
}

impl GameMap<'_> {
    pub fn new<'a>() -> GameMap<'a> {
        GameMap {
            map_tiles: HashMap::new(),
            map_textures: HashMap::new(),
            possible_moves: Vec::new(),
	        possible_attacks: Vec::new(),
	        actual_attacks: Vec::new(),
            damage_indicators: Vec::new(),
        }
    }
}