use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryInto;

use sdl2::video::WindowContext;
use sdl2::render::{Texture, TextureCreator};
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;

use crate::cursor::Cursor;
use crate::banner::Banner;
use crate::button::Button;
use crate::damage_indicator::DamageIndicator;
use crate::unit_interface::UnitInterface;
use crate::objective_manager::ObjectiveManager;
use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::tile::{Tile, Structure};
use crate::unit::{Team, Unit, GUARD_HEALTH_ID, SCOUT_HEALTH_ID};
use crate::pixel_coordinates::PixelCoordinates;
use crate::{CAM_H, CAM_W, TILE_SIZE};
use crate::SDLCore;
use crate::net::util::*;

pub struct GameMap<'a> {
	pub map_tiles: HashMap<(u32, u32), Tile<'a>>,
	pub map_size: (usize, usize),

	//Stuff for enemy AI calculations
	pub objectives: ObjectiveManager,

	pub player_units: HashMap<(u32, u32), Unit<'a>>,
	pub enemy_units: HashMap<(u32, u32), Unit<'a>>,
	pub barbarian_units: HashMap<(u32, u32), Unit<'a>>,

	pub possible_moves: Vec<(u32, u32)>,
	pub possible_attacks: Vec<(u32, u32)>,
	pub actual_attacks: Vec<(u32, u32)>,

	pub unit_interface: Option<UnitInterface<'a>>,
	pub choose_unit_interface: Option<UnitInterface<'a>>,

	pub player_state: PlayerState,

	//Holds all damage indicators (the numbers that appear above a unit when attacked) that are visible
	pub damage_indicators: Vec<DamageIndicator>,

	// various UI elements
	pub banner: Banner,
	pub cursor: Cursor<'a>,
	pub end_turn_button: Button<'a>,

	pub camp_textures: Vec<(&'a Texture<'a>, &'a Texture<'a>)>,

	pub event_list: Vec<Event>,
	pub event_list_index: usize,

	pub winning_team: Option<Team>,
}

