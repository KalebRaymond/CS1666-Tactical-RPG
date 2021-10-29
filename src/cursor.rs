use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::TILE_SIZE;

pub struct Cursor<'a> {
    pub x: i32,
    pub y: i32,
    pub is_visible: bool,
    pub texture: &'a Texture<'a>,
}

impl Cursor<'_> {
    pub fn new<'a>(tex: &'a Texture) -> Cursor<'a> {
        Cursor {
            x: -1,
            y: -1,
            is_visible: false,
            texture: tex,
        }
    }

    pub fn set_cursor(&mut self, coordinates: &PixelCoordinates) {
        self.x = coordinates.x as i32;
        self.y = coordinates.y as i32;
        self.is_visible = true;
    }

    pub fn hide_cursor(&mut self) {
        self.x = -1;
        self.y = -1;
        self.is_visible = false;
    }

    pub fn draw(&self, core: &mut SDLCore) -> Result<(), String> {
        if self.is_visible {
            core.wincan.copy(self.texture, Rect::new(0, 0, TILE_SIZE, TILE_SIZE), Rect::new(self.x, self.y, TILE_SIZE, TILE_SIZE))?;
        }
        
        Ok(())
    }
}