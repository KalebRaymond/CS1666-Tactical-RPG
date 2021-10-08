use sdl2::pixels::Color;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::mouse::MouseState;
use sdl2::keyboard::Keycode;

use crate::GameState;
use crate::SDLCore;

pub fn main_menu(core: &mut SDLCore) -> Result<GameState, String> {
    let mut next_game_state = GameState::SinglePlayer;
	let single_player_button = centered_rect!(core, _, 720/4, 100, 100);
	let credit_button = centered_rect!(core, _, 3*720/4, 100, 100);

	'menuloop: loop {
		let mouse_state: MouseState = core.event_pump.mouse_state();

		if mouse_state.left() {
			let x = mouse_state.x();
			let y = mouse_state.y();
			if single_player_button.contains_point((x, y)) {
				next_game_state = GameState::SinglePlayer;
				break 'menuloop;
			} else if credit_button.contains_point(((x, y)){
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
		
		core.wincan.set_draw_color(Color::RGBA(0,255,0,255));
		core.wincan.draw_rect(credit_button)?;

		core.wincan.present();
	}

	Ok(next_game_state)
}