impl GameMap<'_> {
	pub fn new<'a>(core: &SDLCore<'a>, player_team: Team) -> GameMap<'a> {
		//Load map from file
		let map_data = File::open("maps/map.txt").expect("Unable to open map file");
		let mut map_data = BufReader::new(map_data);
		let mut line = String::new();

		//Sets size of the map from the first line of the text file
		map_data.read_line(&mut line).unwrap();
		let map_width: usize = line.trim().parse().unwrap();
		let map_height: usize = line.trim().parse().unwrap();

		//Creates map from file
		let map_string: Vec<Vec<String>> = map_data.lines()
			.take(map_width)
			.map(|x| x.unwrap().chars().collect::<Vec<char>>())
			.map(|x| x.chunks(2).map(|chunk| chunk[0].to_string()).collect())
			.collect();

		let end_turn_button = Button::new(core, Rect::new((CAM_W - 240).try_into().unwrap(), (CAM_H - 90).try_into().unwrap(), 200, 50), "End Turn").unwrap();

		let mut map: GameMap<'a> = GameMap {
			map_tiles: HashMap::new(),
			map_size: (map_width, map_height),
			objectives: ObjectiveManager::init_default(),
			player_units: HashMap::new(),
			enemy_units: HashMap::new(),
			barbarian_units: HashMap::new(),
			possible_moves: Vec::new(),
			possible_attacks: Vec::new(),
			actual_attacks: Vec::new(),
			unit_interface: None,
			choose_unit_interface: None,
			player_state: PlayerState::new(player_team),
			damage_indicators: Vec::new(),
			banner: Banner::new(),
			cursor: Cursor::new(core.texture_map.get("cursor").unwrap()),
			end_turn_button,
			camp_textures: Vec::new(),
			event_list: Vec::new(),
			event_list_index: 0,
			winning_team: None,
		};

		//Set up the HashMap of Tiles that can be interacted with
		let mut x = 0;
		let mut y = 0;
		let mut pos_player_castle: (u32, u32) = (0, 0);
		let mut pos_enemy_castle: (u32, u32) = (0, 0);
		let mut pos_barbarian_camps: Vec<(u32, u32)> = Vec::new();
		for row in map_string.iter() {
			for col in row.iter() {
				let letter = if player_team == Team::Enemy {
					match col.as_ref() {
						"1" => "2",
						"2" => "1",
						_ => &col[..],
					}
				} else { &col[..] };

				let texture = core.texture_map.get(letter).unwrap();
				match letter {
					"║" | "^" | "v" | "<" | "=" | ">" | "t" => map.map_tiles.insert((x,y), Tile::new(x, y, false, true, None, None, texture)),
					" " => map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, None, texture)),
					"b" =>  {
						pos_barbarian_camps.push((y,x));
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), texture))
					},
					"f" => {
						pos_barbarian_camps.push((y,x));
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), texture))
					}
					"_" => map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), texture)),
					"1" =>  {
						pos_player_castle = (y, x);
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::PCastle), texture))
					},
					"2" =>  {
						pos_enemy_castle = (y, x);
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::ECastle), texture))
					},
					_ => map.map_tiles.insert((x,y), Tile::new(x, y, false, false, None, None, texture)),
				};
				y += 1;
			}
			x += 1;
			y = 0;
		}
		map.camp_textures.push((core.texture_map.get("pc").unwrap(), core.texture_map.get("ec").unwrap()));
		map.camp_textures.push((core.texture_map.get("pf").unwrap(), core.texture_map.get("ef").unwrap()));


		//Now that the locations of the objectives have been found, update the ObjectiveManager
		map.objectives = ObjectiveManager::new(pos_player_castle, pos_enemy_castle, pos_barbarian_camps);

		let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(
			('l', (8,46)), ('l', (10,45)), ('l', (12,46)), ('l', (17,51)), ('l', (17,55)), ('l', (18,53)),
			('r', (9,49)), ('r', (10,47)), ('r', (14,54)), ('r', (16,53)),
			('m', (10,50)), ('m', (13,53)), ('m', (12,48)),
			('g', (10,53)),
			('s', (10,52)), ('s', (11,53)),
		);
		//let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (14, 40))); //Spawns a player unit right next to some barbarians
		//let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (54, 8))); //Spawns a player unit right next to the enemy's castle
		let p2_units_abrev: Vec<(char, (u32,u32))> = vec!(
			('l', (46,8)), ('l', (45,10)), ('l', (46,12)), ('l', (51,17)), ('l', (55,17)), ('l', (53,18)),
			('r', (49,9)), ('r', (47,10)), ('r', (54,14)), ('r', (53,16)),
			('m', (50,10)), ('m', (53,13)), ('m', (48,12)),
			('g', (53,10)),
			('s', (52,10)), ('s', (53,11)),
		);
		//let p2_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (16,44))); //Spawns a single enemy near the player
		let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (4,6)), ('l', (6,8)), ('l', (7,7)), ('r', (8,5)), ('l', (59,56)), ('l', (56,56)), ('l', (54,57)), ('r', (56,59)), ('l', (28,15)), ('l', (29,10)), ('l', (32,11)), ('l', (35,15)), ('r', (30,8)), ('r', (36,10)), ('l', (28,52)), ('l', (28,48)), ('l', (33,51)), ('l', (35,48)), ('r', (32,53)), ('r', (33,56)), ('l', (17,38)), ('l', (16,37)), ('r', (23,36)), ('r', (18,30)), ('l', (46,25)), ('l', (47,26)), ('r', (40,27)), ('r', (45,33)),);
		//let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (32, 60))); //Spawns a single barbarian near the bottom of the map
		//let barb_units_abrev: Vec<(char, (u32,u32))> = Vec::new(); //No barbarians

		prepare_player_units(&mut map.player_units, Team::Player, if player_team == Team::Player { &p1_units_abrev } else { &p2_units_abrev }, &core.texture_map, &mut map.map_tiles);
		prepare_player_units(&mut map.enemy_units, Team::Enemy, if player_team == Team::Player { &p2_units_abrev } else { &p1_units_abrev }, &core.texture_map, &mut map.map_tiles);
		prepare_player_units(&mut map.barbarian_units, Team::Barbarians, &barb_units_abrev, &core.texture_map, &mut map.map_tiles);

		map.initialize_next_turn(Team::Player);

		map
	}

	pub fn draw(&mut self, core: &mut SDLCore) -> Result<(), String> {
		//Camera controls should stay enabled even when it is not the player's turn,
		//which is why this code block is not in player_turn.rs
		if core.input.right_held && !self.banner.banner_visible {
			let max_move = TILE_SIZE as i32;
			core.cam.x = (core.cam.x - (core.input.mouse_x_old - core.input.mouse_x).clamp(-max_move, max_move)).clamp(-core.cam.w + core.wincan.window().size().0 as i32, 0);
			core.cam.y = (core.cam.y - (core.input.mouse_y_old - core.input.mouse_y).clamp(-max_move, max_move)).clamp(-core.cam.h + core.wincan.window().size().1 as i32, 0);
		}

		let (i, j) = PixelCoordinates::matrix_indices_from_pixel(
            core.input.mouse_x.try_into().unwrap(),
            core.input.mouse_y.try_into().unwrap(),
            (-1 * core.cam.x).try_into().unwrap(),
            (-1 * core.cam.y).try_into().unwrap()
        );

		match self.player_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				self.cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {
				self.cursor.hide_cursor();
			},
		}
		match self.enemy_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				self.cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {},
		}
		match self.barbarian_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				self.cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {},
		}

		while self.objectives.taken_over_camps.len() > 0 {
			let pos_to_change = self.objectives.taken_over_camps.pop().unwrap();
			let fort_locations: ((u32, u32), (u32, u32)) = ((31, 10), (31, 52));

			if pos_to_change.1 == Team::Player {
				if pos_to_change.0 == fort_locations.0 || pos_to_change.0 == fort_locations.1 {
					let texture = self.camp_textures[1].0;
					self.map_tiles.insert((pos_to_change.0.1,pos_to_change.0.0), Tile::new(pos_to_change.0.1, pos_to_change.0.0, true, true, None, Some(Structure::Camp), texture));
				} else {
					let texture = self.camp_textures[0].0;
					self.map_tiles.insert((pos_to_change.0.0,pos_to_change.0.1), Tile::new(pos_to_change.0.0, pos_to_change.0.1, true, true, None, Some(Structure::Camp), texture));
				}
			} else {
				if pos_to_change.0 == fort_locations.0 || pos_to_change.0 == fort_locations.1 {
					let texture = self.camp_textures[1].1;
					self.map_tiles.insert((pos_to_change.0.1,pos_to_change.0.0), Tile::new(pos_to_change.0.1, pos_to_change.0.0, true, true, None, Some(Structure::Camp), texture));
				} else {
					let texture = self.camp_textures[0].1;
					self.map_tiles.insert((pos_to_change.0.0,pos_to_change.0.1), Tile::new(pos_to_change.0.0, pos_to_change.0.1, true, true, None, Some(Structure::Camp), texture));
				}
			}
		}

		//Draw tiles & sprites
		for x in 0..self.map_size.0 {
			for y in 0..self.map_size.1 {
				let map_tile = self.map_tiles.get(&(y as u32, x as u32));
				let map_tile_size = match map_tile {
					Some(Tile{contained_structure: Some(Structure::Camp), ..}) => TILE_SIZE * 2,
					_ => TILE_SIZE,
				};

				let pixel_location = PixelCoordinates::from_matrix_indices(y as u32, x as u32);
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, map_tile_size, map_tile_size);

				//Draw map tile at this coordinate
				if let Some(map_tile) = self.map_tiles.get(&(y as u32, x as u32)) {
					core.wincan.copy(map_tile.texture, None, dest)?;
				}

				//Use default sprite size for all non-map sprites
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);

				//Draw player unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(unit) = self.player_units.get_mut(&(x as u32, y as u32)) {
					unit.draw(core, &dest)?;
				}

				//Draw enemy unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(enemy) = self.enemy_units.get_mut(&(x as u32, y as u32)) {
					enemy.draw(core, &dest)?;
				}

				//Draw barbarian unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(barbarian) = self.barbarian_units.get_mut(&(x as u32, y as u32)) {
					barbarian.draw(core, &dest)?;
				}
			}
		}

		// draw UI/banners
		self.cursor.draw(core)?;
		self.banner.draw(core)?;

		// draw possible move grid
		match self.player_units.get(&(self.player_state.active_unit_j as u32, self.player_state.active_unit_i as u32)) {
			Some(_) => {
				match self.player_state.current_player_action {
					PlayerAction::MovingUnit => {
						draw_possible_moves(core, &self.possible_moves, Color::RGBA(0, 89, 178, 50))?;
					},
					PlayerAction::AttackingUnit => {
						draw_possible_moves(core, &self.possible_attacks, Color::RGBA(178, 89, 0, 100))?;
						draw_possible_moves(core, &self.actual_attacks, Color::RGBA(128, 0, 128, 100))?;
					},
					_ => {},
				}
			}
			_ => ()
		};

		//Draw the damage indicators that appear above the units that have received damage
		for damage_indicator in self.damage_indicators.iter_mut() {
			damage_indicator.draw(core)?;
		}
		//Remove the damage indicators that have expired
		self.damage_indicators.retain(|damage_indicator| {
			damage_indicator.is_visible
		});

		if self.player_state.is_turn() {
			//Draw the scroll sprite UI
			let result = if let Some(ui) = self.unit_interface.as_mut() {
				ui.draw(core, core.texture_creator).is_ok()
			} else {
				false
			};

			if !result {
				self.unit_interface = None;
			}

			let result_2 = if let Some(ui) = self.choose_unit_interface.as_mut() {
				ui.draw(core, core.texture_creator).is_ok()
			} else {
				false
			};

			if !result_2 {
				self.choose_unit_interface = None;
			}

			//Draw the button for the player to end their turn, relative to the camera
			self.end_turn_button.draw_relative(core)?;
		}

		Ok(())
	}

	// Function that takes a HashMap of units and sets all has_attacked and has_moved to false so that they can move again
	pub fn initialize_next_turn(&mut self, team: Team) {
		let client_team = team.as_client(&self.player_state);
		self.banner.show_turn(client_team);

		//Fix glitch where castle tile says it's occupied when it's not
		self.correct_map_errors();
		// Checks to see if the player's units are on the opponent's castle tile
		self.objectives.check_objectives(client_team, match client_team {
			Team::Player => &self.player_units,
			Team::Enemy => &self.enemy_units,
			Team::Barbarians => &self.barbarian_units,
		});

		if self.objectives.has_won(client_team) {
			self.set_winner(client_team);

		//Check for total party kill and set the other team as the winner
		//Ideally you would check this whenever a unit on either team gets attacked, but this works
		} else if self.player_units.len() == 0 {
			println!("Enemy team won via Total Party Kill!");
			self.set_winner(Team::Enemy);
		} else if self.enemy_units.len() == 0 {
			println!("Player team won via Total Party Kill!");
			self.set_winner(Team::Player);
		}

		match team {
			Team::Player => {
				for unit in &mut self.player_units.values_mut() {
					unit.next_turn();
				}
			}
			Team::Enemy => {
				for unit in &mut self.enemy_units.values_mut() {
					unit.next_turn();
				}
			}
			Team::Barbarians => {
				for unit in &mut self.barbarian_units.values_mut() {
					unit.next_turn();
				}
			}
		}
	}

	//Sets up the winner's banner so it can start displaying, and returns an Option containing the Team corresponding to the winning team
	pub fn set_winner(&mut self, winner: Team) {
		match winner {
			Team::Player => {
				println!("You win!");
				self.banner.show("p1_win_banner");
			},
			Team::Enemy => {
				println!("You Lose!");
				self.banner.show("p2_win_banner");
			},
			Team::Barbarians => {
				println!("Barbarians win!");
			},
		};

		// send an END_GAME event to the other client
		self.event_list.push(Event::create(EVENT_END_GAME, winner.as_client(&self.player_state).to_id(), (0,0), (0,0), 0));
		self.winning_team = Some(winner);
	}

	/* For some reason there's a glitch where sometimes the spaces where the enemy units spawn are
	 * marked as occupied even when they aren't, which prevents the player from being able to occupy
	 * the enemy's castle. This function checks the spaces around both castles and fixes any disparities
	 * between the units and the tiles' contained unit.
	 */
	pub fn correct_map_errors(&mut self) {
		//Check the player castle
		println!("Player castle:");
		for i in (self.objectives.p1_castle.1 - 1)..=(self.objectives.p1_castle.1 + 1) {
			for j in (self.objectives.p1_castle.0 - 1)..=(self.objectives.p1_castle.0 + 1) {
				let tile_contains_unit = if let Some(tile) = self.map_tiles.get(&(i, j)) {
					if let Some(team) = tile.contained_unit_team {
						if team == Team::Player {
							1
						}
						else {
							0 //Team is not Player
						}
					}
					else {
						2 //Team is None
					}
				}
				else {
					3 //Tile not found
				};

				let hashmap_contains_unit = if let Some(_unit) = self.player_units.get(&(j, i)) {
					1
				}
				else {
					0
				};

				//If the tile says it contains a Player, but the hashmap of units does not,
				//the tile needs to be corrected
				if tile_contains_unit == 1 && !(hashmap_contains_unit == 1) {
					if let Some(mut tile) = self.map_tiles.get_mut(&(i, j)) {
						tile.contained_unit_team = None;
					}
				}

				print!("[{}, {}] ", tile_contains_unit, hashmap_contains_unit);
			}
			println!();
		}

		//Check the enemy castle
		println!("Enemy castle:");
		for i in (self.objectives.p2_castle.1 - 1)..=(self.objectives.p2_castle.1 + 1) {
			for j in (self.objectives.p2_castle.0 - 1)..=(self.objectives.p2_castle.0 + 1) {
				let tile_contains_unit = if let Some(tile) = self.map_tiles.get(&(i, j)) {
					if let Some(team) = tile.contained_unit_team {
						if team == Team::Enemy {
							1
						}
						else {
							0 //Team is not Enemy
						}
					}
					else {
						2 //Team is None
					}
				}
				else {
					3 //Tile not found
				};

				let hashmap_contains_unit = if let Some(_unit) = self.enemy_units.get(&(j, i)) {
					1
				}
				else {
					0
				};

				//If the tile says it contains an Enemy, but the hashmap of units does not,
				//the tile needs to be corrected
				if tile_contains_unit == 1 && !(hashmap_contains_unit == 1) {
					if let Some(mut tile) = self.map_tiles.get_mut(&(i, j)) {
						tile.contained_unit_team = None;
					}
				}

				print!("[{}, {}] ", tile_contains_unit, hashmap_contains_unit);
			}
			println!();
		}
	}
	pub fn get_unit(&self, pos: &(u32, u32)) -> Result<&Unit, String> {
		// for whatever reason, all the event positions are inverted as (y,x), so they need to be flipped to (x,y) to get the map tile
		let unit_tile = self.map_tiles.get(&(pos.1, pos.0)).ok_or("Could not get map tile at unit position")?;

		match unit_tile.contained_unit_team {
			Some(Team::Player) => self.player_units.get(pos),
			Some(Team::Enemy) => self.enemy_units.get(pos),
			Some(Team::Barbarians) => self.barbarian_units.get(pos),
			None => return Err("No unit in the specified tile position".to_string())
		}.ok_or("Could not get unit at position".to_string())
	}
}

