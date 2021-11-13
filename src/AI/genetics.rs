use std::collections::HashMap;
use std::convert::TryInto;

use crate::AI::genetic_params::GeneticParams;
use crate::AI::population_state::*;
use crate::unit::Unit;
use crate::tile::Tile;

//Utility Function Constants
const MIN_DISTANCE: i32 = 5; // Defines the minimum distance a unit can be from an objective to be considered near it
const DEFENDING_WEIGHT: f64 = -0.5;
const SIEGING_WEIGHT: f64 = -0.75;
const CAMP_WEIGHT: f64 = -0.25;
const ATTACK_VALUE: f64 = 10.0;
const MIN_DEFENSE: u32 = 5; //Since one of our AI goals says that some units should stay behind and defend, we need metrics to enforce this
const DEFENSE_PENALTY: f64 = -500.0;

pub fn generate_initial_population(params: &GeneticParams, succinct_units: &Vec<SuccinctUnit>) -> Vec<PopulationState> {
    let mut population: Vec<PopulationState> = Vec::new();
    
    //Generate 1 less state so we can add the initial population
    for i in 1..params.pop_num {
        //let mut state = PopulationState::new();

		for unit in succinct_units {
			//state.push(random selection of one of the units possible moves)
		}

		//population.push(state);
    }

    population
}

pub fn mutate(state: &mut PopulationState) {
	//This function depends on how we implement PopulationState
    /*
    for unit in random_sample of size MUT_NUM of state {
		while new_position = current_position {
			new_position = random selection of one of the unit's possible moves
		}
		state.update(unit, new_position)
	}
    */
}

// pub fn crossover(state_1: PopulationState, state_2: PopulationState) -> (PopulationState, PopulationState) {
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

pub fn elite_selection(params: &GeneticParams, current_population: &mut Vec<PopulationState>) -> Vec<PopulationState> {
	let num_to_keep: usize = ((params.e_perc * (current_population.len() as f32)).round() as i32).try_into().unwrap();
	
    //Assuming current_population is in descending order 
	return current_population[0..num_to_keep].to_vec();
}

pub fn culling(params: &GeneticParams, current_population: &mut Vec<PopulationState>) -> Vec<PopulationState> {
	let num_to_drop: usize = ((params.c_perc * (current_population.len() as f32)).round() as i32).try_into().unwrap();
	
    //Assuming current_population is in descending order 
	return current_population[0..(current_population.len() - num_to_drop)].to_vec();
}

pub fn genetic_algorithm(params: &GeneticParams, units: &HashMap<(u32, u32), Unit>, map: &mut HashMap<(u32, u32), Tile>, current_population: &mut Vec<PopulationState>) -> Vec<PopulationState>{
    //Keep track of all the possible unit movements
    let mut succinct_units: Vec<SuccinctUnit> = Vec::new();
    //let mut original_population: PopulationState = 
    for unit in units.values() {
        succinct_units.push(SuccinctUnit::new(unit.get_tiles_in_movement_range(map), unit.attack_range));
    }

    let mut initial_population = generate_initial_population(params, &succinct_units);
    let mut new_generation: Vec<PopulationState> = Vec::new();
    let mut remaining_population: Vec<PopulationState> = Vec::new();

    for i in 0..params.gen_num {
        new_generation.append(&mut elite_selection(params, current_population));
        remaining_population = culling(params, &mut initial_population);

        //generate probabilities of each individual in the remaining population to be selected (will want to weight this based on score - favor better scored states) - Boltzman distribution is commonly used, but someone more familiar with statistics feel free to suggest a different distribution
        while new_generation.len() < params.pop_num.try_into().unwrap() {
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

        initial_population = new_generation.clone();
    }

    //init_population now generally represents the best possible states that have been found and we can use these to form the considered moves of our minimax and we can repeat this for the enemy to get their "best" move and make the decision from there   
    initial_population
}

//Evaluation/Utility function related
pub fn assign_value_to_state (current_state: &mut PopulationState, current_state_values: Vec<(f64, u32, u32, u32, u32)>) {
    let mut total_value: f64 = 0.0;
    let mut units_defending: u32 = 0; //Units near own castle 
    let mut units_sieging: u32 = 0; //Units near enemy castle
    let mut units_near_camp: u32 = 0;
    let mut units_able_to_attack: u32 = 0;

    //println!("Utility Function Constants:\nMinimum Distance from Objectives: {}, Defending Weight: {}, Sieging Weight: {}, Camp Weight: {}, Value from Attack: {}, Minimum Defending Units: {}, Defense Penalty: {}\n", MIN_DISTANCE, DEFENDING_WEIGHT, SIEGING_WEIGHT, CAMP_WEIGHT, ATTACK_VALUE, MIN_DEFENSE, DEFENSE_PENALTY);

    for value in current_state_values {
        total_value += value.0;
        units_defending += value.1;
        units_sieging += value.2;
        units_near_camp += value.3;
        units_able_to_attack += value.4;
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
pub fn current_unit_value (unit: SuccinctUnit, unit_pos: (u32, u32), map: &mut HashMap<(u32, u32), Tile>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) -> (f64, u32, u32, u32, u32) {    
    let mut value: f64 = 0.0;

    let distance_from_own_castle = (unit_pos.0 as i32 - p2_castle.0 as i32).abs() + (unit_pos.1 as i32 - p2_castle.1 as i32).abs();
    
    let defending: u32 = if distance_from_own_castle <= MIN_DISTANCE {
                        1
                    } else {
                        0
                    };

    let distance_from_enemy_castle = (unit_pos.0 as i32 - p1_castle.0 as i32).abs() + (unit_pos.1 as i32 - p1_castle.1 as i32).abs();

    let sieging: u32 =   if distance_from_enemy_castle <= MIN_DISTANCE {
                        1
                    } else {
                        0
                    };

    let distance_from_nearest_camp = {
        let mut distances_from_camps: Vec<i32> = Vec::new();

        for camp in camp_coords {
            distances_from_camps.push((unit_pos.0 as i32 - camp.0 as i32).abs() + (unit_pos.1 as i32 - camp.1 as i32).abs())
        }
        *distances_from_camps.iter().min().unwrap()
    };

    let near_camp: u32 = if distance_from_nearest_camp <= MIN_DISTANCE {
                        1
                    } else {
                        0
                    };

    let able_to_attack: u32 =   if generalized_tiles_can_attack(map, unit_pos, unit.attack_range).is_empty() {
                                    0
                                } else {
                                    1
                                };
    if defending == 0 {
        value += distance_from_own_castle as f64 * DEFENDING_WEIGHT;
    } 
    if sieging == 0 {
        value += distance_from_enemy_castle as f64 * SIEGING_WEIGHT;
    }
    if near_camp == 0 {
        value += distance_from_nearest_camp as f64 * CAMP_WEIGHT;
    }
    if able_to_attack == 1 {
        value += ATTACK_VALUE;
    }

    println!("Unit at {}, {}\nValue: {}, D(own_castle): {}, D(enemy_castle): {}, D(camp): {}, can_attack: {}\n", unit_pos.0, unit_pos.1, value, distance_from_own_castle, distance_from_enemy_castle, distance_from_nearest_camp, able_to_attack);

    (value, defending, sieging, near_camp, able_to_attack)
}