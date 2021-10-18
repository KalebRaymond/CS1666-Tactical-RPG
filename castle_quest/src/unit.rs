//Rust complains that it can't find rand crate
//extern crate rand;
//use rand::Rng;

use sdl2::render::Texture;
use std::fmt;

use crate::tile::{Tile};

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
    pub texture: &'a Texture<'a>,
}

impl Unit <'_>{
    pub fn new<'a> (x:u32, y:u32, team: Team, hp: u32, movement_range: u32, attack_range: u32, accuracy: u32, max_damage: u32, texture: &'a Texture) -> Unit<'a> {
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
    pub fn get_tiles_in_attack_range(&self, map: &Vec<Vec<Tile>>) -> Vec<(u32, u32)> {
        let mut tiles_in_range: Vec<(u32, u32)> = Vec::new();
        // let lowest_row: u32 = (self.y - self.attack_range).clamp(0, self.y);
        // let highest_row: u32 = (self.y + self.attack_range).clamp(self.y, map.len());
        // let mut offset = 0;

        // for y in lowest_row..=highest_row {
        //     for x in (self.x-offset).clamp(0, self.x)..=(self.x+offset).clamp(self.x, map[0].len()) {
        //         if map[y][x].contains_unit {
        //             tiles_in_range.push((x,y));
        //         }
        //     }
        //     offset += 1;
        //     if y == self.y {
        //         offset = 0;
        //     }
        // }
        tiles_in_range
    }
}

impl fmt::Display for Unit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unit(x:{}, y:{}, hp:{})", self.x, self.y, self.hp)
    }
}