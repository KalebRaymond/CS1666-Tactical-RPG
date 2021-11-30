use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryInto;

use sdl2::video::WindowContext;
use sdl2::render::{Texture, TextureCreator};
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;

use crate::cursor::Cursor;
use crate::input::Input;
use crate::banner::Banner;
use crate::damage_indicator::DamageIndicator;
use crate::tile::{Tile, Structure};
use crate::unit::{Team, Unit};
use crate::pixel_coordinates::PixelCoordinates;
use crate::TILE_SIZE;
use crate::SDLCore;

pub struct GameMap<'a> {
	pub map_tiles: HashMap<(u32, u32), Tile<'a>>,
	pub map_size: (usize, usize),

	//Stuff for enemy AI calculations
	pub pos_player_castle: (u32, u32),
	pub pos_enemy_castle: (u32, u32),
	pub pos_barbarian_camps: Vec<(u32, u32)>,

	pub player_units: HashMap<(u32, u32), Unit<'a>>,
	pub enemy_units: HashMap<(u32, u32), Unit<'a>>,
	pub barbarian_units: HashMap<(u32, u32), Unit<'a>>,

	pub possible_moves: Vec<(u32, u32)>,
	pub possible_attacks: Vec<(u32, u32)>,
	pub actual_attacks: Vec<(u32, u32)>,

	//Holds all damage indicators (the numbers that appear above a unit when attacked) that are visible
	pub damage_indicators: Vec<DamageIndicator<'a>>,

	// various UI elements
	pub banner: Banner,
	pub cursor: Cursor<'a>,
}

impl GameMap<'_> {
	pub fn new<'a>(textures: &'a HashMap<&str, Texture<'a>>) -> GameMap<'a> {
		//Load map from file
		let map_data = File::open("maps/map.txt").expect("Unable to open map file");
		let mut map_data = BufReader::new(map_data);
		let mut line = String::new();

		//Sets size of the map from the first line of the text file
		map_data.read_line(&mut line).unwrap();
		let map_width: usize = line.trim().parse().unwrap();
		let map_height: usize = line.trim().parse().unwrap();

		//Creates map from file
		let map_string: Vec<Vec<String>> = map_data.lines()
			.take(map_width)
			.map(|x| x.unwrap().chars().collect::<Vec<char>>())
			.map(|x| x.chunks(2).map(|chunk| chunk[0].to_string()).collect())
			.collect();

		let mut map: GameMap<'a> = GameMap {
			map_tiles: HashMap::new(),
			map_size: (map_width, map_height),
			pos_player_castle: (0, 0),
			pos_enemy_castle: (0, 0),
			pos_barbarian_camps: Vec::new(),
			player_units: HashMap::new(),
			enemy_units: HashMap::new(),
			barbarian_units: HashMap::new(),
			possible_moves: Vec::new(),
			possible_attacks: Vec::new(),
			actual_attacks: Vec::new(),
			damage_indicators: Vec::new(),
			banner: Banner::new(),
			cursor: Cursor::new(textures.get("cursor").unwrap())
		};

		//Set up the HashMap of Tiles that can be interacted with
		let mut x = 0;
		let mut y = 0;
		for row in map_string.iter() {
			for col in row.iter() {
				let letter = &col[..];
				let texture = textures.get(letter).unwrap();
				match letter {
					"║" | "^" | "v" | "<" | "=" | ">" | "t" => map.map_tiles.insert((x,y), Tile::new(x, y, false, true, None, None, texture)),
					" " => map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, None, texture)),
					"b" =>  {
						map.pos_barbarian_camps.push((y,x));
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), texture))
					},
					"_" => map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), texture)),
					"1" =>  {
						map.pos_player_castle = (y, x);
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::PCastle), texture))
					},
					"2" =>  {
						map.pos_enemy_castle = (y, x);
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::ECastle), texture))
					},
					_ => map.map_tiles.insert((x,y), Tile::new(x, y, false, false, None, None, texture)),
				};
				y += 1;
			}
			x += 1;
			y = 0;
		}

		map
	}

	pub fn draw(&mut self, core: &mut SDLCore) {
		//Camera controls should stay enabled even when it is not the player's turn,
		//which is why this code block is not in player_turn.rs
		if core.input.right_held && !self.banner.banner_visible {
			let max_move = TILE_SIZE as i32;
			core.cam.x = (core.cam.x - (core.input.mouse_x_old - core.input.mouse_x).clamp(-max_move, max_move)).clamp(-core.cam.w + core.wincan.window().size().0 as i32, 0);
			core.cam.y = (core.cam.y - (core.input.mouse_y_old - core.input.mouse_y).clamp(-max_move, max_move)).clamp(-core.cam.h + core.wincan.window().size().1 as i32, 0);
		}

		let (i, j) = PixelCoordinates::matrix_indices_from_pixel(
            core.input.mouse_x.try_into().unwrap(),
            core.input.mouse_y.try_into().unwrap(),
            (-1 * core.cam.x).try_into().unwrap(),
            (-1 * core.cam.y).try_into().unwrap()
        );

		match self.player_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				self.cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {
				self.cursor.hide_cursor();
			},
		}
		match self.enemy_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				self.cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {},
		}
		match self.barbarian_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				self.cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {},
		}

		//Draw tiles & sprites
		for x in 0..self.map_size.0 {
			for y in 0..self.map_size.1 {
				let map_tile = self.map_tiles.get(&(y as u32, x as u32));
				let map_tile_size = match map_tile {
					Some(Tile{contained_structure: Some(Structure::Camp), ..}) => TILE_SIZE * 2,
					_ => TILE_SIZE,
				};

				let pixel_location = PixelCoordinates::from_matrix_indices(y as u32, x as u32);
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, map_tile_size, map_tile_size);

				//Draw map tile at this coordinate
				if let Some(map_tile) = self.map_tiles.get(&(y as u32, x as u32)) {
					core.wincan.copy(map_tile.texture, None, dest);
				}

				//Use default sprite size for all non-map sprites
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);

				//Draw player unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(mut unit) = self.player_units.get_mut(&(x as u32, y as u32)) {
					unit.draw(core, &dest);
				}

				//Draw enemy unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(mut enemy) = self.enemy_units.get_mut(&(x as u32, y as u32)) {
					enemy.draw(core, &dest);
				}

				//Draw barbarian unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(mut barbarian) = self.barbarian_units.get_mut(&(x as u32, y as u32)) {
					barbarian.draw(core, &dest);
				}
			}
		}

		// draw UI/banners
		self.cursor.draw(core);
		self.banner.draw(core);
	}
}

