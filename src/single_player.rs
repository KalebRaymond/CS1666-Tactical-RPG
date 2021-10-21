use std::collections::HashSet;
use std::convert::TryInto;
use std::time::{Instant, Duration};

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseState;
use sdl2::rect::Rect;
use sdl2::render::Texture;

//For accessing map file and reading lines
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::GameState;
use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::{TILE_SIZE, CAM_W, CAM_H};
use crate::unit_interface::UnitInterface;

use crate::unit::{Team, Unit};
use crate::tile::{Tile};

const BANNER_TIMEOUT: u64 = 2500;

pub fn single_player(core: &mut SDLCore) -> Result<GameState, String> {
	let texture_creator = core.wincan.texture_creator();
	
	//Stuff for banner that appears at beginning of each turn
	let mut current_player = Team::Player;
	let mut banner_key = "p1_banner";
	let mut current_banner_transparency = 250;
	let mut banner_colors = Color::RGBA(0, 89, 178, current_banner_transparency);
	let mut initial_banner_output = Instant::now();
	let mut banner_visible = true;

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
	
	//Left mouse button state. If true, then the left mouse button was clicked on the current frame
	let mut left_clicked = false; 

	//Creates map from file
	let map_string: Vec<Vec<String>> = map_data.lines()
		.take(map_width)
		.map(|x| x.unwrap().chars().collect::<Vec<char>>())
		.map(|x| x.chunks(2).map(|chunk| chunk[0].to_string()).collect())
		.collect();

	//Load map textures
	let mut tile_textures: HashMap<&str, Texture> = HashMap::new();
	// Mountains
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
	// Grass
	tile_textures.insert(" ", texture_creator.load_texture("images/tiles/grass_tile.png")?);
	tile_textures.insert("_", texture_creator.load_texture("images/tiles/empty_tile.png")?);
	// Rivers
	tile_textures.insert("=", texture_creator.load_texture("images/tiles/river_tile.png")?);
	tile_textures.insert("║", texture_creator.load_texture("images/tiles/river_vertical.png")?);
	tile_textures.insert("^", texture_creator.load_texture("images/tiles/river_end_vertical_top.png")?);
	tile_textures.insert("v", texture_creator.load_texture("images/tiles/river_end_vertical_bottom.png")?);
	tile_textures.insert(">", texture_creator.load_texture("images/tiles/river_end_right.png")?);
	tile_textures.insert("<", texture_creator.load_texture("images/tiles/river_end_left.png")?);
	// Bases
	tile_textures.insert("b", texture_creator.load_texture("images/tiles/barbarian_camp.png")?);
	tile_textures.insert("1", texture_creator.load_texture("images/tiles/red_castle.png")?);
	tile_textures.insert("2", texture_creator.load_texture("images/tiles/blue_castle.png")?);
	// Tree
	tile_textures.insert("t", texture_creator.load_texture("images/tiles/tree_tile.png")?);

	//Load unit textures
	let mut unit_textures: HashMap<&str, Texture> = HashMap::new();
	unit_textures.insert("pll", texture_creator.load_texture("images/units/player1_melee.png")?);
	unit_textures.insert("plr", texture_creator.load_texture("images/units/player1_archer.png")?);
	unit_textures.insert("plm", texture_creator.load_texture("images/units/player1_mage.png")?);
	unit_textures.insert("bl", texture_creator.load_texture("images/units/barbarian_melee.png")?);
	unit_textures.insert("br", texture_creator.load_texture("images/units/barbarian_archer.png")?);

	let mut text_textures: HashMap<&str, Texture> = HashMap::new();
	{
		let bold_font = core.ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?;
		text_textures.insert("p1_banner", {
			let text_surface = bold_font.render("Player 1's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("p2_banner", {
			let text_surface = bold_font.render("Player 2's Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});

		text_textures.insert("b_banner", {
			let text_surface = bold_font.render("Barbarians' Turn")
					.blended_wrapped(Color::RGBA(0,0,0, current_banner_transparency), 320) //Black font
					.map_err(|e| e.to_string())?;

			texture_creator.create_texture_from_surface(&text_surface)
					.map_err(|e| e.to_string())?
		});
	}

	// Sets up the HashMap of Tiles that can be interacted with
	let mut map_tiles: HashMap<(u32, u32), Tile> = HashMap::new();
	{
		let mut x = 0;
		let mut y = 0;
		for row in map_string.iter() {
			for col in row.iter() { 
				let letter = &col[..];
				match letter {
					// I wanted to do something that wasn't just hardcoding all the textures, but it seems that tile_textures.get() refuses anything that isn't a hard-coded string
					"║" | "^" | "v" | "<" | "=" | ">" | "t" => map_tiles.insert((x,y), Tile::new(x, y, false, true, None, tile_textures.get(&letter).unwrap())),
					"b" | "1" | "2" | " " => map_tiles.insert((x,y), Tile::new(x, y, true, true, None, tile_textures.get(&letter).unwrap())),
					_ => map_tiles.insert((x,y), Tile::new(x, y, false, false, None, tile_textures.get(&letter).unwrap())),
				};
				y += 1;
			}
			x += 1;
			y = 0;
		}
	}

	let mut p1_units: HashMap<(u32, u32), Unit> = HashMap::new();
	let p1_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (0,0)), ('l', (3,3)), ('l', (4,5)));
	prepare_player_units(&mut p1_units, Team::Player, p1_units_abrev, &unit_textures, &mut map_tiles);

	let mut p2_units: HashMap<(u32, u32), Unit> = HashMap::new();
	let mut barbarian_units: HashMap<(u32, u32), Unit> = HashMap::new();
	let barb_units_abrev: Vec<(char, (u32,u32))> = vec!(('l', (7,7)), ('l', (4,6)));
	prepare_player_units(&mut barbarian_units, Team::Barbarians, barb_units_abrev, &unit_textures, &mut map_tiles);
	
	let unit_interface_texture = texture_creator.load_texture("images/interface/unit_interface.png")?;
	let mut unit_interface: Option<UnitInterface> = None;

	'gameloop: loop {
		core.wincan.clear();

		//Check if user tried to quit the program
		for event in core.event_pump.poll_iter() {
			match event {
				Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => break 'gameloop,
				_ => {},
			}
		}

		//Mouse Controls
		{
			let mouse_state: MouseState = core.event_pump.mouse_state();
			//Check right mouse button. Camera controls should stay enabled even when it is not the player's turn
			if mouse_state.right() && !banner_visible{
				if old_mouse_x < 0 || old_mouse_y < 0 {
					old_mouse_x = mouse_state.x();
					old_mouse_y = mouse_state.y();
				}
				core.cam.x = (core.cam.x - (old_mouse_x - mouse_state.x())).clamp(-core.cam.w + core.wincan.window().size().0 as i32, 0);
				core.cam.y = (core.cam.y - (old_mouse_y - mouse_state.y())).clamp(-core.cam.h + core.wincan.window().size().1 as i32, 0,);
				
				old_mouse_x = mouse_state.x();
				old_mouse_y = mouse_state.y();
			}
			else {
				old_mouse_y = -1;
				old_mouse_x = -1;
			}

			//Check left mouse button
			if mouse_state.left() {
				if  !left_clicked {
					left_clicked = true;

					//Get map matrix indices from mouse position
					let (i, j) = PixelCoordinates::matrix_indices_from_pixel(	mouse_state.x().try_into().unwrap(), 
																				mouse_state.y().try_into().unwrap(), 
																				(-1 * core.cam.x).try_into().unwrap(), 
																				(-1 * core.cam.y).try_into().unwrap()
																			);

					unit_interface = match p1_units.get(&(j,i)) {
						Some(_) => { Some(UnitInterface::new(i, j, vec!["Move","Attack"], &unit_interface_texture)) },
						_ => { None },
					}
				}
			}
			else {
				left_clicked = false;
			}
		}

		//Record key inputs
		let keystate: HashSet<Keycode> = core.event_pump
		.keyboard_state()
		.pressed_scancodes()
		.filter_map(Keycode::from_scancode)
		.collect();

		//Handle the current team's move
		match current_player {
			Team::Player => {
				if !banner_visible {
					if keystate.contains(&Keycode::Backspace) {
						//End turn
						current_player = Team::Enemy;

						//Start displaying the enemy's banner
						current_banner_transparency = 250;
						banner_colors = Color::RGBA(207, 21, 24, current_banner_transparency);
						banner_key = "p2_banner";
						banner_visible = true;
					}
				}
			},
			Team::Enemy => {
				if !banner_visible {
					//End turn
				current_player = Team::Barbarians;

				//Start displaying the barbarians' banner
				current_banner_transparency = 250;
				banner_colors = Color::RGBA(163,96,30, current_banner_transparency);
				banner_key = "b_banner";
				banner_visible = true;
				}
			},
			Team::Barbarians => {
				if !banner_visible {
					//End turn
					current_player = Team::Player;

					//Start displaying Player 1's banner
					current_banner_transparency = 250;
					banner_colors = Color::RGBA(0, 89, 178, current_banner_transparency);
					banner_key = "p1_banner";
					banner_visible = true;
				}
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
				if let std::collections::hash_map::Entry::Occupied(entry) = map_tiles.entry((i as u32, j as u32)) {
					core.wincan.copy(entry.get().texture, None, dest)?
				}
				//Draw unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let std::collections::hash_map::Entry::Occupied(entry) = p1_units.entry((j as u32, i as u32)) {
					core.wincan.copy(entry.get().texture, None, dest)?
				}
				//Draw unit at this coordinate (Don't forget i is y and j is x because 2d arrays)
				if let std::collections::hash_map::Entry::Occupied(entry) = barbarian_units.entry((j as u32, i as u32)) {
					core.wincan.copy(entry.get().texture, None, dest)?
				}
			}
		}

		//Draw banner that appears at beginning of turn
		{
			//As long as the banner isn't completely transparent, draw it
			if current_banner_transparency != 0 {
				banner_colors.a = current_banner_transparency;
				draw_player_banner(core, &text_textures, banner_key, banner_colors)?;
			} else if banner_visible {
				banner_visible = false;
			}

			//The first time we draw the banner we need to keep track of when it first appears
			if current_banner_transparency == 250 {
				initial_banner_output = Instant::now();
				current_banner_transparency -= 25;
			}

			//After a set amount of seconds pass and if the banner is still visible, start to make the banner disappear
			if initial_banner_output.elapsed() >= Duration::from_millis(BANNER_TIMEOUT) && current_banner_transparency != 0 {
				current_banner_transparency -= 25;
			}
		}
		
		match &unit_interface {
			Some(ui) => {
				ui.draw(core);
			},
			_ => {},
		}

		core.wincan.set_viewport(core.cam);
		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}

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

//Method for preparing the HashMap of player units whilst also properly marking them in the map
//l melee r ranged m mage
fn prepare_player_units<'a, 'b> (player_units: &mut HashMap<(u32, u32), Unit<'a>>, player_team: Team, units: Vec<(char, (u32, u32))>, unit_textures: &'a HashMap<&str, Texture<'a>>, map: &'b mut HashMap<(u32, u32), Tile>) {
	let melee: &str;
	let range: &str;
	let mage: &str;
	let (melee, range, mage)  = match player_team {
		Team::Player => ("pll", "plr", "plm"),
		Team::Enemy =>  ("pll", "plr", "plm"),
		Team::Barbarians => ("bl", "br", ""),
	};
	for unit in units {
		match player_team {
			Team::Player => map.get_mut(&(unit.1.0, unit.1.1)).unwrap().update_team(Some(Team::Player)),
			Team::Enemy => map.get_mut(&(unit.1.0, unit.1.1)).unwrap().update_team(Some(Team::Player)),
			Team::Barbarians => map.get_mut(&(unit.1.0, unit.1.1)).unwrap().update_team(Some(Team::Player)),
		}
		match unit.0 {
			'l' => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, Team::Player, 20, 6, 2, 90, 5, unit_textures.get(melee).unwrap())),
			'r' => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, Team::Player, 15, 5, 4, 70, 7, unit_textures.get(range).unwrap())),
			 _ => player_units.insert((unit.1.0, unit.1.1), Unit::new(unit.1.0, unit.1.1, Team::Player, 10, 4, 3, 60, 9, unit_textures.get(mage).unwrap())),
		};
	}
}
