extern crate sdl2;

use std::time::Duration;
use std::thread;

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;

const TITLE: &str = "Castle Quest";
const CAM_W: u32 = 1280;
const CAM_H: u32 = 720;
const TIMEOUT: u64 = 7500;


use sdl2::rect::Rect;

pub struct SDLCore {
	sdl_cxt: sdl2::Sdl,
	pub wincan: sdl2::render::WindowCanvas,
	pub event_pump: sdl2::EventPump,
	pub cam: Rect,
}

impl SDLCore {
	pub fn init(
		title: &str,
		vsync: bool,
		width: u32,
		height: u32,
	) -> Result<SDLCore, String>
	{
		let sdl_cxt = sdl2::init()?;
		let video_subsys = sdl_cxt.video()?;

		let window = video_subsys.window(title, width, height)
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

		let event_pump = sdl_cxt.event_pump()?;

		let cam = Rect::new(0, 0, width, height);

		Ok(SDLCore{
			sdl_cxt,
			wincan,
			event_pump,
			cam,
		})
	}
}

pub trait Demo {
	fn init() -> Result<Self, String> where Self: Sized;
	fn run(&mut self) -> Result<(), String>;
}

pub fn runner<F, D>(desc: &str, initter: F)
	where
		F: Fn() -> Result<D, String>,
		D: Demo,
{
	println!("\nRunning {}:", desc);
	print!("\tInitting...");
	match initter() {
		Err(e) => println!("\n\t\tFailed to init: {}", e),
		Ok(mut d) => {
			println!("DONE");

			print!("\tRunning...");
			match d.run() {
				Err(e) => println!("\n\t\tEncountered error while running: {}", e),
				Ok(_) => println!("DONE\nExiting cleanly"),
			};
		},
	};
}

pub struct SDLMAIN {
	core: SDLCore,
}

impl Demo for SDLMAIN {
	fn init() -> Result<Self, String> {
		let core = SDLCore::init(TITLE, true, CAM_W, CAM_H)?;
		Ok(SDLMAIN{ core })
	}

	fn run(&mut self) -> Result<(), String> {
		let texture_creator = self.core.wincan.texture_creator();

		//let ms = texture_creator.load_texture("images/hello_world_win.png")?;
		//let tux = texture_creator.load_texture("images/tuxdoge.png")?;

		self.core.wincan.set_draw_color(Color::RGBA(0, 128, 128, 255));
		self.core.wincan.clear();

		//self.core.wincan.copy(&ms, None, None)?;
		self.core.wincan.present();

		// Note SDL has a timer subsystem, but the Rust SDL bindings recommend
		// the use of std::thread::sleep and std::time instead
		thread::sleep(Duration::from_millis(TIMEOUT));

		//self.core.wincan.copy(&tux, None, None)?;
		self.core.wincan.present();

		thread::sleep(Duration::from_millis(TIMEOUT));

		Ok(())
	}
}

fn main() {
	runner(TITLE, SDLMAIN::init);
}