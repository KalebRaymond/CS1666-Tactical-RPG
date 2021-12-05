use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DistanceMap {
    /* All map coordinates are in (x, y) order */
    pub to_player_castle: HashMap<(u32, u32), u32>,
    pub to_enemy_castle: HashMap<(u32, u32), u32>,
    pub to_barbarian_camps: HashMap<(u32, u32), HashMap<(u32, u32), u32>>,
}

impl DistanceMap {
    pub fn new() -> DistanceMap {
        return DistanceMap::read_from_file("./src/AI/distances.txt".to_string());
    }

    fn read_from_file(path: String) -> DistanceMap {
        let mut to_player_castle: HashMap<(u32, u32), u32> = HashMap::new();
        let mut to_enemy_castle: HashMap<(u32, u32), u32> = HashMap::new();
        let mut to_barbarian_camps: HashMap<(u32, u32), HashMap<(u32, u32), u32>> = HashMap::new();

        let file = File::open(path).expect("Could not open file");
        let file_io = BufReader::new(file);

        //Parse distances.txt and populate the hashmaps with the values from the file
        let mut cur_map = &mut to_player_castle;
        for line in file_io.lines() {
            let line = line.unwrap();
            //Put contents of current line into a vector
            let values: Vec<&str> = line.split(" ").collect();

            if values.len() == 1 {
                match values[0] {
                    "p1_castle" => cur_map = &mut to_player_castle,
                    "enemy_castle" => cur_map = &mut to_enemy_castle,
                    _ => {}, //Do nothing. We only need to reference the nested hashmaps in to_barbarian_camps, which is handled in the len() == 3 block below
                 };
            }
            else if values.len() == 3 {
                if values[0] == "#" {
                    //Add a new nested hashmap to the hashmap of barbarian camps
                    let coords = (values[1].parse::<u32>().unwrap(), values[2].parse::<u32>().unwrap());
                    to_barbarian_camps.insert(coords, HashMap::new());
                    cur_map = to_barbarian_camps.get_mut(&coords).unwrap();
                }
                else {
                    //Convert the string values into u32s
                    let num_vals: Vec<u32> = values.iter().map(|s| s.parse::<u32>().unwrap()).collect();
                    //num_vals[0] = x coordinate
                    //num_vals[1] = y coordinate
                    //num_vals[2] = distance from (x, y) to whichever goal the current map is for
                    cur_map.insert((num_vals[0], num_vals[1]), num_vals[2]);
                }
            }
        }

        DistanceMap {
            to_player_castle,
            to_enemy_castle,
            to_barbarian_camps,
        }
    }
}