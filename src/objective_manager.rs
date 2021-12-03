use std::collections::HashMap;

use crate::unit::{Team, Unit};

const TURNS_TO_CAPTURE: u32 = 3;

pub struct ObjectiveManager {
    pub p1_castle: (u32, u32),
    pub p1_castle_turns: u32,

    pub p2_castle: (u32, u32),
    pub p2_castle_turns: u32,

    pub barbarian_camps: Vec<(u32, u32)>,
    pub barbarian_camps_turns: HashMap<(u32, u32), u32>, //Keeps track of how many consecutive turns each camp has been occupied
    pub barbarian_camps_teams: HashMap<(u32, u32), Option<Team>>, //Keeps track of which team is occupying each camp
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
            barbarian_camps_turns: HashMap::new(),
            barbarian_camps_teams: HashMap::new(),
        }
    }

    pub fn new(p1_castle_location: (u32, u32), p2_castle_location: (u32, u32), barb_camp_locations: Vec<(u32, u32)>) -> ObjectiveManager {
        let mut barbarian_camps_turns: HashMap<(u32, u32), u32> = HashMap::new();
        let mut barbarian_camps_teams: HashMap<(u32, u32), Option<Team>> = HashMap::new();

        for camp in barb_camp_locations.iter() {
            barbarian_camps_turns.insert(*camp, 0);
            barbarian_camps_teams.insert(*camp, None);
        }

        return ObjectiveManager {
            p1_castle: p1_castle_location,
            p1_castle_turns: 0,
            p2_castle: p2_castle_location,
            p2_castle_turns: 0,
            barbarian_camps: barb_camp_locations,
            barbarian_camps_turns,
            barbarian_camps_teams,
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
        let mut camps_to_remove: Vec<(u32, u32)> = Vec::new();
        if team != Team::Barbarians {
            for camp in self.barbarian_camps.iter() {
                if ObjectiveManager::camp_is_occupied_by_team(camp, &team_units) {
                    //Increment camp's turn count
                    self.barbarian_camps_turns.insert(*camp, *self.barbarian_camps_turns.get(camp).unwrap() + 1);
                    
                    //If this is the first turn the team is occupying the camp, set the camp's team
                    if self.barbarian_camps_teams.get(camp).is_none() {
                        self.barbarian_camps_teams.insert(*camp, Some(team));
                    }
                }
                else if self.barbarian_camps_teams.get(camp).is_none() {
                    //If the camp is not occupied by a team, set the turns to 0
                    self.barbarian_camps_turns.insert(*camp, 0);
                }

                //Keep track of captured camps
                if *self.barbarian_camps_turns.get(camp).unwrap() >= TURNS_TO_CAPTURE {
                    camps_to_remove.push((camp.0, camp.1));
                }

                println!("Turns on camp ({}, {}): {}", camp.0, camp.1, *self.barbarian_camps_turns.get(camp).unwrap());
            }

            //Remove captured camps
            for camp_to_remove in camps_to_remove.iter() {
                self.barbarian_camps_turns.remove(camp_to_remove);
                self.barbarian_camps_teams.remove(camp_to_remove);
                self.barbarian_camps.retain(|camp| camp.0 != camp_to_remove.0 || camp.1 != camp_to_remove.1);

                println!("Camp ({}, {}) removed from ObjectiveManager", camp_to_remove.0, camp_to_remove.1);
            }
        }
    }

    fn camp_is_occupied_by_team<'a>(camp_coord: &(u32, u32), team_units: &HashMap<(u32, u32), Unit<'a>>) -> bool {
        //Since camps are 2x2, have to check each tile in the camp for a unit
        //camp_coord is the coordinates of the top left tile in the camp
        return  team_units.contains_key(&camp_coord) ||
                team_units.contains_key(&(camp_coord.0 + 1, camp_coord.1)) ||
                team_units.contains_key(&(camp_coord.0, camp_coord.1 + 1)) ||
                team_units.contains_key(&(camp_coord.0 + 1, camp_coord.1 + 1));
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