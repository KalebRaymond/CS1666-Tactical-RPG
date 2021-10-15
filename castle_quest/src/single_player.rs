use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseState;
use sdl2::rect::Rect;
use sdl2::render::Texture;

//For accessing map file and reading lines
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::GameState;
use crate::level_map::LevelMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::TILE_SIZE;

pub struct GameplayState {
	/* Empty for now, but will hold all the flags and booleans
	 * used to execute different conditions during single player
	 * or multiplayer
	 * 
	 * For example, will be useful when the user clicks on a unit 
	 * to move it somewhere. This struct will have a "moving_unit"
	 * boolean. If it is false, we can check to skip over the logic 
	 * in 'gameloop for moving a unit
	 *
	 * Something like that.
	 */
}

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	//Load map layout from txt file
	let mut map_data = File::open("src/maps/map.txt").expect("Unable to open map file");
	let mut map_data = BufReader::new(map_data);
	let mut line = String::new();

	//Sets size of the map from the first line of the text file
	map_data.read_line(&mut line).unwrap();
	let map_width: usize = line.trim().parse().unwrap();
	let map_height: usize = line.trim().parse().unwrap();

	//Creates map from file
	let map_tiles: Vec<Vec<String>> = map_data.lines()
		.take(map_width)
		.map(|x| x.unwrap().chars().collect::<Vec<char>>())
		.map(|x| x.chunks(2).map(|chunk| chunk[0].to_string()).collect())
		.collect();

	let mut textures: HashMap<&str, Texture> = HashMap::new();
	//Mountains
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
	//Grass
	textures.insert(" ", core.texture_creator.load_texture("images/tiles/grass_tile.png")?);
	//Rivers
	textures.insert("=", core.texture_creator.load_texture("images/tiles/river_tile.png")?);
	textures.insert("║", core.texture_creator.load_texture("images/tiles/river_vertical.png")?);
	textures.insert("^", core.texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	textures.insert("v", core.texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	textures.insert(">", core.texture_creator.load_texture("images/tiles/river_end_right.png")?);
	textures.insert("<", core.texture_creator.load_texture("images/tiles/river_end_left.png")?);
	//Bases
	textures.insert("b", core.texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	textures.insert("1", core.texture_creator.load_texture("images/tiles/red_castle.png")?);
	textures.insert("2", core.texture_creator.load_texture("images/tiles/blue_castle.png")?);
	//Tree
	textures.insert("t", core.texture_creator.load_texture("images/tiles/tree_tile.png")?);

	//Mock units map for testing
	let mut units: Vec<Vec<u32>> = vec![vec![0; map_width]; map_height];
	units[0][0] = 1;
	units[3][3] = 1;
	units[4][5] = 1;

	//Put all of the map matrices into one struct
	let mut level_map = LevelMap {
		map_tiles: map_tiles,
		units: units,
	};

	//Default mouse positions
	let mut old_mouse_x = -1;
	let mut old_mouse_y = -1;

	//Camera
	core.cam.w = (map_width as u32 * TILE_SIZE) as i32;
	core.cam.h = (map_height as u32 * TILE_SIZE) as i32;

	'gameloop: loop {
		core.wincan.clear();

		for event in core.event_pump.poll_iter() {
			match event {
				Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => break 'gameloop,
				_ => {},
			}
		}

		//Mouse Controls
		let mouse_state: MouseState = core.event_pump.mouse_state();
		if mouse_state.right() {
			if old_mouse_x < 0 || old_mouse_y < 0 {
				old_mouse_x = mouse_state.x();
				old_mouse_y = mouse_state.y();
			}
			core.cam.x = (core.cam.x - (old_mouse_x - mouse_state.x())).clamp(-core.cam.w + core.wincan.window().size().0 as i32, 0);
			core.cam.y = (core.cam.y - (old_mouse_y - mouse_state.y())).clamp(-core.cam.h + core.wincan.window().size().1 as i32, 0,);
			
			old_mouse_x = mouse_state.x();
			old_mouse_y = mouse_state.y();
		}
		else {
			old_mouse_y = -1;
			old_mouse_x = -1;
		}

		//Draw tiles & sprites
		for i in 0..map_height {
			for j in 0..map_width {
				let map_tile = level_map.map_tiles[i][j].as_ref();
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
				let unit_texture: Option<Texture> = match level_map.units[i][j] {
					1 => Some(core.texture_creator.load_texture("images/player1_melee.png")?),
					_ => None,
				};

				match unit_texture {
					Some(texture) => core.wincan.copy(&texture, None, dest)?,
					None => {},
				};
			}
		}

		core.wincan.set_viewport(core.cam);
		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}