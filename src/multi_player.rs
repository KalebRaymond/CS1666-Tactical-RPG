use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use crate::net::client::Client;
use crate::net::util::*;

use crate::{Drawable, GameState};
use crate::SDLCore;

pub struct MultiPlayer<'i, 'r> {
	core: &'i mut SDLCore<'r>,
	client: Client,

	bg_texture: Texture<'i>,
	bg_interface: Texture<'i>,
	bg_rect: Rect,

	join_text: Texture<'i>,
    join_text_rect: Rect,
    room_text: Texture<'i>,
    room_text_rect: Rect,
}

impl MultiPlayer<'_, '_> {

	pub fn new<'i, 'r>(core: &'i mut SDLCore<'r>) -> Result<MultiPlayer<'i, 'r>, String> {
		let client = Client::new()?;

		let bg_texture = core.texture_creator.load_texture("images/main_menu_animation/24.png")?;
		let bg_interface = core.texture_creator.load_texture("images/interface/unit_interface.png")?;
		let bg_rect = centered_rect!(core, 800, 650);

		let join_str = "Waiting for another player to join...";
		let (join_w, join_h) = core.bold_font.size_of(&join_str).map_err(|_e| "Could not determine text size")?;
		let join_text = core.texture_creator.create_texture_from_surface(
			core.bold_font.render(&join_str)
				.blended(Color::RGBA(0,0,0,0)) //Black font
				.map_err(|e| e.to_string())?
		).map_err(|e| e.to_string())?;
		let join_text_rect = Rect::new(300, 120, join_w, join_h);

		let room_str = format!("Room Code: {:04}", client.code);
		let (room_w, room_h) = core.bold_font.size_of(&room_str).map_err(|_e| "Could not determine text size")?;
		let room_text = core.texture_creator.create_texture_from_surface(
			core.bold_font.render(&room_str)
				.blended_wrapped(Color::RGBA(0,0,0,0), 320) //Black font
				.map_err(|e| e.to_string())?
		).map_err(|e| e.to_string())?;
		let room_text_rect = centered_rect!(core, _, 350, room_w, room_h);

		Ok(MultiPlayer {
			core,
			client,

			bg_texture,
			bg_interface,
			bg_rect,

			join_text,
			join_text_rect,

			room_text,
			room_text_rect,
		})
	}

}

impl Drawable for MultiPlayer<'_, '_> {

	fn draw(&mut self) -> Result<GameState, String> {
		for event in self.core.event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit{..} | sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Escape), ..} => {
					return Err("Quit keycode".to_string());
				},
				_ => {},
			}
		}

		if let Some(event) = self.client.poll()? {
			match event {
				_ => {},
			}
		}

		self.core.wincan.clear();

		if !self.client.is_joined {
			// waiting for other player to join; draw prompt
			self.core.wincan.copy(&self.bg_texture, None, None)?;
			self.core.wincan.copy(&self.bg_interface, None, self.bg_rect)?;
			self.core.wincan.copy(&self.join_text, None, self.join_text_rect)?;
        	self.core.wincan.copy(&self.room_text, None, self.room_text_rect)?;
			self.core.wincan.present();
			return Ok(GameState::MultiPlayer);
		}

		// TODO: game rendering logic
		self.core.wincan.present();

		Ok(GameState::MultiPlayer)
	}

}