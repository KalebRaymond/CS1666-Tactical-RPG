use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;

//For accessing map file and reading lines
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Instant, Duration};

use crate::game_map::GameMap;
use crate::GameState;
use crate::{TILE_SIZE, CAM_W, CAM_H};
use crate::input::Input;
use crate::pixel_coordinates::PixelCoordinates;
use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::player_turn;
use crate::barbarian_turn;
use crate::SDLCore;
use crate::tile::Tile;
use crate::turn_banner::TurnBanner;
use crate::unit_interface::UnitInterface;
use crate::unit::{Team, Unit};

const BANNER_TIMEOUT: u64 = 1500;

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	let texture_creator = core.wincan.texture_creator();
	
	//Stuff for banner that appears at beginning of each turn
	let mut turn_banner = TurnBanner::new();

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

	let mut text_textures: HashMap<&str, Texture> = HashMap::new();
	{
		text_textures.insert("p1_banner", {
			let text_surface = core.bold_font.render("Player 1's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, turn_banner.current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("p2_banner", {
			let text_surface = core.bold_font.render("Player 2's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, turn_banner.current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("b_banner", {
			let text_surface = core.bold_font.render("Barbarians' Turn")
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
					//I wanted to do something that wasn't just hardcoding all the textures, but it seems that tile_textures.get() refuses anything that isn't a hard-coded string
					"║" | "^" | "v" | "<" | "=" | ">" | "t" => game_map.map_tiles.insert((x,y), Tile::new(x, y, false, true, None, tile_textures.get(&letter).unwrap())),
					"b" | "1" | "2" | " " | "_" => game_map.map_tiles.insert((x,y), Tile::new(x, y, true, true, None, tile_textures.get(&letter).unwrap())),
					_ => game_map.map_tiles.insert((x,y), Tile::new(x, y, false, false, None, tile_textures.get(&letter).unwrap())),
				};
				y += 1;
			}
			x += 1;
			y = 0;
		}
	}

	//Collection of variables useful for determining player's current state
	let mut player_state = PlayerState::new();

	player_state.p1_units = HashMap::new();
	let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (8,46)), ('l', (10,45)), ('l', (10,53)), ('l', (12,46)), ('l', (17,51)), ('l', (17,55)), ('l', (18,53)), ('r', (9,49)), ('r', (10,46)), ('r', (13,50)), ('r', (14,54)), ('r', (16,53)), ('m', (10,50)), ('m', (10,52)), ('m', (11,53)), ('m', (13,53)));
	prepare_player_units(&mut player_state.p1_units, Team::Player, p1_units_abrev, &unit_textures, &mut game_map.map_tiles);

	let mut p2_units: HashMap<(u32, u32), Unit> = HashMap::new();
	let p2_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (46,8)), ('l', (45,10)), ('l', (53,10)), ('l', (46,12)), ('l', (51,17)), ('l', (55,17)), ('l', (53,18)), ('r', (49,9)), ('r', (47,10)), ('r', (50,13)), ('r', (54,14)), ('r', (53,16)), ('m', (50,10)), ('m', (52,10)), ('m', (53,11)), ('m', (53,13)));
	prepare_player_units(&mut p2_units, Team::Enemy, p2_units_abrev, &unit_textures, &mut game_map.map_tiles);

	let mut barbarian_units: HashMap<(u32, u32), Unit> = HashMap::new();
	let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (4,6)), ('l', (6,8)), ('l', (7,7)), ('l', (9,7)), ('r', (6,6)), ('r', (8,5)), ('l', (55,55)), ('l', (59,56)), ('l', (56,56)), ('l', (54,57)), ('r', (56,59)), ('r', (57,57)), ('l', (28,15)), ('l', (29,10)), ('l', (31,12)), ('l', (31,17)), ('l', (32,11)), ('l', (35,15)), ('l', (34,11)), ('r', (31,10)), ('r', (33,9)), ('r', (30,8)), ('r', (36,10)), ('l', (28,52)), ('l', (28,48)), ('l', (33,51)), ('l', (31,46)), ('l', (31,52)), ('l', (35,52)), ('l', (35,48)), ('r', (32,53)), ('r', (33,56)), ('r', (30,54)), ('r', (34,54)),);
	prepare_player_units(&mut barbarian_units, Team::Barbarians, barb_units_abrev, &unit_textures, &mut game_map.map_tiles);
	
	let unit_interface_texture = texture_creator.load_texture("images/interface/unit_interface.png")?;
	let mut unit_interface: Option<UnitInterface> = None;

	// Do this right before the game starts so that player 1 starts
	player_state.p1_units = initialize_next_turn(player_state.p1_units);
	
	let mut current_player = Team::Player;
	
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
		//which is why this code block is not in the player's match statement below
		if input.mouse_state.right() && !turn_banner.banner_visible{
			if old_mouse_x < 0 || old_mouse_y < 0 {
				old_mouse_x = input.mouse_state.x();
				old_mouse_y = input.mouse_state.y();
			}
			core.cam.x = (core.cam.x - (old_mouse_x - input.mouse_state.x())).clamp(-core.cam.w + core.wincan.window().size().0 as i32, 0);
			core.cam.y = (core.cam.y - (old_mouse_y - input.mouse_state.y())).clamp(-core.cam.h + core.wincan.window().size().1 as i32, 0,);
			
			old_mouse_x = input.mouse_state.x();
			old_mouse_y = input.mouse_state.y();
		}
		else {
			old_mouse_y = -1;
			old_mouse_x = -1;
		}

		//Handle the current team's move
		match current_player {
			Team::Player => {
				player_turn::handle_player_turn(&core, &mut player_state, &mut game_map, &input, &mut turn_banner, &mut unit_interface, &unit_interface_texture, &mut current_player);
			},
			Team::Enemy => {
				if !turn_banner.banner_visible {
					//End turn
					current_player = Team::Barbarians;

					//Start displaying the barbarians' banner
					turn_banner.current_banner_transparency = 250;
					turn_banner.banner_colors = Color::RGBA(163,96,30, turn_banner.current_banner_transparency);
					turn_banner.banner_key = "b_banner";
					turn_banner.banner_visible = true;
				}
			},
			Team::Barbarians => {
				barbarian_turn::handle_barbarian_turn(&mut barbarian_units, &mut game_map, &mut turn_banner, &mut current_player);
				player_state.p1_units = initialize_next_turn(player_state.p1_units);
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
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);
				//Draw player unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let Some(unit) = player_state.p1_units.get(&(j as u32, i as u32)) {
					core.wincan.copy(unit.texture, None, dest)?
				}

				//Draw enemy unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let Some(enemy) = p2_units.get(&(j as u32, i as u32)) {
					core.wincan.copy(enemy.texture, None, dest)?
				}

				//Draw barbarian unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let Some(barbarian) = barbarian_units.get(&(j as u32, i as u32)) {
					core.wincan.copy(barbarian.texture, None, dest)?
				}
			}
		}

		//Draw banner that appears at beginning of turn
		{
			//As long as the banner isn't completely transparent, draw it
			if turn_banner.current_banner_transparency != 0 {
				turn_banner.banner_colors.a = turn_banner.current_banner_transparency;
				draw_player_banner(core, &text_textures, turn_banner.banner_key, turn_banner.banner_colors)?;
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
		
		core.wincan.set_viewport(core.cam);
		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}

// Function that takes a HashMap of units and sets all has_attacked and has_moved to false so that they can move again
fn initialize_next_turn(mut team_units: HashMap<(u32, u32), Unit>) -> HashMap<(u32, u32), Unit>{
	for unit in &mut team_units.values_mut() {
		unit.next_turn();
	}
	team_units
}

// Draws a banner at the center of the camera to signify whose turn it currently is
fn draw_player_banner(core: &mut SDLCore, text_textures: &HashMap<&str, Texture>, text_index: &str, rect_color: Color) -> Result< (), String> {
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
			'l' => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 20, 4, 1, 90, 5, unit_textures.get(melee).unwrap())),
			'r' => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 15, 2, 4, 70, 7, unit_textures.get(range).unwrap())),
			 _ => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, player_team, 10, 3, 3, 60, 9, unit_textures.get(mage).unwrap())),
		};
	}
}