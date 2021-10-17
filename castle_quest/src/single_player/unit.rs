//Rust complains that it can't find rand crate
//extern crate rand;
//use rand::Rng;

use sdl2::render::Texture;


pub enum Team {
	Player,
	Enemy,
	Barbarians,
}

pub struct Unit<'a> {
    pub x: u32,
    pub y: u32,
    pub team: Team, 
    pub hp: u32,
    movement_range: u32,
    attack_range: u32,
    accuracy: u32,
    max_damage: u32,
    pub texture: Texture<'a>,
}

impl Unit <'_>{
    pub fn new<'a> (x:u32, y:u32, team: Team, hp: u32, movement_range: u32, attack_range: u32, accuracy: u32, max_damage: u32, texture: Texture<'a>) -> Unit<'a> {
        Unit {
            x,
            y,
            team,
            hp,
            movement_range,
            attack_range,
            accuracy,
            max_damage,
            texture,
        }
    }
    pub fn get_attack_damage(&self) -> u32 {
        // let chance = rand::thread_rng().gen_range(0..100);
        // if chance < self.accuracy {
        //     rand::thread_rng().gen_range(1..=self.max_damage);
        // } else {
        //     0
        // }
        0
    }
    pub fn get_tiles_in_attack_range(&self) -> Vec<(u32, u32)> {
        vec!((0,0))
    }
}