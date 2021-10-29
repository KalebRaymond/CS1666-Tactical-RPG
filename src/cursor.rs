use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use crate::SDLCore;

pub struct Cursor<'a> {
    pub position_i: i32,
    pub position_j: i32,
    pub is_visible: bool,
    pub texture: &'a Texture<'a>,
}

impl Cursor<'_> {
    pub fn new<'a>(tex: &'a Texture) -> Cursor<'a> {
        Cursor {
            position_i: -1,
            position_j: -1,
            is_visible: false,
            texture: tex,
        }
    }

    pub fn set_cursor(&mut self, i: i32, j: i32) {
        self.position_i = i;
        self.position_j = j;
        self.is_visible = true;
    }

    pub fn hide_cursor(&mut self) {
        self.position_i = -1;
        self.position_j = -1;
        self.is_visible = false;
    }

    pub fn draw(&self, core: &mut SDLCore) -> Result<(), String> {
        if self.is_visible {
            core.wincan.copy(self.texture, Rect::new(0, 0, 64, 16), Rect::new(self.position_i, self.position_j, 64, 16))?;
        }
        
        Ok(())
    }
}