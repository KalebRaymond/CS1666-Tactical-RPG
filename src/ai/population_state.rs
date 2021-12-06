use rand::Rng;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use sdl2::render::Texture;

use crate::tile::Tile;
use crate::unit::{Unit, Team, QueueObject};
use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::damage_indicator::DamageIndicator;
use crate::SDLCore;

const MAP_WIDTH: u32 = 64;
const MAP_HEIGHT: u32 = 64;

#[derive(Clone)]
pub struct PopulationState {
    //Will likely need a struct to keep track of individuals in a population (all units current position and the value of that state)
    pub units_and_utility: Vec<((u32, u32), (f64, bool, bool, bool, bool))>,
    pub overall_utility: f64,
}

impl PopulationState {
    pub fn new(units_and_utility: Vec<((u32, u32), (f64, bool, bool, bool, bool))>, overall_utility: f64) -> PopulationState {
        PopulationState{
            units_and_utility,
            overall_utility,
        }
    }
    // To reduce some of the issues with units moving to the same tile, adding a quick check to see if there is already a unit at this tile
    // returns true if a unit is already at this tile and false if otherwise
    pub fn is_dupe_unit_placement(&self, coordinates:&(u32,u32)) -> bool {
        for unit in self.units_and_utility.iter() {
            if unit.0 == *coordinates {
                //println!("{},{} == {},{}; this move already exists...", coordinates.0, coordinates.1, unit.0.0, unit.0.1);
                return true;
            }
        }
        false
    }

    // It does not matter if there is a duplicate move after only before
    // This is used in actually converting the state to action
    pub fn is_dupe_unit_placement_ending_at(&self, coordinates:&(u32,u32), ending_point: usize) -> bool {
        for index in 0..ending_point {
            if self.units_and_utility[index].0 == *coordinates {
                //println!("{},{} == {},{}; this move already exists...", coordinates.0, coordinates.1, self.units_and_utility[index].0.0, self.units_and_utility[index].0.1);
                return true;
            }
        }
        false
    }

