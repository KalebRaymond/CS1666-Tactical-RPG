use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::mouse::MouseState;

use crate::GameState;
use crate::SDLCore;

pub fn main_menu(core: &mut SDLCore) -> Result<GameState, String> {
    let mut next_game_state = GameState::SinglePlayer;
	let single_player_button = centered_rect!(core, _, 720/4, 100, 100);
	let credit_button = centered_rect!(core, _, 3*720/4, 100, 100);
	let join_code_textbox = Rect::new(750, 200, 400, 60);
	let mut join_code = "test";

	let regular_font = core.ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 32)?; //From https://www.fontsquirrel.com/fonts/open-sans

	'menuloop: loop {
		let mouse_state: MouseState = core.event_pump.mouse_state();

		if mouse_state.left() {
			let x = mouse_state.x();
			let y = mouse_state.y();
			if single_player_button.contains_point((x, y)) {
				next_game_state = GameState::SinglePlayer;
				break 'menuloop;
			} else if credit_button.contains_point((x, y)){
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
				_ => {},
			}
		}

		core.wincan.set_draw_color(Color::RGBA(0, 0, 0, 255)); //Black Screen
		core.wincan.clear();

		core.wincan.set_draw_color(Color::RGBA(255,0,0,255));
		core.wincan.draw_rect(single_player_button)?;
		
		core.wincan.set_draw_color(Color::RGBA(255,255,255,255));
		core.wincan.draw_rect(join_code_textbox)?;
		let text_surface = regular_font.render(join_code)
			.blended(Color::RGBA(255,255,255,255))
			.map_err(|e| e.to_string())?;
		let text_texture = core.texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;
		let text_size = regular_font.size_of(join_code);
		match text_size {
			Ok((w, h)) => {
				println!("{},{}", w, h);
				core.wincan.copy(&text_texture, None, Rect::new(760, 200 + (60-h as i32)/2, w, h))?;
			},
			_ => {},
		}

		core.wincan.set_draw_color(Color::RGBA(0,255,0,255));
		core.wincan.draw_rect(credit_button)?;

		core.wincan.present();
	}

	Ok(next_game_state)
}