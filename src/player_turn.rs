use rand::Rng;

use sdl2::keyboard::Keycode;

use std::convert::TryInto;

use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::player_action::PlayerAction;
use crate::SDLCore;
use crate::unit_interface::UnitInterface;
use crate::net::util::*;

pub fn handle_player_turn<'a>(core: &SDLCore<'a>, game_map: &mut GameMap<'a>) -> Result<(), String> {
    if game_map.banner.banner_visible {
        return Ok(());
    }

    //Check if player ended turn by pressing backspace
    if core.input.keystate.contains(&Keycode::Backspace) && match game_map.player_state.current_player_action {
        PlayerAction::ChoosingNewUnit => false,
        _ => true,
    }{
        end_player_turn(game_map);
        return Ok(());
    }

    //Check if user clicked the end turn button
    if core.input.left_clicked && game_map.end_turn_button.is_mouse(core) && match game_map.player_state.current_player_action {
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

    match game_map.player_state.current_player_action {
        PlayerAction::Default => {
            //If player hovers over a unit, display cursor above that unit
            match game_map.player_units.get_mut(&(j,i)) {
                Some(active_unit) => {
                    //Now check if the player actually clicked on the unit they hovered over
                    if core.input.left_clicked {
                        game_map.player_state.active_unit_i = i as i32;
                        game_map.player_state.active_unit_j = j as i32;

                        //If the user did click on a unit, allow the player to move the unit
                        game_map.unit_interface = Some(UnitInterface::from_unit(active_unit, core.texture_map.get("unit_interface").unwrap()));
                        game_map.player_state.current_player_action = PlayerAction::ChoosingUnitAction;
                    }
                },
                _ => {},
            }
        },
        PlayerAction::ChoosingUnitAction => {
            if core.input.left_clicked {
                // Handle clicking based on unit interface
                let active_unit = game_map.player_units.get(&(game_map.player_state.active_unit_j as u32, game_map.player_state.active_unit_i as u32)).unwrap();
                game_map.player_state.current_player_action = game_map.unit_interface.as_ref().unwrap().get_click_selection(glob_x, glob_y);
                match game_map.player_state.current_player_action {
                    PlayerAction::Default => {
                        // Deselect the active unit
                        game_map.player_state.active_unit_i = -1;
                        game_map.player_state.active_unit_j = -1;
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
                game_map.player_state.active_unit_i = -1;
                game_map.player_state.active_unit_j = -1;
                game_map.player_state.current_player_action = PlayerAction::Default;
            }
            else if core.input.left_clicked {
                // Ensure valid tile to move to
                if game_map.possible_moves.contains(&(j,i)) {
                    game_map.event_list.push(Event::create(EVENT_MOVE, 0, (game_map.player_state.active_unit_j as u32, game_map.player_state.active_unit_i as u32), (j, i), 0));
                }

                //Now that the unit has moved, deselect
                game_map.player_state.active_unit_i = -1;
                game_map.player_state.active_unit_j = -1;
                game_map.player_state.current_player_action = PlayerAction::Default;
            }
        },
        PlayerAction::AttackingUnit => {
            if core.input.right_clicked {
                // Deselect the active unit
                game_map.player_state.active_unit_i = -1;
                game_map.player_state.active_unit_j = -1;
                game_map.player_state.current_player_action = PlayerAction::Default;
            } else if core.input.left_clicked {
                // Attack unit clicked on
                // The player should only be able to attack if the tile they clicked on contains an opposing unit within their range
                if game_map.actual_attacks.contains(&(j, i)) {
                    let active_unit = game_map.get_unit(&(game_map.player_state.active_unit_j as u32, game_map.player_state.active_unit_i as u32))?;
                    let atk_unit = game_map.get_unit(&(j, i))?;
                    let atk_damage = active_unit.get_attack_damage(atk_unit);
                    let atk_kill = atk_unit.hp <= atk_damage;
                    println!("Player: Attacking unit at {:?} with {} damage.", (j, i), atk_damage);

                    game_map.event_list.push(Event::create(EVENT_ATTACK, 0, (game_map.player_state.active_unit_j as u32, game_map.player_state.active_unit_i as u32), (j, i), atk_damage as u8));

                    // if the attacked Barbarian will die on this turn...
                    if atk_kill {
                        //Need to check and see if this barbarian was converted - currently a 45% chance
                        let chance = rand::thread_rng().gen_range(0..100);
                        if chance < 45 {
                            game_map.player_state.current_player_action = PlayerAction::ChoosePrimer;
                        }
                        else {
                            println!("Failed to get new unit");
                        }
                    }
                }
                // After attack, deselect
                game_map.player_state.active_unit_i = -1;
                game_map.player_state.active_unit_j = -1;
                match game_map.player_state.current_player_action {
                    PlayerAction::ChoosePrimer => {},
                    _ => game_map.player_state.current_player_action = PlayerAction::Default,
                }                }
            }
            PlayerAction::ChoosePrimer => {
                game_map.choose_unit_interface = Some(UnitInterface::from_conversion(core, core.texture_map.get("unit_interface").unwrap()));
                game_map.player_state.current_player_action = PlayerAction::ChoosingNewUnit;
            }
            PlayerAction::ChoosingNewUnit => {
                let castle_coord = &game_map.objectives.p1_castle;
                if core.input.left_clicked {
                    // Handle clicking based on unit interface
                    game_map.player_state.current_player_action = game_map.choose_unit_interface.as_ref().unwrap().get_choose_unit_click_selection(glob_x, glob_y);
                    let respawn_location = crate::unit::respawn_loc((castle_coord.0-5, castle_coord.1+5), &mut game_map.map_tiles, *castle_coord);
                    let unit_id = match game_map.player_state.current_player_action {
                        PlayerAction::ChosenRanger => EVENT_UNIT_ARCHER,
                        PlayerAction::ChosenMelee => EVENT_UNIT_MELEE,
                        PlayerAction::ChosenMage => EVENT_UNIT_MAGE,
                        _ => 100 // arbitrary unused id value
                    };

                    if unit_id != 100 {
                        game_map.event_list.push(Event::create(EVENT_SPAWN_UNIT, unit_id, (0,0), respawn_location, unit_id));

                        game_map.choose_unit_interface.as_mut().unwrap().animate_close();
                        game_map.player_state.current_player_action = PlayerAction::Default;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn end_player_turn<'a>(game_map: &mut GameMap<'a>) {
        //Clear the player UI if it is still visible
        game_map.unit_interface = None;
        game_map.cursor.hide_cursor();

        //Deselect the active unit
        game_map.player_state.active_unit_i = -1;
        game_map.player_state.active_unit_j = -1;
        game_map.player_state.current_player_action = PlayerAction::Default;

        game_map.event_list.push(Event::create(EVENT_END_TURN, EVENT_ID_PLAYER, (0, 0), (0, 0), 0));
    }