pub fn apply_events<'a>(core: &SDLCore<'a>, game_map: &mut GameMap<'a>) -> Result<Vec<Event>, String> {
	let mut ret: Vec<Event> = Vec::new();

	// process any new events in the event_list
	let new_index = game_map.event_list.len();
	for i in game_map.event_list_index..new_index {
		if let Some(event) = game_map.event_list.get(i).map(|e| e.clone()) {
			println!("Applying event #{}: {}", i, event);
			apply_event(core, game_map, event)?;
			ret.push(event.clone());
		}
	}
	game_map.event_list_index = new_index;

	// remove any (dead) units that have reached 0 hp
	let mut dead_units: Vec<(u32, u32)> = Vec::new();
	dead_units.extend(
		game_map.player_units.values().filter(|u| u.hp == 0).map(|u| (u.x, u.y))
	);
	dead_units.extend(
		game_map.enemy_units.values().filter(|u| u.hp == 0).map(|u| (u.x, u.y))
	);
	dead_units.extend(
		game_map.barbarian_units.values().filter(|u| u.hp == 0).map(|u| (u.x, u.y))
	);

	for pos in dead_units {
		game_map.player_units.remove(&pos);
		game_map.enemy_units.remove(&pos);
		game_map.barbarian_units.remove(&pos);

		game_map.map_tiles.get_mut(&(pos.1, pos.0)).map(|t| t.update_team(None));
	}

	Ok(ret)
}

