use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;

// For accessing map file and reading lines
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryInto;
use std::collections::HashMap;


use crate::GameState;
use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::TILE_SIZE;

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	//Basic mock map, 48x48 2d vector filled with 1's
	//let mut map: Vec<Vec<&str>> = vec![vec![" "; 64]; 64];
	//let map_width = map[0].len();
	//let map_height = map.len();

	let mut map_data = File::open("src/maps/map.txt").expect("Unable to open map file");
	let mut map_data = BufReader::new(map_data);
	let mut line = String::new();

	map_data.read_line(&mut line).unwrap();
	let map_width: usize = line.trim().parse().unwrap();
	let map_height: usize = line.trim().parse().unwrap();

	let map: Vec<Vec<String>> = map_data.lines()
		.take(map_width)
		.map(|x| x.unwrap().chars().collect::<Vec<char>>())
		.map(|x| x.chunks(2).map(|chunk| chunk[0].to_string()).collect())
		.collect();

	let mut textures: HashMap<&str, Texture> = HashMap::new();
	// Mountains
	textures.insert("▉", core.texture_creator.load_texture("images/tiles/mountain_tile.png")?);
	textures.insert("▒", core.texture_creator.load_texture("images/tiles/mountain2_tile.png")?);
	textures.insert("▀", core.texture_creator.load_texture("images/tiles/mountain_side_top.png")?);
	textures.insert("▐", core.texture_creator.load_texture("images/tiles/mountain_side_vertical_right.png")?);
	textures.insert("▃", core.texture_creator.load_texture("images/tiles/mountain_side_bottom.png")?);
	textures.insert("▍", core.texture_creator.load_texture("images/tiles/mountain_side_vertical_left.png")?);
	textures.insert("▛", core.texture_creator.load_texture("images/tiles/mountain_top_left.png")?);
	textures.insert("▜", core.texture_creator.load_texture("images/tiles/mountain_top_right.png")?);
	textures.insert("▙", core.texture_creator.load_texture("images/tiles/mountain_bottom_left.png")?);
	textures.insert("▟", core.texture_creator.load_texture("images/tiles/mountain_bottom_right.png")?);
	// Grass
	textures.insert(" ", core.texture_creator.load_texture("images/tiles/grass_tile.png")?);
	// Rivers
	textures.insert("=", core.texture_creator.load_texture("images/tiles/river_tile.png")?);
	textures.insert("║", core.texture_creator.load_texture("images/tiles/river_vertical.png")?);
	textures.insert("^", core.texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	textures.insert("v", core.texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	textures.insert(">", core.texture_creator.load_texture("images/tiles/river_end_right.png")?);
	textures.insert("<", core.texture_creator.load_texture("images/tiles/river_end_left.png")?);
	// Bases
	textures.insert("b", core.texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	textures.insert("1", core.texture_creator.load_texture("images/tiles/red_castle.png")?);
	textures.insert("2", core.texture_creator.load_texture("images/tiles/blue_castle.png")?);
	// Tree
	textures.insert("t", core.texture_creator.load_texture("images/tiles/tree_tile.png")?);

	//Mock units map for testing
	let mut units: Vec<Vec<u32>> = vec![vec![0; map_width]; map_height];
	units[0][0] = 1;
	units[3][3] = 1;
	units[4][5] = 1;

	'gameloop: loop {
		core.wincan.clear();

		for event in core.event_pump.poll_iter() {
			match event {
				Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => break 'gameloop,
				_ => {},
			}
		}

		//Draw tiles & sprites
		for i in 0..map_height {
			for j in 0..map_width {
				let map_tile = map[i][j].as_ref();
				let map_tile_size = match map_tile {
					"b" => TILE_SIZE * 2,
					_ => TILE_SIZE,
				};

				let pixel_location = PixelCoordinates::from_matrix_indices(i as u32, j as u32);
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, map_tile_size, map_tile_size);

				//Draw map tile at this coordinate
				if let std::collections::hash_map::Entry::Occupied(entry) = textures.entry(map_tile) {
					core.wincan.copy(&entry.get(), None, dest)?
				}

				//Draw unit at this coordinate if there is one
				let unit_texture: Option<Texture> = match units[i][j] {
					1 => Some(core.texture_creator.load_texture("images/player1_melee.png")?),
					_ => None,
				};

				match unit_texture {
					Some(texture) => core.wincan.copy(&texture, None, dest)?,
					None => {},
				};
			}
		}

		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}