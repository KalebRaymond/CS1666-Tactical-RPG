use crate::AI::*;
use crate::AI::distance_map::*;
use crate::game_map::GameMap;
use crate::SDLCore;
use crate::net::util::*;

pub fn handle_enemy_turn<'a>(core: &SDLCore<'a>, game_map: &mut GameMap<'a>, distance_map: &DistanceMap) -> Result<(), String> {
    if !game_map.banner.banner_visible {
        let best_moves = genetics::genetic_algorithm(game_map, distance_map);

        //Currently just base movements off the best individual, will convert to minimax later...
        let best_individual = best_moves.iter().max().unwrap();
        best_individual.convert_state_to_action(core, core.texture_map, game_map)?;

        //End turn
        game_map.event_list.push(Event::create(EVENT_END_TURN, EVENT_ID_ENEMY, (0,0), (0,0), 0));
    }
    Ok(())
}