use rand::{seq::IteratorRandom, Rng, thread_rng};
use std::cmp::Reverse;
use std::collections::{HashMap, BinaryHeap};
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::ai::population_state::*;
use crate::ai::distance_map::*;
use crate::game_map::GameMap;
use crate::unit::*;
use crate::tile::Tile;

//Genetic Algorithm Constants (instead of a struct to make things easier to modify and less things to pass around)
const POP_NUM: usize = 120; //Population size
const GEN_NUM: u32 = 60; //Number of generations to run
const MUT_PROB: f32 = 0.3; //Probability of an individual being mutated
const MUT_NUM: usize = 6; //How many units should be changed on mutation
const C_PERC: f32 = 0.2; //Percentage of the least fit individuals to be removed
const E_PERC: f32 = 0.1; //Proportion of best individuals to carry over from one generation to the next

//Utility Function Constants
const MIN_DISTANCE: u32 = 5; // Defines the minimum distance a unit can be from an objective to be considered near it
const DEFENDING_WEIGHT: f64 = 5.0;
const SIEGING_WEIGHT: f64 = 7.5;
const CAMP_WEIGHT: f64 = 2.5;
const ATTACK_VALUE: f64 = 1.0;
const MIN_DEFENSE: u32 = 5; //Since one of our AI goals says that some units should stay behind and defend, we need metrics to enforce this
const DEFENSE_PENALTY: f64 = 5.0;

const MAP_WIDTH: u32 = 64;
const MAP_HEIGHT: u32 = 64;

fn generate_initial_population(succinct_units: &Vec<SuccinctUnit>, map: &mut HashMap<(u32, u32), Tile>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>, distance_map: &DistanceMap) -> Vec<PopulationState> {
    let mut rng_thread = thread_rng();
    let mut population: Vec<PopulationState> = Vec::new();

    //Generate 1 less state so we can add the initial population
    for i in 1..POP_NUM {
        let mut unit_movements: Vec<((u32,u32), (f64, bool, bool, bool, bool))> = Vec::new();

        for unit in succinct_units.iter() {
            let selected_move: (u32, u32) = *unit.possible_moves.iter().choose(&mut rng_thread).unwrap();
            let move_value = current_unit_value(unit.attack_range, selected_move, map, p2_castle, p1_castle, camp_coords, distance_map);
            unit_movements.push((selected_move, move_value));
        }
        let mut state = PopulationState::new(unit_movements, 0.0);
        assign_value_to_state(&mut state);
		population.push(state);
    }

    population
}

//Randomly selects unit within a state and reassigns them a new position
//After we mutate a state we also need to be able to update its value
fn mutate(state: &mut PopulationState, succinct_units: &Vec<SuccinctUnit>, map: &mut HashMap<(u32, u32), Tile>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>, distance_map: &DistanceMap) {
    let mut rng_thread = thread_rng();
    let index_of_units_to_mutate = (0..state.units_and_utility.len() as usize).choose_multiple(&mut rng_thread, MUT_NUM);
    for index in index_of_units_to_mutate {
        //If the unit only has 1 move to choose from, nothing will change. So move on to next unit to mutate...
        if succinct_units[index].possible_moves.len() == 1 {
            continue;
        }
        //If the unit is currently capturing a camp or the enemy castle, it should not move
        if state.units_and_utility[index].1.2 || state.units_and_utility[index].1.2 {
            continue;
        }
        let mut index_of_new_move: usize = (0..succinct_units[index].possible_moves.len() as usize).choose(&mut rng_thread).unwrap();
        let mut new_move = succinct_units[index].possible_moves.get(index_of_new_move).unwrap();
        let mut attempts: u32 = 0;
        //Although is_dupe_unit_placement also takes care of the case where the current placement is the same move as before, this might allow for constant check in the best case
        while *new_move == state.units_and_utility[index].0 || state.is_dupe_unit_placement(&new_move){
            //println!("Generating new mutation {:?} has issues...", new_move);
            index_of_new_move = (0..succinct_units[index].possible_moves.len() as usize).choose(&mut rng_thread).unwrap();
            new_move = succinct_units[index].possible_moves.get(index_of_new_move).unwrap();
            //println!("New move {:?} selected", new_move);
            attempts += 1;
            //println!("Len of possible moves at index {}:{}", index, succinct_units[index].possible_moves.len());
            if attempts == 10 {
                if index_of_new_move == 0 {
                    new_move = succinct_units[index].possible_moves.get(index_of_new_move+1).unwrap();
                } else {
                    new_move = succinct_units[index].possible_moves.get(index_of_new_move-1).unwrap();
                }
                break;
            }
        }
        let move_value = current_unit_value(succinct_units[index].attack_range, *new_move, map, p2_castle, p1_castle, camp_coords, distance_map);
        state.units_and_utility[index] = (*new_move, move_value);
	}
    //Don't forget to update the overall value of the state (can't just substract the difference in values from the state as we are also checking overall conditions)
    assign_value_to_state(state);
}

