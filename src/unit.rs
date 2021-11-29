use rand::Rng;

use sdl2::rect::Rect;
use sdl2::render::Texture;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use crate::SDLCore;
use crate::tile::Tile;

const MAP_WIDTH: u32 = 64;
const MAP_HEIGHT: u32 = 64;

pub enum Team {
	Player,
	Enemy,
	Barbarians,
}
impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Team::Player, Team::Player) => true,
            (Team::Enemy, Team::Enemy) => true,
            (Team::Barbarians, Team::Barbarians) => true,
            _ => false,
        }
    }
}
impl Copy for Team {}
impl Clone for Team {
    fn clone(&self) -> Team {
        match self {
            Team::Player => Team::Player,
            Team::Enemy => Team::Enemy,
            Team::Barbarians => Team::Barbarians,
        }
    }
}
pub struct QueueObject {
    pub coords: (u32, u32),
    pub cost: u32, //Moves remaining if the unit goes to that tile
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
    pub max_hp: u32,
    movement_range: u32,
    pub attack_range: u32,
    accuracy: u32,
    min_damage: u32,
    max_damage: u32,
    pub texture: &'a Texture<'a>,
    pub has_attacked: bool,
    pub has_moved: bool,

    default_sprite_src: Rect,
    red_sprite_src: Rect,
    gray_sprite_src: Rect,

    is_attacked: bool,
    last_damaged_drawn: Instant,
    time_since_damaged: f32,

    //Used for barbarians to make sure they roam within a small radius
    pub starting_x: u32,
    pub starting_y: u32,
}

