use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::render::Texture;

use crate::pixel_coordinates::PixelCoordinates;
use crate::unit::{Team, Unit};
use crate::SDLCore;
use crate::TILE_SIZE;

const HEALTH_WIDTH: u32 = 5;

pub struct Cursor<'a> {
    pub x: i32,
    pub y: i32,
    pub is_visible: bool,
    pub texture: &'a Texture<'a>,
    pub unit_team: Team,  // team that the unit belongs to
    pub unit_hp: u32,     // amount of unit health to draw in health bar
    pub unit_max_hp: u32,
}

impl Cursor<'_> {
    pub fn new<'a>(tex: &'a Texture) -> Cursor<'a> {
        Cursor {
            x: -1,
            y: -1,
            is_visible: false,
            texture: tex,
            unit_team: Team::Player,
            unit_hp: 0,
            unit_max_hp: 0,
        }
    }

    pub fn set_cursor(&mut self, coordinates: &PixelCoordinates, unit: &Unit) {
        self.x = coordinates.x as i32;
        self.y = coordinates.y as i32;
        self.is_visible = true;

        self.unit_team = unit.team;
        self.unit_hp = unit.hp;
        self.unit_max_hp = unit.max_hp;
    }

    pub fn hide_cursor(&mut self) {
        self.x = -1;
        self.y = -1;
        self.is_visible = false;
    }

    pub fn draw(&self, core: &mut SDLCore) -> Result<(), String> {
        if self.is_visible && self.unit_team == Team::Player {
            core.wincan.copy(self.texture, Rect::new(0, 0, TILE_SIZE, TILE_SIZE), Rect::new(self.x, self.y, TILE_SIZE, TILE_SIZE))?;
        }

        if self.is_visible {
            let back_health = Rect::new(self.x, self.y-2, self.unit_max_hp*2, HEALTH_WIDTH);
            core.wincan.set_draw_color(Color::GRAY);
            core.wincan.fill_rect(back_health)?;
            core.wincan.draw_rect(back_health)?;

            core.wincan.set_draw_color(Color::RED);
            core.wincan.fill_rect(Rect::new(self.x, self.y-2, self.unit_hp*2, HEALTH_WIDTH))?;
        }

        Ok(())
    }
}