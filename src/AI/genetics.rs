use rand::{seq::IteratorRandom, thread_rng};
use std::collections::HashMap;
use std::convert::TryInto;

use crate::AI::genetic_params::GeneticParams;
use crate::AI::population_state::*;
use crate::game_map::GameMap;
use crate::unit::Unit;
use crate::tile::Tile;

//Genetic Algorithm Constants (instead of a struct to make things easier to modify and less things to pass around)
const POP_NUM: u32 = 30; //Population size
const GEN_NUM: u32 = 0; //Number of generations to run
const MUT_PROB: f32 = 0.1; //Probability of an individual being mutated
const MUT_NUM: usize = 5; //How many units should be changed on mutation
const C_PERC: f32 = 0.2; //Percentage of the least fit individuals to be removed
const E_PERC: f32 = 0.1; //Proportion of best individuals to carry over from one generation to the next

//Utility Function Constants
const MIN_DISTANCE: i32 = 5; // Defines the minimum distance a unit can be from an objective to be considered near it
const DEFENDING_WEIGHT: f64 = -0.5;
const SIEGING_WEIGHT: f64 = -0.75;
const CAMP_WEIGHT: f64 = -0.25;
const ATTACK_VALUE: f64 = 10.0;
const MIN_DEFENSE: u32 = 5; //Since one of our AI goals says that some units should stay behind and defend, we need metrics to enforce this
const DEFENSE_PENALTY: f64 = -500.0;

