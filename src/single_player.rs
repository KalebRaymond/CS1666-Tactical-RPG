use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Instant, Duration};

use crate::button::Button;
use crate::cursor::Cursor;
use crate::damage_indicator::DamageIndicator;
use crate::game_map::GameMap;
use crate::GameState;
use crate::{CAM_H, CAM_W, TILE_SIZE};
use crate::input::Input;
use crate::pixel_coordinates::PixelCoordinates;
use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::player_turn;
use crate::enemy_turn;
use crate::barbarian_turn;
use crate::SDLCore;
use crate::tile::{Tile, Structure};
use crate::banner::Banner;
use crate::unit_interface::UnitInterface;
use crate::unit::{Team, Unit};

const BANNER_TIMEOUT: u64 = 1500;
const TURNS_ON_BASE: u32 = 3;

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	let texture_creator = core.wincan.texture_creator();

	//Stuff for enemy AI calculations
	let mut camp_coords: Vec<(u32, u32)> = Vec::new();
	let mut player_castle: (u32, u32) = (0, 0);
	let mut enemy_castle: (u32, u32) = (0, 0);

	//Load map from file
	let map_data = File::open("maps/map.txt").expect("Unable to open map file");
	let mut map_data = BufReader::new(map_data);
	let mut line = String::new();

	//Sets size of the map from the first line of the text file
	map_data.read_line(&mut line).unwrap();
	let map_width: usize = line.trim().parse().unwrap();
	let map_height: usize = line.trim().parse().unwrap();

	//Set camera size based on map size
	core.cam.w = (map_width as u32 * TILE_SIZE) as i32;
	core.cam.h = (map_height as u32 * TILE_SIZE) as i32;
	//Start camera in lower left corner, to start with the player castle in view
	core.cam.x = 0;
	core.cam.y = -core.cam.h + core.wincan.window().size().1 as i32;

	//Initial mouse positions
	let mut old_mouse_x = -1;
	let mut old_mouse_y = -1;

	//User input
	let mut input = Input::new(&core.event_pump);

	//Creates map from file
	let map_string: Vec<Vec<String>> = map_data.lines()
		.take(map_width)
		.map(|x| x.unwrap().chars().collect::<Vec<char>>())
		.map(|x| x.chunks(2).map(|chunk| chunk[0].to_string()).collect())
		.collect();

	//Load map textures
	let mut tile_textures: HashMap<&str, Texture> = HashMap::new();
	//Mountains
	tile_textures.insert("▉", texture_creator.load_texture("images/tiles/mountain_tile.png")?);
	tile_textures.insert("▒", texture_creator.load_texture("images/tiles/mountain2_tile.png")?);
	tile_textures.insert("▀", texture_creator.load_texture("images/tiles/mountain_side_top.png")?);
	tile_textures.insert("▐", texture_creator.load_texture("images/tiles/mountain_side_vertical_right.png")?);
	tile_textures.insert("▃", texture_creator.load_texture("images/tiles/mountain_side_bottom.png")?);
	tile_textures.insert("▍", texture_creator.load_texture("images/tiles/mountain_side_vertical_left.png")?);
	tile_textures.insert("▛", texture_creator.load_texture("images/tiles/mountain_top_left.png")?);
	tile_textures.insert("▜", texture_creator.load_texture("images/tiles/mountain_top_right.png")?);
	tile_textures.insert("▙", texture_creator.load_texture("images/tiles/mountain_bottom_left.png")?);
	tile_textures.insert("▟", texture_creator.load_texture("images/tiles/mountain_bottom_right.png")?);
	//Grass
	tile_textures.insert(" ", texture_creator.load_texture("images/tiles/grass_tile.png")?);
	tile_textures.insert("_", texture_creator.load_texture("images/tiles/empty_tile.png")?);
	//Rivers
	tile_textures.insert("=", texture_creator.load_texture("images/tiles/river_tile.png")?);
	tile_textures.insert("║", texture_creator.load_texture("images/tiles/river_vertical.png")?);
	tile_textures.insert("^", texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	tile_textures.insert("v", texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	tile_textures.insert(">", texture_creator.load_texture("images/tiles/river_end_right.png")?);
	tile_textures.insert("<", texture_creator.load_texture("images/tiles/river_end_left.png")?);
	//Bases
	tile_textures.insert("b", texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	tile_textures.insert("1", texture_creator.load_texture("images/tiles/red_castle.png")?);
	tile_textures.insert("2", texture_creator.load_texture("images/tiles/blue_castle.png")?);
	//Tree
	tile_textures.insert("t", texture_creator.load_texture("images/tiles/tree_tile.png")?);

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

	//Stuff for banner that appears at beginning of each turn
	let mut turn_banner = Banner::new();
	let mut turn_text_textures: HashMap<&str, Texture> = HashMap::new();
	{
		turn_text_textures.insert("p1_banner", {
			let text_surface = core.bold_font.render("Player 1's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, turn_banner.current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		turn_text_textures.insert("p2_banner", {
			let text_surface = core.bold_font.render("Player 2's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, turn_banner.current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		turn_text_textures.insert("b_banner", {
			let text_surface = core.bold_font.render("Barbarians' Turn")
					.blended_wrapped(Color::RGBA(0,0,0, turn_banner.current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});
	}

	//Stuff for banner that appears when a team has won
	let mut winner_banner = Banner::new();
	let mut win_text_textures: HashMap<&str, Texture> = HashMap::new();
	{
		win_text_textures.insert("p1_banner", {
			let text_surface = core.bold_font.render("Player 1 wins!")
					.blended_wrapped(Color::RGBA(0,0,0, turn_banner.current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		win_text_textures.insert("p2_banner", {
			let text_surface = core.bold_font.render("Player 2 wins!")
					.blended_wrapped(Color::RGBA(0,0,0, turn_banner.current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});
	}

	//Collection of variables useful for handling map interaction & tile overlays
	let mut game_map = GameMap::new();
	//Set up the HashMap of Tiles that can be interacted with
	//Ideally this would be handled by the GameMap struct
	{
		let mut x = 0;
		let mut y = 0;
		for row in map_string.iter() {
			for col in row.iter() { 
				let letter = &col[..];
				match letter {
					"║" | "^" | "v" | "<" | "=" | ">" | "t" => game_map.map_tiles.insert((x,y), Tile::new(x, y, false, true, None, None, tile_textures.get(&letter).unwrap())),
					" " => game_map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, None, tile_textures.get(&letter).unwrap())),
					"b" =>  { 
								camp_coords.push((y,x));
								game_map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), tile_textures.get(&letter).unwrap()))
							},
					"_" => game_map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::Camp), tile_textures.get(&letter).unwrap())),
					"1" =>  {
								player_castle = (y, x);
								game_map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::PCastle), tile_textures.get(&letter).unwrap()))
							},
					"2" =>  {
								enemy_castle = (y, x);
								game_map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, Some(Structure::ECastle), tile_textures.get(&letter).unwrap()))
							},
					  _ => game_map.map_tiles.insert((x,y), Tile::new(x, y, false, false, None, None, tile_textures.get(&letter).unwrap())),
				};
				y += 1;
			}
			x += 1;
			y = 0;
		}
	}

	//Collection of variables useful for determining player's current state
	let mut player_state = PlayerState::new();

	//Cursor that appears when you hover over one of your units
	let cursor_texture = texture_creator.load_texture("images/interface/cursor.png")?;
	let mut cursor = Cursor::new(&cursor_texture);

	player_state.p1_units = HashMap::new();
	let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (8,46)), ('l', (10,45)), ('l', (10,53)), ('l', (12,46)), ('l', (17,51)), ('l', (17,55)), ('l', (18,53)), ('r', (9,49)), ('r', (10,46)), ('r', (13,50)), ('r', (14,54)), ('r', (16,53)), ('m', (10,50)), ('m', (10,52)), ('m', (11,53)), ('m', (13,53)));
	//let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (14, 40))); //Spawns a player unit right next to some barbarians
	prepare_player_units(&mut player_state.p1_units, Team::Player, p1_units_abrev, &unit_textures, &mut game_map.map_tiles);

	let mut p2_units: HashMap<(u32, u32), Unit> = HashMap::new();
	let p2_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (46,8)), ('l', (45,10)), ('l', (53,10)), ('l', (46,12)), ('l', (51,17)), ('l', (55,17)), ('l', (53,18)), ('r', (49,9)), ('r', (47,10)), ('r', (50,13)), ('r', (54,14)), ('r', (53,16)), ('m', (50,10)), ('m', (52,10)), ('m', (53,11)), ('m', (53,13)));
	prepare_player_units(&mut p2_units, Team::Enemy, p2_units_abrev, &unit_textures, &mut game_map.map_tiles);

	let mut barbarian_units: HashMap<(u32, u32), Unit> = HashMap::new();
	let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (4,6)), ('l', (6,8)), ('l', (7,7)), ('l', (9,7)), ('r', (6,6)), ('r', (8,5)), ('l', (55,55)), ('l', (59,56)), ('l', (56,56)), ('l', (54,57)), ('r', (56,59)), ('r', (57,57)), ('l', (28,15)), ('l', (29,10)), ('l', (31,12)), ('l', (31,17)), ('l', (32,11)), ('l', (35,15)), ('l', (34,11)), ('r', (31,10)), ('r', (33,9)), ('r', (30,8)), ('r', (36,10)), ('l', (28,52)), ('l', (28,48)), ('l', (33,51)), ('l', (31,46)), ('l', (31,52)), ('l', (35,52)), ('l', (35,48)), ('r', (32,53)), ('r', (33,56)), ('r', (30,54)), ('r', (34,54)), ('l', (17,38)), ('l', (16,37)), ('r', (23,36)), ('r', (18,30)), ('l', (46,25)), ('l', (47,26)), ('r', (40,27)), ('r', (45,33)),);
	//let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (32, 60))); //Spawns a single barbarian near the bottom of the map
	prepare_player_units(&mut barbarian_units, Team::Barbarians, barb_units_abrev, &unit_textures, &mut game_map.map_tiles);
	
	let unit_interface_texture = texture_creator.load_texture("images/interface/unit_interface.png")?;
	let mut unit_interface: Option<UnitInterface> = None;

	// Do this right before the game starts so that player 1 starts
	initialize_next_turn(&mut player_state.p1_units);

	let mut current_player = Team::Player;

	//Button for player to end their turn
    let mut end_turn_button = Button::new(core, Rect::new((CAM_W - 240).try_into().unwrap(), (CAM_H - 90).try_into().unwrap(), 200, 50), "End Turn")?;

	//Winning team. Is set to None until one of the Teams wins
	let mut winning_team: Option<Team> = None;
	let mut player1_on_base = 0;
	let mut player2_on_base = 0;
	// Not sure how else to check on_base once per turn
	let mut next_team_check = Team::Player;
	
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
		input.update(&core.event_pump);

		//Camera controls should stay enabled even when it is not the player's turn,
		//which is why this code block is not in player_turn.rs
		if input.mouse_state.right() && !turn_banner.banner_visible{
			if old_mouse_x < 0 || old_mouse_y < 0 {
				old_mouse_x = input.mouse_state.x();
				old_mouse_y = input.mouse_state.y();
			}
			core.cam.x = (core.cam.x - (old_mouse_x - input.mouse_state.x())).clamp(-core.cam.w + core.wincan.window().size().0 as i32, 0);
			core.cam.y = (core.cam.y - (old_mouse_y - input.mouse_state.y())).clamp(-core.cam.h + core.wincan.window().size().1 as i32, 0);
			
			old_mouse_x = input.mouse_state.x();
			old_mouse_y = input.mouse_state.y();
		}
		else {
			old_mouse_y = -1;
			old_mouse_x = -1;
		}

		let (i, j) = PixelCoordinates::matrix_indices_from_pixel(
            input.mouse_state.x().try_into().unwrap(), 
            input.mouse_state.y().try_into().unwrap(), 
            (-1 * core.cam.x).try_into().unwrap(), 
            (-1 * core.cam.y).try_into().unwrap()
        );

		match player_state.p1_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {
				cursor.hide_cursor();
			},
		}
		match p2_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {},
		}
		match barbarian_units.get_mut(&(j,i)) {
			Some(active_unit) => {
				cursor.set_cursor(&PixelCoordinates::from_matrix_indices(i, j), &active_unit);
			},
			_ => {},
		}

		//Handle the current team's move
		match current_player {
			Team::Player => {
				player_turn::handle_player_turn(&core, &mut player_state, &mut p2_units, &mut barbarian_units, &mut game_map, &input, &mut turn_banner, &mut unit_interface, &unit_interface_texture, &mut current_player, &mut cursor, &mut end_turn_button)?;
			},
			Team::Enemy => {
				enemy_turn::handle_enemy_turn(&core, &mut p2_units, &mut player_state.p1_units, &mut barbarian_units, &mut game_map, &mut turn_banner, &mut current_player, &enemy_castle, &player_castle, &camp_coords);
			},
			Team::Barbarians => {
				barbarian_turn::handle_barbarian_turn(&core, &mut barbarian_units, &mut player_state.p1_units, &mut p2_units, &mut game_map, &mut turn_banner, &mut current_player)?;
			},

		}
		

		//Draw tiles & sprites
		for i in 0..map_height {
			for j in 0..map_width {
				let map_tile = map_string[i][j].as_ref();
				let map_tile_size = match map_tile {
					"b" => TILE_SIZE * 2,
					_ => TILE_SIZE,
				};

				let pixel_location = PixelCoordinates::from_matrix_indices(i as u32, j as u32);
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, map_tile_size, map_tile_size);

				//Draw map tile at this coordinate
				if let Some(map_tile) = game_map.map_tiles.get(&(i as u32, j as u32)) {
					core.wincan.copy(map_tile.texture, None, dest)?
				}

				//Use default sprite size for all non-map sprites
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);
				
				//Draw player unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let Some(mut unit) = player_state.p1_units.get_mut(&(j as u32, i as u32)) {
					unit.draw(core, &dest)?;
				}

				//Draw enemy unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let Some(mut enemy) = p2_units.get_mut(&(j as u32, i as u32)) {
					enemy.draw(core, &dest)?;
				}

				//Draw barbarian unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let Some(mut barbarian) = barbarian_units.get_mut(&(j as u32, i as u32)) {
					barbarian.draw(core, &dest)?;
				}
			}
		}

		match player_state.p1_units.get(&(player_state.active_unit_j as u32, player_state.active_unit_i as u32)) {
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
			//Draw the cursor
			cursor.draw(core)?;

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

			//Draw the button for the player to end their turn, relative to the camera
			end_turn_button.draw_relative(core)?;
		}

		//Draw banner that appears at beginning of turn
		if winning_team.is_none() {
			//As long as the banner isn't completely transparent, draw it
			if turn_banner.current_banner_transparency != 0 {
				turn_banner.banner_colors.a = turn_banner.current_banner_transparency;
				draw_banner(core, &turn_text_textures, turn_banner.banner_key, turn_banner.banner_colors)?;
			} else if turn_banner.banner_visible {
				turn_banner.banner_visible = false;
			}

			//The first time we draw the banner we need to keep track of when it first appears
			if turn_banner.current_banner_transparency == 250 {
				turn_banner.initial_banner_output = Instant::now();
				turn_banner.current_banner_transparency -= 25;
			}

			//After a set amount of seconds pass and if the banner is still visible, start to make the banner disappear
			if turn_banner.initial_banner_output.elapsed() >= Duration::from_millis(BANNER_TIMEOUT) && turn_banner.current_banner_transparency != 0 {
				turn_banner.current_banner_transparency -= 25;
			}
		}
		else {
			//Draw the winner's banner if someone has won
			//As long as the banner isn't completely transparent, draw it
			if winner_banner.current_banner_transparency != 0 {
				winner_banner.banner_colors.a = winner_banner.current_banner_transparency;
				draw_banner(core, &win_text_textures, winner_banner.banner_key, winner_banner.banner_colors)?;
			} else if winner_banner.banner_visible {
				winner_banner.banner_visible = false;
				
				//End the game by returning to the main menu
				return Ok(GameState::MainMenu);
			}

			//The first time we draw the banner we need to keep track of when it first appears
			if winner_banner.current_banner_transparency == 250 {
				winner_banner.initial_banner_output = Instant::now();
				winner_banner.current_banner_transparency -= 25;
			}

			//After a set amount of seconds pass and if the banner is still visible, start to make the banner disappear
			if winner_banner.initial_banner_output.elapsed() >= Duration::from_millis(BANNER_TIMEOUT) && winner_banner.current_banner_transparency != 0 {
				winner_banner.current_banner_transparency -= 25;
			}
		}
		
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

