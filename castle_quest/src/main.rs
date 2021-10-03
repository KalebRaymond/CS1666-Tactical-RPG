extern crate sdl2;

const TITLE: &str = "Castle Quest";
const CAM_W: u32 = 1280;
const CAM_H: u32 = 720;

use sdl2::rect::Rect;

mod credits;

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
	match run(vsync) {
		Err(e) => println!("\n\t\tEncountered error while running: {}", e),
		Ok(_) => println!("DONE\nExiting cleanly"),
	};
}

fn run(vsync:bool) -> Result<(), String> {
	let mut core = init_sdl_core(vsync)?;
	credits::credits(&mut core)?;

	Ok(())
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

	Ok( SDLCore{
			sdl_ctx,
			ttf_ctx,
			wincan,
			event_pump,
			cam,
			texture_creator,
	})
}

fn main() {
	runner(true);
}