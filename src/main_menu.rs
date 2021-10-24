use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::mouse::MouseState;
use sdl2::image::LoadTexture;
use std::time::Instant;

use crate::GameState;
use crate::SDLCore;

use std::path::Path;
use sdl2::mixer::{InitFlag, AUDIO_S32SYS, DEFAULT_CHANNELS};

pub fn main_menu(core: &mut SDLCore) -> Result<GameState, String> {
    let texture_creator = core.wincan.texture_creator();

	let bold_font = core.ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?; //From https://www.fontsquirrel.com/fonts/open-sans
	let regular_font = core.ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 16)?; //From https://www.fontsquirrel.com/fonts/open-sans

	let mut next_game_state = GameState::SinglePlayer;
	//Single player button
	let single_player_button = Rect::new(40, 600, 380, 100);
	let text_surface = bold_font.render("Single Player")
		.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320) //White font
		.map_err(|e| e.to_string())?;

	let text_texture = texture_creator.create_texture_from_surface(&text_surface)
		.map_err(|e| e.to_string())?;

	//Multiplayer button
	let multiplayer_button = Rect::new(450, 600, 380, 100);
	let text_surface_multi = bold_font.render("Multiplayer")
		.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320) //White font
		.map_err(|e| e.to_string())?;

	let text_texture_multi = texture_creator.create_texture_from_surface(&text_surface_multi)
		.map_err(|e| e.to_string())?;

	//Credit button
	let credit_button = Rect::new(860, 600, 380, 100);
	let text_surface2 = bold_font.render("Credits")
		.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320) //White font
		.map_err(|e| e.to_string())?;

	let text_texture2 = texture_creator.create_texture_from_surface(&text_surface2)
		.map_err(|e| e.to_string())?;


	// Multiplayer menu
	let mut multiplayer_selected = false;

	let multiplayer_create_rect = centered_rect!(core, _, 200, 400, 100);
	let multiplayer_create_text = texture_creator.create_texture_from_surface(
		&bold_font.render("Create Room")
			.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320)
			.map_err(|e| e.to_string())?
	).map_err(|e| e.to_string())?;

	let multiplayer_join_rect = centered_rect!(core, _, 480, 400, 80);
	let multiplayer_join_text = texture_creator.create_texture_from_surface(
		&bold_font.render("Join Room")
			.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320)
			.map_err(|e| e.to_string())?
	).map_err(|e| e.to_string())?;

	//Join code textbox
	let join_code_textbox = centered_rect!(core, _, 400, 400, 60);
	let mut join_code = String::from("");
	let mut textbox_selected = false;
	let mut textbox_select_time = Instant::now();

	sdl2::mixer::open_audio(44100, AUDIO_S32SYS, DEFAULT_CHANNELS, 1024)?;
	let _mixer_filetypes = sdl2::mixer::init(InitFlag::MP3)?;
	let music = sdl2::mixer::Music::from_file(Path::new("./music/main_menu.mp3"))?;

	music.play(-1);

	//For animation
	let mut i = 0;

	'menuloop: loop {
		let mouse_state: MouseState = core.event_pump.mouse_state();
		let mouse_pos = (mouse_state.x(), mouse_state.y());

		if mouse_state.left() && multiplayer_selected {
			if join_code_textbox.contains_point(mouse_pos) {
				textbox_selected = true;
				textbox_select_time = Instant::now();
			} else if multiplayer_create_rect.contains_point(mouse_pos) {
				println!("TODO: create multiplayer room");
				multiplayer_selected = false;
			} else if multiplayer_join_rect.contains_point(mouse_pos) {
				println!("TODO: join multiplayer room");
				multiplayer_selected = false;
			} else {
				textbox_selected = false;
				multiplayer_selected = false;
			}
		}

		if mouse_state.left() && !multiplayer_selected {
			if single_player_button.contains_point(mouse_pos) {
				return Ok(GameState::SinglePlayer);
			} else if multiplayer_button.contains_point(mouse_pos) {
				multiplayer_selected = true;
			} else if credit_button.contains_point(mouse_pos) {
				return Ok(GameState::Credits);
			}
		}

		for event in core.event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit{..} | sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Escape), ..} => {
					break 'menuloop;
				},
				sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Backspace), ..} => {
					if textbox_selected && join_code.chars().count() > 0 {
						let mut char_iter = join_code.chars();
						char_iter.next_back();
						join_code = char_iter.as_str().to_string();
					}
				}
				sdl2::event::Event::KeyDown{keycode: Some(key), ..} => {
					let parsed_key = key.to_string();
					if textbox_selected && join_code.chars().count() < 4 && parsed_key.chars().count() == 1 && parsed_key.chars().next().unwrap().is_numeric() {
						join_code.push_str(&key.to_string());
					}
				},
				_ => {},
			}
		}
		i +=1;
		if i < 24 {
			sleep_poll!(core, 40);
			let fr = format!("images/main_menu_animation/{}.png", i);
			let mm_frame = texture_creator.load_texture(fr)?;
			core.wincan.copy(&mm_frame, None, None)?;
		} else {
			let fr = format!("images/main_menu_animation/{}.png", 24);
			let mm_frame = texture_creator.load_texture(fr)?;
			core.wincan.copy(&mm_frame, None, None)?;
		}
		if i > 800{
			i = 1;
		}

		//let fr = format!("images/main_menu_animation/{}.png", i);

		if !multiplayer_selected {
			// Draw singleplayer
			core.wincan.set_draw_color(Color::RGBA(0,100,0,255));
			core.wincan.fill_rect(single_player_button)?;
			core.wincan.copy(&text_texture, None, Rect::new(90, 600, 300, 90))?;

			// Draw multiplayer
			core.wincan.set_draw_color(Color::RGBA(0,100,0,255));
			core.wincan.fill_rect(multiplayer_button)?;
			core.wincan.copy(&text_texture_multi, None, Rect::new(500, 600, 300, 90))?;

			// Draw credits
			core.wincan.set_draw_color(Color::RGBA(0,100,0,255));
			core.wincan.fill_rect(credit_button)?;
			core.wincan.copy(&text_texture2, None, Rect::new(890, 600, 300, 90))?;
		} else {
			core.wincan.set_draw_color(Color::RGBA(0, 0, 0, 100));
			core.wincan.fill_rect(centered_rect!(core, 800, 500))?;

			// Draw create room box
			core.wincan.set_draw_color(Color::RGBA(50,50,50,255));
			core.wincan.fill_rect(multiplayer_create_rect)?;
			core.wincan.copy(&multiplayer_create_text, None, centered_rect!(core, _, 210, 300, 80))?;

			// Draw join room box
			core.wincan.set_draw_color(Color::RGBA(50,50,50,255));
			core.wincan.fill_rect(multiplayer_join_rect)?;
			core.wincan.copy(&multiplayer_join_text, None, centered_rect!(core, _, 480, 300, 80))?;

			// Draw join code box
			core.wincan.set_draw_color(Color::RGBA(255,255,255,255));
			core.wincan.draw_rect(join_code_textbox)?;

			//Render text for join code textbox
			let display_text = format!("{}{}", join_code, if textbox_selected && textbox_select_time.elapsed().subsec_millis()<500 { "|" } else { "" });
			match regular_font.size_of(&display_text) {
				Ok((w, h)) => {
					if w > 0 {
						let text_surface = regular_font.render(&display_text)
							.blended(Color::RGBA(255,255,255,255))
							.map_err(|e| e.to_string())?;
						let text_texture = texture_creator.create_texture_from_surface(&text_surface)
							.map_err(|e| e.to_string())?;
						core.wincan.copy(&text_texture, None, Rect::new(join_code_textbox.x + 20, join_code_textbox.y + (60-h as i32)/2, w, h))?;
					}
				},
				_ => {},
			}
		}

		core.wincan.present();

	}

	Ok(GameState::Quit)
}