pub fn apply_event<'a>(core: &SDLCore<'a>, game_map: &mut GameMap<'a>, event: Event) -> Result<(), String> {
	// for whatever reason, all the event positions are inverted as (y,x), so they need to be flipped to (x,y) to get the map tile
	let from_tile = (event.from_pos.1, event.from_pos.0);
	let to_tile = (event.to_pos.1, event.to_pos.0);

	let from_team = game_map.map_tiles.get_mut(&from_tile).ok_or("Could not obtain 'from' tile")?.contained_unit_team;
	let to_team = game_map.map_tiles.get_mut(&to_tile).ok_or("Could not obtain 'from' tile")?.contained_unit_team;

	match event.action {
		EVENT_MOVE => {
			let unit_map = match from_team {
				Some(Team::Player) => &mut game_map.player_units,
				Some(Team::Enemy) => &mut game_map.enemy_units,
				Some(Team::Barbarians) => &mut game_map.barbarian_units,
				None => {
					return Err("No specified unit on event 'from' tile".to_string());
				}
			};

			if from_tile == to_tile {
				// moving unit to the same tile; no action required
				unit_map.get_mut(&event.from_pos).map(|u| u.has_moved = true);
				return Ok(());
			}

			if to_team != None {
				return Err("Could not apply event: 'to' tile already contains another unit".to_string());
			}

			let unit_ref = unit_map.get(&event.from_pos).ok_or("Could not get selected unit for event")?;
			if unit_ref.has_moved {
				return Err("Could not apply event: selected unit has already been moved in this turn".to_string());
			}

			let mut unit = unit_map.remove(&event.from_pos).ok_or("Could not remove selected unit for event")?;
			unit.update_pos(event.to_pos.0, event.to_pos.1);
			unit.has_moved = true;
			unit_map.insert((event.to_pos.0, event.to_pos.1), unit);

			// Update map tiles
			game_map.map_tiles.get_mut(&from_tile).map(|t| t.update_team(None));
			game_map.map_tiles.get_mut(&to_tile).map(|t| t.update_team(from_team));
		},
		EVENT_ATTACK => {
			let (attacking_unit_map, defending_unit_map) = match (from_team, to_team) {
				(Some(Team::Player), Some(Team::Enemy)) => (&mut game_map.player_units, &mut game_map.enemy_units),
				(Some(Team::Player), Some(Team::Barbarians)) => (&mut game_map.player_units, &mut game_map.barbarian_units),
				(Some(Team::Enemy), Some(Team::Player)) => (&mut game_map.enemy_units, &mut game_map.player_units),
				(Some(Team::Enemy), Some(Team::Barbarians)) => (&mut game_map.enemy_units, &mut game_map.barbarian_units),
				(Some(Team::Barbarians), Some(Team::Player)) => (&mut game_map.barbarian_units, &mut game_map.player_units),
				(Some(Team::Barbarians), Some(Team::Enemy)) => (&mut game_map.barbarian_units, &mut game_map.enemy_units),
				_ => {
					return Err("No specified attacking unit on event 'from' tile".to_string());
				},
			};

			let attacking_unit = attacking_unit_map.get_mut(&event.from_pos).ok_or("Could not get selected attacker unit for event")?;
			let unit = defending_unit_map.get_mut(&event.to_pos).ok_or("Could not get selected defender unit for event")?;

			attacking_unit.has_attacked = true;
			attacking_unit.starting_x = attacking_unit.x;
			attacking_unit.starting_y = attacking_unit.y;
			unit.receive_damage(event.value as u32, &attacking_unit);
			game_map.damage_indicators.push(DamageIndicator::new(core, event.value as u32, PixelCoordinates::from_matrix_indices(
				unit.y.checked_sub(1).unwrap_or(unit.y),
				unit.x
			))?);
		},
		EVENT_END_TURN => {
			let next_team = game_map.player_state.advance_turn();
			println!("Ending turn: preparing turn for {}", next_team.to_string());
			game_map.initialize_next_turn(next_team);
		},
		EVENT_SPAWN_UNIT => {
			let unit_team = if event.from_self { Team::Player } else { Team::Enemy };

			let unit_map = match unit_team {
				Team::Player => &mut game_map.player_units,
				Team::Enemy => &mut game_map.enemy_units,
				Team::Barbarians => &mut game_map.barbarian_units,
			};

			let (melee, range, mage)  = match unit_team {
				Team::Player => ("pll", "plr", "plm"),
				Team::Enemy =>  ("pl2l", "pl2r", "pl2m"),
				Team::Barbarians => ("bl", "br", ""),
			};

			let (x, y) = event.to_pos;
			// update unit team on map tile
			game_map.map_tiles.get_mut(&(y, x)).unwrap().update_team(Some(unit_team));

			let mut new_unit = match event.value {
				EVENT_UNIT_MELEE =>  Unit::new(x, y, unit_team, 20, 7, 1, 95, 1, 5, core.texture_map.get(melee).unwrap(), false),
				EVENT_UNIT_ARCHER => Unit::new(x, y, unit_team, 15, 5, 4, 85, 3, 7, core.texture_map.get(range).unwrap(), true),
				_ =>                 Unit::new(x, y, unit_team, 10, 6, 3, 75, 5, 9, core.texture_map.get(mage).unwrap(), true),
			};

			new_unit.has_moved = true;
			new_unit.has_attacked = true;

			unit_map.insert((x, y), new_unit);
			println!("Unit spawned at {:?}", (x, y));
		},
		EVENT_END_GAME => {
			if game_map.winning_team == None {
				let team = Team::from_id(event.id)?;
				game_map.set_winner(team.as_client(&game_map.player_state));
			}
		},
		_ => {

		},
	}

	Ok(())
}

