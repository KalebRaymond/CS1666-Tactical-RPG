use sdl2::mouse::MouseState;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;

use crate::SDLCore;

pub struct Button<'r> {
	rect: Rect,
	text: String,
	text_size: (u32, u32),
	texture: sdl2::render::Texture<'r>,
}

impl Button<'_> {
	pub fn new<'r>(core: &SDLCore<'r>, rect: Rect, text: &str) -> Result<Button<'r>, String> {
		let texture = core.texture_creator.create_texture_from_surface(
			&core.bold_font.render(text)
				.blended_wrapped(Color::RGBA(255, 255, 255, 128), 320)
				.map_err(|e| e.to_string())?
		).map_err(|e| e.to_string())?;

		Ok(Button {
			rect,
			text: text.to_string(),
			text_size: core.bold_font.size_of(text).map_err(|_e| "Could not determine text size")?,
			texture,
		})
	}

	pub fn is_mouse(&self, core: &SDLCore) -> bool {
		let mouse_state: MouseState = core.event_pump.mouse_state();
		let mouse_pos = (mouse_state.x(), mouse_state.y());

		self.rect.contains_point(mouse_pos)
	}

	pub fn draw<'r>(&self, core: &mut SDLCore<'r>) -> Result<(), String> {
		let color = if self.is_mouse(core) { Color::RGBA(100,100,100,100) } else { Color::RGBA(50,50,50,100) };
		let (w, h) = self.text_size;
		let x: i32 = (self.rect.width() as i32 - w as i32) / 2;
		let y: i32 = (self.rect.height() as i32 - h as i32) / 2;

		core.wincan.set_blend_mode(BlendMode::Blend);
		core.wincan.set_draw_color(color);
		core.wincan.fill_rect(self.rect)?;
		core.wincan.copy(&self.texture, None, Rect::new(self.rect.x() + x, self.rect.y() + y, w, h))?;
		Ok(())
	}

	pub fn draw_relative<'r>(&self, core: &mut SDLCore<'r>) -> Result<(), String> {
		let color = if self.is_mouse(core) { Color::RGBA(100,100,100,100) } else { Color::RGBA(50,50,50,100) };
		let (w, h) = self.text_size;
		let x: i32 = (self.rect.width() as i32 - w as i32) / 2;
		let y: i32 = (self.rect.height() as i32 - h as i32) / 2;

		let button_rect = Rect::new(self.rect.x() - core.cam.x, self.rect.y() - core.cam.y, self.rect.width(), self.rect.height());
		let text_rect = Rect::new(self.rect.x() + x - core.cam.x, self.rect.y() + y - core.cam.y, w, h);

		core.wincan.set_blend_mode(BlendMode::Blend);
		core.wincan.set_draw_color(color);
		core.wincan.fill_rect(button_rect)?;
		core.wincan.copy(&self.texture, None, text_rect)?;
		Ok(())
	}
}
