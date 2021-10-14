use sdl2::image::LoadTexture;
use sdl2::render::Texture;

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
    anim_progress: f32,
    anim_state: AnimState,
}

impl UnitInterface<'_> {
    pub fn new<'a>(i: u32, j: u32, t: Vec<&'a str>) -> UnitInterface<'a> {
        UnitInterface { 
            x: j * crate::TILE_SIZE,
            y: i * crate::TILE_SIZE,
            txt: t,
            anim_progress: 0.0,
            anim_state: AnimState::Open,
        }
    }

    pub fn get_texture(core: &mut SDLCore) -> Result<Texture, String> {
        core.texture_creator.load_texture("images/interface/unit_interface.png")
    }
}