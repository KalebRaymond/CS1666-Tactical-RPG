use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture,TextureCreator};
use sdl2::video::WindowContext;

use crate::SDLCore;

enum AnimState {
	Stop,
    Open,
    Close,
}

pub struct UnitInterface<'a> {
    pub x: i32,
    pub y: i32,
    txt: Vec<&'a str>,
    texture: Option<&'a Texture<'a>>,
    anim_progress: f32,
    anim_state: AnimState,
}

impl<'a> UnitInterface<'a> {
    pub fn new(i: u32, j: u32, t: Vec<&'a str>, tex: &'a Texture<'a>) -> UnitInterface<'a> {
        UnitInterface { 
            x: ((j-2) * crate::TILE_SIZE) as i32,
            y: ((i-1) * crate::TILE_SIZE) as i32,
            txt: t,
            texture: Some(tex),
            anim_progress: 0.0,
            anim_state: AnimState::Open,
        }
    }

    pub fn draw(&self, core: &mut SDLCore, texture_creator: &TextureCreator<WindowContext>) -> Result<(), String> {
        match &self.texture {
            Some(texture) => {
                core.wincan.copy(texture, Rect::new(0,0,64,16), Rect::new(self.x,self.y,64,16))?;
                core.wincan.copy(texture, Rect::new(0,16,64,16), Rect::new(self.x,self.y+16,64,16))?;
                core.wincan.copy(texture, Rect::new(0,16,64,16), Rect::new(self.x,self.y+32,64,16))?;
                core.wincan.copy(texture, Rect::new(0,32,64,16), Rect::new(self.x,self.y+48,64,16))?;

                let font = core.ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 16)?;
                let text_size = font.size_of("test");
                match text_size {
                    Ok((w, h)) => {
                        if w > 0 {
                            println!("{},{}",w,h);
                        }
                    },
                    _ => {},
                }

                Ok(())
            },
            _ => { Err("Texture not defined.".to_string()) },
        }
    }
}