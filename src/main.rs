extern crate sdl2;

#[macro_use] mod sdl_macros;


mod AI;
mod banner;
mod barbarian_turn;
mod credits;
mod cursor;
mod damage_indicator;
mod enemy_turn;
mod game_map;
mod input;
mod main_menu;
mod multi_player;
mod net;
mod objective_manager;
mod pixel_coordinates;
mod player_action;
mod player_state;
mod player_turn;
mod single_player;
mod unit_interface;
pub mod button;
pub mod tile;
pub mod unit;

use std::env;
use std::path::Path;
use std::collections::HashMap;

use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, Texture};
use sdl2::mixer::{InitFlag, AUDIO_S32SYS, DEFAULT_CHANNELS};

use crate::main_menu::MainMenu;
use crate::single_player::SinglePlayer;
use crate::multi_player::MultiPlayer;
use crate::input::Input;

const TITLE: &str = "Castle Quest";
const CAM_W: u32 = 1280;
const CAM_H: u32 = 720;
pub const TILE_SIZE: u32 = 32;

#[derive(PartialEq)]
pub enum GameState {
	MainMenu,
	SinglePlayer,
	MultiPlayer,
	Credits,
	Quit,
}

pub trait Drawable {
	fn draw(&mut self) -> Result<GameState, String>;
}

pub struct SDLCore<'t> {
	pub sdl_ctx: sdl2::Sdl,
	pub bold_font: sdl2::ttf::Font<'t, 't>,
	pub regular_font: sdl2::ttf::Font<'t, 't>,
	pub tiny_font: sdl2::ttf::Font<'t, 't>,
	pub wincan: sdl2::render::WindowCanvas,
	pub texture_creator: &'t TextureCreator<sdl2::video::WindowContext>,
	pub texture_map: &'t HashMap<String, Texture<'t>>,
	pub event_pump: sdl2::EventPump,
	pub cam: Rect,
	pub input: Input,
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
	let input = Input::new(&event_pump);

	let cam = Rect::new(0, 0, CAM_W, CAM_H);

	let texture_creator = wincan.texture_creator();

	let bold_font = ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?; //From https://www.fontsquirrel.com/fonts/open-sans
	let regular_font = ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 16)?; //From https://www.fontsquirrel.com/fonts/open-sans
	let tiny_font = ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 12)?; //From https://www.fontsquirrel.com/fonts/open-sans

	let mut texture_map = HashMap::new();

	crate::game_map::load_textures(&mut texture_map, &texture_creator)?;
	crate::damage_indicator::load_textures(&mut texture_map, &texture_creator, &bold_font)?;
	crate::banner::load_textures(&mut texture_map, &texture_creator, &bold_font)?;

	let mut core = SDLCore{
		sdl_ctx,
		bold_font,
		regular_font,
		tiny_font,
		wincan,
		texture_creator: &texture_creator,
		texture_map: &texture_map,
		event_pump,
		cam,
		input,
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
	let mut scene: Box<dyn Drawable> = match game_state {
		GameState::MainMenu => {
			// background music for main menu
			// - This had to be moved into a scope that exists outside of both the `main_menu.rs` functions as it needs a persistent lifetime (otherwise it segfaults)
			// - in the future, this should be abstracted; we could create a `sound_queue: Vec<Path>` in SDLCore that any function can push to in order to play sounds
			sdl2::mixer::open_audio(44100, AUDIO_S32SYS, DEFAULT_CHANNELS, 1024)?;
			let _mixer_filetypes = sdl2::mixer::init(InitFlag::MP3)?;
			let bg_music = sdl2::mixer::Music::from_file(Path::new("./music/main_menu.mp3"))?;

			bg_music.play(-1)?;

			Box::new(MainMenu::new(core)?)
		},
		GameState::SinglePlayer => Box::new(SinglePlayer::new(core)?),
		GameState::MultiPlayer => Box::new(MultiPlayer::new(core)?),
		GameState::Credits => {
			return Ok(credits::credits(core)?);
		},
		_ => return Err("Exit game state".to_string())
	};

	loop {
		let state = scene.draw()?;
		if state != *game_state {
			return Ok(state);
		}
	}
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