    // Currently just returns the movements for each unit (will eventually also handle attacks)
    pub fn convert_state_to_action<'a> (&self, core: &SDLCore<'a>, unit_textures: &'a HashMap<String, Texture<'a>>, game_map: &mut GameMap<'a>) -> Result<(), String> {
        let mut actual_units_mut = game_map.enemy_units.values_mut();
        let mut actual_moves: Vec<((u32, u32), (u32, u32))> = Vec::new();  //Original coordinates followed by new coordinates
        //println!("{} == {}", actual_units_mut.len(), self.units_and_utility.len()); //Check to make sure same size
        //Both the hashmap of units and the vector of moves should be the same length; if not something went wrong and should panic
        for index in 0..self.units_and_utility.len() {
            let mut new_move = self.units_and_utility[index].0;
            let mut actual_unit = actual_units_mut.next().unwrap(); //Units should be in order so we can just use next to get corresponding unit (nth panics)

            // If this move exists in the moves of the unit, move to it...
            if !self.is_dupe_unit_placement_ending_at(&new_move, index) {
                //Would like to update the hashmap of units but borrow checker says otherwise...
                actual_moves.push(((actual_unit.x, actual_unit.y), new_move));
            } else { // Else, we need to move to the closest possible tile
                println!("Best move not possible; need to find closest tile...");
                print!("Old:{},{} -> ", new_move.0, new_move.1);
                new_move = actual_unit.get_closest_move(new_move, &mut game_map.map_tiles);
                println!("New:{},{}\n", new_move.0, new_move.1);
                actual_moves.push(((actual_unit.x, actual_unit.y), new_move));
            }
            // Update map tiles (even though we are not updating units, should still update map to properly restrict movements)
            // Have to remember that map indexing is swapped
            if let Some(old_map_tile) = game_map.map_tiles.get_mut(&(actual_unit.y, actual_unit.x)) {
                old_map_tile.update_team(None);
            }
            if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(new_move.1, new_move.0)) {
                new_map_tile.update_team(Some(Team::Enemy));
            }
        }

        //Now need to actually act on these moves now that units are no longer being borrowed
        for (ogcoord, newcoord) in actual_moves {
            let mut active_unit = game_map.enemy_units.remove(&(ogcoord.0, ogcoord.1)).unwrap();
            active_unit.update_pos(newcoord.0, newcoord.1);

            //Also need to handle the attack at this tile if there is an attack
            let enemies_to_attack = active_unit.get_tiles_can_attack(&mut game_map.map_tiles);
            let mut dead_barb: bool = false;
            if !enemies_to_attack.is_empty() {
                //The enemy should attack the unit with the least health
                let mut tile_with_least_health: (u32, u32) = enemies_to_attack[0];
                let mut least_health: u32 = 1000;
                for possible_attack in enemies_to_attack.iter() {
                    if let Some(tile_under_attack) = game_map.map_tiles.get_mut(&(possible_attack.1, possible_attack.0)) {
                        match tile_under_attack.contained_unit_team {
                            Some(Team::Player) => {
                                if let Some(unit) = game_map.player_units.get_mut(&(possible_attack.0, possible_attack.1)) {
                                    if unit.hp < least_health {
                                        least_health = unit.hp;
                                        tile_with_least_health = (possible_attack.0, possible_attack.1);
                                    }
                                }
                            },
                            _ => {
                                if let Some(unit) = game_map.barbarian_units.get_mut(&(possible_attack.0, possible_attack.1)) {
                                    if unit.hp < least_health {
                                        least_health = unit.hp;
                                        tile_with_least_health = (possible_attack.0, possible_attack.1);
                                    }
                                }
                            } //This handles the barbarian case and also prevents rust from complaining about unchecked cases,
                        }
                    }
                }
                if let Some(tile_under_attack) = game_map.map_tiles.get_mut(&(tile_with_least_health.1, tile_with_least_health.0)) {
                    match tile_under_attack.contained_unit_team {
                        Some(Team::Player) => {
                            if let Some(unit) = game_map.player_units.get_mut(&(tile_with_least_health.0, tile_with_least_health.1)) {
                                let damage_done = active_unit.get_attack_damage(unit);
                                println!("Unit starting at {} hp.", unit.hp);
                                if unit.hp <= damage_done {
                                    game_map.player_units.remove(&(tile_with_least_health.0, tile_with_least_health.1));
                                    println!("Player unit at {}, {} is dead after taking {} damage.", tile_with_least_health.0, tile_with_least_health.1, damage_done);
                                    tile_under_attack.update_team(None);
                                } else {
                                    unit.receive_damage(damage_done, &active_unit);
                                    game_map.damage_indicators.push(DamageIndicator::new(core, damage_done, PixelCoordinates::from_matrix_indices(
                                        unit.y.checked_sub(1).unwrap_or(unit.y),
                                        unit.x
                                    ))?);
                                    println!("Enemy at {}, {} attacking player unit at {}, {} for {} damage. Unit now has {} hp.", active_unit.x, active_unit.y, tile_with_least_health.0, tile_with_least_health.1, damage_done, unit.hp);
                                }
                            }
                        },
                        _ => {
                            if let Some(unit) = game_map.barbarian_units.get_mut(&(tile_with_least_health.0, tile_with_least_health.1)) {
                                let damage_done = active_unit.get_attack_damage(unit);
                                println!("Barbarian unit starting at {} hp.", unit.hp);
                                if unit.hp <= damage_done {
                                    game_map.barbarian_units.remove(&(tile_with_least_health.0, tile_with_least_health.1));
                                    println!("Barbarian unit at {}, {} is dead after taking {} damage.", tile_with_least_health.0, tile_with_least_health.1, damage_done);
                                    tile_under_attack.update_team(None);
                                    dead_barb = true;
                                } else {
                                    unit.receive_damage(damage_done, &active_unit);
                                    game_map.damage_indicators.push(DamageIndicator::new(core, damage_done, PixelCoordinates::from_matrix_indices(
                                        unit.y.checked_sub(1).unwrap_or(unit.y),
                                        unit.x
                                    ))?);
                                    println!("Enemy at {}, {} attacking barbarian unit at {}, {} for {} damage. Unit now has {} hp.", active_unit.x, active_unit.y, tile_with_least_health.0, tile_with_least_health.1, damage_done, unit.hp);
                                }
                            }
                        } //This handles the enemy case and also prevents rust from complaining about unchecked cases,
                    }
                }
            }

            //Don't forget to reinsert the unit into the hashmap
            game_map.enemy_units.insert((newcoord.0, newcoord.1), active_unit);
            if dead_barb {
                //Need to check and see if this barbarian was converted - currently a 45% chance
                let chance = rand::thread_rng().gen_range(0..100);
                if chance < 45 {
                    print!("Barbarian has been converted.");
                    //Create the new unit with default stats and update the position of it accordingly
                    let castle_coord = &game_map.objectives.p2_castle;
                    //Since all the units are of relatively equal value at base stats, we can randomly choose among them similar to how a player would
                    let mut new_unit = if chance < 15 {
                            println!(" Melee selected.");
                            Unit::new(castle_coord.0+5, castle_coord.1-5, Team::Enemy, 20, 7, 1, 95, 1, 5, unit_textures.get("pl2l").unwrap(), false)
                        } else if chance < 30 {
                            println!(" Ranged selected.");
                            Unit::new(castle_coord.0+5, castle_coord.1-5, Team::Enemy, 15, 5, 4, 85, 3, 7, unit_textures.get("pl2r").unwrap(), true)
                        } else {
                            println!(" Mage selected.");
                            Unit::new(castle_coord.0+5, castle_coord.1-5, Team::Enemy, 10, 6, 3, 75,  5, 9, unit_textures.get("pl2m").unwrap(), true)
                        };

                    let respawn_location = new_unit.respawn_loc(&mut game_map.map_tiles, *castle_coord);
                    new_unit.update_pos(respawn_location.0, respawn_location.1);
                    println!("Unit spawned at {}, {}", respawn_location.0, respawn_location.1);
                    //Don't forget to update the players units and the hash map
                    game_map.enemy_units.insert(respawn_location, new_unit);
                    if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(respawn_location.1, respawn_location.0)) {
                        new_map_tile.update_team(Some(Team::Enemy));
                    }
                }
            }
        }
        Ok(())
    }

}