// Produces 2 new states by randomly selecting 2 endpoints within the units and joining the two states at these end points
// No easy way to check for duplicates here, so we will need to do so when actually processing the move
fn crossover(state_1: &PopulationState, state_2: &PopulationState) -> (PopulationState, PopulationState) {
    let mut rng_thread = thread_rng();
    let endpoints = (0..state_1.units_and_utility.len() as usize).choose_multiple(&mut rng_thread, 2);
    let upper_endpoint = *endpoints.iter().max().unwrap();
    let lower_endpoint = *endpoints.iter().min().unwrap();
    let mut state_1_copy = state_1.clone();
    let mut state_2_copy = state_2.clone();

    let mut new_state_1_unit_movements: Vec<((u32,u32), (f64, bool, bool, bool, bool))> = Vec::new();
    let mut new_state_2_unit_movements: Vec<((u32,u32), (f64, bool, bool, bool, bool))> = Vec::new();

    new_state_1_unit_movements.append(&mut state_2_copy.units_and_utility[0..lower_endpoint].to_vec());
    new_state_1_unit_movements.append(&mut state_1_copy.units_and_utility[lower_endpoint..upper_endpoint].to_vec());
    new_state_1_unit_movements.append(&mut state_2_copy.units_and_utility[upper_endpoint..state_1.units_and_utility.len() as usize].to_vec());


    new_state_2_unit_movements.append(&mut state_1_copy.units_and_utility[0..lower_endpoint].to_vec());
    new_state_2_unit_movements.append(&mut state_2_copy.units_and_utility[lower_endpoint..upper_endpoint].to_vec());
    new_state_2_unit_movements.append(&mut state_1_copy.units_and_utility[upper_endpoint..state_1.units_and_utility.len() as usize].to_vec());

    let mut new_state_1 = PopulationState::new(new_state_1_unit_movements, 0.0);
    let mut new_state_2 = PopulationState::new(new_state_2_unit_movements, 0.0);

    //println!("len of state_1:{}, len of state_2: {}", state_1.units_and_utility.len(), state_2.units_and_utility.len());

    assign_value_to_state(&mut new_state_1);
    assign_value_to_state(&mut new_state_2);

    (new_state_1, new_state_2)
}

fn elite_selection(current_population: &Vec<PopulationState>) -> Vec<PopulationState> {
	let num_to_keep: usize = ((E_PERC * (current_population.len() as f32)).round() as i32).try_into().unwrap();

    //Assuming current_population is in descending order
	return current_population[0..num_to_keep].to_vec();
}

fn culling(current_population: &Vec<PopulationState>) -> Vec<PopulationState> {
	let num_to_drop: usize = ((C_PERC * (current_population.len() as f32)).round() as i32).try_into().unwrap();

    //Assuming current_population is in descending order
	return current_population[0..(current_population.len() - num_to_drop)].to_vec();
}

