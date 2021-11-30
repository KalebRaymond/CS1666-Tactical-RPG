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
use crate::GameState;
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

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	let texture_creator = core.wincan.texture_creator();

	//Load unit textures
	let mut unit_textures: HashMap<&str, Texture> = HashMap::new();
	unit_textures.insert("pll", texture_creator.load_texture("images/units/player1_melee.png")?);
	unit_textures.insert("plr", texture_creator.load_texture("images/units/player1_archer.png")?);
	unit_textures.insert("plm", texture_creator.load_texture("images/units/player1_mage.png")?);
	unit_textures.insert("pl2l", texture_creator.load_texture("images/units/player2_melee.png")?);
	unit_textures.insert("pl2r", texture_creator.load_texture("images/units/player2_archer.png")?);
	unit_textures.insert("pl2m", texture_creator.load_texture("images/units/player2_mage.png")?);
	unit_textures.insert("bl", texture_creator.load_texture("images/units/barbarian_melee.png")?);
	unit_textures.insert("br", texture_creator.load_texture("images/units/barbarian_archer.png")?);

	let unit_interface_texture = texture_creator.load_texture("images/interface/unit_interface.png")?;
	let mut unit_interface: Option<UnitInterface> = None;

	let mut choose_unit_interface: Option<UnitInterface> = None;

	//Collection of variables useful for handling map interaction & tile overlays
	let mut game_map = GameMap::new(core.texture_map);

	//Set camera size based on map size
	core.cam.w = (game_map.map_size.0 as u32 * TILE_SIZE) as i32;
	core.cam.h = (game_map.map_size.1 as u32 * TILE_SIZE) as i32;
	//Start camera in lower left corner, to start with the player castle in view
	core.cam.x = 0;
	core.cam.y = -core.cam.h + core.wincan.window().size().1 as i32;

	//Collection of variables useful for determining player's current state
	let mut player_state = PlayerState::new(Team::Player);

	let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (8,46)), ('l', (10,45)), ('l', (10,53)), ('l', (12,46)), ('l', (17,51)), ('l', (17,55)), ('l', (18,53)), ('r', (9,49)), ('r', (10,46)), ('r', (13,50)), ('r', (14,54)), ('r', (16,53)), ('m', (10,50)), ('m', (10,52)), ('m', (11,53)), ('m', (13,53)));
	//let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (14, 40))); //Spawns a player unit right next to some barbarians
	//let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (54, 8))); //Spawns a player unit right next to the enemy's castle
	prepare_player_units(&mut game_map.player_units, Team::Player, p1_units_abrev, &unit_textures, &mut game_map.map_tiles);

	let p2_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (46,8)), ('l', (45,10)), ('l', (53,10)), ('l', (46,12)), ('l', (51,17)), ('l', (55,17)), ('l', (53,18)), ('r', (49,9)), ('r', (47,10)), ('r', (50,13)), ('r', (54,14)), ('r', (53,16)), ('m', (50,10)), ('m', (52,10)), ('m', (53,11)), ('m', (53,13)));
	//let p2_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (16,44))); //Spawns a single enemy near the player
	prepare_player_units(&mut game_map.enemy_units, Team::Enemy, p2_units_abrev, &unit_textures, &mut game_map.map_tiles);

	let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (4,6)), ('l', (6,8)), ('l', (7,7)), ('r', (8,5)), ('l', (59,56)), ('l', (56,56)), ('l', (54,57)), ('r', (56,59)), ('l', (28,15)), ('l', (29,10)), ('l', (32,11)), ('l', (35,15)), ('r', (30,8)), ('r', (36,10)), ('l', (28,52)), ('l', (28,48)), ('l', (33,51)), ('l', (35,48)), ('r', (32,53)), ('r', (33,56)), ('l', (17,38)), ('l', (16,37)), ('r', (23,36)), ('r', (18,30)), ('l', (46,25)), ('l', (47,26)), ('r', (40,27)), ('r', (45,33)),);
	//let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (32, 60))); //Spawns a single barbarian near the bottom of the map
	//let barb_units_abrev: Vec<(char, (u32,u32))> = Vec::new(); //No barbarians
	prepare_player_units(&mut game_map.barbarian_units, Team::Barbarians, barb_units_abrev, &unit_textures, &mut game_map.map_tiles);

	// Do this right before the game starts so that player 1 starts
	initialize_next_turn(&mut game_map.player_units);

	let mut current_player = Team::Player;

	//Button for player to end their turn
    let mut end_turn_button = Button::new(core, Rect::new((CAM_W - 240).try_into().unwrap(), (CAM_H - 90).try_into().unwrap(), 200, 50), "End Turn")?;

	//Winning team. Is set to None until one of the Teams wins
	let mut winning_team: Option<Team> = None;
	let mut player1_on_base = 0;
	let mut player2_on_base = 0;
	// Not sure how else to check on_base once per turn
	let mut next_team_check = Team::Player;

	//Precalculating distances to each goal from each tile, used for enemy's AI.
	//If AI/distances.txt already exists, this line can be commented out.
	//genetics::get_goal_distances(&mut game_map.map_tiles, player_castle, enemy_castle, &camp_coords)?;

	//Distance from each tile to each goal, used for enemy's AI
	let distance_map = distance_map::DistanceMap::new();

	'gameloop: loop {
		core.wincan.clear();

		//Check if user tried to quit the program
		for event in core.event_pump.poll_iter() {
			match event {
				Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => break 'gameloop,
				_ => {},
			}
		}

		//Record user inputs
		core.input.update(&core.event_pump);

		//If no one has won so far...
		if winning_team.is_none() {
			//Handle the current team's move
			match current_player {
				Team::Player => {
					player_turn::handle_player_turn(&core, &mut player_state, &mut game_map, &mut unit_interface, &mut choose_unit_interface, &unit_textures, &unit_interface_texture, &mut current_player, &mut end_turn_button)?;

					// Checks to see if the player's units are on the opponent's castle tile
					if next_team_check == Team::Player {
						match game_map.player_units.get_mut(&game_map.pos_enemy_castle) {
							Some(_player1_unit) => {
								player1_on_base += 1;
								if player1_on_base >= TURNS_ON_BASE {
									winning_team = set_winner(Team::Player, &mut game_map.banner);
								}
							},
							_ => {
								player1_on_base = 0;
							},
						}
						println!("Turns on enemy castle: {}/{}", player1_on_base, TURNS_ON_BASE);
						// Makes it so that this isn't checked every time it loops through
						next_team_check = Team::Enemy;
					}
				},
				Team::Enemy => {
					enemy_turn::handle_enemy_turn(&core, &mut game_map, &mut current_player, &distance_map, &unit_textures)?;

					if next_team_check == Team::Enemy {
						match game_map.enemy_units.get_mut(&game_map.pos_player_castle) {
							Some(_player2_unit) => {
								player2_on_base += 1;
								if player2_on_base >= TURNS_ON_BASE {
									winning_team = set_winner(Team::Enemy, &mut game_map.banner);
								}
							},
							_ => {
								player2_on_base = 0;
							},
						}
						println!("Turns on player castle: {}/{}", player2_on_base, TURNS_ON_BASE);
						next_team_check = Team::Player;
					}
				},
				Team::Barbarians => {
					barbarian_turn::handle_barbarian_turn(&core, &mut game_map, &mut current_player)?;
				},
			}

			//Check for total party kill and set the other team as the winner
			//Ideally you would check this whenever a unit on either team gets attacked, but this works
			if game_map.player_units.len() == 0 {
				println!("Enemy team won via Total Party Kill!");
				winning_team = set_winner(Team::Enemy, &mut game_map.banner);
			}
			else if game_map.enemy_units.len() == 0 {
				println!("Player team won via Total Party Kill!");
				winning_team = set_winner(Team::Player, &mut game_map.banner);
			}
		}

		game_map.draw(core);

		match game_map.player_units.get(&(player_state.active_unit_j as u32, player_state.active_unit_i as u32)) {
			Some(_) => {
				match player_state.current_player_action {
					PlayerAction::MovingUnit => {
						draw_possible_moves(core, &game_map.possible_moves, Color::RGBA(0, 89, 178, 50))?;
					},
					PlayerAction::AttackingUnit => {
						draw_possible_moves(core, &game_map.possible_attacks, Color::RGBA(178, 89, 0, 100))?;
						draw_possible_moves(core, &game_map.actual_attacks, Color::RGBA(128, 0, 128, 100))?;
					},
					_ => {},
				}
			}
			_ => ()
		};

		//Draw the damage indicators that appear above the units that have received damage
		for damage_indicator in game_map.damage_indicators.iter_mut() {
			damage_indicator.draw(core)?;
		}
		//Remove the damage indicators that have expired
		game_map.damage_indicators.retain(|damage_indicator| {
			damage_indicator.is_visible
		});

		if current_player == Team::Player
		{
			//Draw the scroll sprite UI
			unit_interface = match unit_interface {
				Some(mut ui) => {
					match ui.draw(core, &texture_creator) {
						Ok(_) => { Some(ui) },
						_ => { None },
					}
				},
				_ => { None },
			};

			choose_unit_interface = match choose_unit_interface {
				Some(mut ui) => {
					match ui.draw(core, &texture_creator) {
						Ok(_) => { Some(ui) },
						_ => { None },
					}
				},
				_ => { None },
			};

			//Draw the button for the player to end their turn, relative to the camera
			end_turn_button.draw_relative(core)?;
		}

		//Draw banner that appears at beginning of turn

		core.wincan.set_viewport(core.cam);
		core.wincan.present();
	}

	//Single player somehow finished without a winner, automatically quit game
	Ok(GameState::Quit)
}


