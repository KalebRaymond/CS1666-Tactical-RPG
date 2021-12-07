use std::time::Instant;

use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use crate::net::client::{Client, ClientBuffer};

use crate::game_map::GameMap;
use crate::{Drawable, GameState};
use crate::unit::Team;
use crate::{SDLCore, TILE_SIZE};

pub struct MultiPlayer<'i, 'r> {
	core: &'i mut SDLCore<'r>,
	client: Client,
	client_buffer: ClientBuffer,

	bg_texture: Texture<'i>,
	bg_interface: Texture<'i>,
	bg_rect: Rect,

	join_text: Texture<'i>,
	join_text_period: Texture<'i>,
    join_text_rects: Vec<Rect>,
	join_text_anim_start: Instant,
    room_text: Texture<'i>,
	room_text_rect: Rect,

	game_map: GameMap<'i>,
}

impl MultiPlayer<'_, '_> {

	pub fn new<'i, 'r>(core: &'i mut SDLCore<'r>) -> Result<MultiPlayer<'i, 'r>, String> {
		let client = Client::new()?;

		let bg_texture = core.texture_creator.load_texture("images/main_menu_animation/24.png")?;
		let bg_interface = core.texture_creator.load_texture("images/interface/unit_interface.png")?;
		let bg_rect = centered_rect!(core, 800, 650);

		let join_str = "Waiting for another player to join.";
		let (join_w, join_h) = core.bold_font.size_of(&join_str).map_err(|_e| "Could not determine text size")?;
		let join_text = core.texture_creator.create_texture_from_surface(
			core.bold_font.render(&join_str)
				.blended(Color::RGBA(0,0,0,0)) //Black font
				.map_err(|e| e.to_string())?
		).map_err(|e| e.to_string())?;
		let (join_period_w, join_period_h) = core.bold_font.size_of(".").map_err(|_e| "Could not determine text size")?;
		let join_text_period = core.texture_creator.create_texture_from_surface(
			core.bold_font.render(".")
				.blended(Color::RGBA(0,0,0,0)) //Black font
				.map_err(|e| e.to_string())?
		).map_err(|e| e.to_string())?;
		let join_text_rects = vec![
			Rect::new(300, 120, join_w, join_h),
			Rect::new(300+join_w as i32, 120, join_period_w, join_period_h),
			Rect::new(300+(join_w+join_period_w) as i32, 120, join_period_w, join_period_h)
		];

		let room_str = format!("Room Code: {:04}", client.code);
		let (room_w, room_h) = core.bold_font.size_of(&room_str).map_err(|_e| "Could not determine text size")?;
		let room_text = core.texture_creator.create_texture_from_surface(
			core.bold_font.render(&room_str)
				.blended_wrapped(Color::RGBA(0,0,0,0), 320) //Black font
				.map_err(|e| e.to_string())?
		).map_err(|e| e.to_string())?;
		let room_text_rect = centered_rect!(core, _, 350, room_w, room_h);

		let game_map = GameMap::new(core, if client.is_host { Team::Player } else { Team::Enemy });

		//Set camera size based on map size
		core.cam.w = (game_map.map_size.0 as u32 * TILE_SIZE) as i32;
		core.cam.h = (game_map.map_size.1 as u32 * TILE_SIZE) as i32;
		if client.is_host {
			//Start camera in lower left corner, to start with the player castle in view
			core.cam.x = 0;
			core.cam.y = -core.cam.h + core.wincan.window().size().1 as i32;
		} else {
			//Start camera in lower left corner, to start with the player castle in view
			core.cam.x = -core.cam.w + core.wincan.window().size().0 as i32;
			core.cam.y = 0;
		}

		Ok(MultiPlayer {
			core,
			client,
			client_buffer: ClientBuffer::new(),

			bg_texture,
			bg_interface,
			bg_rect,

			join_text,
			join_text_period,
			join_text_rects,
			join_text_anim_start: Instant::now(),

			room_text,
			room_text_rect,

			game_map,
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

		// receive a new event from the server
		if !self.core.is_animating {
			match self.client_buffer.poll(&mut self.client) {
				Ok(Some(event)) => self.game_map.event_list.push(event),
				Err(e) => println!("Error polling server: {}", e),
				_ => {},
			}
		} else {
			self.core.is_animating = false;
		}

		self.core.wincan.clear();

		if !self.client.is_joined {
			// calculate time elapsed for join text
			let millis = self.join_text_anim_start.elapsed().subsec_millis();
			let anim_state = if millis == 999 { 2 } else { millis/333 }; // math to never be 3 if millis = 999
			// waiting for other player to join; draw prompt
			self.core.wincan.copy(&self.bg_texture, None, None)?;
			self.core.wincan.copy(&self.bg_interface, None, self.bg_rect)?;
			self.core.wincan.copy(&self.join_text, None, self.join_text_rects[0])?;
			if anim_state > 0 {
				self.core.wincan.copy(&self.join_text_period, None, self.join_text_rects[1])?;
				if anim_state > 1 {
					self.core.wincan.copy(&self.join_text_period, None, self.join_text_rects[2])?;
				}
			}
        	self.core.wincan.copy(&self.room_text, None, self.room_text_rect)?;
			self.core.wincan.present();
			return Ok(GameState::MultiPlayer);
		}

		//If no one has won so far...
		if self.game_map.winning_team.is_none() {
			//Handle the current team's move
			// handle the current player's turn
			if self.game_map.player_state.is_turn() {
				crate::player_turn::handle_player_turn(&self.core, &mut self.game_map)?;
			}

			// handle the barbarians' turn (only on the host client)
			if self.client.is_host && self.game_map.player_state.current_turn == Team::Barbarians {
				crate::barbarian_turn::handle_barbarian_turn(&self.core, &mut self.game_map)?;
			}
		}

		//Record user inputs
		self.core.input.update(&self.core.event_pump);

		crate::game_map::apply_events(&self.core, &mut self.game_map)?.iter()
			.filter(|e| e.from_self).copied()
			.for_each(|e| self.client_buffer.send(e));

		// render the current game board
		self.game_map.draw(self.core)?;

		self.core.wincan.set_viewport(self.core.cam);
		self.core.wincan.present();

		if !self.game_map.winning_team.is_none() && !self.game_map.banner.banner_visible && self.core.input.left_clicked {
			Ok(GameState::MainMenu)
		} else {
			Ok(GameState::MultiPlayer)
		}
	}

}