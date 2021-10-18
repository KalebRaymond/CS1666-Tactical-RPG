use std::collections::HashSet;
use std::time::{Instant, Duration};

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
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
use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::{TILE_SIZE, CAM_W, CAM_H};

use crate::unit::{Team, Unit};

const BANNER_TIMEOUT: u64 = 2500;

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	let texture_creator = core.wincan.texture_creator();
	
	let mut current_player = Team::Player;
	let mut banner_key = "p1_banner";
	let mut current_banner_transparency = 250;
	let mut banner_colors = Color::RGBA(0, 89, 178, current_banner_transparency);

	let mut initial_banner_output = Instant::now();

	let mut banner_visible = true;

	//Load map from file
	let mut map_data = File::open("maps/map.txt").expect("Unable to open map file");
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

	//Load map textures
	let mut tile_textures: HashMap<&str, Texture> = HashMap::new();
	// Mountains
	tile_textures.insert("▉", texture_creator.load_texture("images/tiles/mountain_tile.png")?);
	tile_textures.insert("▒", texture_creator.load_texture("images/tiles/mountain2_tile.png")?);
	tile_textures.insert("▀", texture_creator.load_texture("images/tiles/mountain_side_top.png")?);
	tile_textures.insert("▐", texture_creator.load_texture("images/tiles/mountain_side_vertical_right.png")?);
	tile_textures.insert("▃", texture_creator.load_texture("images/tiles/mountain_side_bottom.png")?);
	tile_textures.insert("▍", texture_creator.load_texture("images/tiles/mountain_side_vertical_left.png")?);
	tile_textures.insert("▛", texture_creator.load_texture("images/tiles/mountain_top_left.png")?);
	tile_textures.insert("▜", texture_creator.load_texture("images/tiles/mountain_top_right.png")?);
	tile_textures.insert("▙", texture_creator.load_texture("images/tiles/mountain_bottom_left.png")?);
	tile_textures.insert("▟", texture_creator.load_texture("images/tiles/mountain_bottom_right.png")?);
	// Grass
	tile_textures.insert(" ", texture_creator.load_texture("images/tiles/grass_tile.png")?);
	// Rivers
	tile_textures.insert("=", texture_creator.load_texture("images/tiles/river_tile.png")?);
	tile_textures.insert("║", texture_creator.load_texture("images/tiles/river_vertical.png")?);
	tile_textures.insert("^", texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	tile_textures.insert("v", texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	tile_textures.insert(">", texture_creator.load_texture("images/tiles/river_end_right.png")?);
	tile_textures.insert("<", texture_creator.load_texture("images/tiles/river_end_left.png")?);
	// Bases
	tile_textures.insert("b", texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	tile_textures.insert("1", texture_creator.load_texture("images/tiles/red_castle.png")?);
	tile_textures.insert("2", texture_creator.load_texture("images/tiles/blue_castle.png")?);
	// Tree
	tile_textures.insert("t", texture_creator.load_texture("images/tiles/tree_tile.png")?);

	//Load unit textures
	let mut unit_textures: HashMap<&str, Texture> = HashMap::new();
	unit_textures.insert("p1m", texture_creator.load_texture("images/units/player1_melee.png")?);

	
	let mut text_textures: HashMap<&str, Texture> = HashMap::new();
	{
		let bold_font = core.ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?;
		text_textures.insert("p1_banner", {
			let text_surface = bold_font.render("Player 1's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("p2_banner", {
			let text_surface = bold_font.render("Player 2's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("b_banner", {
			let text_surface = bold_font.render("Barbarians' Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});
	}

	//Tried to get this to work with 2d vectors and Option(Unit) but it was not having the macro 
	let mut p1_units: HashMap<(u32, u32), Unit> = HashMap::new();
	p1_units.insert((0,0), Unit::new(0, 0, Team::Player, 10, 5, 2, 90, 5, unit_textures.get("p1m").unwrap()));
	p1_units.insert((3,3), Unit::new(3, 3, Team::Player, 10, 5, 2, 90, 5, unit_textures.get("p1m").unwrap()));
	p1_units.insert((4,5), Unit::new(4, 5, Team::Player, 10, 5, 2, 90, 5, unit_textures.get("p1m").unwrap()));	

	//Default mouse positions
	let mut old_mouse_x = -1;
	let mut old_mouse_y = -1;

	//Camera
	core.cam.w = (map_width as u32 * TILE_SIZE) as i32;
	core.cam.h = (map_height as u32 * TILE_SIZE) as i32;

	'gameloop: loop {
		core.wincan.clear();

		//Check if user tried to quit the program
		for event in core.event_pump.poll_iter() {
			match event {
				Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => break 'gameloop,
				_ => {},
			}
		}

		//Mouse Controls
		let mouse_state: MouseState = core.event_pump.mouse_state();
		//Check right mouse button. Camera controls should stay enabled even when it is not the player's turn
		if mouse_state.right() && !banner_visible{
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

		//Record key inputs
		let keystate: HashSet<Keycode> = core.event_pump
		.keyboard_state()
		.pressed_scancodes()
		.filter_map(Keycode::from_scancode)
		.collect();

		//Handle the current team's move
		match current_player {
			Team::Player => {
				if !banner_visible {
					if keystate.contains(&Keycode::Backspace) {
						//End turn
						current_player = Team::Enemy;

						//Start displaying the enemy's banner
						current_banner_transparency = 250;
						banner_colors = Color::RGBA(207, 21, 24, current_banner_transparency);
						banner_key = "p2_banner";
						banner_visible = true;
					}
				}
			},
			Team::Enemy => {
				if !banner_visible {
					//End turn
				current_player = Team::Barbarians;

				//Start displaying the barbarians' banner
				current_banner_transparency = 250;
				banner_colors = Color::RGBA(163,96,30, current_banner_transparency);
				banner_key = "b_banner";
				banner_visible = true;
				}
			},
			Team::Barbarians => {
				if !banner_visible {
					//End turn
					current_player = Team::Player;

					//Start displaying Player 1's banner
					current_banner_transparency = 250;
					banner_colors = Color::RGBA(0, 89, 178, current_banner_transparency);
					banner_key = "p1_banner";
					banner_visible = true;
				}
			},
		}
		
		//Draw tiles & sprites
		for i in 0..map_height {
			for j in 0..map_width {
				let map_tile = map_tiles[i][j].as_ref();
				let map_tile_size = match map_tile {
					"b" => TILE_SIZE * 2,
					_ => TILE_SIZE,
				};

				let pixel_location = PixelCoordinates::from_matrix_indices(i as u32, j as u32);
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, map_tile_size, map_tile_size);

				//Draw map tile at this coordinate
				if let std::collections::hash_map::Entry::Occupied(entry) = tile_textures.entry(map_tile) {
					core.wincan.copy(&entry.get(), None, dest)?
				}
				//Draw unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let std::collections::hash_map::Entry::Occupied(entry) = p1_units.entry((j as u32, i as u32)) {
					core.wincan.copy(entry.get().texture, None, dest)?
				}
			}
		}

		//Draw banner that appears at beginning of turn
		{
			//As long as the banner isn't completely transparent, draw it
			if current_banner_transparency != 0 {
				banner_colors.a = current_banner_transparency;
				draw_player_banner(core, &text_textures, banner_key, banner_colors)?;
			} else if banner_visible {
				banner_visible = false;
			}

			//The first time we draw the banner we need to keep track of when it first appears
			if current_banner_transparency == 250 {
				initial_banner_output = Instant::now();
				current_banner_transparency -= 25;
			}

			//After a set amount of seconds pass and if the banner is still visible, start to make the banner disappear
			if initial_banner_output.elapsed() >= Duration::from_millis(BANNER_TIMEOUT) && current_banner_transparency != 0 {
				current_banner_transparency -= 25;
			}
		}
		
		core.wincan.set_viewport(core.cam);
		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}

fn draw_player_banner(core: &mut SDLCore, text_textures: &HashMap<&str, Texture>, text_index: &str, rect_color: Color) -> Result< (), String> {
	let banner_rect = Rect::new(core.cam.x.abs(), core.cam.y.abs() + (360-64), CAM_W, 128);
	let text_rect = Rect::new(core.cam.x.abs() + (640-107), core.cam.y.abs() + (360-64), CAM_W/6, 128);
	core.wincan.set_blend_mode(BlendMode::Blend);
	core.wincan.set_draw_color(rect_color);
	core.wincan.draw_rect(banner_rect)?;
	core.wincan.fill_rect(banner_rect)?;

	match text_textures.get(text_index) {
		Some(texture) => core.wincan.copy(&texture, None, text_rect)?,
		None => {},
	};
	
	Ok(())
}