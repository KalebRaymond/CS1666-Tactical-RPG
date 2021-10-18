use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};

use crate::SDLCore;

enum AnimState {
	Stop,
    Open,
    Close,
}

pub struct UnitInterface<'a> {
    pub x: u32,
    pub y: u32,
    txt: Vec<&'a str>,
    texture: Option<Texture<'a>>,
    anim_progress: f32,
    anim_state: AnimState,
}

impl<'a> UnitInterface<'a> {
    pub fn new(i: u32, j: u32, t: Vec<&'a str>) -> UnitInterface<'a> {
        UnitInterface { 
            x: j * crate::TILE_SIZE,
            y: i * crate::TILE_SIZE,
            txt: t,
            texture: None,
            anim_progress: 0.0,
            anim_state: AnimState::Open,
        }
    }

    pub fn get_texture(&self) -> &Option<Texture> {
        &self.texture
    }

    fn generate_texture(&mut self, core: &'a SDLCore) {
        let texture_creator = core.wincan.texture_creator();

        let base_texture = texture_creator.load_texture("images/interface/unit_interface.png");
        self.texture = match base_texture {
            Ok(spritesheet) => {
                texture_creator.create_texture_target(None, 64, 32+16*self.txt.len() as u32).ok()
            },
            _ => None
        }
    }
}