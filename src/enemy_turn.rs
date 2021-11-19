use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::AI::*;
use crate::game_map::GameMap;
use crate::unit::{Team, Unit};
use crate::banner::Banner;
use crate::SDLCore;

//Utility Function Constants
const MIN_DISTANCE: i32 = 5; // Defines the minimum distance a unit can be from an objective to be considered near it
const DEFENDING_WEIGHT: f64 = -0.5;
const SIEGING_WEIGHT: f64 = -0.75;
const CAMP_WEIGHT: f64 = -0.25;
const ATTACK_VALUE: f64 = 10.0;
const MIN_DEFENSE: u32 = 5; //Since one of our AI goals says that some units should stay behind and defend, we need metrics to enforce this
const DEFENSE_PENALTY: f64 = -500.0;

pub fn handle_enemy_turn<'a, 'b>(core: &SDLCore<'b>, p2_units: &mut HashMap<(u32, u32), Unit<'a>>, p1_units: &mut HashMap<(u32, u32), Unit<'a>>, barbarian_units: &mut HashMap<(u32, u32), Unit<'a>>, game_map: &mut GameMap<'b>, turn_banner: &mut Banner, current_player: &mut Team, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) -> Result<(), String> {
    if !turn_banner.banner_visible {
        let best_moves = genetics::genetic_algorithm(p2_units, game_map, p2_castle, p1_castle, camp_coords);
        
        //Currently just base movements off the best individual, will convert to minimax later...
        let best_individual = best_moves.iter().max().unwrap();
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