// Function that takes a HashMap of units and sets all has_attacked and has_moved to false so that they can move again
pub fn initialize_next_turn(team_units: &mut HashMap<(u32, u32), Unit>) {
	for unit in &mut team_units.values_mut() {
		unit.next_turn();
	}
}

// Draws a rect of a certain color over all tiles contained within the vector
fn draw_possible_moves(core: &mut SDLCore, tiles: &Vec<(u32, u32)>, color:Color) -> Result< (), String> {
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

// Method for preparing the HashMap of player units whilst also properly marking them in the map
// l melee r ranged m mage
fn prepare_player_units<'a, 'b> (player_units: &mut HashMap<(u32, u32), Unit<'a>>, player_team: Team, units: Vec<(char, (u32, u32))>, unit_textures: &'a HashMap<&str, Texture<'a>>, map: &'b mut HashMap<(u32, u32), Tile>) {
	let (melee, range, mage)  = match player_team {
		Team::Player => ("pll", "plr", "plm"),
		Team::Enemy =>  ("pl2l", "pl2r", "pl2m"),
		Team::Barbarians => ("bl", "br", ""),
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
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 10, 7, 1, 95, 1, 3, unit_textures.get(melee).unwrap()));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 20, 7, 1, 95, 1, 5, unit_textures.get(melee).unwrap()));
				}
			},
			'r' => {
				if player_team == Team::Barbarians {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 8, 5, 4, 85, 2, 4, unit_textures.get(range).unwrap()));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 15, 5, 4, 85, 3, 7, unit_textures.get(range).unwrap()));
				}
			},
			_ => {
				if player_team == Team::Barbarians {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 5, 6, 3, 75,  3, 6, unit_textures.get(mage).unwrap()));
				}
				else {
					player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 10, 6, 3, 75,  5, 9, unit_textures.get(mage).unwrap()));
				}
			},
		};
	}
}

//Sets up the winner's banner so it can start displaying, and returns an Option containing the Team corresponding to the winning team
pub fn set_winner(winner: Team, winner_banner: &mut Banner) -> Option<Team> {
	match winner {
		Team::Player => {
			println!("Player 1 wins!");
			winner_banner.show("p1_win_banner");
		},
		Team::Enemy => {
			println!("Enemy wins!");
			winner_banner.show("p2_win_banner");
		},
		Team::Barbarians => {
			println!("Barbarians win!");
		},
	};

	return Some(winner);
}