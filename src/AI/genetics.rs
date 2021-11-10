/*
Will likely need a struct to keep track of individuals in a population (all units current position and the value of that state)
Will also need to determine the best way to keep track of each units possible moves as this will be important for mutations
 
generate_initial_population (){
	population = []
	for i in 0..POP_NUM {
		state = []
		for each unit in units {
			state.push(random selection of one of the units possible moves)
		}
		population.push(state)
	}
}
 
mutate (state){
	for unit in random_sample of size MUT_NUM of state {
		while new_position = current_position {
			new_position = random selection of one of the units possible moves
		}
		state.update(unit, new_position)
	}
}
 
crossover (state1, state2) {
	randomly select bounds for state1
	new_state.push(the units within these bounds from state1)
	new_state.push(the remaining units from state2)
	new_state2.push(the units outside the bounds of state1)
	new_state2.push(the remaining units from state 2)
	return new_state, new_state2
}
 
elite_selection (current_population) {
	num_to_keep = E_PERC * size(current_population)
	assuming current_population is in descending order 
	return current_population[0..num_to_keep]
}
 
culling (current_population) {
	num_to_drop = C_PERC * size(current_population)
	assuming current_population is in descending order 
	return current_population[0..size(current_population)-num_to_drop]
}
 
genetic algorithm() {
	init_population = generate_initial_population()
	for i in 0..GEN_NUM {
		new_generation.append(elite_selection(current_population))
		remaining_population = culling(init_population)
		generate probabilities of each individual in the remaining population to be selected (will want to weight this based on score - favor better scored states) - Boltzman distribution is commonly used, but someone more familiar with statistics feel free to suggest a different distribution
		while size of new_generation < POP_NUM {
			s_1 = randomly sample this distribution to get first individual
			s_2 = randomly sample again to get second individual (ensure individuals are not the same)
			new_individuals = crossover(s_1, s_2)
			if size of new_generation + size of new_individuals > POP_NUM {
				add only one of the new_individuals to new_generation
			} else {
				add both of the new_individuals to new_generation
			}
		}
		randomly sample according to MUT_PERC
		selected_individual = mutate(selected_individual)
 
		init_population = new_generation
	}
	init_population now generally represents the best possible states that have been found and we can use these to form the considered moves of our minimax and we can repeat this for the enemy to get their "best" move and make the decision from there
}
*/