pub fn genetic_algorithm(game_map: &mut GameMap, distance_map: &DistanceMap) -> Vec<PopulationState>{
    let mut rng_thread = thread_rng();
    //Keeps track of all the possible unit movements
    let mut succinct_units: Vec<SuccinctUnit> = Vec::new();

    //Also want to include the unmodified initial state among possible candidate states
    let mut original_unit_movements: Vec<((u32,u32), (f64, bool, bool, bool, bool))> = Vec::new();

    println!("Utility Function Constants:\nMinimum Distance from Objectives: {}, Defending Weight: {}, Sieging Weight: {}, Camp Weight: {}, Value from Attack: {}, Minimum Defending Units: {}, Defense Penalty: {}\n", MIN_DISTANCE, DEFENDING_WEIGHT, SIEGING_WEIGHT, CAMP_WEIGHT, ATTACK_VALUE, MIN_DEFENSE, DEFENSE_PENALTY);
    println!("Genetic Algorithm Constants:\nPopulation Size: {}, Number of Generations: {}, Mutation Probability: {}, Number of Units Changed on Mutate: {}, Elite Percentage: {}, Culling Percentage: {}\n", POP_NUM, GEN_NUM, MUT_PROB, MUT_NUM, E_PERC, C_PERC);

    for unit in game_map.enemy_units.values() {
        let move_value = current_unit_value(unit.attack_range, (unit.x, unit.y), &mut game_map.map_tiles, &game_map.objectives.p2_castle, &game_map.objectives.p1_castle, &game_map.objectives.barbarian_camps, distance_map);
        original_unit_movements.push(((unit.x, unit.y), move_value));

        //If a unit is currently in the process of capturing, it should not consider other moves
        let current_unit = if move_value.2 || move_value.3 {
            SuccinctUnit::new(vec![(unit.x, unit.y)], unit.attack_range)
        } else {
            SuccinctUnit::new(unit.get_tiles_in_movement_range(&mut game_map.map_tiles), unit.attack_range)
        };

        succinct_units.push(current_unit);
    }

    let mut initial_population = generate_initial_population(&succinct_units, &mut game_map.map_tiles, &game_map.objectives.p2_castle, &game_map.objectives.p1_castle, &game_map.objectives.barbarian_camps, distance_map);
    let mut original_state = PopulationState::new(original_unit_movements, 0.0);
    assign_value_to_state(&mut original_state);
    initial_population.push(original_state);

    let mut new_generation: Vec<PopulationState> = Vec::new();
    let mut remaining_population: Vec<PopulationState> = Vec::new();

    for i in 0..GEN_NUM {
        initial_population.sort_unstable();
        initial_population.reverse();

        new_generation.append(&mut elite_selection(&initial_population));
        remaining_population = culling(&initial_population);

        let utilities: Vec<f64> = remaining_population.iter().map(|pop| pop.overall_utility).collect();
        let probabilities: Vec<f64> = convert_utilities_to_probabilities(utilities);

        //While we still need to fill our generation, generate new individuals using cross over
        while new_generation.len() < POP_NUM {
            let mut num_attempts = 0; //Although it should be unlikely, there is a chance that we reselct the same index multiple times, so we need to ensure otherwise

            let mut index_of_state_1 = choose_index_from_distribution(&probabilities);
            //Need to ensure that the index we selected is actually in bounds
            while index_of_state_1 == probabilities.len() {
                //println!("Selecting new index to cross; out of bounds...");
                index_of_state_1 = choose_index_from_distribution(&probabilities);
                num_attempts += 1;
                if num_attempts == 10 {
                    index_of_state_1 = probabilities.len()-1;
                }
            }

            num_attempts = 0;

            let mut index_of_state_2 = choose_index_from_distribution(&probabilities);
            //Need to make sure that we do not select the same index as crossing a state with itself produces nothing new
            while index_of_state_2 == index_of_state_1 || index_of_state_2 == probabilities.len(){
                //println!("Selecting new index to cross; either out of bounds or duplicate...");
                index_of_state_2 = choose_index_from_distribution(&probabilities);
                num_attempts += 1;
                if num_attempts == 10 {
                    if index_of_state_1 == 0 {
                        index_of_state_2 = index_of_state_1+1;
                    } else {
                        index_of_state_2 = index_of_state_1-1;
                    }
                }
            }

            let new_individuals = crossover(&remaining_population[index_of_state_1], &remaining_population[index_of_state_2]);

            if new_generation.len() + 2 > POP_NUM {
				new_generation.push(new_individuals.0);
			} else {
				new_generation.push(new_individuals.0);
                new_generation.push(new_individuals.1);
			}
        }
        //In order to mutate the states we need to calculate how many to mutate and then randomly select them as mutable
        let num_to_mutate: usize = ((MUT_PROB * (new_generation.len() as f32)).round() as i32).try_into().unwrap();
        let mut states_to_mutate = new_generation.iter_mut().choose_multiple(&mut rng_thread, num_to_mutate);
        for state in states_to_mutate.iter_mut() {
            mutate(state, &succinct_units, &mut game_map.map_tiles, &game_map.objectives.p2_castle, &game_map.objectives.p1_castle, &game_map.objectives.barbarian_camps, distance_map);
        }

        initial_population = new_generation.clone();
        let best_individual = initial_population.iter().max().unwrap();
        //Only print every 5 generations to save console from becoming unreadable
        if i % 5 == 0 {
            println!("Best score in generation {}:{}", i, best_individual.overall_utility);
            let moves: Vec<(u32, u32)> = best_individual.units_and_utility.iter().map(|tup| tup.0).collect();
            println!("Moves:{:?}\n", moves);
        }
        //println!("Num units: {}", best_individual.units_and_utility.len());
        //Also need to remember to reset the corresponding vectors for the next generation
        new_generation = Vec::new();
        remaining_population = Vec::new();
    }

    //init_population now generally represents the best possible states that have been found and we can use these to form the considered moves of our minimax and we can repeat this for the enemy to get their "best" move and make the decision from there
    initial_population
}

