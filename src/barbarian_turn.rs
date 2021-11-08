use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::damage_indicator::DamageIndicator;
use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::SDLCore;
use crate::TILE_SIZE;
use crate::turn_banner::TurnBanner;
use crate::unit::{Team, Unit};

pub fn handle_barbarian_turn<'i, 'r>(core: &SDLCore<'r>, barb_units: &mut HashMap<(u32, u32), Unit<'i>>, p1_units: &mut HashMap<(u32, u32), Unit<'i>>, p2_units: &mut HashMap<(u32, u32), Unit<'i>>, game_map: &mut GameMap<'r>, turn_banner: &mut TurnBanner, current_player: &mut Team) -> Result<(), String> {
    if !turn_banner.banner_visible {
        //First set of coords is the new coordinates and second set are the old ones
        let mut moving_barbs: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
        for barbarian in barb_units.values_mut() {
            let (original_x, original_y) = (barbarian.x, barbarian.y);
            let possible_moves: Vec<(u32, u32)> = barbarian.get_tiles_in_movement_range(&mut game_map.map_tiles);
            for possible_move in possible_moves {
                barbarian.x = possible_move.0; 
                barbarian.y = possible_move.1;
                let actual_attacks: Vec<(u32, u32)> = barbarian.get_tiles_can_attack(&mut game_map.map_tiles);
                if !actual_attacks.is_empty() {
                    // Need to check and make sure that a barbarian has not already moved to this tile
                    if let Some(coordinates) = moving_barbs.get(&(barbarian.x, barbarian.y)) {
                        continue;
                    }
                    //Need to update map outside of this loop as this will allow for easier updating movement later on if we want
                    moving_barbs.insert((barbarian.x, barbarian.y), (original_x, original_y));

                    //All attack handling is done here
                    let damage_done = barbarian.get_attack_damage();
                    if let Some(tile_under_attack) = game_map.map_tiles.get_mut(&(actual_attacks[0].1, actual_attacks[0].0)) {
                        match tile_under_attack.contained_unit_team {
                            Some(Team::Player) => {
                                if let Some(unit) = p1_units.get_mut(&(actual_attacks[0].0, actual_attacks[0].1)) {
                                    println!("Unit starting at {} hp.", unit.hp);
                                    if unit.hp <= damage_done {
                                        p1_units.remove(&(actual_attacks[0].0, actual_attacks[0].1));
                                        println!("Player unit at {}, {} is dead after taking {} damage.", actual_attacks[0].0, actual_attacks[0].1, damage_done);
                                        tile_under_attack.update_team(None);
                                    } else {
                                        unit.receive_damage(damage_done);
                                        game_map.damage_indicators.push(DamageIndicator::new(core, damage_done, PixelCoordinates::from_matrix_indices(unit.y - 1, unit.x))?);
                                        println!("Barbarian at {}, {} attacking player unit at {}, {} for {} damage. Unit now has {} hp.", barbarian.x, barbarian.y, actual_attacks[0].0, actual_attacks[0].1, damage_done, unit.hp);
                                    }
                                }
                            },
                            _ => {
                                if let Some(unit) = p2_units.get_mut(&(actual_attacks[0].0, actual_attacks[0].1)) {
                                    println!("Enemy unit starting at {} hp.", unit.hp);
                                    if unit.hp <= damage_done {
                                        p2_units.remove(&(actual_attacks[0].0, actual_attacks[0].1));
                                        println!("Enemy unit at {}, {} is dead after taking {} damage.", actual_attacks[0].0, actual_attacks[0].1, damage_done);
                                        tile_under_attack.update_team(None);
                                    } else {
                                        unit.receive_damage(damage_done);
                                        game_map.damage_indicators.push(DamageIndicator::new(core, damage_done, PixelCoordinates::from_matrix_indices(unit.y - 1, unit.x))?);
                                        println!("Barbarian at {}, {} attacking enemy unit at {}, {} for {} damage. Unit now has {} hp.", barbarian.x, barbarian.y, actual_attacks[0].0, actual_attacks[0].1, damage_done, unit.hp);
                                    }
                                }
                            } //This handles the enemy case and also prevents rust from complaining about unchecked cases,
                        }
                    }
                    
                    // If we want to implement random movement, we can add a boolean here and then do some probability calculations
                    break;
                }
            }
            // Make sure to reset it back to its normal position as we cannot update the hashmap after already borrowing it
            barbarian.x = original_x; 
            barbarian.y = original_y; 
        }
        
        for (newcoord, ogcoord) in moving_barbs.into_iter() {
            let mut active_unit = barb_units.remove(&(ogcoord.0, ogcoord.1)).unwrap();
            active_unit.update_pos(newcoord.0, newcoord.1);
            barb_units.insert((newcoord.0, newcoord.1), active_unit);
            // Update map tiles
            // Have to remember that map indexing is swapped
            if let Some(old_map_tile) = game_map.map_tiles.get_mut(&(ogcoord.1, ogcoord.0)) {
                old_map_tile.update_team(None);
            }
            if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(newcoord.1, newcoord.0)) {
                new_map_tile.update_team(Some(Team::Barbarians));
            }
        }

        //End turn
        *current_player = Team::Player;

        //Start displaying Player 1's banner
        turn_banner.current_banner_transparency = 250;
        turn_banner.banner_colors = Color::RGBA(0, 89, 178, turn_banner.current_banner_transparency);
        turn_banner.banner_key = "p1_banner";
        turn_banner.banner_visible = true;

        //Reactivate any grayed out barbarian units
        crate::single_player::initialize_next_turn(barb_units);
    }

    Ok(())
}
