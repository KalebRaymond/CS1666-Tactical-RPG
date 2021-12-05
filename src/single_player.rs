use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::ai::*;
use crate::game_map::GameMap;
use crate::{Drawable, GameState};
use crate::TILE_SIZE;
use crate::player_turn;
use crate::enemy_turn;
use crate::barbarian_turn;
use crate::SDLCore;
use crate::unit::Team;

pub struct SinglePlayer<'i, 'r> {
	core: &'i mut SDLCore<'r>,

	game_map: GameMap<'i>,

	distance_map: distance_map::DistanceMap,
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
		if self.game_map.winning_team.is_none() {
			//Handle the current team's move
			match self.game_map.player_state.current_turn {
				Team::Player => player_turn::handle_player_turn(&self.core, &mut self.game_map)?,
				Team::Enemy => enemy_turn::handle_enemy_turn(&self.core, &mut self.game_map, &self.distance_map)?,
				Team::Barbarians => barbarian_turn::handle_barbarian_turn(&self.core, &mut self.game_map)?,
			}
		}

		//Record user inputs
		self.core.input.update(&self.core.event_pump);

		crate::game_map::apply_events(&self.core, &mut self.game_map)?;

		self.game_map.draw(self.core)?;

		self.core.wincan.set_viewport(self.core.cam);
		self.core.wincan.present();

		if !self.game_map.winning_team.is_none() && !self.game_map.banner.banner_visible {
			Ok(GameState::MainMenu)
		} else {
			Ok(GameState::SinglePlayer)
		}
	}
}
