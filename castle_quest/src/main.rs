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

	let _event_pump = sdl_cxt.event_pump()?;

	let _cam = Rect::new(0, 0, CAM_W, CAM_H);

	let texture_creator = wincan.texture_creator();
	
	{ //Credits
		let bold_font = ttf_cxt.load_font("fonts/OpenSans-Bold.ttf", 32)?; //From https://www.fontsquirrel.com/fonts/open-sans
		let regular_font = ttf_cxt.load_font("fonts/OpenSans-Regular.ttf", 16)?; //From https://www.fontsquirrel.com/fonts/open-sans

		wincan.set_draw_color(Color::RGBA(63, 191, 191, 128));
		wincan.clear();

		//Game title
		let text_surface = bold_font.render("Castle Quest")
								.blended_wrapped(Color::RGBA(0, 0, 0, 96), 100) //Black font
								.map_err(|e| e.to_string())?;
								
		let text_texture = texture_creator.create_texture_from_surface(&text_surface) 
											.map_err(|e| e.to_string())?;
		
		wincan.copy(&text_texture, None, None)?;
		wincan.present();

		thread::sleep(Duration::from_millis(TIMEOUT));
		wincan.set_draw_color(Color::RGBA(60, 163, 62, 128));
		wincan.clear();

		//AI Team centered
		let w = 500;
		let r = Rect::new((CAM_W/2 - w/2) as i32, (CAM_H /2 - w/2) as i32, w, w);


		let text_surface = regular_font.render("AI Subteam")
								.blended_wrapped(Color::RGBA(0, 0, 0, 96), 160) //Black font
								.map_err(|e| e.to_string())?;
								
		let text_texture = texture_creator.create_texture_from_surface(&text_surface) 
											.map_err(|e| e.to_string())?;
		
		wincan.copy(&text_texture, None, r)?;
		wincan.present();

		thread::sleep(Duration::from_millis(TIMEOUT));
		wincan.set_draw_color(Color::RGBA(201, 196, 196, 128));
		wincan.clear();

		//Alex Kwiatkowski Credit Image
		let alex_credit = texture_creator.load_texture("images/AlexKCreditImageWithText.png")?;
		wincan.copy(&alex_credit, None, None)?;
		wincan.present();

		//thread::sleep(Duration::from_millis(TIMEOUT));
		//wincan.clear();

		//........

		thread::sleep(Duration::from_millis(TIMEOUT));
		wincan.set_draw_color(Color::RGBA(238, 46, 21, 128));
		wincan.clear();
		
		let text_surface = regular_font.render("Networking Subteam")
								.blended_wrapped(Color::RGBA(0, 0, 0, 96), 320) //Black font
								.map_err(|e| e.to_string())?;
								
		let text_texture = texture_creator.create_texture_from_surface(&text_surface) 
											.map_err(|e| e.to_string())?;
		
		wincan.copy(&text_texture, None, r)?;
		wincan.present();

		// Note SDL has a timer subsystem, but the Rust SDL bindings recommend
		// the use of std::thread::sleep and std::time instead
		thread::sleep(Duration::from_millis(TIMEOUT));

		//self.core.wincan.copy(&tux, None, None)?;
		wincan.present();

		thread::sleep(Duration::from_millis(TIMEOUT));

		Ok(())
	}
}

fn main() {
	runner(true);
}