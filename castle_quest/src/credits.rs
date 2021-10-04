use std::time::Duration;
use std::thread;

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;

use crate::SDLCore;

const CREDITS_TIMEOUT: u64 = 4500;

/// Credits page macro: surrounds the provided "closure" (not really a closure) with canvas present() and thread::sleep calls
macro_rules! credits_page {
	($core: ident, $closure: expr) => {
		$core.wincan.set_draw_color(Color::RGBA(0, 0, 0, 128));
		$core.wincan.clear();
		$closure();

		$core.wincan.present();
		thread::sleep(Duration::from_millis(CREDITS_TIMEOUT));
	}
}

pub fn credits(core: &mut SDLCore) -> Result<(), String> {
	let bold_font = core.ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?; //From https://www.fontsquirrel.com/fonts/open-sans
	let regular_font = core.ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 16)?; //From https://www.fontsquirrel.com/fonts/open-sans

	// Game title
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(63, 191, 191, 128));
		core.wincan.clear();

		let text_surface = bold_font.render("Castle Quest")
			.blended_wrapped(Color::RGBA(0, 0, 0, 96), 100) //Black font
			.map_err(|e| e.to_string())?;

		let text_texture = core.texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, 500, 500))?;
	});

	// ----- AI team
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(60, 163, 62, 128));
		core.wincan.clear();

		let text_surface = regular_font.render("AI Subteam")
			.blended_wrapped(Color::RGBA(0, 0, 0, 128), 160) //Black font
			.map_err(|e| e.to_string())?;

		let text_texture = core.texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, 500, 250))?;
	});

	//Alex Kwiatkowski Credit Image
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(201, 196, 196, 128));
		core.wincan.clear();

		let alex_credit = core.texture_creator.load_texture("images/AlexKCreditImageWithText.png")?;
		core.wincan.copy(&alex_credit, None, None)?;
	});

	//Kaleb Raymond Credit Image
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(201, 196, 196, 128));
		core.wincan.clear();

		let kaleb_credit = core.texture_creator.load_texture("images/KalebRCreditImage.png")?;
		core.wincan.copy(&kaleb_credit, None, None)?;
	});
	
	//Jared Carl Credit Image
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(255, 255, 255, 128));
		core.wincan.clear();

		let jared_credit = core.texture_creator.load_texture("images/JaredCCreditImage.png")?;
		core.wincan.copy(&jared_credit, None, centered_rect!(core, _, 200, 256, 200))?;
	
		let text_surface = regular_font.render("Jared Carl")
			.blended_wrapped(Color::RGBA(0, 0, 0, 128), 320)
			.map_err(|e| e.to_string())?;
	
		let text_texture = core.texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;
	
		core.wincan.copy(&text_texture, None, centered_rect!(core, _, 400, 550, 200))?;
	});

	// ----- Networking team
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(238, 46, 21, 128));
		core.wincan.clear();

		let text_surface = regular_font.render("Networking Subteam")
			.blended_wrapped(Color::RGBA(0, 0, 0, 128), 320) //Black font
			.map_err(|e| e.to_string())?;

		let text_texture = core.texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, 1000, 250))?;
	});

	// James Fenn Credit Image
	credits_page!(core, {
		let james_credit = core.texture_creator.load_texture("images/JamesFCreditImage.png")?;
		core.wincan.copy(&james_credit, None, centered_rect!(core, _, 200, 256, 256))?;

		let text_surface = regular_font.render("James Fenn")
			.blended_wrapped(Color::RGBA(128, 128, 128, 128), 320) //Black font
			.map_err(|e| e.to_string())?;

		let text_texture = core.texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, _, 400, 500, 125))?;
	});

	credits_page!(core, {
		let jake_credit = core.texture_creator.load_texture("images/JakeBCreditImageWithText.png")?;
		core.wincan.copy(&jake_credit, None, None)?;
	});

	Ok(())
}
