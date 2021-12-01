use std::collections::HashMap;

use crate::unit::{Team, Unit};

const TURNS_TO_CAPTURE: u32 = 3;

pub struct ObjectiveManager {
    pub p1_castle: (u32, u32),
    pub p1_castle_turns: u32,

    pub p2_castle: (u32, u32),
    pub p2_castle_turns: u32,

    pub barbarian_camps: Vec<(u32, u32)>,
    pub barbarian_camps_turns: Vec<u32>,
}

impl ObjectiveManager {
    /* Hacky workaround since GameMap::new() creates a GameMap struct before the locations of
     * the objectives are determined, and because rust doesn't support function overloading.
     * Creates an ObjectiveManager struct with default values, intended to be used as a placeholder
     * until ObjectiveManager::new() can be called.
     */
    pub fn init_default() -> ObjectiveManager {
        ObjectiveManager {
            p1_castle: (0, 0),
            p1_castle_turns: 0,
            p2_castle: (0, 0),
            p2_castle_turns: 0,
            barbarian_camps: Vec::new(),
            barbarian_camps_turns: Vec::new(),
        }
    }

    pub fn new(p1_castle_location: (u32, u32), p2_castle_location: (u32, u32), barb_camp_locations: Vec<(u32, u32)>) -> ObjectiveManager {
        let mut barbarian_camps_turns: Vec<u32> = Vec::new();
        for _camp in barb_camp_locations.iter() {
            barbarian_camps_turns.push(0);
        }

        return ObjectiveManager {
            p1_castle: p1_castle_location,
            p1_castle_turns: 0,
            p2_castle: p2_castle_location,
            p2_castle_turns: 0,
            barbarian_camps: barb_camp_locations,
            barbarian_camps_turns,
        };
    }

    pub fn check_objectives<'a>(&mut self, team: Team, team_units: &HashMap<(u32, u32), Unit<'a>>) {
        //Check if enemy is occupying player castle
        if team == Team::Enemy {
            match team_units.get(&self.p1_castle) {
                Some(_unit) => {
                    self.p1_castle_turns += 1;
                },
                _ => {
                    self.p1_castle_turns = 0;
                },
            }
        }

        //Check if player is occupying enemy castle
        if team == Team::Player {
            match team_units.get(&self.p2_castle) {
                Some(_unit) => {
                    self.p2_castle_turns += 1;
                },
                _ => {
                    self.p2_castle_turns = 0;
                },
            }
        }

        //Check barbarian camps
        /*
        if team != barbarian
            for each camp in camps
                if team has a unit in this camp
        */
    }

    //Returns true if the given team has captured their opponent's castle
    pub fn has_won(&self, team: Team) -> bool {
        if team == Team::Player {
            return self.p2_castle_turns >= TURNS_TO_CAPTURE;
        }
        else if team == Team::Enemy {
            return self.p1_castle_turns >= TURNS_TO_CAPTURE;
        }
        else {
            return false;
        }
    }
}