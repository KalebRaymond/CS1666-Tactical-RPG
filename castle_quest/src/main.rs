extern crate sdl2;

use std::time::Duration;
use std::thread;

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;

const TITLE: &str = "Castle Quest";
const CAM_W: u32 = 1280;
const CAM_H: u32 = 720;
const TIMEOUT: u64 = 4500;

use sdl2::rect::Rect;

fn runner(vsync:bool) {
	println!("\nRunning {}:", TITLE);
	match run(vsync) {
		Err(e) => println!("\n\t\tEncountered error while running: {}", e),
		Ok(_) => println!("DONE\nExiting cleanly"),
	};
}

fn run(vsync:bool) -> Result<(), String> {
	let sdl_cxt = sdl2::init()?;
	let ttf_cxt = sdl2::ttf::init().map_err(|e| e.to_string())?;
	let video_subsys = sdl_cxt.video()?;

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
	
	let mut wincan = wincan.build()
		.map_err(|e| e.to_string())?;

	let event_pump = sdl_cxt.event_pump()?;

	let cam = Rect::new(0, 0, CAM_W, CAM_H);

	let texture_creator = wincan.texture_creator();
	
	{ //Credits
		
	}
}

fn main() {
	runner(true);
}