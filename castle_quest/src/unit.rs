use sdl2::render::Texture;
use rand::Rng;

use crate::Team;

pub struct Unit {
    pub x: u32,
    pub y: u32,
    pub team: Team, 
    pub hp: u32,
    movement_range: u32,
    attack_range: u32,
    accuracy: u32,
    max_damage: u32,
    pub texture: Texture,
}

impl Unit {
    pub fn new(x:u32, y:u32, team: Team, hp: u32, movement: u32, attack: u32, accuracy: u32, damage: u32, texture: Texture) -> Unit {
        Unit {
            x,
            y,
            team,
            hp,
            movement,
            attack,
            accuracy,
            damage,
            texture,
        }
    }
}