impl Ord for PopulationState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.overall_utility.partial_cmp(&other.overall_utility).unwrap()
    }
}

impl PartialOrd for PopulationState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PopulationState {
    fn eq(&self, other: &Self) -> bool {
        self.overall_utility == other.overall_utility
    }
}

impl Eq for PopulationState {
}

//A succinct way to represent units since we will only be concerned with possible_moves and attack_range
pub struct SuccinctUnit {
    pub possible_moves: Vec<(u32, u32)>,
    pub attack_range: u32,
}

impl SuccinctUnit {
    pub fn new(possible_moves: Vec<(u32, u32)>, attack_range: u32) -> SuccinctUnit {
        SuccinctUnit{
            possible_moves,
            attack_range,
        }
    }
}

//Since we won't be passing around units, we need to create a generalized way to get units that can be attacked
//Unlike the regular can attack function we only care about the units distance from that tile here
pub fn generalized_tiles_can_attack(map: &mut HashMap<(u32, u32), Tile>, coordinates: (u32, u32), range: u32) -> Vec<u32> {
    let mut tiles_in_range: Vec<u32> = Vec::new();
    let mut visited: HashMap<(u32,u32), bool> = HashMap::new();
    let mut heap = BinaryHeap::new();
    heap.push(QueueObject{coords: (coordinates.0, coordinates.1), cost: range});
    visited.insert((coordinates.0, coordinates.1), true);
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
                            if team != Team::Enemy {
                                tiles_in_range.push(range-(cost-1));
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
                            if team != Team::Enemy {
                                tiles_in_range.push(range-(cost-1));
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
                            if team != Team::Enemy {
                                tiles_in_range.push(range-(cost-1));
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
                            if team != Team::Enemy {
                                tiles_in_range.push(range-(cost-1));
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