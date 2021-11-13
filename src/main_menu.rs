use sdl2::image::LoadTexture;
use sdl2::mouse::MouseState;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use std::time::Instant;

use crate::{Drawable, GameState};
use crate::button::Button;
use crate::net::client;
use crate::SDLCore;

pub struct MainMenu<'i, 'r> {
	core: &'i mut SDLCore<'r>,

	bg_frame: usize,
	bg_textures: Vec<Texture<'i>>,
	bg_interface: Texture<'i>,

	// main menu buttons
	singleplayer_button: Button<'i>,
	multiplayer_button: Button<'i>,
	credits_button: Button<'i>,

	// multiplayer sub-menu buttons
	is_multiplayer_open: bool,
	multiplayer_rect: Rect,
	multiplayer_create_button: Button<'i>,
	multiplayer_join_button: Button<'i>,

	// join code
	join_code_rect: Rect,
	join_code: String,
	join_code_selected: bool,
	join_code_selected_time: Instant,
}

impl MainMenu<'_, '_> {

	pub fn new<'i, 'r>(core: &'i mut SDLCore<'r>) -> Result<MainMenu<'i, 'r>, String> {
		// bg animation textures
		let bg_textures: Vec<Texture> = (1..25).map(|i| {
			core.texture_creator.load_texture(format!("images/main_menu_animation/{}.png", i)).unwrap()
		}).collect();
		let bg_interface = core.texture_creator.load_texture("images/interface/unit_interface.png")?;

		// main menu buttons
		let singleplayer_button = Button::new(core, Rect::new(40, 600, 380, 100), "Single Player")?;
		let multiplayer_button = Button::new(core, Rect::new(450, 600, 380, 100), "Multiplayer")?;
		let credits_button = Button::new(core, Rect::new(860, 600, 380, 100), "Credits")?;

		// multiplayer sub-menu buttons
		let multiplayer_rect = centered_rect!(core, 800, 650);
		let multiplayer_create_button = Button::new(core, centered_rect!(core, _, 90, 400, 100), "Create Room")?;
		let multiplayer_join_button = Button::new(core, centered_rect!(core, _, 520, 400, 100), "Join Room")?;

		let join_code_rect = centered_rect!(core, _, 400, 400, 60);

		Ok(MainMenu {
			core,
			bg_frame: 0,
			bg_textures,
			bg_interface,

			singleplayer_button,
			multiplayer_button,
			credits_button,

			is_multiplayer_open: false,
			multiplayer_rect,
			multiplayer_create_button,
			multiplayer_join_button,

			join_code_rect,
			join_code: String::from(""),
			join_code_selected: false,
			join_code_selected_time: Instant::now(),
		})
	}

}

impl Drawable for MainMenu<'_, '_> {

	fn draw(&mut self) -> Result<GameState, String> {
		let mouse_state: MouseState = self.core.event_pump.mouse_state();
		let mouse_pos = (mouse_state.x(), mouse_state.y());

		if mouse_state.left() && self.is_multiplayer_open {
			if self.join_code_rect.contains_point(mouse_pos) {
				self.join_code_selected = true;
				self.join_code_selected_time = Instant::now();
			} else if self.multiplayer_create_button.is_mouse(self.core) {
				// create a new multiplayer room
				client::set_code(None);
				return Ok(GameState::MultiPlayer);
			} else if self.multiplayer_join_button.is_mouse(self.core) {
				// join multiplayer room with code
				let code: u32 = self.join_code.parse().map_err(|_e| "Couldn't parse join code")?;
				client::set_code(Some(code));
				return Ok(GameState::MultiPlayer);
			} else {
				self.join_code_selected = false;
				self.is_multiplayer_open = false;
			}
		}

		if mouse_state.left() && !self.is_multiplayer_open {
			if self.singleplayer_button.is_mouse(self.core) {
				return Ok(GameState::SinglePlayer);
			} else if self.multiplayer_button.is_mouse(self.core) {
				self.is_multiplayer_open = true;
			} else if self.credits_button.is_mouse(self.core) {
				return Ok(GameState::Credits);
			}
		}

		for event in self.core.event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit{..} | sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Escape), ..} => {
					return Err("Quit keycode".to_string());
				},
				sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Backspace), ..} => {
					if self.join_code_selected && self.join_code.chars().count() > 0 {
						let mut char_iter = self.join_code.chars();
						char_iter.next_back();
						self.join_code = char_iter.as_str().to_string();
					}
				}
				sdl2::event::Event::KeyDown{keycode: Some(key), ..} => {
					let parsed_key = key.to_string();
					if self.join_code_selected && self.join_code.chars().count() < 4 && parsed_key.chars().count() == 1 && parsed_key.chars().next().unwrap().is_numeric() {
						self.join_code.push_str(&key.to_string());
					}
				},
				_ => {},
			}
		}

		// background animation
		self.bg_frame += 1;
		if self.bg_frame < self.bg_textures.len() {
			self.core.wincan.copy(&self.bg_textures[self.bg_frame], None, None)?;
		} else if let Some(texture) = self.bg_textures.last() {
			self.core.wincan.copy(&texture, None, None)?;
		}

		if self.bg_frame > 800 {
			self.bg_frame = 0;
		}

		// buttons
		if !self.is_multiplayer_open {
			self.singleplayer_button.draw(self.core)?;
			self.multiplayer_button.draw(self.core)?;
			self.credits_button.draw(self.core)?;
		} else {
			// multiplayer sub-menu background
			self.core.wincan.copy(&self.bg_interface, None, self.multiplayer_rect)?;

			self.multiplayer_create_button.draw(self.core)?;
			self.multiplayer_join_button.draw(self.core)?;

			// Draw join code box
			self.core.wincan.set_draw_color(Color::RGBA(0,0,0,255));
			self.core.wincan.draw_rect(self.join_code_rect)?;

			//Render text for join code textbox
			let display_text = format!("{}{}", self.join_code, if self.join_code_selected && self.join_code_selected_time.elapsed().subsec_millis()<500 { "|" } else { "" });
			if let Ok((w, h)) = self.core.regular_font.size_of(&*display_text) {
				if w > 0 {
					let text_surface = self.core.regular_font.render(&*display_text)
						.blended(Color::RGBA(0,0,0,255))
						.map_err(|e| e.to_string())?;
					let text_texture = self.core.texture_creator.create_texture_from_surface(&text_surface)
						.map_err(|e| e.to_string())?;
					self.core.wincan.copy(&text_texture, None, Rect::new(self.join_code_rect.x + 20, self.join_code_rect.y + (60-h as i32)/2, w, h))?;
				}
			}
		}

		self.core.wincan.present();
		Ok(GameState::MainMenu)
	}

}
