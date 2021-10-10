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
	let mut current_player = 1; //Very basic counter to keep track of player turn (will be changed to something more powerful later on) - start with 1 since 0 will be drawn initially
	let mut player_text = "Player 1's Turn";
	let mut current_red  = 0;
	let mut current_green = 89;
	let mut current_blue = 178;
	let mut current_transparency = 250;

	let mut initial_banner_output = Instant::now();
	let banner_timeout = Duration::new(3,500);
	
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

		if keystate.contains(&Keycode::Backspace) && current_transparency == 0{ //Very basic next turn
			current_transparency = 250; //Restart transparency in order to display next banner
			if current_player == 0 {
				player_text = "Player 1's Turn";
				current_red  = 0;
				current_green = 89;
				current_blue = 178;
			} else if current_player == 1 {
				player_text = "Player 2's Turn";
				current_red  = 207;
				current_green = 21;
				current_blue = 24;
			} else {
				player_text = "Barbarians's Turn";
				current_red  = 163;
				current_green = 96;
				current_blue = 30;
				current_player = -1;
			}
			current_player += 1;
		}

		//As long as the banner won't be completely transparent, draw it
		if current_transparency != 0 {
			draw_player_banner(core, player_text, Color::RGBA(current_red, current_green, current_blue, current_transparency), Color::RGBA(0,0,0, current_transparency))?;
		}

		//The first time we draw the banner we need to keep track of when it first appears
		if current_transparency == 250 {
			initial_banner_output = Instant::now();
			current_transparency -= 10;
		}

		//After a set amount of seconds pass, start to make the banner disappear
		if Instant::now()-initial_banner_output >= banner_timeout && current_transparency != 0{
			current_transparency -= 10;
		}

		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}

fn draw_player_banner(core: &mut SDLCore, text: &str, rect_color: Color, text_color: Color) -> Result< (), String> {
	let bold_font = core.ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?;
	let banner_rect = centered_rect!(core, CAM_W, 128);

	core.wincan.set_blend_mode(BlendMode::Blend);
	core.wincan.set_draw_color(rect_color);
	core.wincan.draw_rect(banner_rect)?;
	core.wincan.fill_rect(banner_rect)?;
	
	let text_surface = bold_font.render(text)
			.blended_wrapped(text_color, 320) //Black font
			.map_err(|e| e.to_string())?;

	let text_texture = core.texture_creator.create_texture_from_surface(&text_surface)
		.map_err(|e| e.to_string())?;

	core.wincan.copy(&text_texture, None, centered_rect!(core, CAM_W/6, 128))?;
	
	Ok(())
}