//Evaluation/Utility function related
fn assign_value_to_state (current_state: &mut PopulationState) {
    let mut total_value: f64 = 0.0;
    let mut units_defending: u32 = 0; //Units near own castle
    let mut units_sieging: u32 = 0; //Units near enemy castle
    let mut units_near_camp: u32 = 0;
    let mut units_able_to_attack: u32 = 0;

    //println!("Utility Function Constants:\nMinimum Distance from Objectives: {}, Defending Weight: {}, Sieging Weight: {}, Camp Weight: {}, Value from Attack: {}, Minimum Defending Units: {}, Defense Penalty: {}\n", MIN_DISTANCE, DEFENDING_WEIGHT, SIEGING_WEIGHT, CAMP_WEIGHT, ATTACK_VALUE, MIN_DEFENSE, DEFENSE_PENALTY);

    for value in current_state.units_and_utility.iter() {
        total_value += value.1.0;
        units_defending += value.1.1 as u32;
        units_sieging += value.1.2 as u32;
        units_near_camp += value.1.3 as u32;
        units_able_to_attack += value.1.4 as u32;
    }

    // Calculations for state as a whole (not individual units)
    if units_defending < MIN_DEFENSE {
        total_value = total_value/DEFENSE_PENALTY;
    }
    //Will eventually want to add on values for units sieging, near camps, attacking, etc (ie prefer sieging a castle with x units over y)

    //println!("Total value: {}\nUnits near p2 castle: {}\nUnits near p1 castle: {}\nUnits near camps: {}\nUnits able to attack: {}\n", total_value, units_defending, units_sieging, units_near_camp, units_able_to_attack);

    current_state.overall_utility = total_value;
}

