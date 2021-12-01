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
use crate::input::Input;
use crate::banner::Banner;
use crate::button::Button;
use crate::damage_indicator::DamageIndicator;
use crate::unit_interface::UnitInterface;
use crate::objectives::ObjectiveManager;
use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::tile::{Tile, Structure};
use crate::unit::{Team, Unit};
use crate::pixel_coordinates::PixelCoordinates;
use crate::{CAM_H, CAM_W, TILE_SIZE};
use crate::SDLCore;

pub struct GameMap<'a> {
	pub map_tiles: HashMap<(u32, u32), Tile<'a>>,
	pub map_size: (usize, usize),

	//Stuff for enemy AI calculations
	/*
	pub pos_player_castle: (u32, u32),
	pub pos_enemy_castle: (u32, u32),
	pub pos_barbarian_camps: Vec<(u32, u32)>,
	*/
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
	pub damage_indicators: Vec<DamageIndicator<'a>>,

	// various UI elements
	pub banner: Banner,
	pub cursor: Cursor<'a>,
	pub end_turn_button: Button<'a>,
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
			/*
			pos_player_castle: (0, 0),
			pos_enemy_castle: (0, 0),
			pos_barbarian_camps: Vec::new(),
			*/
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
		};

		//Set up the HashMap of Tiles that can be interacted with
		let mut x = 0;
		let mut y = 0;
		let mut pos_player_castle: (u32, u32) = (0, 0);
		let mut pos_enemy_castle: (u32, u32) = (0, 0);
		let mut pos_barbarian_camps: Vec<(u32, u32)> = Vec::new();
		for row in map_string.iter() {
			for col in row.iter() {
				let letter = &col[..];
				let texture = core.texture_map.get(letter).unwrap();
				match letter {
					"║" | "^" | "v" | "<" | "=" | ">" | "t" => map.map_tiles.insert((x,y), Tile::new(x, y, false, true, None, None, texture)),
					" " => map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, None, texture)),
					"b" =>  {
						pos_barbarian_camps.push((y,x));
						map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), texture))
					},
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

		//Now that the locations of the objectives have been found, update the ObjectiveManager
		map.objectives = ObjectiveManager::new(pos_player_castle, pos_enemy_castle, pos_barbarian_camps);

		let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (8,46)), ('l', (10,45)), ('l', (10,53)), ('l', (12,46)), ('l', (17,51)), ('l', (17,55)), ('l', (18,53)), ('r', (9,49)), ('r', (10,46)), ('r', (13,50)), ('r', (14,54)), ('r', (16,53)), ('m', (10,50)), ('m', (10,52)), ('m', (11,53)), ('m', (13,53)));
		//let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (14, 40))); //Spawns a player unit right next to some barbarians
		//let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (54, 8))); //Spawns a player unit right next to the enemy's castle
		let p2_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (46,8)), ('l', (45,10)), ('l', (53,10)), ('l', (46,12)), ('l', (51,17)), ('l', (55,17)), ('l', (53,18)), ('r', (49,9)), ('r', (47,10)), ('r', (50,13)), ('r', (54,14)), ('r', (53,16)), ('m', (50,10)), ('m', (52,10)), ('m', (53,11)), ('m', (53,13)));
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

	pub fn draw(&mut self, core: &mut SDLCore) {
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
					core.wincan.copy(map_tile.texture, None, dest);
				}

				//Use default sprite size for all non-map sprites
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);

				//Draw player unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(mut unit) = self.player_units.get_mut(&(x as u32, y as u32)) {
					unit.draw(core, &dest);
				}

				//Draw enemy unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(mut enemy) = self.enemy_units.get_mut(&(x as u32, y as u32)) {
					enemy.draw(core, &dest);
				}

				//Draw barbarian unit at this coordinate (Don't forget row is y and col is x because 2d arrays)
				if let Some(mut barbarian) = self.barbarian_units.get_mut(&(x as u32, y as u32)) {
					barbarian.draw(core, &dest);
				}
			}
		}

		// draw UI/banners
		self.cursor.draw(core);
		self.banner.draw(core);

		// draw possible move grid
		match self.player_units.get(&(self.player_state.active_unit_j as u32, self.player_state.active_unit_i as u32)) {
			Some(_) => {
				match self.player_state.current_player_action {
					PlayerAction::MovingUnit => {
						draw_possible_moves(core, &self.possible_moves, Color::RGBA(0, 89, 178, 50));
					},
					PlayerAction::AttackingUnit => {
						draw_possible_moves(core, &self.possible_attacks, Color::RGBA(178, 89, 0, 100));
						draw_possible_moves(core, &self.actual_attacks, Color::RGBA(128, 0, 128, 100));
					},
					_ => {},
				}
			}
			_ => ()
		};

		//Draw the damage indicators that appear above the units that have received damage
		for damage_indicator in self.damage_indicators.iter_mut() {
			damage_indicator.draw(core);
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
			self.end_turn_button.draw_relative(core);
		}
	}

	// Function that takes a HashMap of units and sets all has_attacked and has_moved to false so that they can move again
	pub fn initialize_next_turn(&mut self, team: Team) {
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
	pub fn set_winner(&mut self, winner: Team) -> Option<Team> {
		match winner {
			Team::Player => {
				println!("Player 1 wins!");
				self.banner.show("p1_win_banner");
			},
			Team::Enemy => {
				println!("Enemy wins!");
				self.banner.show("p2_win_banner");
			},
			Team::Barbarians => {
				println!("Barbarians win!");
			},
		};

		return Some(winner);
	}
}

