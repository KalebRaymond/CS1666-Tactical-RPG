use rand::Rng;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Texture;

use std::convert::TryInto;
use std::collections::HashMap;

use crate::button::Button;
use crate::cursor::Cursor;
use crate::damage_indicator::DamageIndicator;
use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::SDLCore;
use crate::banner::Banner;
use crate::unit_interface::UnitInterface;
use crate::unit::{Team, Unit};

pub fn handle_player_turn<'a>(core: &SDLCore<'a>, game_map: &mut GameMap<'a>) -> Result<(), String> {
    let player_state = &mut game_map.player_state;

    if !game_map.banner.banner_visible {
        //Check if player ended turn by pressing backspace
        if core.input.keystate.contains(&Keycode::Backspace) && match player_state.current_player_action {
            PlayerAction::ChoosingNewUnit => false,
            _ => true,
        }{
            end_player_turn(game_map);
            return Ok(());
        }

        //Check if user clicked the end turn button
		if core.input.left_clicked && game_map.end_turn_button.is_mouse(core) && match player_state.current_player_action {
            PlayerAction::ChoosingNewUnit => false,
            _ => true,
        }{
			end_player_turn(game_map);
            return Ok(());
		}

        //Get map matrix indices from mouse position
        let (i, j) = PixelCoordinates::matrix_indices_from_pixel(
            core.input.mouse_state.x().try_into().unwrap(),
            core.input.mouse_state.y().try_into().unwrap(),
            (-1 * core.cam.x).try_into().unwrap(),
            (-1 * core.cam.y).try_into().unwrap()
        );
        let (glob_x, glob_y) = PixelCoordinates::global_coordinates(
            core.input.mouse_state.x().try_into().unwrap(),
            core.input.mouse_state.y().try_into().unwrap(),
            (-1 * core.cam.x).try_into().unwrap(),
            (-1 * core.cam.y).try_into().unwrap()
        );

        match player_state.current_player_action {
            PlayerAction::Default => {
                //If player hovers over a unit, display cursor above that unit
                match game_map.player_units.get_mut(&(j,i)) {
                    Some(active_unit) => {
                        //Now check if the player actually clicked on the unit they hovered over
                        if core.input.left_clicked {
                            player_state.active_unit_i = i as i32;
                            player_state.active_unit_j = j as i32;

                            //If the user did click on a unit, allow the player to move the unit
                            game_map.unit_interface = Some(UnitInterface::from_unit(active_unit, core.texture_map.get("unit_interface").unwrap()));
                            player_state.current_player_action = PlayerAction::ChoosingUnitAction;
                        }
                    },
                    _ => {},
                }
            },
            PlayerAction::ChoosingUnitAction => {
                if core.input.left_clicked {
                    // Handle clicking based on unit interface
                    let active_unit = game_map.player_units.get(&(player_state.active_unit_j as u32, player_state.active_unit_i as u32)).unwrap();
                    player_state.current_player_action = game_map.unit_interface.as_ref().unwrap().get_click_selection(glob_x, glob_y);
                    match player_state.current_player_action {
                        PlayerAction::Default => {
                            // Deselect the active unit
                            player_state.active_unit_i = -1;
                            player_state.active_unit_j = -1;
                            // Close interface
                            game_map.unit_interface.as_mut().unwrap().animate_close();
                        },
                        PlayerAction::ChoosingUnitAction => {},
                        PlayerAction::MovingUnit => {
                            game_map.possible_moves = active_unit.get_tiles_in_movement_range(&mut game_map.map_tiles);
                            // Close interface
                            game_map.unit_interface.as_mut().unwrap().animate_close();
                        },
                        PlayerAction::AttackingUnit => {
                            game_map.possible_attacks = active_unit.get_tiles_in_attack_range(&mut game_map.map_tiles);
                            game_map.actual_attacks = active_unit.get_tiles_can_attack(&mut game_map.map_tiles);
                            // Close interface
                            game_map.unit_interface.as_mut().unwrap().animate_close();
                        },
                        _ => {},
                    }
                }
            },
            PlayerAction::MovingUnit => {
                if core.input.right_clicked {
                    // Deselect the active unit
                    player_state.active_unit_i = -1;
                    player_state.active_unit_j = -1;
                    player_state.current_player_action = PlayerAction::Default;
                }
                else if core.input.left_clicked {
                    // Ensure valid tile to move to
                    if game_map.possible_moves.contains(&(j,i)) {
                        // Move unit
                        let mut active_unit = game_map.player_units.remove(&(player_state.active_unit_j as u32, player_state.active_unit_i as u32)).unwrap();
                        active_unit.update_pos(j, i);
                        active_unit.has_moved = true;
                        game_map.player_units.insert((j, i), active_unit);
                        // Update map tiles
                        if let Some(old_map_tile) = game_map.map_tiles.get_mut(&(player_state.active_unit_i as u32, player_state.active_unit_j as u32)) {
                            old_map_tile.update_team(None);
                        }
                        if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(i, j)) {
                            new_map_tile.update_team(Some(Team::Player));
                        }
                    }

                    //Now that the unit has moved, deselect
                    player_state.active_unit_i = -1;
                    player_state.active_unit_j = -1;
                    player_state.current_player_action = PlayerAction::Default;
                }
            },
            PlayerAction::AttackingUnit => {
                if core.input.right_clicked {
                    // Deselect the active unit
                    player_state.active_unit_i = -1;
                    player_state.active_unit_j = -1;
                    player_state.current_player_action = PlayerAction::Default;
                } else if core.input.left_clicked {
                    // Attack unit clicked on
                    // The player should only be able to attack if the tile they clicked on contains an opposing unit within their range
                    if game_map.actual_attacks.contains(&(j, i)) {
                        let mut active_unit = game_map.player_units.get_mut(&(player_state.active_unit_j as u32, player_state.active_unit_i as u32)).unwrap();
                        let mut dead_barb: bool = false;
                        //All attack handling is done here
                        let damage_done = active_unit.get_attack_damage();
                        if let Some(tile_under_attack) = game_map.map_tiles.get_mut(&(i, j)) {
                            match tile_under_attack.contained_unit_team {
                                Some(Team::Enemy) => {
                                    if let Some(unit) = game_map.enemy_units.get_mut(&(j, i)) {
                                        println!("Enemy unit starting at {} hp.", unit.hp);
                                        if unit.hp <= damage_done {
                                            game_map.enemy_units.remove(&(j, i));
                                            println!("Enemy unit at {}, {} is dead after taking {} damage.", j, i, damage_done);
                                            tile_under_attack.update_team(None);
                                        } else {
                                            unit.receive_damage(damage_done);
                                            game_map.damage_indicators.push(DamageIndicator::new(core, damage_done, PixelCoordinates::from_matrix_indices(unit.y - 1, unit.x))?);
                                            println!("Unit at {}, {} attacking enemy unit at {}, {} for {} damage. Unit now has {} hp.", active_unit.x, active_unit.y, j, i, damage_done, unit.hp);
                                        }
                                    }
                                },
                                _ => {
                                    if let Some(unit) = game_map.barbarian_units.get_mut(&(j, i)) {
                                        println!("Barbarian unit starting at {} hp.", unit.hp);
                                        if unit.hp <= damage_done {
                                            game_map.barbarian_units.remove(&(j, i));
                                            println!("Barbarian unit at {}, {} is dead after taking {} damage.", j, i, damage_done);
                                            tile_under_attack.update_team(None);
                                            dead_barb = true;
                                        } else {
                                            unit.receive_damage(damage_done);
                                            game_map.damage_indicators.push(DamageIndicator::new(core, damage_done, PixelCoordinates::from_matrix_indices(unit.y - 1, unit.x))?);
                                            println!("Unit at {}, {} attacking barbarian unit at {}, {} for {} damage. Unit now has {} hp.", active_unit.x, active_unit.y, j, i, damage_done, unit.hp);
                                        }
                                    }
                                } //This handles the barbarian case and also prevents rust from complaining about unchecked cases,
                            }
                        }
                        active_unit.has_attacked = true;
                        if dead_barb {
                            //Need to check and see if this barbarian was converted - currently a 45% chance
                            let chance = rand::thread_rng().gen_range(0..100);
                            if chance < 45 {
                                player_state.current_player_action = PlayerAction::ChoosePrimer;
                            }
                            else {
                                println!("Failed to get new unit");
                            }
                        }
                    }
                    // After attack, deselect
                    player_state.active_unit_i = -1;
                    player_state.active_unit_j = -1;
                    match player_state.current_player_action {
                        PlayerAction::ChoosePrimer => {},
                        _ => player_state.current_player_action = PlayerAction::Default,
                    }                }
            }
            PlayerAction::ChoosePrimer => {
                game_map.choose_unit_interface = Some(UnitInterface::from_conversion(core, core.texture_map.get("unit_interface").unwrap()));
                player_state.current_player_action = PlayerAction::ChoosingNewUnit;
            }
            PlayerAction::ChoosingNewUnit => {
                let castle_coord = &game_map.pos_player_castle;
                if core.input.left_clicked {
                    // Handle clicking based on unit interface
                    player_state.current_player_action = game_map.choose_unit_interface.as_ref().unwrap().get_choose_unit_click_selection(glob_x, glob_y);
                    match player_state.current_player_action {
                        PlayerAction::ChosenRanger => {
                            let mut new_unit = Unit::new(castle_coord.0-5, castle_coord.1+5, Team::Player, 15, 5, 4, 85, 3, 7, core.texture_map.get("plr").unwrap());
                            let respawn_location = new_unit.respawn_loc(&mut game_map.map_tiles, *castle_coord);
                            new_unit.update_pos(respawn_location.0, respawn_location.1);
                            //The new unit should not be able to move immediately after being converted
                            new_unit.has_moved = true;
                            new_unit.has_attacked = true;
                            println!("Unit spawned at {}, {}", respawn_location.0, respawn_location.1);

                            //Don't forget to update the players units and the hash map
                            game_map.player_units.insert(respawn_location, new_unit);
                            if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(respawn_location.1, respawn_location.0)) {
                                new_map_tile.update_team(Some(Team::Player));
                            }
                            game_map.choose_unit_interface.as_mut().unwrap().animate_close();
                            player_state.current_player_action = PlayerAction::Default;
                         },
                         PlayerAction::ChosenMelee => {
                            let mut new_unit = Unit::new(castle_coord.0-5, castle_coord.1+5, Team::Player, 20, 7, 1, 95, 1, 5, core.texture_map.get("pll").unwrap());
                            let respawn_location = new_unit.respawn_loc(&mut game_map.map_tiles, *castle_coord);
                            new_unit.update_pos(respawn_location.0, respawn_location.1);
                            //The new unit should not be able to move immediately after being converted
                            new_unit.has_moved = true;
                            new_unit.has_attacked = true;
                            println!("Unit spawned at {}, {}", respawn_location.0, respawn_location.1);

                            //Don't forget to update the players units and the hash map
                            game_map.player_units.insert(respawn_location, new_unit);
                            if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(respawn_location.1, respawn_location.0)) {
                                new_map_tile.update_team(Some(Team::Player));
                            }
                            game_map.choose_unit_interface.as_mut().unwrap().animate_close();
                            player_state.current_player_action = PlayerAction::Default;
                        }
                        PlayerAction::ChosenMage => {
                            let mut new_unit = Unit::new(castle_coord.0-5, castle_coord.1+5, Team::Player, 10, 6, 3, 75,  5, 9, core.texture_map.get("plm").unwrap());
                            let respawn_location = new_unit.respawn_loc(&mut game_map.map_tiles, *castle_coord);
                            new_unit.update_pos(respawn_location.0, respawn_location.1);
                            //The new unit should not be able to move immediately after being converted
                            new_unit.has_moved = true;
                            new_unit.has_attacked = true;
                            println!("Unit spawned at {}, {}", respawn_location.0, respawn_location.1);

                            //Don't forget to update the players units and the hash map
                            game_map.player_units.insert(respawn_location, new_unit);
                            if let Some(new_map_tile) = game_map.map_tiles.get_mut(&(respawn_location.1, respawn_location.0)) {
                                new_map_tile.update_team(Some(Team::Player));
                            }
                            game_map.choose_unit_interface.as_mut().unwrap().animate_close();
                            player_state.current_player_action = PlayerAction::Default;
                        }
                        _ => {},
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn end_player_turn<'a>(game_map: &mut GameMap<'a>) {
    //End player's turn
    game_map.player_state.current_turn = Team::Enemy;

    //Clear the player UI if it is still visible
    game_map.unit_interface = None;
    game_map.cursor.hide_cursor();

    //Deselect the active unit
    game_map.player_state.active_unit_i = -1;
    game_map.player_state.active_unit_j = -1;
    game_map.player_state.current_player_action = PlayerAction::Default;

    //Reactivate any grayed out player units
    game_map.initialize_next_turn(Team::Player);

    //Start displaying the enemy's banner
    game_map.banner.show("p2_banner");
}