//Load map textures
pub fn load_textures<'r>(textures: &mut HashMap<&str, Texture<'r>>, texture_creator: &'r TextureCreator<WindowContext>) -> Result<(), String> {
	//Mountains
	textures.insert("▉", texture_creator.load_texture("images/tiles/mountain_tile.png")?);
	textures.insert("▒", texture_creator.load_texture("images/tiles/mountain2_tile.png")?);
	textures.insert("▀", texture_creator.load_texture("images/tiles/mountain_side_top.png")?);
	textures.insert("▐", texture_creator.load_texture("images/tiles/mountain_side_vertical_right.png")?);
	textures.insert("▃", texture_creator.load_texture("images/tiles/mountain_side_bottom.png")?);
	textures.insert("▍", texture_creator.load_texture("images/tiles/mountain_side_vertical_left.png")?);
	textures.insert("▛", texture_creator.load_texture("images/tiles/mountain_top_left.png")?);
	textures.insert("▜", texture_creator.load_texture("images/tiles/mountain_top_right.png")?);
	textures.insert("▙", texture_creator.load_texture("images/tiles/mountain_bottom_left.png")?);
	textures.insert("▟", texture_creator.load_texture("images/tiles/mountain_bottom_right.png")?);
	//Grass
	textures.insert(" ", texture_creator.load_texture("images/tiles/grass_tile.png")?);
	textures.insert("_", texture_creator.load_texture("images/tiles/empty_tile.png")?);
	//Rivers
	textures.insert("=", texture_creator.load_texture("images/tiles/river_tile.png")?);
	textures.insert("║", texture_creator.load_texture("images/tiles/river_vertical.png")?);
	textures.insert("^", texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	textures.insert("v", texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	textures.insert(">", texture_creator.load_texture("images/tiles/river_end_right.png")?);
	textures.insert("<", texture_creator.load_texture("images/tiles/river_end_left.png")?);
	//Bases
	textures.insert("b", texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	textures.insert("1", texture_creator.load_texture("images/tiles/blue_castle.png")?);
	textures.insert("2", texture_creator.load_texture("images/tiles/red_castle.png")?);
	//Tree
	textures.insert("t", texture_creator.load_texture("images/tiles/tree_tile.png")?);

	//Load unit textures
	textures.insert("pll", texture_creator.load_texture("images/units/player1_melee.png")?);
	textures.insert("plr", texture_creator.load_texture("images/units/player1_archer.png")?);
	textures.insert("plm", texture_creator.load_texture("images/units/player1_mage.png")?);
	textures.insert("pl2l", texture_creator.load_texture("images/units/player2_melee.png")?);
	textures.insert("pl2r", texture_creator.load_texture("images/units/player2_archer.png")?);
	textures.insert("pl2m", texture_creator.load_texture("images/units/player2_mage.png")?);
	textures.insert("bl", texture_creator.load_texture("images/units/barbarian_melee.png")?);
	textures.insert("br", texture_creator.load_texture("images/units/barbarian_archer.png")?);

	//Load UI textures
	textures.insert("cursor", texture_creator.load_texture("images/interface/cursor.png")?);

	Ok(())
}