// Order of values in return
// 0: value of state
// 1: near_own_castle
// 2: sieging
// 3: capturing_camp
// 4: able_to_attack
// Minus "being able to attack" all other values will be calculated using heuristics (relative manhattan distance)
// Additionally not calculating closest unit to save time since based on the distance from objectives and the ability to attack this distance should be implied
fn current_unit_value (unit_attack_range: u32, unit_pos: (u32, u32), map: &mut HashMap<(u32, u32), Tile>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>, distance_map: &DistanceMap) -> (f64, bool, bool, bool, bool) {
    let mut value: f64 = 0.0;

    //let distance_from_own_castle = (unit_pos.0 as i32 - p2_castle.0 as i32).abs() + (unit_pos.1 as i32 - p2_castle.1 as i32).abs();
    let distance_from_own_castle: u32 = if let Some(dist) = distance_map.to_enemy_castle.get(&unit_pos) {
                                        *dist
                                    } else {
                                        panic!();
                                        100000
                                    };

    let defending: bool = if distance_from_own_castle <= MIN_DISTANCE {
                        true
                    } else {
                        false
                    };

    let distance_from_enemy_castle = if let Some(dist) = distance_map.to_player_castle.get(&unit_pos) {
                                        *dist
                                    } else {
                                        panic!();
                                        100000
                                    };

    let sieging: bool =   if distance_from_enemy_castle == 0 {
                        true
                    } else {
                        false
                    };

    let distance_from_nearest_camp: i32 = if camp_coords.len() > 0 {
        let mut min_distance_index = 0;
        let mut min_distance = 1000;
        for index in 0..camp_coords.len() {
            let camp = camp_coords.get(index).unwrap();
            let current_distance = (unit_pos.0 as i32 - camp.0 as i32).abs() + (unit_pos.1 as i32 - camp.1 as i32).abs();
            if current_distance < min_distance {
                min_distance = current_distance;
                min_distance_index = index;
            }
        }
        if let Some(hash_map) = distance_map.to_barbarian_camps.get(&camp_coords.get(min_distance_index).unwrap()) {
            if let Some(dist) = hash_map.get(&unit_pos) {
                let min_camp = camp_coords.get(min_distance_index).unwrap();
                if unit_pos == (min_camp.0+1, min_camp.1) || unit_pos == (min_camp.0, min_camp.1+1) || unit_pos == (min_camp.0+1, min_camp.1+1) {
                    0
                } else {
                    *dist as i32
                }
            } else {
                panic!();
                100000
            }
        } else {
            panic!();
        }
    } else {
        -1
    };

    let capturing_camp: bool = if distance_from_nearest_camp == 0 {
                        true
                    } else {
                        false
                    };

    let tiles_to_attack = generalized_tiles_can_attack(map, unit_pos, unit_attack_range);
    let able_to_attack: bool =  if tiles_to_attack.is_empty() {
                                    false
                                } else {
                                    true
                                };
    //Currently commenting this out for now, I don't know if we don't want to punish units for not defending or just if there isn't enough defending
    // if defending == false {
    //     value += distance_from_own_castle as f64 * DEFENDING_WEIGHT;
    // }
    if distance_from_enemy_castle != 0 {
        value += SIEGING_WEIGHT/(distance_from_enemy_castle as f64);
    } else {
        value += SIEGING_WEIGHT*2.0;
    }
    if distance_from_nearest_camp > 0 {
        value += CAMP_WEIGHT/(distance_from_nearest_camp as f64);
    } else if distance_from_nearest_camp == 0 {
        value += CAMP_WEIGHT*3.0;
    }

    if able_to_attack == true {
        value += ATTACK_VALUE * (*tiles_to_attack.iter().min().unwrap() as f64); //Should favor moves that allows unit to attack from further away
    }

    //println!("Unit at {}, {}\nValue: {}, D(own_castle): {}, D(enemy_castle): {}, D(camp): {}, can_attack: {}\n", unit_pos.0, unit_pos.1, value, distance_from_own_castle, distance_from_enemy_castle, distance_from_nearest_camp, able_to_attack);

    (value, defending, sieging, capturing_camp, able_to_attack)
}

//In order to convert utilities into probabilities, we are using the Boltzman distribution (slightly flipped since we are aiming for max instead of min)
fn convert_utilities_to_probabilities(utilities: Vec<f64>) -> Vec<f64>{
    let min_utility = *utilities.iter().min_by(|a,b| a.partial_cmp(&b).unwrap()).unwrap();
    let max_utility = *utilities.iter().max_by(|a,b| a.partial_cmp(&b).unwrap()).unwrap();
    let temperature = max_utility - min_utility;
    let utilities_to_p_accept: Vec<f64> = utilities.iter().map(|current_utility| (-(max_utility - current_utility)/temperature).exp()).collect();
    let p_accept_sum:f64 = utilities_to_p_accept.iter().sum();
    utilities_to_p_accept.iter().map(|p_accept| p_accept/p_accept_sum).collect()
}