// Draws a banner at the center of the camera to signify whose turn it currently is
fn draw_banner(core: &mut SDLCore, text_textures: &HashMap<&str, Texture>, text_index: &str, rect_color: Color) -> Result< (), String> {
	let banner_rect = Rect::new(core.cam.x.abs(), core.cam.y.abs() + (360-64), CAM_W, 128);
	let text_rect = Rect::new(core.cam.x.abs() + (640-107), core.cam.y.abs() + (360-64), CAM_W/6, 128);
	core.wincan.set_blend_mode(BlendMode::Blend);
	core.wincan.set_draw_color(rect_color);
	core.wincan.draw_rect(banner_rect)?;
	core.wincan.fill_rect(banner_rect)?;

	match text_textures.get(text_index) {
		Some(texture) => core.wincan.copy(&texture, None, text_rect)?,
		None => {},
	};
	
	Ok(())
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
	let melee: &str;
	let range: &str;
	let mage: &str;
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
		match unit.0 {
			'l' => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 20, 6, 1, 95, 1, 5, unit_textures.get(melee).unwrap())),
			'r' => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 15, 5, 4, 85, 3, 7, unit_textures.get(range).unwrap())),
			 _ => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 10, 7, 3, 75,  5, 9, unit_textures.get(mage).unwrap())),
		};
	}
}

//Sets up the winner's banner so it can start displaying, and returns an Option containing the Team corresponding to the winning team
pub fn set_winner(winner: Team, winner_banner: &mut Banner) -> Option<Team> {
	//Set up the winner's banner
	winner_banner.current_banner_transparency = 250;
	winner_banner.banner_visible = true;

	match winner {
		Team::Player => {
			println!("Player 1 wins!");
			winner_banner.banner_key = "p1_banner";
			winner_banner.banner_colors = Color::RGBA(0, 89, 178, winner_banner.current_banner_transparency);
		},
		Team::Enemy => {
			println!("Enemy wins!");
			winner_banner.banner_key = "p2_banner";
			winner_banner.banner_colors = Color::RGBA(207, 21, 24, winner_banner.current_banner_transparency);
		},
		Team::Barbarians => {
			println!("Barbarians win!");
			winner_banner.banner_key = "b_banner";
			winner_banner.banner_colors = Color::RGBA(163, 96, 30, winner_banner.current_banner_transparency);
		},
	};

	return Some(winner);
}