impl Unit <'_>{
    pub fn new<'a> (x:u32, y:u32, team: Team, hp: u32, movement_range: u32, attack_range: u32, accuracy: u32, min_damage:u32, max_damage: u32, texture: &'a Texture) -> Unit<'a> {
        Unit {
            x,
            y,
            team,
            hp,
            max_hp: hp,
            movement_range,
            attack_range,
            accuracy,
            min_damage,
            max_damage,
            texture,
            
            has_attacked: false,
            has_moved: false,

            default_sprite_src: Rect::new(0, 0, 32, 32),
            red_sprite_src: Rect::new(32, 0, 32, 32),
            gray_sprite_src: Rect::new(64, 0, 32, 32),

            is_attacked: false,
            last_damaged_drawn: Instant::now(),
            time_since_damaged: 0.0,

            starting_x: x,
            starting_y: y,
        }
    }

    pub fn get_attack_damage(&self) -> u32 {
        let chance = rand::thread_rng().gen_range(0..100);
        if chance < self.accuracy {
            rand::thread_rng().gen_range(self.min_damage..=self.max_damage)
        } else {
            0
        }
    }

    pub fn update_pos(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }

    pub fn next_turn(&mut self) {
        self.has_attacked = false;
        self.has_moved = false;
    }

    pub fn respawn_loc(&self, map: &mut HashMap<(u32, u32), Tile>, where_to_spawn: (u32,u32)) -> (u32, u32) {
        if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((where_to_spawn.1, where_to_spawn.0)) {
            //As long as a unit can move to this tile return it otherwise find closest available
            if entry.get().unit_can_move_here() {
                where_to_spawn
            } else {
                self.get_closest_move(where_to_spawn, map)
            }
        } else {
            panic!("Trying to spawn unit off map")
        }
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

    // There is a chance that the best move for an enemy unit is no longer possible once we actually start moving units
    // Thus we should try to find the closest possible tile to move to
    pub fn get_closest_move(&self, desired_move:(u32,u32), map: &mut HashMap<(u32, u32), Tile>,) -> (u32,u32) {
        let mut y_increment: i32 = 1;
        let mut x_increment: i32 = 1;
        let mut current_x:i32 = desired_move.0 as i32;
        let mut current_y:i32 = desired_move.1 as i32;

        if desired_move.1 > self.y { //If the coorodinate is below then our increment should be -1;
            y_increment = -1;
        } else if desired_move.1 == self.y { //If the coordinates are at the same y focus on moving x first
            y_increment = 0;
        }
        if desired_move.0 > self.x { //If the coorodinate is to the right then our increment should be -1;
            x_increment = -1;
        } else if desired_move.0 == self.x { //If the coordinates are at the same x focus on moving y first
            x_increment = 0;
        }
        loop {
            if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((current_y as u32, (current_x + x_increment) as u32)) {
                //As long as a unit can move to this tile and we have not already visited this tile
                if entry.get().unit_can_move_here() {
                    return ((current_x + x_increment) as u32, current_y as u32)
                }
            }
            if let std::collections::hash_map::Entry::Occupied(entry) = map.entry(((current_y + y_increment) as u32, current_x as u32)) {
                //As long as a unit can move to this tile and we have not already visited this tile
                if entry.get().unit_can_move_here() {
                    return (current_x as u32, (current_y + y_increment) as u32)
                }
            }
            if let std::collections::hash_map::Entry::Occupied(entry) = map.entry(((current_y + y_increment) as u32, (current_x + x_increment) as u32)) {
                //As long as a unit can move to this tile and we have not already visited this tile
                if entry.get().unit_can_move_here() {
                    return ((current_x + x_increment) as u32, (current_y + y_increment) as u32)
                }
            }
            current_x += x_increment;
            current_y += y_increment;
            if current_x == self.x as i32 && current_y == self.y as i32 {
                break;
            }
        }
        //In the event that no closer moves are found, stay at current position
        (self.x, self.y)
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
                    //As long as we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0-1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0-1, coords.1), cost:cost-1});
                        visited.insert((coords.0-1, coords.1), true);
                        tiles_in_range.push((coords.0-1, coords.1));
                    }
                }
            }
            if coords.0 < MAP_WIDTH-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0+1 as u32)) {
                    //As long as we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0+1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0+1, coords.1), cost:cost-1});
                        visited.insert((coords.0+1, coords.1), true);
                        tiles_in_range.push((coords.0+1, coords.1));
                    }
                }
            }
            if coords.1 > 0 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1-1 as u32, coords.0 as u32)) {
                    //As long as we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0, coords.1-1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1-1), cost:cost-1});
                        visited.insert((coords.0, coords.1-1), true);
                        tiles_in_range.push((coords.0, coords.1-1));
                    }
                }
            }
            if coords.1 < MAP_HEIGHT-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1+1 as u32, coords.0 as u32)) {
                    //As long as we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0, coords.1+1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1+1), cost:cost-1});
                        visited.insert((coords.0, coords.1+1), true);
                        tiles_in_range.push((coords.0, coords.1+1));
                    }
                }
            }
        }
        tiles_in_range
    }

    pub fn get_tiles_can_attack(&self, map: &mut HashMap<(u32, u32), Tile>,) -> Vec<(u32, u32)> {
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
                    //As we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0-1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0-1, coords.1), cost:cost-1});
                        visited.insert((coords.0-1, coords.1), true);
                        match entry.get().contained_unit_team {
                            Some(team) => {
                                if team != self.team {
                                    tiles_in_range.push((coords.0-1, coords.1));
                                }
                            },
                            None => {}
                        };
                    }
                }
            }
            if coords.0 < MAP_WIDTH-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0+1 as u32)) {
                    //As long as we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0+1, coords.1)){
                        heap.push(QueueObject { coords: (coords.0+1, coords.1), cost:cost-1});
                        visited.insert((coords.0+1, coords.1), true);
                        match entry.get().contained_unit_team {
                            Some(team) => {
                                if team != self.team {
                                    tiles_in_range.push((coords.0+1, coords.1));
                                }
                            },
                            None => {}
                        };
                    }
                }
            }
            if coords.1 > 0 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1-1 as u32, coords.0 as u32)) {
                    //As long as we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0, coords.1-1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1-1), cost:cost-1});
                        visited.insert((coords.0, coords.1-1), true);
                        match entry.get().contained_unit_team {
                            Some(team) => {
                                if team != self.team {
                                    tiles_in_range.push((coords.0, coords.1-1));
                                }
                            },
                            None => {}
                        };
                    }
                }
            }
            if coords.1 < MAP_HEIGHT-1 {
                if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1+1 as u32, coords.0 as u32)) {
                    //As long as we have not already visited this tile
                    if entry.get().can_attack_through && !visited.contains_key(&(coords.0, coords.1+1)){
                        heap.push(QueueObject { coords: (coords.0, coords.1+1), cost:cost-1});
                        visited.insert((coords.0, coords.1+1), true);
                        match entry.get().contained_unit_team {
                            Some(team) => {
                                if team != self.team {
                                    tiles_in_range.push((coords.0, coords.1+1));
                                }
                            },
                            None => {}
                        };
                    }
                }
            }
        }
        
        tiles_in_range
    }

    pub fn receive_damage(&mut self, damage: u32) {
        self.hp -= damage;

        //Make the unit turn red after taking damage
        self.is_attacked = true;
        self.last_damaged_drawn = Instant::now();
    }

    pub fn draw(&mut self, core: &mut SDLCore, dest: &Rect) -> Result<(), String> {
        let src = if self.has_attacked && self.has_moved {
            //Draw the darkened sprite
            self.gray_sprite_src
        } 
        else if self.is_attacked {
            self.time_since_damaged += self.last_damaged_drawn.elapsed().as_secs_f32();
            self.last_damaged_drawn = Instant::now();

            //Remove red tint after 1 second
            if self.time_since_damaged >= 1.0 {
                self.is_attacked = false;
                self.time_since_damaged = 0.0;
            }

            //Draw the sprite that's tinted red
            self.red_sprite_src
        }
        else {
            //Draw the default sprite
            self.default_sprite_src
        };
        
        //Draw the sprite
        core.wincan.copy(self.texture, src, *dest)?;

        Ok(())
    }
}

impl fmt::Display for Unit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unit(x:{}, y:{}, hp:{})", self.x, self.y, self.hp)
    }
}