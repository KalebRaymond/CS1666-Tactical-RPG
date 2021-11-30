use sdl2::pixels::Color;
use sdl2::render::Texture;
use std::collections::HashMap;

use crate::AI::*;
use crate::AI::distance_map::*;
use crate::game_map::GameMap;
use crate::unit::{Team, Unit};
use crate::banner::Banner;
use crate::SDLCore;

pub fn handle_enemy_turn<'a>(core: &SDLCore<'a>, game_map: &mut GameMap<'a>, current_player: &mut Team, distance_map: &DistanceMap, unit_textures: &'a HashMap<&str, Texture<'a>>,) -> Result<(), String> {
    if !game_map.banner.banner_visible {
        let best_moves = genetics::genetic_algorithm(game_map, distance_map);

        //Currently just base movements off the best individual, will convert to minimax later...
        let best_individual = best_moves.iter().max().unwrap();
        best_individual.convert_state_to_action(core, unit_textures, game_map)?;

        //End turn
        *current_player = Team::Barbarians;

        //Start displaying the barbarians' banner
        game_map.banner.show("b_banner");
    }
    Ok(())
}