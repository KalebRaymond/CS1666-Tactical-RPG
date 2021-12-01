use std::collections::HashMap;

use crate::unit::{Team, Unit};

const TURNS_TO_CAPTURE: u32 = 3;

struct Objective {
    pub location: (u32, u32),
    pub height: u32,
    pub width: u32,
    pub turns_occupied: u32,
}

impl Objective {
    pub fn new(location: (u32, u32), height: u32, width: u32) -> Objective {
        Objective {
            location,
            height,
            width,
            turns_occupied: 0,
        }
    }

    /*
    pub fn contains_location(location: u32, u32) -> bool {

    }
    */
}

pub struct ObjectiveManager {
    pub p1_castle: Objective, 
    pub p2_castle: Objective,
    pub barbarian_camps: Vec<Objective>,
}

impl ObjectiveManager {
    /* Hacky workaround since GameMap::new() creates a GameMap struct before the locations of
     * the objectives are determined, and because rust doesn't support function overloading.
     * Creates an ObjectiveManager struct with default values, intended to be used as a placeholder
     * until ObjectiveManager::new() can be called.
     */
    pub fn init_default() -> ObjectiveManager {
        ObjectiveManager {
            p1_castle: Objective::new((0, 0), 0, 0),
            p2_castle: Objective::new((0, 0), 0, 0),
            barbarian_camps: Vec::new(),
        }
    }

    pub fn new(p1_castle_location: (u32, u32), p2_castle_location: (u32, u32), barb_camp_locations: Vec<(u32, u32)>) -> ObjectiveManager {
        let p1_castle = Objective::new(p1_castle_location, 1, 1);
        let p2_castle = Objective::new(p2_castle_location, 1, 1);
        let barbarian_camps: Vec<Objective> = barb_camp_locations.iter()
                                                .map(|location| Objective::new(location, 2, 2))
                                                .collect();

        return ObjectiveManager {
            p1_castle,
            p2_castle,
            barbarian_camps,
        };
    }

    pub fn check_objectives<'a>(&self, team: Team, team_units: HashMap<(u32, u32), Unit<'a>>) {
        //Check if enemy is occupying player castle
        if team == Team::Enemy {
            match team_units.get_mut(&self.p1_castle.location) {
                Some(_unit) => {
                    self.p1_castle.turns_occupied += 1;
                },
                _ => {
                    self.p1_castle.turns_occupied = 0;
                },
            }
        }

        //Check if player is occupying enemy castle
        if team == Team::Player {
            match team_units.get_mut(&self.p2_castle.location) {
                Some(_unit) => {
                    self.p2_castle.turns_occupied += 1;
                },
                _ => {
                    self.p2_castle.turns_occupied = 0;
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
        match team {
            Team::Player => return self.p2_castle.turns_occupied >= TURNS_TO_CAPTURE,
            Team::Enemy => return self.p1_castle.turns_occupied >= TURNS_TO_CAPTURE,
            _ => return false,
        }
    }
}