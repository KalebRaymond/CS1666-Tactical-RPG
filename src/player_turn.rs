use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Texture;

use std::convert::TryInto;

use crate::game_map::GameMap;
use crate::input::Input;
use crate::pixel_coordinates::PixelCoordinates;
use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::SDLCore;
use crate::unit_interface::UnitInterface;
use crate::unit::{Team, Unit};
use crate::turn_banner::TurnBanner;

pub fn handle_player_turn<'a>(core: &SDLCore, player_state: &mut PlayerState, game_map: &mut GameMap, input: &Input, turn_banner: &mut TurnBanner, unit_interface: &mut Option<UnitInterface<'a>>, unit_interface_texture: &'a Texture<'a>, current_player: &mut Team) {
    if !turn_banner.banner_visible {
        if input.keystate.contains(&Keycode::Backspace) {
            //End turn
            *current_player = Team::Enemy;

            //Start displaying the enemy's banner
            turn_banner.current_banner_transparency = 250;
            turn_banner.banner_colors = Color::RGBA(207, 21, 24, turn_banner.current_banner_transparency);
            turn_banner.banner_key = "p2_banner";
            turn_banner.banner_visible = true;
        }

        //Get map matrix indices from mouse position
        let (i, j) = PixelCoordinates::matrix_indices_from_pixel(
            input.mouse_state.x().try_into().unwrap(), 
            input.mouse_state.y().try_into().unwrap(), 
            (-1 * core.cam.x).try_into().unwrap(), 
            (-1 * core.cam.y).try_into().unwrap()
        );
        let (glob_x, glob_y) = PixelCoordinates::global_coordinates(
            input.mouse_state.x().try_into().unwrap(), 
            input.mouse_state.y().try_into().unwrap(), 
            (-1 * core.cam.x).try_into().unwrap(), 
            (-1 * core.cam.y).try_into().unwrap()
        );

        match player_state.current_player_action {
            PlayerAction::Default => {
                if input.left_clicked {
                    //Get the unit that is located at the mouse position, if there is one
                    match player_state.p1_units.get_mut(&(j,i)) {
                        Some(active_unit) => {
                            player_state.active_unit_i = i as i32;
                            player_state.active_unit_j = j as i32;

                            //If the user did click on a unit, allow the player to move the unit
                            *unit_interface = Some(UnitInterface::from_unit(active_unit, unit_interface_texture));
                            player_state.current_player_action = PlayerAction::ChoosingUnitAction;
                        },
                        _ => {},
                    }
                }	
            },
            PlayerAction::ChoosingUnitAction => {
                if input.left_clicked {
                    // Handle clicking based on unit interface
                    let active_unit = player_state.p1_units.get(&(player_state.active_unit_j as u32, player_state.active_unit_i as u32)).unwrap();
                    player_state.current_player_action = unit_interface.as_ref().unwrap().get_click_selection(glob_x, glob_y);
                    match player_state.current_player_action {
                        PlayerAction::Default => {
                            // Deselect the active unit
                            player_state.active_unit_i = -1;
                            player_state.active_unit_j = -1;
                            // Close interface
                            unit_interface.as_mut().unwrap().animate_close();
                        },
                        PlayerAction::ChoosingUnitAction => {},
                        PlayerAction::MovingUnit => {
                            game_map.possible_moves = active_unit.get_tiles_in_movement_range(&mut game_map.map_tiles);
                            // Close interface
                            unit_interface.as_mut().unwrap().animate_close();
                        },
                        PlayerAction::AttackingUnit => {
                            game_map.possible_attacks = active_unit.get_tiles_in_attack_range(&mut game_map.map_tiles);
                            game_map.actual_attacks = active_unit.get_tiles_can_attack(&mut game_map.map_tiles);
                            // Close interface
                            unit_interface.as_mut().unwrap().animate_close();
                        },
                    }
                }
            },		
            PlayerAction::MovingUnit => {
                if input.right_clicked {
                    // Deselect the active unit
                    player_state.active_unit_i = -1;
                    player_state.active_unit_j = -1;
                    player_state.current_player_action = PlayerAction::Default;
                }
                else if input.left_clicked {
                    // Ensure valid tile to move to
                    if game_map.possible_moves.contains(&(j,i)) {
                        // Move unit
                        let mut active_unit = player_state.p1_units.remove(&(player_state.active_unit_j as u32, player_state.active_unit_i as u32)).unwrap();
                        active_unit.update_pos(j, i);
                        active_unit.has_moved = true;
                        player_state.p1_units.insert((j, i), active_unit);
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
                if input.right_clicked {
                    // Deselect the active unit
                    player_state.active_unit_i = -1;
                    player_state.active_unit_j = -1;
                    player_state.current_player_action = PlayerAction::Default;
                } else if input.left_clicked {
                    // Attack unit clicked on
                    // After attack, deselect
                    player_state.active_unit_i = -1;
                    player_state.active_unit_j = -1;
                    player_state.current_player_action = PlayerAction::Default;
                }
            },				
        }
    }
}