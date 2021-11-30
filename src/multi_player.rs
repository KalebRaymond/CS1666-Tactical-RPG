use std::convert::TryInto;
use std::time::Instant;

use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use crate::net::client::Client;
use crate::net::util::*;

use crate::game_map::GameMap;
use crate::{Drawable, GameState};
use crate::player_state::PlayerState;
use crate::unit::Team;
use crate::button::Button;
use crate::{SDLCore, CAM_W, CAM_H, TILE_SIZE};

pub struct MultiPlayer<'i, 'r> {
	core: &'i mut SDLCore<'r>,
	client: Client,

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
	player_state: PlayerState,
	end_turn_button: Button<'i>,
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

		let game_map = GameMap::new(core.texture_map);
		let player_state = PlayerState::new(if client.is_host { Team::Player } else { Team::Enemy });
		let end_turn_button = Button::new(core, Rect::new((CAM_W - 240).try_into().unwrap(), (CAM_H - 90).try_into().unwrap(), 200, 50), "End Turn")?;

		//Set camera size based on map size
		core.cam.w = (game_map.map_size.0 as u32 * TILE_SIZE) as i32;
		core.cam.h = (game_map.map_size.1 as u32 * TILE_SIZE) as i32;
		//Start camera in lower left corner, to start with the player castle in view
		core.cam.x = 0;
		core.cam.y = -core.cam.h + core.wincan.window().size().1 as i32;

		Ok(MultiPlayer {
			core,
			client,

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
			player_state,
			end_turn_button,
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
				Event{action: EVENT_END_TURN, ..} => {
					// event.id == 0: signals the end of the opposing player's turn
					if event.id == 0 && !self.player_state.is_turn() && self.player_state.current_turn != Team::Barbarians {
						let next_team = self.player_state.advance_turn();
						self.game_map.banner.show_turn(next_team);
					}
					// event.id == 1: signals the end of the barbarian turn by the host
					else if event.id == 1 && !self.client.is_host && self.player_state.current_turn == Team::Barbarians {
						let next_team = self.player_state.advance_turn();
						self.game_map.banner.show_turn(next_team);
					}
				},
				_ => {},
			}
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

		//Record user inputs
		self.core.input.update(&self.core.event_pump);

		// render the current game board
		self.game_map.draw(self.core);

		// handle the current player's turn
		if self.player_state.is_turn() {
			self.end_turn_button.draw_relative(self.core)?;

			if self.core.input.left_clicked && self.end_turn_button.is_mouse(self.core) {
				// end the player turn
				self.client.send(Event::create(EVENT_END_TURN, 0, (0,0), (0,0)))?;
				let next_team = self.player_state.advance_turn();
				self.game_map.banner.show_turn(next_team);
			}
		}

		// handle the barbarians' turn (only on the host client)
		if self.client.is_host && self.player_state.current_turn == Team::Barbarians {
			if !self.game_map.banner.banner_visible {
				// end the barbarians turn
				self.client.send(Event::create(EVENT_END_TURN, 1, (0,0), (0,0)))?;
				let next_team = self.player_state.advance_turn();
				self.game_map.banner.show_turn(next_team);
			}
		}

		self.core.wincan.set_viewport(self.core.cam);
		self.core.wincan.present();

		Ok(GameState::MultiPlayer)
	}

}