// Method for preparing the HashMap of player units whilst also properly marking them in the map
// l melee r ranged m mage
pub fn prepare_player_units<'a, 'b> (player_units: &mut HashMap<(u32, u32), Unit<'a>>, player_team: Team, units: &Vec<(char, (u32, u32))>, unit_textures: &'a HashMap<String, Texture<'a>>, map: &'b mut HashMap<(u32, u32), Tile>) {
	let (melee, range, mage, guard, scout)  = match player_team {
		Team::Player => ("pll", "plr", "plm", "plg", "pls"),
		Team::Enemy =>  ("pl2l", "pl2r", "pl2m", "pl2g", "pl2s"),
		Team::Barbarians => ("bl", "br", "", "", ""),
	};

	for unit in units {
		//Remember map is flipped indexing
		match player_team {
			Team::Player => map.get_mut(&(unit.1.1, unit.1.0)).unwrap().update_team(Some(Team::Player)),
			Team::Enemy => map.get_mut(&(unit.1.1, unit.1.0)).unwrap().update_team(Some(Team::Enemy)),
			Team::Barbarians => map.get_mut(&(unit.1.1, unit.1.0)).unwrap().update_team(Some(Team::Barbarians)),
		}

		//Add unit to team. Barbarian units get half as much HP and do half as much max damage
		match unit.0 {
			'l' => {
				if player_team == Team::Barbarians {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 10, 7, 1, 95, 1, 3, unit_textures.get(melee).unwrap(), false));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 20, 7, 1, 95, 1, 5, unit_textures.get(melee).unwrap(), false));
				}
			},
			'r' => {
				if player_team == Team::Barbarians {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 8, 5, 4, 85, 2, 4, unit_textures.get(range).unwrap(), true));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 15, 5, 4, 85, 3, 7, unit_textures.get(range).unwrap(), true));
				}
			},
			'g' => {
				if player_team == Team::Barbarians {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 16, 4, 1, 90, 1, 5, unit_textures.get(guard).unwrap(), false));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, GUARD_HEALTH_ID, 4, 1, 90, 1, 5, unit_textures.get(guard).unwrap(), false));
				}
			}
			's' => {
				if player_team == Team::Barbarians {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 6, 9, 2, 100, 4, 4, unit_textures.get(scout).unwrap(), false));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, SCOUT_HEALTH_ID, 9, 2, 100, 4, 4, unit_textures.get(scout).unwrap(), false));
				}
			}
			_ => {
				if player_team == Team::Barbarians {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 5, 6, 3, 75,  3, 6, unit_textures.get(mage).unwrap(), true));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 10, 6, 3, 75,  5, 9, unit_textures.get(mage).unwrap(), true));
				}
			},
		};
	}
}

