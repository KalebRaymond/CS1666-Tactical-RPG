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
	let single_player_button = Rect::new(100, 600, 400, 100);
	let text_surface = bold_font.render("Single Player")
		.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320) //White font
		.map_err(|e| e.to_string())?;

	let text_texture = texture_creator.create_texture_from_surface(&text_surface)
		.map_err(|e| e.to_string())?;

	//Credit button
	let credit_button = Rect::new(600, 600, 400, 100);
	let text_surface2 = bold_font.render("Credits")
		.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320) //White font
		.map_err(|e| e.to_string())?;

	let text_texture2 = texture_creator.create_texture_from_surface(&text_surface2)
		.map_err(|e| e.to_string())?;

	//Join code textbox
	let join_code_textbox = Rect::new(750, 200, 400, 60);
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

		if mouse_state.left() {
			let x = mouse_state.x();
			let y = mouse_state.y();
			textbox_selected = false;
			if single_player_button.contains_point((x, y)) {
				sdl2::mixer::Music::fade_out(600);
				next_game_state = GameState::SinglePlayer;
				break 'menuloop;
			} else if join_code_textbox.contains_point((x,y)) {
				textbox_selected = true;
				textbox_select_time = Instant::now();
			} else if credit_button.contains_point((x, y)){
				sdl2::mixer::Music::fade_out(600);
				next_game_state = GameState::Credits;
				break 'menuloop;
			}
		}

		for event in core.event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit{..} | sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Escape), ..} => {
					next_game_state = GameState::Quit;
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
		

		core.wincan.set_draw_color(Color::RGBA(255,0,0,255));
		core.wincan.draw_rect(single_player_button)?;
		core.wincan.copy(&text_texture, None, Rect::new(150, 600, 300, 90))?;
		
		core.wincan.set_draw_color(Color::RGBA(255,255,255,255));
		core.wincan.draw_rect(join_code_textbox)?;
		
		//Render text for join code textbox
		let display_text = format!("{}{}", join_code, if textbox_selected && textbox_select_time.elapsed().subsec_millis()<500 { "|" } else { "" });
		let text_size = regular_font.size_of(&display_text);
		match text_size {
			Ok((w, h)) => {
				if w > 0 {
					let text_surface = regular_font.render(&display_text)
						.blended(Color::RGBA(255,255,255,255))
						.map_err(|e| e.to_string())?;
					let text_texture = texture_creator.create_texture_from_surface(&text_surface)
						.map_err(|e| e.to_string())?;
					core.wincan.copy(&text_texture, None, Rect::new(760, 200 + (60-h as i32)/2, w, h))?;
				}
			},
			_ => {},
		}

		core.wincan.set_draw_color(Color::RGBA(0,255,0,255));
		core.wincan.draw_rect(credit_button)?;
		core.wincan.copy(&text_texture2, None, Rect::new(650, 600, 300, 90))?;

		core.wincan.present();

	}

	Ok(next_game_state)
}