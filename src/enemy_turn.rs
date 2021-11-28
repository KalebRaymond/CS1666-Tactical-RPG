use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::AI::*;
use crate::AI::distance_map::*;
use crate::game_map::GameMap;
use crate::unit::{Team, Unit};
use crate::banner::Banner;
use crate::SDLCore;

pub fn handle_enemy_turn<'a, 'b>(core: &SDLCore<'b>, p2_units: &mut HashMap<(u32, u32), Unit<'a>>, p1_units: &mut HashMap<(u32, u32), Unit<'a>>, barbarian_units: &mut HashMap<(u32, u32), Unit<'a>>, game_map: &mut GameMap<'b>, turn_banner: &mut Banner, current_player: &mut Team, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>, distance_map: &DistanceMap) -> Result<(), String> {
    if !turn_banner.banner_visible {
        let best_moves = genetics::genetic_algorithm(p2_units, game_map, p2_castle, p1_castle, camp_coords, distance_map);
        
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