pub fn draw_possible_moves(core: &mut SDLCore, tiles: &Vec<(u32, u32)>, color:Color) -> Result< (), String> {
	for (x,y) in tiles.into_iter() {
		let pixel_location = PixelCoordinates::from_matrix_indices(*y, *x);
		let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);
		core.wincan.set_blend_mode(BlendMode::Blend);
		core.wincan.set_draw_color(color);
		core.wincan.draw_rect(dest)?;
		core.wincan.fill_rect(dest)?;
	}
	Ok(())
}

//Load map textures
pub fn load_textures<'r>(textures: &mut HashMap<String, Texture<'r>>, texture_creator: &'r TextureCreator<WindowContext>) -> Result<(), String> {
	//Mountains
	textures.insert("▉".to_string(), texture_creator.load_texture("images/tiles/mountain_tile.png")?);
	textures.insert("▒".to_string(), texture_creator.load_texture("images/tiles/mountain2_tile.png")?);
	textures.insert("▀".to_string(), texture_creator.load_texture("images/tiles/mountain_side_top.png")?);
	textures.insert("▐".to_string(), texture_creator.load_texture("images/tiles/mountain_side_vertical_right.png")?);
	textures.insert("▃".to_string(), texture_creator.load_texture("images/tiles/mountain_side_bottom.png")?);
	textures.insert("▍".to_string(), texture_creator.load_texture("images/tiles/mountain_side_vertical_left.png")?);
	textures.insert("▛".to_string(), texture_creator.load_texture("images/tiles/mountain_top_left.png")?);
	textures.insert("▜".to_string(), texture_creator.load_texture("images/tiles/mountain_top_right.png")?);
	textures.insert("▙".to_string(), texture_creator.load_texture("images/tiles/mountain_bottom_left.png")?);
	textures.insert("▟".to_string(), texture_creator.load_texture("images/tiles/mountain_bottom_right.png")?);
	//Grass
	textures.insert(" ".to_string(), texture_creator.load_texture("images/tiles/grass_tile.png")?);
	textures.insert("_".to_string(), texture_creator.load_texture("images/tiles/empty_tile.png")?);
	//Rivers
	textures.insert("=".to_string(), texture_creator.load_texture("images/tiles/river_tile.png")?);
	textures.insert("║".to_string(), texture_creator.load_texture("images/tiles/river_vertical.png")?);
	textures.insert("^".to_string(), texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	textures.insert("v".to_string(), texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	textures.insert(">".to_string(), texture_creator.load_texture("images/tiles/river_end_right.png")?);
	textures.insert("<".to_string(), texture_creator.load_texture("images/tiles/river_end_left.png")?);
	//Bases
	textures.insert("b".to_string(), texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	textures.insert("pc".to_string(), texture_creator.load_texture("images/tiles/player_camp.png")?);
	textures.insert("ec".to_string(), texture_creator.load_texture("images/tiles/enemy_camp.png")?);
	textures.insert("f".to_string(), texture_creator.load_texture("images/tiles/barbarian_fort.png")?);
	textures.insert("pf".to_string(), texture_creator.load_texture("images/tiles/player_fort.png")?);
	textures.insert("ef".to_string(), texture_creator.load_texture("images/tiles/enemy_fort.png")?);
	textures.insert("1".to_string(), texture_creator.load_texture("images/tiles/blue_castle.png")?);
	textures.insert("2".to_string(), texture_creator.load_texture("images/tiles/red_castle.png")?);
	//Tree
	textures.insert("t".to_string(), texture_creator.load_texture("images/tiles/tree_tile.png")?);

	//Load unit textures
	textures.insert("pll".to_string(), texture_creator.load_texture("images/units/player1_melee.png")?);
	textures.insert("plr".to_string(), texture_creator.load_texture("images/units/player1_archer.png")?);
	textures.insert("plm".to_string(), texture_creator.load_texture("images/units/player1_mage.png")?);
	textures.insert("plg".to_string(), texture_creator.load_texture("images/units/player1_guard.png")?);
	textures.insert("pls".to_string(), texture_creator.load_texture("images/units/player1_scout.png")?);
	textures.insert("pl2l".to_string(), texture_creator.load_texture("images/units/player2_melee.png")?);
	textures.insert("pl2r".to_string(), texture_creator.load_texture("images/units/player2_archer.png")?);
	textures.insert("pl2m".to_string(), texture_creator.load_texture("images/units/player2_mage.png")?);
	textures.insert("pl2g".to_string(), texture_creator.load_texture("images/units/player2_guard.png")?);
	textures.insert("pl2s".to_string(), texture_creator.load_texture("images/units/player2_scout.png")?);
	textures.insert("bl".to_string(), texture_creator.load_texture("images/units/barbarian_melee.png")?);
	textures.insert("br".to_string(), texture_creator.load_texture("images/units/barbarian_archer.png")?);

	//Load UI textures
	textures.insert("cursor".to_string(), texture_creator.load_texture("images/interface/cursor.png")?);
	textures.insert("unit_interface".to_string(), texture_creator.load_texture("images/interface/unit_interface.png")?);

	Ok(())
}
