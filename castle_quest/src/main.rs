extern crate sdl2;

const TITLE: &str = "Castle Quest";
const CAM_W: u32 = 1280;
const CAM_H: u32 = 720;
const TILE_SIZE: u32 = 32;

use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

#[macro_use] mod sdl_macros;

mod credits;
mod pixel_coordinates;

use pixel_coordinates::PixelCoordinates;

pub enum GameState {
	MainMenu,
	SinglePlayer,
	MultiPlayer,
	Credits,
	Quit,
}

pub struct SDLCore {
	pub sdl_ctx: sdl2::Sdl,
	pub ttf_ctx: sdl2::ttf::Sdl2TtfContext,
	pub wincan: sdl2::render::WindowCanvas,
	pub event_pump: sdl2::EventPump,
	pub cam: Rect,
	pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>
}

fn runner(vsync:bool) {
	println!("\nRunning {}:", TITLE);
	print!("\tInitting...");
	match init_sdl_core(vsync) {
		Err(e) => println!("\n\t\tFailed to init: {}", e),
		Ok(mut core) => {
			println!("DONE");
			print!("\tRunning...");

			//Start the game in the Single Player mode
			let mut game_state = GameState::SinglePlayer;

			loop {
				match run_game_state(&mut core, &game_state) {
					Err(e) => {
						println!("\n\t\tEncountered error while running: {}", e)
					},
					Ok(next_game_state) => {
						match next_game_state {
							GameState::Quit => break,
							_ => { game_state = next_game_state; },
						}
					},
				};
			}

			println!("DONE\nExiting cleanly");
		},
	};
}

fn run_game_state(core: &mut SDLCore, game_state: &GameState) -> Result<GameState, String> {
	let next_game_state = match game_state {
		GameState::SinglePlayer => run_single_player(core)?,
		GameState::Credits => credits::credits(core)?,
		GameState::Quit => GameState::Quit,
		_ => return Err("Invalid game state".to_string()),
	};

	Ok(next_game_state)
}

fn init_sdl_core(vsync:bool) -> Result<SDLCore, String> {
	let sdl_ctx = sdl2::init()?;
	let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;
	let video_subsys = sdl_ctx.video()?;

	let window = video_subsys.window(TITLE, CAM_W, CAM_H)
		.build()
		.map_err(|e| e.to_string())?;

	let wincan = window.into_canvas().accelerated();

	// Check if we should lock to vsync
	let wincan = if vsync {
		wincan.present_vsync()
	}
	else {
		wincan
	};

	let wincan = wincan.build()
		.map_err(|e| e.to_string())?;

	let event_pump = sdl_ctx.event_pump()?;

	let cam = Rect::new(0, 0, CAM_W, CAM_H);

	let texture_creator = wincan.texture_creator();

	let core = SDLCore{
		sdl_ctx,
		ttf_ctx,
		wincan,
		event_pump,
		cam,
		texture_creator,
	};

	Ok(core)
}

fn run_single_player(core: &mut SDLCore) -> Result<GameState, String> {
	//Basic mock map, 48x48 2d vector filled with 1s
	let mut map: Vec<Vec<u32>> = vec![vec![1; 48]; 48];
	let map_width = map[0].len();
	let map_height = map.len();

	'gameloop: loop {
		core.wincan.clear();

		for event in core.event_pump.poll_iter() {
			match event {
				Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => break 'gameloop,
				_ => {},
			}
		}

		//Draw tiles & sprites
		for i in 0..map_height {
			for j in 0..map_width {
				let pixel_location = PixelCoordinates::from_matrix_indices(i as u32, j as u32);
				let dest = Rect::new(pixel_location.x as i32, pixel_location.y as i32, TILE_SIZE, TILE_SIZE);

				//Draw map tile at this coordinate
				let map_tile_texture = match map[i][j] {
					1 => core.texture_creator.load_texture("images/grass_tile.png")?,
					_ => return Err("Invalid map tile id".to_string()),
				};

				core.wincan.copy(&map_tile_texture, None, dest)?; 
			}
		}

		core.wincan.present();
	}

	//Single player finished running cleanly, automatically quit game
	Ok(GameState::Quit)
}

fn main() {
	runner(true);
}