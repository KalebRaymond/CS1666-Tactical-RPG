use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use std::collections::HashMap;
use std::convert::TryInto;
use crate::AI::*;
use crate::button::Button;
use crate::cursor::Cursor;
use crate::game_map::GameMap;
use crate::{Drawable, GameState};
use crate::{CAM_H, CAM_W, TILE_SIZE};
use crate::pixel_coordinates::PixelCoordinates;
use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::player_turn;
use crate::enemy_turn;
use crate::barbarian_turn;
use crate::SDLCore;
use crate::tile::Tile;
use crate::banner::Banner;
use crate::unit_interface::UnitInterface;
use crate::unit::{Team, Unit};

const TURNS_ON_BASE: u32 = 3;

pub struct SinglePlayer<'i, 'r> {
	core: &'i mut SDLCore<'r>,

	game_map: GameMap<'i>,

	distance_map: distance_map::DistanceMap,

	winning_team: Option<Team>,
	player1_on_base: u32,
	player2_on_base: u32,
	next_team_check: Team,
}

impl SinglePlayer<'_,'_> {
	pub fn new<'i, 'r>(core: &'i mut SDLCore<'r>) -> Result<SinglePlayer<'i, 'r>, String> {
		let game_map = GameMap::new(core, Team::Player);

		//Set camera size based on map size
		core.cam.w = (game_map.map_size.0 as u32 * TILE_SIZE) as i32;
		core.cam.h = (game_map.map_size.1 as u32 * TILE_SIZE) as i32;
		//Start camera in lower left corner, to start with the player castle in view
		core.cam.x = 0;
		core.cam.y = -core.cam.h + core.wincan.window().size().1 as i32;



		let distance_map = distance_map::DistanceMap::new();

		Ok(SinglePlayer {
			core,
			game_map,
			distance_map,
			winning_team: None,
			player1_on_base: 0,
			player2_on_base: 0,
			next_team_check: Team::Player,
		})
	}
}

impl Drawable for SinglePlayer<'_,'_> {

	fn draw(&mut self) -> Result<GameState, String> {
		self.core.wincan.clear();

		//Check if user tried to quit the program
		for event in self.core.event_pump.poll_iter() {
			match event {
				Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
					return Err("Quit keycode".to_string());
				},
				_ => {},
			}
		}

		//If no one has won so far...
		if self.winning_team.is_none() {
			//Handle the current team's move
			match self.game_map.player_state.current_turn {
				Team::Player => {
					player_turn::handle_player_turn(&self.core, &mut self.game_map)?;

					// Checks to see if the player's units are on the opponent's castle tile
					if self.next_team_check == Team::Player {
						match self.game_map.player_units.get_mut(&self.game_map.pos_enemy_castle) {
							Some(_player1_unit) => {
								self.player1_on_base += 1;
								if self.player1_on_base >= TURNS_ON_BASE {
									self.winning_team = self.game_map.set_winner(Team::Player);
								}
							},
							_ => {
								self.player1_on_base = 0;
							},
						}
						println!("Turns on enemy castle: {}/{}", self.player1_on_base, TURNS_ON_BASE);
						// Makes it so that this isn't checked every time it loops through
						self.next_team_check = Team::Enemy;
					}
				},
				Team::Enemy => {
					enemy_turn::handle_enemy_turn(&self.core, &mut self.game_map, &self.distance_map)?;

					if self.next_team_check == Team::Enemy {
						match self.game_map.enemy_units.get_mut(&self.game_map.pos_player_castle) {
							Some(_player2_unit) => {
								self.player2_on_base += 1;
								if self.player2_on_base >= TURNS_ON_BASE {
									self.winning_team = self.game_map.set_winner(Team::Enemy);
								}
							},
							_ => {
								self.player2_on_base = 0;
							},
						}
						println!("Turns on player castle: {}/{}", self.player2_on_base, TURNS_ON_BASE);
						self.next_team_check = Team::Player;
					}
				},
				Team::Barbarians => {
					barbarian_turn::handle_barbarian_turn(&self.core, &mut self.game_map)?;
				},
			}

			//Check for total party kill and set the other team as the winner
			//Ideally you would check this whenever a unit on either team gets attacked, but this works
			if self.game_map.player_units.len() == 0 {
				println!("Enemy team won via Total Party Kill!");
				self.winning_team = self.game_map.set_winner(Team::Enemy);
			}
			else if self.game_map.enemy_units.len() == 0 {
				println!("Player team won via Total Party Kill!");
				self.winning_team = self.game_map.set_winner(Team::Player);
			}
		}

		//Record user inputs
		self.core.input.update(&self.core.event_pump);

		self.game_map.draw(self.core);

		self.core.wincan.set_viewport(self.core.cam);
		self.core.wincan.present();
		Ok(GameState::SinglePlayer)
	}
}
