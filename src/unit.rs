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
use crate::player_state::PlayerState;
use crate::net::util::*;

const MAP_WIDTH: u32 = 64;
const MAP_HEIGHT: u32 = 64;

pub const GUARD_HEALTH_ID: u32 = 25;
pub const SCOUT_HEALTH_ID: u32 = 9;

pub enum Team {
	Player,
	Enemy,
	Barbarians,
}
impl Team {
    // Swaps the player/enemy enum values if invoked on the 'peer' client.
    // Since the player state is shared between clients, some properties will
    // use the "player" value to represent the host and "enemy" as the peer.
    pub fn as_client(self, player_state: &PlayerState) -> Team {
        if player_state.team == Team::Enemy {
            match self {
                Team::Player => Team::Enemy,
                Team::Enemy => Team::Player,
                _ => self,
            }
        } else { self }
    }

    pub fn to_id(self) -> u8 {
        match self {
            Team::Player => EVENT_ID_PLAYER,
            Team::Enemy => EVENT_ID_ENEMY,
            Team::Barbarians => EVENT_ID_BARBARIAN,
        }
    }

    pub fn from_id(id: u8) -> Result<Team, String> {
        match id {
            EVENT_ID_PLAYER => Ok(Team::Player),
            EVENT_ID_ENEMY => Ok(Team::Enemy),
            EVENT_ID_BARBARIAN => Ok(Team::Barbarians),
            _ => Err("Invalid team id".to_string())
        }
    }
}
impl ToString for Team {
    fn to_string(&self) -> String {
        match self {
            Team::Player => "player",
            Team::Enemy => "enemy",
            Team::Barbarians => "barbarians",
        }.to_string()
    }
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
    pub draw_x: f64,
    pub draw_y: f64,
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

    ranged_attacker: bool,

    //Used for barbarians to make sure they roam within a small radius
    pub starting_x: u32,
    pub starting_y: u32,
}

impl Unit <'_>{
    pub fn new<'a> (x:u32, y:u32, team: Team, hp: u32, movement_range: u32, attack_range: u32, accuracy: u32, min_damage:u32, max_damage: u32, texture: &'a Texture, ranged_attacker: bool) -> Unit<'a> {
        Unit {
            draw_x: -1.0,
            draw_y: -1.0,
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

            ranged_attacker,

            starting_x: x,
            starting_y: y,
        }
    }

    pub fn get_attack_damage(&self, other: &Unit) -> u32 {
        let chance = rand::thread_rng().gen_range(0..100);
        let scout_debuff = if other.max_hp == SCOUT_HEALTH_ID {
            20
        } else {
            0
        };
        if chance < self.accuracy.checked_sub(scout_debuff).unwrap_or(0) {
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
        respawn_loc((self.x, self.y), map, where_to_spawn)
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

    pub fn get_tiles_can_attack_from_pos(&self, from_pos: (u32, u32), map: &mut HashMap<(u32, u32), Tile>,) -> Vec<(u32, u32)> {
        let mut tiles_in_range: Vec<(u32, u32)> = Vec::new();
        let mut visited: HashMap<(u32,u32), bool> = HashMap::new();
        let mut heap = BinaryHeap::new();
        heap.push(QueueObject{coords: (from_pos.0, from_pos.1), cost: self.attack_range});
        visited.insert((from_pos.0, from_pos.1), true);
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

    pub fn receive_damage(&mut self, damage: u32, other: &Unit) {
        let mut do_damage = damage;
        if self.max_hp == GUARD_HEALTH_ID && other.ranged_attacker && damage > 1 {
            do_damage /= 2;
        }
        self.hp = self.hp.checked_sub(do_damage).unwrap_or(0);

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

        if self.draw_x < 0.0 || self.draw_y < 0.0 {
            self.draw_x = dest.x as f64;
            self.draw_y = dest.y as f64;
        }

        self.draw_x = (self.draw_x + dest.x as f64) / 2.0;
        self.draw_y = (self.draw_y + dest.y as f64) / 2.0;
        let rect = Rect::new(self.draw_x as i32, self.draw_y as i32, dest.width(), dest.height());

        //Draw the sprite
        core.wincan.copy(self.texture, src, rect)?;

        Ok(())
    }
}

impl fmt::Display for Unit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unit(x:{}, y:{}, hp:{})", self.x, self.y, self.hp)
    }
}

pub fn respawn_loc(castle_coords: (u32, u32), map: &mut HashMap<(u32, u32), Tile>, where_to_spawn: (u32,u32)) -> (u32, u32) {
    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((where_to_spawn.1, where_to_spawn.0)) {
        //As long as a unit can move to this tile return it otherwise find closest available
        if entry.get().unit_can_move_here() {
            where_to_spawn
        } else {
            let mut y_increment: i32 = 1;
            let mut x_increment: i32 = 1;
            let mut current_x:i32 = where_to_spawn.0 as i32;
            let mut current_y:i32 = where_to_spawn.1 as i32;

            if where_to_spawn.1 > castle_coords.1 { //If the coorodinate is below then our increment should be -1;
                y_increment = -1;
            } else if where_to_spawn.1 == castle_coords.1 { //If the coordinates are at the same y focus on moving x first
                y_increment = 0;
            }
            if where_to_spawn.0 > castle_coords.0 { //If the coorodinate is to the right then our increment should be -1;
                x_increment = -1;
            } else if where_to_spawn.0 == castle_coords.0 { //If the coordinates are at the same x focus on moving y first
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
                if current_x == castle_coords.0 as i32 && current_y == castle_coords.1 as i32 {
                    break;
                }
            }
            //In the event that no closer moves are found, stay at current position
            (castle_coords.0, castle_coords.1)
        }
    } else {
        panic!("Trying to spawn unit off map")
    }
}
