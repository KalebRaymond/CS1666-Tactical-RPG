use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::Font;

use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Instant;

use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;

pub struct DamageIndicator {
    pub damage: u32,
    pub x: i32,
    pub y: i32,
    pub is_visible: bool,

    last_drawn: Instant,
    elapsed_time: f32,

    text: String,
    text_size: (u32, u32),
}

impl DamageIndicator {
    pub fn new(core: &SDLCore, damage: u32, position: PixelCoordinates) -> Result<DamageIndicator, String> {
        let text = "-".to_string() + &damage.to_string();
        let text_size = core.bold_font.size_of(&text).map_err(|_e| "Could not determine text size")?;

        Ok(DamageIndicator {
            damage: damage,
            x: position.x.try_into().unwrap(),
            y: position.y.try_into().unwrap(),
            is_visible: true,

            last_drawn: Instant::now(),
            elapsed_time: 0.0,

            text,
            text_size,
        })
    }

    pub fn draw<'r>(&mut self, core: &mut SDLCore<'r>) -> Result<(), String> {
        let texture = core.texture_map.get(&self.text).ok_or("Could not obtain a valid damage texture")?;

        let (w, h) = self.text_size;
        core.wincan.copy(&texture, None, Rect::new(self.x, self.y, w, h))?;

        self.elapsed_time += self.last_drawn.elapsed().as_secs_f32();
        self.last_drawn = Instant::now();

        //Make numbers float upwards
        self.y -= 1;

        //Set is_visible to false after 1 second. This object should now be destroyed
        if self.elapsed_time >= 1.0 {
            self.is_visible = false;
        }

        Ok(())
    }
}

pub fn load_textures<'r>(textures: &mut HashMap<String, Texture<'r>>, texture_creator: &'r TextureCreator<WindowContext>, bold_font: &Font<'r, 'r>) -> Result<(), String> {
    // create damage indicator textures
	for i in 0..10 {
		let text = format!("-{}", i);
		textures.insert(
			text.to_string(),
            //Create texture to display the damage as a string
			texture_creator.create_texture_from_surface(
				&bold_font.render(&text)
                	.blended_wrapped(Color::RGBA(255, 0, 0, 255), 320)
                	.map_err(|e| e.to_string())?
			).map_err(|e| e.to_string())?
		);
	}

    Ok(())
}
