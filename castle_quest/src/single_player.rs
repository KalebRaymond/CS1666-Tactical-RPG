use std::collections::HashSet;
use std::time::{Instant, Duration};

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use crate::GameState;
use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::TILE_SIZE;
use crate::CAM_W;
use crate::CAM_H;

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	let mut current_player = 0; //Very basic counter to keep track of player turn (will be changed to something more powerful later on)
	let mut current_transparency = 250;
	let mut initial_banner_output = Instant::now();
	let banner_timeout = Duration::new(5,0);
	
	//Basic mock map, 48x48 2d vector filled with 1s
	let mut map: Vec<Vec<u32>> = vec![vec![1; 48]; 48];
	let map_width = map[0].len();
	let map_height = map.len();

	//Mock units maps for testing
	let mut units: Vec<Vec<u32>> = vec![vec![0; map_width]; map_height];
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

		let keystate: HashSet<Keycode> = core.event_pump
				.keyboard_state()
				.pressed_scancodes()
				.filter_map(Keycode::from_scancode)
				.collect();
		
		//Draw tiles & sprites
		for i in 0..map_height {
			for j in 0..map_width {
				let pixel_location = PixelCoordinates::from_matrix_indices(i as u32, j as u32);
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);

				//Draw map tile at this coordinate
				let map_tile_texture: Option<Texture> = match map[i][j] {
					1 => Some(core.texture_creator.load_texture("images/grass_tile.png")?),
					_ => None,
				};

				match map_tile_texture {
					Some(texture) => core.wincan.copy(&texture, None, dest)?,
					None => {},
				};

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

		if keystate.contains(&Keycode::Backspace) {
			if current_player == 0 {
				println!("Player 1");
			} else if current_player == 1 {
				println!("Player 2");
			} else {
				println!("Barbarians");
				current_player = -1;
			}
			current_player += 1;
		}

		//As long as the banner won't be completely transparent, draw it
		if current_transparency != 0 {
			draw_player_banner(core, "hi", Color::RGBA(255, 0, 0, current_transparency), Color::RGBA(255,255,255, current_transparency))?;
		}

		//The first time we draw the banner we need to keep track of when it first appears
		if current_transparency == 250 {
			initial_banner_output = Instant::now();
			current_transparency -= 10;
		}
		//After a set amount of seconds pass, start to make the banner disappear
		if Instant::now()-initial_banner_output >= banner_timeout && current_transparency != 0{
			println!("Time has passed! {}", current_transparency);
			current_transparency -= 10;
		}

		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}

fn draw_player_banner(core: &mut SDLCore, text: &str, rect_color: Color, text_color: Color) -> Result< (), String> {
	let banner_rect = centered_rect!(core, CAM_W, 128);

	core.wincan.set_blend_mode(BlendMode::Blend);
	core.wincan.set_draw_color(rect_color);
	core.wincan.draw_rect(banner_rect)?;
	core.wincan.fill_rect(banner_rect)?;
	
	Ok(())
}