//Randomly select an index by summing values of distribution until we exceed a random value
//since our higher valued utilities are first they have a higher likelihood of being selected
fn choose_index_from_distribution(probabilities: &Vec<f64>) -> usize {
    let mut rng_thread = thread_rng();
    let rand_num: f64 = rng_thread.gen();
    let mut sum:f64 = 0.0;
    for index in 0..probabilities.len() {
        sum += probabilities[index];
        if rand_num <= sum {
            return index;
        }
    }
    return probabilities.len();
}

// Perform a bidirectional search to find the actual distance of the unit from the goal
pub fn get_actual_distance_from_goal(unit_pos: (u32, u32), goal_pos: (u32, u32), map: &mut HashMap<(u32, u32), Tile>) -> u32 {
    let mut visited_init: HashMap<(u32,u32), u32> = HashMap::new();
    let mut visited_goal: HashMap<(u32,u32), u32> = HashMap::new();
    let mut init_heap = BinaryHeap::new();
    let mut goal_heap = BinaryHeap::new();

    //Base case
    if unit_pos == goal_pos {
        return 0;
    }

    init_heap.push(Reverse(QueueObject{coords: (unit_pos.0, unit_pos.1), cost: 0}));
    visited_init.insert((unit_pos.0, unit_pos.1), 0);
    goal_heap.push(Reverse(QueueObject{coords: (goal_pos.0, goal_pos.1), cost: 0}));
    visited_goal.insert((goal_pos.0, goal_pos.1), 0);

    while !init_heap.is_empty() && !goal_heap.is_empty() {
        //If the init_heap is further along, then we should work on expanding goal
        if init_heap.peek().unwrap() < goal_heap.peek().unwrap() {
            if let Some(Reverse(QueueObject { coords, cost })) = goal_heap.pop() {
                if coords.0 > 0 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0-1 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_init.get(&(coords.0-1, coords.1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_goal.contains_key(&(coords.0-1, coords.1)){
                            goal_heap.push(Reverse(QueueObject { coords: (coords.0-1, coords.1), cost:cost+1}));
                            visited_goal.insert((coords.0-1, coords.1), cost+1);
                        }
                    }
                }
                if coords.0 < MAP_WIDTH-1 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0+1 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_init.get(&(coords.0+1, coords.1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_goal.contains_key(&(coords.0+1, coords.1)){
                            goal_heap.push(Reverse(QueueObject { coords: (coords.0+1, coords.1), cost:cost+1}));
                            visited_goal.insert((coords.0+1, coords.1), cost+1);
                        }
                    }
                }
                if coords.1 > 0 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1-1 as u32, coords.0 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_init.get(&(coords.0, coords.1-1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_goal.contains_key(&(coords.0, coords.1-1)){
                            goal_heap.push(Reverse(QueueObject { coords: (coords.0, coords.1-1), cost:cost+1}));
                            visited_goal.insert((coords.0, coords.1-1), cost+1);
                        }
                    }
                }
                if coords.1 < MAP_HEIGHT-1 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1+1 as u32, coords.0 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_init.get(&(coords.0, coords.1+1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_goal.contains_key(&(coords.0, coords.1+1)){
                            goal_heap.push(Reverse(QueueObject { coords: (coords.0, coords.1+1), cost:cost+1}));
                            visited_goal.insert((coords.0, coords.1+1), cost+1);
                        }
                    }
                }
            }
        } else {
            if let Some(Reverse(QueueObject { coords, cost })) = init_heap.pop() {
                if coords.0 > 0 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0-1 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_goal.get(&(coords.0-1, coords.1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_init.contains_key(&(coords.0-1, coords.1)){
                            init_heap.push(Reverse(QueueObject { coords: (coords.0-1, coords.1), cost:cost+1}));
                            visited_init.insert((coords.0-1, coords.1), cost+1);
                        }
                    }
                }
                if coords.0 < MAP_WIDTH-1 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1 as u32, coords.0+1 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_goal.get(&(coords.0+1, coords.1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_init.contains_key(&(coords.0+1, coords.1)){
                            init_heap.push(Reverse(QueueObject { coords: (coords.0+1, coords.1), cost:cost+1}));
                            visited_init.insert((coords.0+1, coords.1), cost+1);
                        }
                    }
                }
                if coords.1 > 0 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1-1 as u32, coords.0 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_goal.get(&(coords.0, coords.1-1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_init.contains_key(&(coords.0, coords.1-1)){
                            init_heap.push(Reverse(QueueObject { coords: (coords.0, coords.1-1), cost:cost+1}));
                            visited_init.insert((coords.0, coords.1-1), cost+1);
                        }
                    }
                }
                if coords.1 < MAP_HEIGHT-1 {
                    if let std::collections::hash_map::Entry::Occupied(entry) = map.entry((coords.1+1 as u32, coords.0 as u32)) {
                        //If we have already visited this tile from the other direction, the sum of the costs is the actual distance
                        if let Some(num) = visited_goal.get(&(coords.0, coords.1+1)) {
                            return num + cost+1;
                        }
                        //As long as a unit can move to this tile and we have not already visited this tile
                        if entry.get().is_traversable && !visited_init.contains_key(&(coords.0, coords.1+1)){
                            init_heap.push(Reverse(QueueObject { coords: (coords.0, coords.1+1), cost:cost+1}));
                            visited_init.insert((coords.0, coords.1+1), cost+1);
                        }
                    }
                }
            }
        }
    }
    0
}

