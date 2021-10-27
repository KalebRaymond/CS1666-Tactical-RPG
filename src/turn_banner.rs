use sdl2::pixels::Color;
use std::time::Instant;

pub struct TurnBanner<'a> {
	pub banner_key: &'a str,
	pub current_banner_transparency: u8,
	pub banner_colors: Color,
	pub initial_banner_output: Instant,
	pub banner_visible: bool,
}

impl TurnBanner<'_> {
    pub fn new<'a>() -> TurnBanner<'a> {
        TurnBanner {
            banner_key: "p1_banner",
            current_banner_transparency: 250,
            banner_colors: Color::RGBA(0, 89, 178, 250),
            initial_banner_output: Instant::now(),
            banner_visible: true,
        }
    }
}