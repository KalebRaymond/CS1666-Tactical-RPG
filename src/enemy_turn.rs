use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::AI::distance_map::*;
use crate::AI::genetics;
use crate::AI::population_state::*;
use crate::banner::Banner;
use crate::game_map::GameMap;
use crate::SDLCore;
use crate::tile::Tile;
use crate::unit::{Team, Unit};

pub fn handle_enemy_turn<'a, 'b>(core: &SDLCore<'b>, p2_units: &mut HashMap<(u32, u32), Unit<'a>>, p1_units: &mut HashMap<(u32, u32), Unit<'a>>, barbarian_units: &mut HashMap<(u32, u32), Unit<'a>>, game_map: &mut GameMap<'b>, turn_banner: &mut Banner, current_player: &mut Team, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>, distance_map: &DistanceMap) -> Result<(), String> {
    if !turn_banner.banner_visible {
        let best_moves = genetics::genetic_algorithm(p2_units, &mut game_map.map_tiles, p2_castle, p1_castle, camp_coords, distance_map);
        
        //Currently just base movements off the best individual, will convert to minimax later...
        let best_individual = best_moves.iter().max().unwrap();

        /*
        let best_individual = minimax(/*cur_state*/, 3, true);
        */
        best_individual.convert_state_to_action(core, p2_units, p1_units, barbarian_units, game_map);

        //End turn
        *current_player = Team::Barbarians;

        //Start displaying the barbarians' banner
        turn_banner.current_banner_transparency = 250;
        turn_banner.banner_colors = Color::RGBA(163,96,30, turn_banner.current_banner_transparency);
        turn_banner.banner_key = "b_banner";
        turn_banner.banner_visible = true;
    }
    Ok(())
}

fn minimax<'a>(cur_state: PopulationState, depth: u32, maximizing_enemy: bool, cur_team_units: &mut HashMap<(u32, u32), Unit>, map_tiles: &mut HashMap<(u32, u32), Tile<'a>>, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>, distance_map: &DistanceMap) -> PopulationState {
    if depth == 0 /* or cur_state has a game over condition */ {
        return cur_state;
    }

    if maximizing_enemy {
        let mut max_utility = f64::NEG_INFINITY;
        let mut best_state = cur_state;

        //Get possible PopulationStates from the current game state
        let potential_states = genetics::genetic_algorithm(cur_team_units, map_tiles, p2_castle, p1_castle, camp_coords, distance_map);

        for potential_state in potential_states.into_iter() {
            //Get the list of units that have moved in the current potential state
            let moved_units = ();

            //Find the best next possible pontential state based on the current potential state
            let best_state_from_current = minimax(potential_state.clone(), depth - 1, false, cur_team_units, map_tiles, p2_castle, p1_castle, camp_coords, distance_map);

            if best_state_from_current.overall_utility > max_utility {
                max_utility = best_state_from_current.overall_utility;
                best_state = potential_state.clone();
            }
        }

        //Return the potential PopulationState with the most utility
        return best_state;
    }
    else {
        let mut min_utility = f64::INFINITY;
        let mut best_state = cur_state;

        //Get possible PopulationStates from the current PopulationState
        let potential_states = genetics::genetic_algorithm(cur_team_units, map_tiles, p2_castle, p1_castle, camp_coords, distance_map);

        for potential_state in potential_states.into_iter() {
            //Get the list of units that have moved in the current potential state

            //Find the best next possible pontential state based on the current potential state
            let best_state_from_current = minimax(potential_state.clone(), depth - 1, true, cur_team_units, map_tiles, p2_castle, p1_castle, camp_coords, distance_map);
            
            if best_state_from_current.overall_utility < min_utility {
                min_utility = best_state_from_current.overall_utility;
                best_state = potential_state.clone();
            }
        }

        //Return the potential PopulationState with the least utility
        return best_state;
    }
}

fn create_potential_game_map_tiles<'a>(map_tiles: &mut HashMap<(u32, u32), Tile<'a>>, potential_state: PopulationState) -> HashMap<(u32, u32), Tile<'a>> {
    let potential_map_tiles: HashMap<(u32, u32), Tile<'a>>  = map_tiles.clone();

    return potential_map_tiles;
}