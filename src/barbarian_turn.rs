use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::unit::{Team, Unit};
use crate::turn_banner::TurnBanner;

pub fn handle_barbarian_turn<'a>(barb_units: &mut HashMap<(u32, u32), Unit<'a>>, game_map: &mut GameMap, turn_banner: &mut TurnBanner, current_player: &mut Team) {
    if !turn_banner.banner_visible {
        for barbarian in barb_units.values_mut() {
            let (original_x, original_y) = (barbarian.x, barbarian.y);
            let mut barb_moved = false;
            let possible_moves: Vec<(u32, u32)> = barbarian.get_tiles_in_movement_range(&mut game_map.map_tiles);
            for possible_move in possible_moves {
                barbarian.x = possible_move.0; 
                barbarian.y = possible_move.1;
                let actual_attacks: Vec<(u32, u32)> = barbarian.get_tiles_can_attack(&mut game_map.map_tiles);
                if !actual_attacks.is_empty() {
                    println!("Should move to this tile.");
                    // //INSTEAD COMPILE A LIST OF ALL BARBARIANS THAT MOVED AND THE TILE THAT THEY MOVE TO
                    // barb_units.remove(&(original_x, original_y)).unwrap();
                    // barb_units.insert((barbarian.x, barbarian.y), *barbarian);
                    // // Update map tiles
                    // if let Some(old_map_tile) = game_map.map_tiles.get_mut(&(original_x, original_y)) {
                    //     old_map_tile.update_team(None);
                    // }
                    // if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(barbarian.x, barbarian.y)) {
                    //     new_map_tile.update_team(Some(Team::Barbarians));
                    // }
                    // barb_moved = true;
                    // break;
                }
            }
            //If a barbarian did not move, make sure to reset it back to its normal position
            if !barb_moved {
                barbarian.x = original_x; 
                barbarian.y = original_y;
            }   
        }
        
        //End turn
        *current_player = Team::Player;

        //Start displaying Player 1's banner
        turn_banner.current_banner_transparency = 250;
        turn_banner.banner_colors = Color::RGBA(0, 89, 178, turn_banner.current_banner_transparency);
        turn_banner.banner_key = "p1_banner";
        turn_banner.banner_visible = true;
    }
}