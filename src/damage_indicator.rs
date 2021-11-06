use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use std::convert::TryInto;
use std::time::Instant;

use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;

pub struct DamageIndicator<'r> {
    pub damage: u32,
    pub x: u32,
    pub y: u32,
    width: u32,
    height: u32,
    pub is_visible: bool,

    last_drawn: Instant,
    elapsed_time: f32,

    texture: Texture<'r>,
    text_size: (u32, u32),
}

impl DamageIndicator<'_> {
    pub fn new<'r>(core: &SDLCore<'r>, damage: u32, position: PixelCoordinates) -> Result<DamageIndicator<'r>, String> {
        //Create texture to display the damage as a string
        let text = "-".to_string() + &damage.to_string();
        let texture = core.texture_creator.create_texture_from_surface(
			&core.bold_font.render(&text)
                .blended_wrapped(Color::RGBA(255, 0, 0, 255), 320)
                .map_err(|e| e.to_string())?
		).map_err(|e| e.to_string())?;

        Ok(DamageIndicator {
            damage: damage,
            x: position.x,
            y: position.y,
            width: 64,
            height: 32,
            is_visible: true,
            
            last_drawn: Instant::now(),
            elapsed_time: 0.0,

            texture: texture,
            text_size: core.bold_font.size_of(&text).map_err(|e| "Could not determine text size")?,
        })  
    }

    pub fn draw<'r>(&mut self, core: &mut SDLCore<'r>) -> Result<(), String> {
        let (w, h) = self.text_size;
        core.wincan.copy(&self.texture, None, Rect::new(self.x.try_into().unwrap(), self.y.try_into().unwrap(), w, h))?;

        self.elapsed_time += self.last_drawn.elapsed().as_secs_f32();
        self.last_drawn = Instant::now();

        //Set is_visible to false after 1 second. This object should now be destroyed
        if self.elapsed_time >= 1.0 {
            self.is_visible = false;
        }

        Ok(())
    }
}