// Method for preparing the HashMap of player units whilst also properly marking them in the map
// l melee r ranged m mage
pub fn prepare_player_units<'a, 'b> (player_units: &mut HashMap<(u32, u32), Unit<'a>>, player_team: Team, units: &Vec<(char, (u32, u32))>, unit_textures: &'a HashMap<&str, Texture<'a>>, map: &'b mut HashMap<(u32, u32), Tile>) {
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
pub fn load_textures<'r>(textures: &mut HashMap<&str, Texture<'r>>, texture_creator: &'r TextureCreator<WindowContext>) -> Result<(), String> {
	//Mountains
	textures.insert("▉", texture_creator.load_texture("images/tiles/mountain_tile.png")?);
	textures.insert("▒", texture_creator.load_texture("images/tiles/mountain2_tile.png")?);
	textures.insert("▀", texture_creator.load_texture("images/tiles/mountain_side_top.png")?);
	textures.insert("▐", texture_creator.load_texture("images/tiles/mountain_side_vertical_right.png")?);
	textures.insert("▃", texture_creator.load_texture("images/tiles/mountain_side_bottom.png")?);
	textures.insert("▍", texture_creator.load_texture("images/tiles/mountain_side_vertical_left.png")?);
	textures.insert("▛", texture_creator.load_texture("images/tiles/mountain_top_left.png")?);
	textures.insert("▜", texture_creator.load_texture("images/tiles/mountain_top_right.png")?);
	textures.insert("▙", texture_creator.load_texture("images/tiles/mountain_bottom_left.png")?);
	textures.insert("▟", texture_creator.load_texture("images/tiles/mountain_bottom_right.png")?);
	//Grass
	textures.insert(" ", texture_creator.load_texture("images/tiles/grass_tile.png")?);
	textures.insert("_", texture_creator.load_texture("images/tiles/empty_tile.png")?);
	//Rivers
	textures.insert("=", texture_creator.load_texture("images/tiles/river_tile.png")?);
	textures.insert("║", texture_creator.load_texture("images/tiles/river_vertical.png")?);
	textures.insert("^", texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	textures.insert("v", texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	textures.insert(">", texture_creator.load_texture("images/tiles/river_end_right.png")?);
	textures.insert("<", texture_creator.load_texture("images/tiles/river_end_left.png")?);
	//Bases
	textures.insert("b", texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	textures.insert("1", texture_creator.load_texture("images/tiles/blue_castle.png")?);
	textures.insert("2", texture_creator.load_texture("images/tiles/red_castle.png")?);
	//Tree
	textures.insert("t", texture_creator.load_texture("images/tiles/tree_tile.png")?);

	//Load unit textures
	textures.insert("pll", texture_creator.load_texture("images/units/player1_melee.png")?);
	textures.insert("plr", texture_creator.load_texture("images/units/player1_archer.png")?);
	textures.insert("plm", texture_creator.load_texture("images/units/player1_mage.png")?);
	textures.insert("pl2l", texture_creator.load_texture("images/units/player2_melee.png")?);
	textures.insert("pl2r", texture_creator.load_texture("images/units/player2_archer.png")?);
	textures.insert("pl2m", texture_creator.load_texture("images/units/player2_mage.png")?);
	textures.insert("bl", texture_creator.load_texture("images/units/barbarian_melee.png")?);
	textures.insert("br", texture_creator.load_texture("images/units/barbarian_archer.png")?);

	//Load UI textures
	textures.insert("cursor", texture_creator.load_texture("images/interface/cursor.png")?);
	textures.insert("unit_interface", texture_creator.load_texture("images/interface/unit_interface.png")?);

	Ok(())
}
