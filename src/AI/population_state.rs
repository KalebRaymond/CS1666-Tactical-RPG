use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use crate::tile::Tile;
use crate::unit::{Unit, Team, QueueObject};
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
pub fn generalized_tiles_can_attack(map: &mut HashMap<(u32, u32), Tile>, coordinates: (u32, u32), range: u32) -> Vec<(u32, u32)> {
    let mut tiles_in_range: Vec<(u32, u32)> = Vec::new();
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
                            if team != Team::Enemy {
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
                            if team != Team::Enemy {
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
                            if team != Team::Enemy {
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