fn generate_initial_population(succinct_units: &Vec<SuccinctUnit>, map: &mut HashMap<(u32, u32), Tile>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) -> Vec<PopulationState> {
    let mut rng_thread = thread_rng();
    let mut population: Vec<PopulationState> = Vec::new();
    
    //Generate 1 less state so we can add the initial population
    for i in 1..POP_NUM {
        let mut unit_movements: Vec<((u32,u32), (f64, bool, bool, bool, bool))> = Vec::new();

        for unit in succinct_units.iter() {
            let selected_move: (u32, u32) = *unit.possible_moves.iter().choose(&mut rng_thread).unwrap();
            let move_value = current_unit_value(unit.attack_range, selected_move, map, p2_castle, p1_castle, camp_coords);
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
fn mutate(state: &mut PopulationState, succinct_units: &Vec<SuccinctUnit>, map: &mut HashMap<(u32, u32), Tile>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) {
    let mut rng_thread = thread_rng();
    let index_of_units_to_mutate = (0..state.units_and_utility.len() as usize).choose_multiple(&mut rng_thread, MUT_NUM); 
    for index in index_of_units_to_mutate {
		let mut new_move: (u32, u32) = *succinct_units[index].possible_moves.iter().choose(&mut rng_thread).unwrap();
        while new_move == state.units_and_utility[index].0 {
            new_move = *succinct_units[index].possible_moves.iter().choose(&mut rng_thread).unwrap();   
        }
        let move_value = current_unit_value(succinct_units[index].attack_range, new_move, map, p2_castle, p1_castle, camp_coords);
        state.units_and_utility[index] = (new_move, move_value);
	}
    //Don't forget to update the overall value of the state (can't just substract the difference in values from the state as we are also checking overall conditions)
    assign_value_to_state(state); 
}

// fn crossover(state_1: PopulationState, state_2: PopulationState) -> (PopulationState, PopulationState) {
//     //let new_state_1 = PopulationState::new();
//     //let new_state_2 = PopulationState::new();

//     /*
//     randomly select bounds for state1
// 	new_state.push(the units within these bounds from state1)
// 	new_state.push(the remaining units from state2)
// 	new_state2.push(the units outside the bounds of state1)
// 	new_state2.push(the remaining units from state 2)
//     */

//     //(new_state_1, new_state_2)
// }

fn elite_selection(current_population: &mut Vec<PopulationState>) -> Vec<PopulationState> {
	let num_to_keep: usize = ((E_PERC * (current_population.len() as f32)).round() as i32).try_into().unwrap();
	
    //Assuming current_population is in descending order 
	return current_population[0..num_to_keep].to_vec();
}

fn culling(current_population: &mut Vec<PopulationState>) -> Vec<PopulationState> {
	let num_to_drop: usize = ((C_PERC * (current_population.len() as f32)).round() as i32).try_into().unwrap();
	
    //Assuming current_population is in descending order 
	return current_population[0..(current_population.len() - num_to_drop)].to_vec();
}

pub fn genetic_algorithm(units: &HashMap<(u32, u32), Unit>, game_map: &mut GameMap, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) -> Vec<PopulationState>{
    let mut rng_thread = thread_rng();
    //Keeps track of all the possible unit movements
    let mut succinct_units: Vec<SuccinctUnit> = Vec::new();

    //Also want to include the unmodified initial state among possible candidate states
    let mut original_unit_movements: Vec<((u32,u32), (f64, bool, bool, bool, bool))> = Vec::new();
    
    for unit in units.values() {  
        let current_unit = SuccinctUnit::new(unit.get_tiles_in_movement_range(&mut game_map.map_tiles), unit.attack_range);
        
        let move_value = current_unit_value(current_unit.attack_range, (unit.x, unit.y), &mut game_map.map_tiles, p2_castle, p1_castle, camp_coords);
        original_unit_movements.push(((unit.x, unit.y), move_value));
        
        succinct_units.push(current_unit);
    }

    let mut initial_population = generate_initial_population(&succinct_units, &mut game_map.map_tiles, p2_castle, p1_castle, camp_coords);
    let mut original_state = PopulationState::new(original_unit_movements, 0.0);
    assign_value_to_state(&mut original_state);
    initial_population.push(original_state);

    let mut new_generation: Vec<PopulationState> = Vec::new();
    let mut remaining_population: Vec<PopulationState> = Vec::new();

    for i in 0..GEN_NUM {
        //new_generation.append(&mut elite_selection(params, current_population));
        remaining_population = culling(&mut initial_population);

        //generate probabilities of each individual in the remaining population to be selected (will want to weight this based on score - favor better scored states) - Boltzman distribution is commonly used, but someone more familiar with statistics feel free to suggest a different distribution
        while new_generation.len() < POP_NUM.try_into().unwrap() {
            /*
            state_1 = randomly sample this distribution to get first individual
			state_2 = randomly sample again to get second individual (ensure individuals are not the same)
            */
            //let state_1 = PopulationState::new();
            //let state_2 = PopulationState::new();

            //let new_individuals = crossover(state_1, state_2);

            /*
            if size of new_generation + size of new_individuals > POP_NUM {
				add only one of the new_individuals to new_generation
			} else {
				add both of the new_individuals to new_generation
			}
            */
        }
        //In order to mutate the states we need to calculate how many to mutate and then randomly select them as mutable
        let num_to_mutate: usize = ((MUT_PROB * (new_generation.len() as f32)).round() as i32).try_into().unwrap();
        let mut states_to_mutate = new_generation.iter_mut().choose_multiple(&mut rng_thread, num_to_mutate); 
        for state in states_to_mutate.iter_mut() {
            mutate(state, &succinct_units, &mut game_map.map_tiles, p2_castle, p1_castle, camp_coords);
        }

        initial_population = new_generation.clone();
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
        total_value += DEFENSE_PENALTY;
    }
    //Will eventually want to add on values for units sieging, near camps, attacking, etc (ie prefer sieging a castle with x units over y)

    //println!("Total value: {}\nUnits near p2 castle: {}\nUnits near p1 castle: {}\nUnits near camps: {}\nUnits able to attack: {}\n", total_value, units_defending, units_sieging, units_near_camp, units_able_to_attack);

    current_state.overall_utility = total_value;
}

// Order of values in return 
// 0: value of state
// 1: near_own_castle
// 2: near_enemy_castle
// 3: near_camp
// 4: able_to_attack
// Minus "being able to attack" all other values will be calculated using heuristics (relative manhattan distance)
// Additionally not calculating closest unit to save time since based on the distance from objectives and the ability to attack this distance should be implied
fn current_unit_value (unit_attack_range: u32, unit_pos: (u32, u32), map: &mut HashMap<(u32, u32), Tile>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) -> (f64, bool, bool, bool, bool) {    
    let mut value: f64 = 0.0;

    let distance_from_own_castle = (unit_pos.0 as i32 - p2_castle.0 as i32).abs() + (unit_pos.1 as i32 - p2_castle.1 as i32).abs();
    
    let defending: bool = if distance_from_own_castle <= MIN_DISTANCE {
                        true
                    } else {
                        false
                    };

    let distance_from_enemy_castle = (unit_pos.0 as i32 - p1_castle.0 as i32).abs() + (unit_pos.1 as i32 - p1_castle.1 as i32).abs();

    let sieging: bool =   if distance_from_enemy_castle <= MIN_DISTANCE {
                        true
                    } else {
                        false
                    };

    let distance_from_nearest_camp = {
        let mut distances_from_camps: Vec<i32> = Vec::new();

        for camp in camp_coords {
            distances_from_camps.push((unit_pos.0 as i32 - camp.0 as i32).abs() + (unit_pos.1 as i32 - camp.1 as i32).abs())
        }
        *distances_from_camps.iter().min().unwrap()
    };

    let near_camp: bool = if distance_from_nearest_camp <= MIN_DISTANCE {
                        true
                    } else {
                        false
                    };

    let able_to_attack: bool =  if generalized_tiles_can_attack(map, unit_pos, unit_attack_range).is_empty() {
                                    false
                                } else {
                                    true
                                };
    //Currently commenting this out for now, I don't know if we don't want to punish units for not defending or just if there isn't enough defending
    // if defending == false {
    //     value += distance_from_own_castle as f64 * DEFENDING_WEIGHT;
    // } 
    if sieging == false {
        value += distance_from_enemy_castle as f64 * SIEGING_WEIGHT;
    }
    if near_camp == false {
        value += distance_from_nearest_camp as f64 * CAMP_WEIGHT;
    }
    if able_to_attack == true {
        value += ATTACK_VALUE;
    }

    //println!("Unit at {}, {}\nValue: {}, D(own_castle): {}, D(enemy_castle): {}, D(camp): {}, can_attack: {}\n", unit_pos.0, unit_pos.1, value, distance_from_own_castle, distance_from_enemy_castle, distance_from_nearest_camp, able_to_attack);

    (value, defending, sieging, near_camp, able_to_attack)
}