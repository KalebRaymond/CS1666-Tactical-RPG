use std::collections::HashSet;
use std::time::{Instant, Duration};

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::mouse::MouseState;

// For accessing map file and reading lines
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryInto;
use std::collections::HashMap;


use crate::GameState;
use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::TILE_SIZE;
use crate::CAM_W;
use crate::CAM_H;

const BANNER_TIMEOUT: u64 = 2500;

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	let texture_creator = core.wincan.texture_creator();
	
	let mut current_player = 1; //Very basic counter to keep track of player turn (will be changed to something more powerful later on) - start with 1 since 0 will be drawn initially
	let mut banner_key = "p1_banner";
	let mut current_transparency = 250;
	let mut banner_colors = Color::RGBA(0, 89, 178, current_transparency);

	let mut initial_banner_output = Instant::now();

	let mut ui_visible = true;
	
	//Basic mock map, 48x48 2d vector filled with 1s
	/*
	let mut map: Vec<Vec<u32>> = vec![vec![1; 48]; 48];
	let map_width = map[0].len();
	let map_height = map.len();
	*/

	let mut map_data = File::open("src/maps/map.txt").expect("Unable to open map file");
	let mut map_data = BufReader::new(map_data);
	let mut line = String::new();

	// Sets size of the map from the first line of the text file
	map_data.read_line(&mut line).unwrap();
	let map_width: usize = line.trim().parse().unwrap();
	let map_height: usize = line.trim().parse().unwrap();
	core.cam.w = (map_width as u32 * TILE_SIZE) as i32;
	core.cam.h = (map_height as u32 * TILE_SIZE) as i32;

	// Previous mouse positions
	let mut old_mouse_x = -1;
	let mut old_mouse_y = -1;

	// Creates map from file
	let map: Vec<Vec<String>> = map_data.lines()
		.take(map_width)
		.map(|x| x.unwrap().chars().collect::<Vec<char>>())
		.map(|x| x.chunks(2).map(|chunk| chunk[0].to_string()).collect())
		.collect();

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

	let mut unit_textures: HashMap<&str, Texture> = HashMap::new();
	unit_textures.insert("p1m", texture_creator.load_texture("images/player1_melee.png")?);

	
	let mut text_textures: HashMap<&str, Texture> = HashMap::new();
	{
		let bold_font = core.ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?;
		text_textures.insert("p1_banner", {
			let text_surface = bold_font.render("Player 1's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("p2_banner", {
			let text_surface = bold_font.render("Player 2's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("b_banner", {
			let text_surface = bold_font.render("Barbarians' Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});
	}
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

		//Mouse Controls
		let mouse_state: MouseState = core.event_pump.mouse_state();
		if mouse_state.right() && !ui_visible{
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
				if let std::collections::hash_map::Entry::Occupied(entry) = tile_textures.entry(map_tile) {
					core.wincan.copy(&entry.get(), None, dest)?
				}

				//Draw unit at this coordinate if there is one
				let unit_texture: Option<&Texture<'_>> = match units[i][j] {
					1 => unit_textures.get("p1m"),
					_ => None,
				};

				match unit_texture {
					Some(texture) => core.wincan.copy(&texture, None, dest)?,
					None => {},
				};
			}
		}

		//Draw the banner that appears at the beginning of a turn on top of the map tiles
		if keystate.contains(&Keycode::Backspace) && current_transparency == 0{ //Very basic next turn
			current_transparency = 250; //Restart transparency in order to display next banner
			ui_visible = true;
			if current_player == 0 {
				banner_key = "p1_banner";
				banner_colors = Color::RGBA(0, 89, 178, current_transparency);
			} else if current_player == 1 {
				banner_key = "p2_banner";
				banner_colors = Color::RGBA(207, 21, 24, current_transparency);
			} else {
				banner_key = "b_banner";
				banner_colors = Color::RGBA(163,96,30, current_transparency);
				current_player = -1;
			}
			current_player += 1;
		}

		//As long as the banner won't be completely transparent, draw it
		if current_transparency != 0 {
			banner_colors.a = current_transparency;
			draw_player_banner(core, &text_textures, banner_key, banner_colors, Color::RGBA(0,0,0, current_transparency))?;
		} else if ui_visible {
			ui_visible = false;
		}

		//The first time we draw the banner we need to keep track of when it first appears
		if current_transparency == 250 {
			initial_banner_output = Instant::now();
			current_transparency -= 25;
		}

		//After a set amount of seconds pass and if the banner is still visible, start to make the banner disappear
		if initial_banner_output.elapsed() >= Duration::from_millis(BANNER_TIMEOUT) && current_transparency != 0 {
			current_transparency -= 25;
		}
		
		core.wincan.set_viewport(core.cam);
		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}

fn draw_player_banner(core: &mut SDLCore, text_textures: &HashMap<&str, Texture>, text_index: &str, rect_color: Color, text_color: Color) -> Result< (), String> {
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