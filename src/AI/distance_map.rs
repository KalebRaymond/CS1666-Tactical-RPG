use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DistanceMap {
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
        let mut file_io = BufReader::new(file);
    
        let mut s: String = "".to_string();
        for line in file_io.lines() {
            s += &line.unwrap();
        }

        DistanceMap {
            to_player_castle,
            to_enemy_castle,
            to_barbarian_camps,
        }
    }
}