//Creates a txt file containing rust code that initializes a bunch of hashmaps that contain the distance from each tile to each goal area
pub fn get_goal_distances(map: &mut HashMap<(u32, u32), Tile>, p1_castle: (u32, u32), enemy_castle: (u32, u32), camp_coords: &Vec<(u32, u32)>) -> Result<(), String>{
    println!("Calculating distances to each goal from each tile");

    let file = File::create("./src/AI/distances.txt").expect("Could not create src/AI/distances.txt");
    let mut file_io = BufWriter::new(file);

    //Get distance from each tile to the p1 castle
    writeln!(file_io, "p1_castle").expect("Write error");
    for i in 0..MAP_HEIGHT {
        for j in 0..MAP_WIDTH {
            //Flip i & j so that they are in (x, y) order in the file
            let dist = get_actual_distance_from_goal((j, i), p1_castle, map);
            writeln!(file_io, "{} {} {}", j, i, dist).expect("Write error");
        }
    }
    writeln!(file_io, "end").expect("Write error");
    writeln!(file_io).expect("Write error");

    //Get distance from each tile to the enemy castle
    writeln!(file_io, "enemy_castle").expect("Write error");
    for i in 0..MAP_HEIGHT {
        for j in 0..MAP_WIDTH {
            //Flip i & j so that they are in (x, y) order in the file
            let dist = get_actual_distance_from_goal((j, i), enemy_castle, map);
            writeln!(file_io, "{} {} {}", j, i, dist).expect("Write error");
        }
    }
    writeln!(file_io, "end").expect("Write error");
    writeln!(file_io).expect("Write error");

    //Get the distance from each tile to each barbarian camp
    writeln!(file_io, "barb_camps").expect("Write error");
    for cur_camp in camp_coords.iter() {
        writeln!(file_io, "# {} {}", cur_camp.0, cur_camp.1).expect("Write error");
        for i in 0..MAP_HEIGHT {
            for j in 0..MAP_WIDTH {
                //Flip i & j so that they are in (x, y) order in the file
                let dist = get_actual_distance_from_goal((j, i), *cur_camp, map);
                writeln!(file_io, "{} {} {}", j, i, dist).expect("Write error");
            }
        }
    }
    writeln!(file_io, "end").expect("Write error");

    Ok(())
}