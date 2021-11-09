use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use crate::GameState;
use crate::SDLCore;

pub struct MultiplayerMenu<'i, 'r> {
	core: &'i mut SDLCore<'r>,

	bg_texture: Texture<'i>,
	bg_interface: Texture<'i>,

    room_text: Texture<'i>,
    room_text_rect: Rect,

    multiplayer_rect: Rect,
}

impl MultiplayerMenu<'_, '_> {

    pub fn new<'i, 'r>(core: &'i mut SDLCore<'r>, room_code: u32) -> Result<MultiplayerMenu<'i, 'r>, String> {
        let bg_texture = core.texture_creator.load_texture("images/main_menu_animation/24.png")?;
        let bg_interface = core.texture_creator.load_texture("images/interface/unit_interface.png")?;

        let room_text = format!("Room {:04}", room_code);
        let text_surface = core.bold_font.render(&room_text[..])
            .blended_wrapped(Color::RGBA(0,0,0,0), 320) //Black font
            .map_err(|e| e.to_string())?;
        let room_text = core.texture_creator.create_texture_from_surface(&text_surface)
            .map_err(|e| e.to_string())?;
        let room_text_rect = centered_rect!(core, _, 75, 300, 75);

        let multiplayer_rect = centered_rect!(core, 800, 650);
        Ok(MultiplayerMenu {
            core,
            bg_texture,
            bg_interface,
            room_text,
            room_text_rect,
            multiplayer_rect,
        })
    }

    pub fn draw(&mut self) -> Result<GameState, String> {
        for event in self.core.event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit{..} | sdl2::event::Event::KeyDown{keycode: Some(sdl2::keyboard::Keycode::Escape), ..} => {
					return Err("Quit keycode".to_string());
				},
				_ => {},
			}
		}

        self.core.wincan.copy(&self.bg_texture, None, None);
        self.core.wincan.copy(&self.bg_interface, None, self.multiplayer_rect)?;
        self.core.wincan.copy(&self.room_text, None, self.room_text_rect)?;
        self.core.wincan.present();
        Ok(GameState::MultiPlayer)
    }

}