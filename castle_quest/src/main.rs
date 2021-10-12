extern crate sdl2;

const TITLE: &str = "Castle Quest";
const CAM_W: u32 = 1280;
const CAM_H: u32 = 720;
pub const TILE_SIZE: u32 = 32;

use sdl2::rect::Rect;
use sdl2::render::Texture;

#[macro_use] mod sdl_macros;

mod credits;
mod main_menu;
mod pixel_coordinates;
mod single_player;

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
}

fn runner(vsync:bool) {
	println!("\nRunning {}:", TITLE);
	print!("\tInitting...");
	match init_sdl_core(vsync) {
		Err(e) => println!("\n\t\tFailed to init: {}", e),
		Ok(mut core) => {
			println!("DONE");
			print!("\tRunning...");

			//Start the game in the menu
			let mut game_state = GameState::MainMenu;

			loop {
				match run_game_state(&mut core, &game_state) {
					Err(e) => {
						panic!("\n\t\tEncountered error while running: {}", e);
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
		GameState::MainMenu => main_menu::main_menu(core)?,
		GameState::SinglePlayer => single_player::single_player(core)?,
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

	let core = SDLCore{
		sdl_ctx,
		ttf_ctx,
		wincan,
		event_pump,
		cam,
	};

	Ok(core)
}

fn main() {
	runner(true);
}