use sdl2::pixels::Color;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;

use crate::GameState;
use crate::SDLCore;

use std::path::Path;
use sdl2::mixer::{InitFlag, AUDIO_S32SYS, DEFAULT_CHANNELS};

const CREDITS_TIMEOUT: u64 = 3500;

/// Credits page macro: surrounds the provided "closure" (not really a closure) with canvas present() and thread::sleep calls
macro_rules! credits_page {
	($core: ident, $closure: expr) => {
		$core.wincan.set_draw_color(Color::RGBA(0, 0, 0, 128));
		$core.wincan.clear();
		$closure();

		$core.wincan.present();
		sleep_poll!($core, CREDITS_TIMEOUT);
	}
}

pub fn credits(core: &mut SDLCore) -> Result<GameState, String> {
	let texture_creator = core.wincan.texture_creator();

	let bold_font = core.ttf_ctx.load_font("fonts/OpenSans-Bold.ttf", 32)?; //From https://www.fontsquirrel.com/fonts/open-sans
	let regular_font = core.ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 16)?; //From https://www.fontsquirrel.com/fonts/open-sans

	//Music
	sdl2::mixer::open_audio(44100, AUDIO_S32SYS, DEFAULT_CHANNELS, 1024)?;
	let _mixer_filetypes = sdl2::mixer::init(InitFlag::MP3)?;
	let music = sdl2::mixer::Music::from_file(Path::new("./music/end_credits.mp3"))?;

	music.play(-1);

	// Game title
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(63, 191, 191, 128));
		core.wincan.clear();

		let text_surface = bold_font.render("Castle Quest")
			.blended_wrapped(Color::RGBA(0, 0, 0, 96), 100) //Black font
			.map_err(|e| e.to_string())?;

		let text_texture = texture_creator.create_texture_from_surface(&text_surface)
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

		let text_texture = texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, 500, 250))?;
	});

	//Alex Kwiatkowski Credit Image
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(201, 196, 196, 128));
		core.wincan.clear();

		let alex_credit = texture_creator.load_texture("images/credits/AlexKCreditImageWithText.png")?;
		core.wincan.copy(&alex_credit, None, None)?;
	});

	//Kaleb Raymond Credit Image
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(201, 196, 196, 128));
		core.wincan.clear();

		let kaleb_credit = texture_creator.load_texture("images/credits/KalebRCreditImage.png")?;
		core.wincan.copy(&kaleb_credit, None, None)?;
	});

	//Jared Carl Credit Image
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(255, 255, 255, 128));
		core.wincan.clear();

		let jared_credit = texture_creator.load_texture("images/credits/JaredCCreditImage.png")?;
		core.wincan.copy(&jared_credit, None, centered_rect!(core, _, 200, 256, 200))?;

		let text_surface = regular_font.render("Jared Carl")
			.blended_wrapped(Color::RGBA(0, 0, 0, 128), 320)
			.map_err(|e| e.to_string())?;

		let text_texture = texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, _, 400, 550, 200))?;
	});

	//Colin Woelfel Credit Image
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(201, 196, 196, 128));
		core.wincan.clear();

		let colin_credit = texture_creator.load_texture("images/credits/ColinWCreditImage.png")?;
		core.wincan.copy(&colin_credit, None, None)?;
	});

	// ----- Networking team
	credits_page!(core, {
		core.wincan.set_draw_color(Color::RGBA(238, 46, 21, 128));
		core.wincan.clear();

		let text_surface = regular_font.render("Networking Subteam")
			.blended_wrapped(Color::RGBA(0, 0, 0, 128), 320) //Black font
			.map_err(|e| e.to_string())?;

		let text_texture = texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, 1000, 250))?;
	});

	// James Fenn Credit Image
	credits_page!(core, {
		let james_credit = texture_creator.load_texture("images/credits/JamesFCreditImage.png")?;
		core.wincan.copy(&james_credit, None, centered_rect!(core, _, 200, 256, 256))?;

		let text_surface = regular_font.render("James Fenn")
			.blended_wrapped(Color::RGBA(128, 128, 128, 128), 320) //Black font
			.map_err(|e| e.to_string())?;

		let text_texture = texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?;

		core.wincan.copy(&text_texture, None, centered_rect!(core, _, 400, 500, 125))?;
	});

	//Bianca Finamore Credit Image
	credits_page!(core, {
		let bianca_credit = texture_creator.load_texture("images/credits/BiancaCredit.png")?;
		core.wincan.copy(&bianca_credit, None, None)?;
	});

	//Jake Baumbaugh Credit Image
	credits_page!(core, {
		let jake_credit = texture_creator.load_texture("images/credits/JakeBCreditImageWithText.png")?;
		core.wincan.copy(&jake_credit, None, None)?;
	});

	// Shane Josapak Credit Image
	credits_page!(core, {
		let shane_credit = texture_creator.load_texture("images/credits/ShaneJCreditImage.png")?;
		core.wincan.copy(&shane_credit, None, None)?;
	});

	sdl2::mixer::Music::fade_out(600);
	//Credits finished playing, automatically quit game
	Ok(GameState::MainMenu)
}
