extern crate sdl2;

#[macro_use] mod sdl_macros;

mod barbarian_turn;
mod credits;
mod cursor;
mod damage_indicator;
mod game_map;
mod input;
mod main_menu;
mod multiplayer_menu;
mod net;
mod pixel_coordinates;
mod player_action;
mod player_state;
mod player_turn;
mod single_player;
mod turn_banner;
mod unit_interface;
pub mod button;
pub mod tile;
pub mod unit;

use std::env;
use std::path::Path;
use std::time::Instant;

use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::mixer::{InitFlag, AUDIO_S32SYS, DEFAULT_CHANNELS};

use crate::net::client::Client;
use crate::main_menu::MainMenu;
use crate::multiplayer_menu::MultiplayerMenu;

const TITLE: &str = "Castle Quest";
const CAM_W: u32 = 1280;
const CAM_H: u32 = 720;
pub const TILE_SIZE: u32 = 32;

pub enum GameState {
	MainMenu,
	SinglePlayer,
	MultiPlayer,
	Credits,
	Quit,
}

pub struct SDLCore<'t> {
	pub sdl_ctx: sdl2::Sdl,
	pub bold_font: sdl2::ttf::Font<'t, 't>,
	pub regular_font: sdl2::ttf::Font<'t, 't>,
	pub tiny_font: sdl2::ttf::Font<'t, 't>,
	pub wincan: sdl2::render::WindowCanvas,
	pub texture_creator: &'t TextureCreator<sdl2::video::WindowContext>,
	pub event_pump: sdl2::EventPump,
	pub cam: Rect,
}

fn runner(vsync:bool) -> Result<(), String> {
	println!("\tRunning...");

	// ----- Initialize SDLCore -----
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

	let bold_font = ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?; //From https://www.fontsquirrel.com/fonts/open-sans
	let regular_font = ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 16)?; //From https://www.fontsquirrel.com/fonts/open-sans
	let tiny_font = ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 12)?; //From https://www.fontsquirrel.com/fonts/open-sans

	let mut core = SDLCore{
		sdl_ctx,
		bold_font,
		regular_font,
		tiny_font,
		wincan,
		texture_creator: &texture_creator,
		event_pump,
		cam,
	};

	// ----- Start the game loop in the menu -----
	let mut game_state = GameState::MainMenu;

	loop {
		game_state = match game_state {
			GameState::Quit => break,
			_ => run_game_state(&mut core, &game_state)?,
		}
	}

	println!("DONE\nExiting cleanly");
	Ok(())
}

fn run_game_state<'i, 'r>(core: &'i mut SDLCore<'r>, game_state: &GameState) -> Result<GameState, String> {
	let state = match game_state {
		// This is a specific loop implementation for the MainMenu struct, as it is the only scene with this structure
		// - this function could be abstracted as the other files are refactored; ideally, `GameState` is a Option<Fn(SDLCore) -> Scene>, where Scene is a trait impemented for every scene struct
		// - this way, `main_menu` can choose to `return Ok(SinglePlayer::init);`, which this function will use to initialize the singleplayer scene and move its iteration to that function's return value
		GameState::MainMenu => {
			let mut scene_menu = MainMenu::new(core)?;

			// background music for main menu
			// - This had to be moved into a scope that exists outside of both the `main_menu.rs` functions as it needs a persistent lifetime (otherwise it segfaults)
			// - in the future, this should be abstracted; we could create a `sound_queue: Vec<Path>` in SDLCore that any function can push to in order to play sounds
			sdl2::mixer::open_audio(44100, AUDIO_S32SYS, DEFAULT_CHANNELS, 1024)?;
			let _mixer_filetypes = sdl2::mixer::init(InitFlag::MP3)?;
			let bg_music = sdl2::mixer::Music::from_file(Path::new("./music/main_menu.mp3"))?;

			bg_music.play(-1)?;

			loop {
				let state = scene_menu.draw()?;
				match state {
					GameState::MainMenu => continue,
					_ => return Ok(state),
				}
			}
		},
		GameState::SinglePlayer => single_player::single_player(core)?,
		GameState::MultiPlayer => {
			let client = Client::new()?;
			
			// PSEUDOCODE:
			// 1. Poll for join event, then can enter game
			// 2. Enter game

			{ // Menu screen waiting for join
				let mut scene_menu = MultiplayerMenu::new(core, client.code)?;
				let mut last_poll_instant = Instant::now();
				loop {
					// Poll events

					// Draw scene
					let state = scene_menu.draw()?;
					match state {
						GameState::MultiPlayer => continue,
						_ => return Ok(state),
					}
				}
			}

			// poll every 1000ms for second player join event
			// TODO: should be integrated with map rendering to poll every 1s between frame draws
			// (e.g. store a let mut last_poll = Instant::now(); in this scope, compare on each frame & update on each poll)
			loop {
				sleep_poll!(core, 1000);

				if let Some(event) = client.poll()? {
					// listen for the join event, indicating that the other player has connected
					if event.action == net::util::EVENT_JOIN {
						println!("The other player has joined the room.");
					}
				}
			}

			return Ok(GameState::MainMenu);
		},
		GameState::Credits => credits::credits(core)?,
		_ => return Err("Exit game state".to_string())
	};

	Ok(state)
}

static mut ARGS: Vec<String> = Vec::new();

// to start client: `cargo run -- tcp://server-address.example.com:0000`
// to start server: `cargo run -- --server tcp://127.0.0.1:0000`
fn main() {
	// give args a static lifetime
	// (only unsafe for threading concerns; since we don't use multithreading, this is not a problem)
	unsafe { ARGS.extend(env::args()) };

	// if address is specified in args, update static variable
	if let Some(addr) = unsafe { ARGS.last() } {
		if addr.get(..6) == Some("tcp://") {
			if let Some(address) = addr.get(6..) {
				unsafe {
					net::SERVER_ADDR = address;
				}
			}
		}
	}

	if unsafe { ARGS.iter() }.any(|s| s == "--server") {
		net::server::run();
	} else {
		if let Err(e) = runner(true) {
			println!("Exiting: {}", e);
		}
	}
}