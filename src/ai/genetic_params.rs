pub struct GeneticParams {
    pub pop_num: u32, //Population size
    pub gen_num: u32, //Number of generations to run
    pub mut_prob: f32, //Probability of an individual being mutated
    pub mut_num: u32, //How many units should be changed on mutation
    pub c_perc: f32, //Percentage of the least fit individuals to be removed
    pub e_perc: f32, //Proportion of best individuals to carry over from one generation to the next
}