//Rust complains that it can't find rand crate
//extern crate rand;
//use rand::Rng;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use sdl2::render::Texture;
use std::fmt;
use crate::tile::{Tile};

const MAP_WIDTH:u32 = 64;
const MAP_HEIGHT:u32 = 64;

pub enum Team {
	Player,
	Enemy,
	Barbarians,
}

struct QueueObject {
    coords: (u32, u32),
    cost: u32, //Moves remaining if the unit goes to that tile
}

//Need to implement this and Eq for comparison to work
impl PartialEq for QueueObject {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.coords == other.coords
    }
}

impl Eq for QueueObject {}

impl Ord for QueueObject {
    //Only really want to compare based on the number of moves remaining
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}

impl PartialOrd for QueueObject {
    //Also need to implement partial ordering as per the rust docs
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
    pub has_attacked: bool,
    pub has_moved: bool,
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
            // Initially both are set to true, when it becomes someone's turn, both will need to be set to false for each unit on team
            has_attacked: true,
            has_moved: true,
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
    pub fn get_tiles_in_movement_range(&self, map: &mut HashMap<(u32, u32), Tile>,) -> Vec<(u32, u32)> {
        let mut tiles_in_range: Vec<(u32, u32)> = Vec::new();
        let mut visited: HashMap<(u32,u32), bool> = HashMap::new();
        let mut heap = BinaryHeap::new();
        heap.push(QueueObject{coords: (self.x, self.y), cost: self.movement_range});
        visited.insert((self.x, self.y), true);
        tiles_in_range.push((self.x, self.y));
        while let Some(QueueObject { coords, cost }) = heap.pop() {
            if cost == 0 {
                continue
            }
            //Since we know that we can make a move here need to check each of the 4 sides of the current position to see if we can make a move
            if coords.0 > 0 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0-1 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if entry.get().unit_can_move_here() && !visited.contains_key(&(coords.0-1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0-1, coords.1), cost:cost-1});
                        visited.insert((coords.0-1, coords.1), true);
                        tiles_in_range.push((coords.0-1, coords.1));
                    }
                }
            }
            if coords.0 < MAP_WIDTH-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0+1 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if entry.get().unit_can_move_here() && !visited.contains_key(&(coords.0+1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0+1, coords.1), cost:cost-1});
                        visited.insert((coords.0+1, coords.1), true);
                        tiles_in_range.push((coords.0+1, coords.1));
                    }
                }
            }
            if coords.1 > 0 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1-1 as u32, coords.0 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if entry.get().unit_can_move_here() && !visited.contains_key(&(coords.0, coords.1-1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1-1), cost:cost-1});
                        visited.insert((coords.0, coords.1-1), true);
                        tiles_in_range.push((coords.0, coords.1-1));
                    }
                }
            }
            if coords.1 < MAP_HEIGHT-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1+1 as u32, coords.0 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if entry.get().unit_can_move_here() && !visited.contains_key(&(coords.0, coords.1+1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1+1), cost:cost-1});
                        visited.insert((coords.0, coords.1+1), true);
                        tiles_in_range.push((coords.0, coords.1+1));
                    }
                }
            }
        }
        tiles_in_range
    }
    pub fn get_tiles_in_attack_range(&self, map: &mut HashMap<(u32, u32), Tile>,) -> Vec<(u32, u32)> {
        let mut tiles_in_range: Vec<(u32, u32)> = Vec::new();
        let mut visited: HashMap<(u32,u32), bool> = HashMap::new();
        let mut heap = BinaryHeap::new();
        heap.push(QueueObject{coords: (self.x, self.y), cost: self.attack_range});
        visited.insert((self.x, self.y), true);
        while let Some(QueueObject { coords, cost }) = heap.pop() {
            if cost == 0 {
                continue
            }
            //Since we know that we can make a move here need to check each of the 4 sides of the current position to see if we can make a move
            if coords.0 > 0 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0-1 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if !visited.contains_key(&(coords.0-1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0-1, coords.1), cost:cost-1});
                        visited.insert((coords.0-1, coords.1), true);
                        tiles_in_range.push((coords.0-1, coords.1));
                    }
                }
            }
            if coords.0 < MAP_WIDTH-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0+1 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if !visited.contains_key(&(coords.0+1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0+1, coords.1), cost:cost-1});
                        visited.insert((coords.0+1, coords.1), true);
                        tiles_in_range.push((coords.0+1, coords.1));
                    }
                }
            }
            if coords.1 > 0 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1-1 as u32, coords.0 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if !visited.contains_key(&(coords.0, coords.1-1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1-1), cost:cost-1});
                        visited.insert((coords.0, coords.1-1), true);
                        tiles_in_range.push((coords.0, coords.1-1));
                    }
                }
            }
            if coords.1 < MAP_HEIGHT-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1+1 as u32, coords.0 as u32)) {
                    //As long as a unit can move to this tile and we have not already visited this tile
                    if !visited.contains_key(&(coords.0, coords.1+1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1+1), cost:cost-1});
                        visited.insert((coords.0, coords.1+1), true);
                        tiles_in_range.push((coords.0, coords.1+1));
                    }
                }
            }
        }
        tiles_in_range
    }
}

impl fmt::Display for Unit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unit(x:{}, y:{}, hp:{})", self.x, self.y, self.hp)
    }
}