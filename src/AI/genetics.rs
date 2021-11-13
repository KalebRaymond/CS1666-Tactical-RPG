use std::collections::HashMap;
use std::convert::TryInto;

use crate::AI::genetic_params::GeneticParams;
use crate::AI::population_state::*;
use crate::unit::Unit;
use crate::tile::Tile;

//Will likely need a struct to keep track of individuals in a population (all units current position and the value of that state)
//Will also need to determine the best way to keep track of each units possible moves as this will be important for mutations

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