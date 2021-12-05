use std::time::{Instant, Duration};
use std::result::Result;
use std::collections::HashMap;

use sdl2::video::WindowContext;
use sdl2::render::{Texture, TextureCreator, BlendMode};
use sdl2::pixels::Color;
use sdl2::ttf::Font;
use sdl2::rect::Rect;

use crate::{CAM_W, SDLCore};
use crate::unit::Team;

const BANNER_TIMEOUT: u64 = 1500;
const BANNER_ALPHA: u8 = 250;

const BANNER_TURN_P1: &str = "p1_banner";
const BANNER_TURN_P2: &str = "p2_banner";
const BANNER_TURN_BARB: &str = "b_banner";
const BANNER_WIN_P1: &str = "p1_win_banner";
const BANNER_WIN_P2: &str = "p2_win_banner";

pub struct Banner {
	pub banner_key: String,
	pub current_banner_transparency: u8,
	pub banner_colors: Color,
	pub initial_banner_output: Instant,
	pub banner_visible: bool,
}

impl Banner {
	pub fn new() -> Banner {
		Banner {
			banner_key: "p1_banner".to_string(),
			current_banner_transparency: BANNER_ALPHA,
			banner_colors: Color::RGBA(0, 89, 178, 250),
			initial_banner_output: Instant::now(),
			banner_visible: true,
		}
	}

	pub fn show_turn(&mut self, banner_team: Team) {
		self.show(match banner_team {
			Team::Player => BANNER_TURN_P1,
			Team::Enemy => BANNER_TURN_P2,
			Team::Barbarians => BANNER_TURN_BARB,
		});
	}

	pub fn show(&mut self, banner_key: &str) {
		self.banner_colors = match banner_key {
			BANNER_TURN_P1 => Color::RGBA(0, 89, 178, BANNER_ALPHA),
			BANNER_TURN_P2 => Color::RGBA(207, 21, 24, BANNER_ALPHA),
			BANNER_TURN_BARB => Color::RGBA(163, 96, 30, BANNER_ALPHA),
			BANNER_WIN_P1 => Color::RGBA(0, 89, 178, BANNER_ALPHA),
			BANNER_WIN_P2 => Color::RGBA(207, 21, 24, BANNER_ALPHA),
			_ => Color::RGBA(0, 89, 178, BANNER_ALPHA),
		};

		self.banner_key = banner_key.to_string();
		self.current_banner_transparency = 250;
		self.banner_visible = true;
	}

	pub fn draw<'r>(&mut self, core: &mut SDLCore<'r>) -> Result<(), String> {
		if self.current_banner_transparency == 0 {
			self.banner_visible = false;
			return Ok(());
		}

		//As long as the banner isn't completely transparent, draw it
		self.banner_colors.a = self.current_banner_transparency;

		let banner_rect = Rect::new(core.cam.x.abs(), core.cam.y.abs() + (360-64), CAM_W, 128);
		let text_rect = Rect::new(core.cam.x.abs() + (640-107), core.cam.y.abs() + (360-64), CAM_W/6, 128);
		core.wincan.set_blend_mode(BlendMode::Blend);
		core.wincan.set_draw_color(self.banner_colors);
		core.wincan.draw_rect(banner_rect)?;
		core.wincan.fill_rect(banner_rect)?;

		if let Some(texture) = core.texture_map.get(&self.banner_key) {
			core.wincan.copy(&texture, None, text_rect)?;
		}

		//The first time we draw the banner we need to keep track of when it first appears
		if self.current_banner_transparency == 250 {
			self.initial_banner_output = Instant::now();
			self.current_banner_transparency -= 25;
		}

		//After a set amount of seconds pass and if the banner is still visible, start to make the banner disappear
		if self.initial_banner_output.elapsed() >= Duration::from_millis(BANNER_TIMEOUT) && self.current_banner_transparency != 0 {
			self.current_banner_transparency -= 25;
		}

		Ok(())
	}
}

pub fn load_textures<'r>(textures: &mut HashMap<String, Texture<'r>>, texture_creator: &'r TextureCreator<WindowContext>, bold_font: &Font<'r, 'r>) -> Result<(), String> {
	textures.insert(BANNER_TURN_P1.to_string(), {
		let text_surface = bold_font.render("Your Turn")
			.blended_wrapped(Color::RGBA(0,0,0,BANNER_ALPHA), 320) //Black font
			.map_err(|e| e.to_string())?;

		texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?
	});

	textures.insert(BANNER_TURN_P2.to_string(), {
		let text_surface = bold_font.render("Enemy's Turn")
			.blended_wrapped(Color::RGBA(0,0,0,BANNER_ALPHA), 320) //Black font
			.map_err(|e| e.to_string())?;

		texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?
	});

	textures.insert(BANNER_TURN_BARB.to_string(), {
		let text_surface = bold_font.render("Barbarians' Turn")
			.blended_wrapped(Color::RGBA(0,0,0,BANNER_ALPHA), 320) //Black font
			.map_err(|e| e.to_string())?;

		texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?
	});

	textures.insert(BANNER_WIN_P1.to_string(), {
		let text_surface = bold_font.render("You win!")
			.blended_wrapped(Color::RGBA(0,0,0,BANNER_ALPHA), 320) //Black font
			.map_err(|e| e.to_string())?;

		texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?
	});

	textures.insert(BANNER_WIN_P2.to_string(), {
		let text_surface = bold_font.render("You lost!")
			.blended_wrapped(Color::RGBA(0,0,0,BANNER_ALPHA), 320) //Black font
			.map_err(|e| e.to_string())?;

		texture_creator.create_texture_from_surface(&text_surface)
			.map_err(|e| e.to_string())?
	});

	Ok(())
}
