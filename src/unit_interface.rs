use std::time::Instant;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture,TextureCreator};
use sdl2::video::WindowContext;

use crate::SDLCore;

const ANIM_LENGTH: f32 = 0.15;

enum AnimState {
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
    last_drawn: Instant,
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
            last_drawn: Instant::now(),
        }
    }

    pub fn draw(&mut self, core: &mut SDLCore, texture_creator: &TextureCreator<WindowContext>) -> Result<(), String> {
        // Update animation
        let time_elapsed = self.last_drawn.elapsed().as_secs_f32();
        match self.anim_state {
            AnimState::Open => {
                let new_progress = self.anim_progress + time_elapsed/ANIM_LENGTH;
                self.anim_progress = if new_progress > 1.0 { 1.0 } else { new_progress };
            },
            AnimState::Close => {
                if self.anim_progress == 0.0 {
                    return Err("End of animation reached.".to_string());
                }
                let new_progress = self.anim_progress - time_elapsed/ANIM_LENGTH;
                self.anim_progress = if new_progress < 0.0 { 0.0 } else { new_progress };
            },
        }
        self.last_drawn = Instant::now();

        // Draw
        match &self.texture {
            Some(texture) => {
                core.wincan.copy(texture, Rect::new(0,0,64,16), Rect::new(self.x,self.y,64,16))?;
                core.wincan.copy(texture, Rect::new(0,16,64,16), Rect::new(self.x,self.y+16,64,16))?;
                if self.anim_progress > 0.5 {
                    core.wincan.copy(texture, Rect::new(0,16,64,16), Rect::new(self.x,self.y+32,64,16))?;
                }
                
                let font = core.ttf_ctx.load_font("fonts/OpenSans-Regular.ttf", 10)?;
                for (i, text) in self.txt.iter().enumerate() {
                    if i == 1 && self.anim_progress <= 0.5 {
                        continue;
                    }
                    let (text_w, text_h) = font.size_of(text)
                    .map_err( |e| e.to_string() )?;
                    let text_ratio = text_w as f32 / text_h as f32;
                    let text_surface = font.render(text)
                    .blended_wrapped(Color::RGBA(0, 0, 0, 0), 320)
                    .map_err(|e| e.to_string())?;
                    let text_texture = texture_creator.create_texture_from_surface(&text_surface)
                    .map_err(|e| e.to_string())?;
                    core.wincan.copy(&text_texture, None, Rect::new(self.x+10, self.y+16*(i+1)as i32, (16.0*text_ratio)as u32, 16))?;
                }
                
                core.wincan.copy(texture, Rect::new(0,32,64,16), Rect::new(self.x,self.y+16+(32.0*self.anim_progress)as i32,64,16))?;

                Ok(())
            },
            _ => { Err("Texture not defined.".to_string()) },
        }
    }

    pub fn animate_close(&mut self) {
        self.anim_state = AnimState::Close;
    }
}