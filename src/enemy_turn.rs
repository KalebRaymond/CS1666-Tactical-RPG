use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::unit::{Team, Unit};
use crate::turn_banner::TurnBanner;

pub fn handle_enemy_turn<'a>(p2_units: &mut HashMap<(u32, u32), Unit<'a>>, p1_units: &mut HashMap<(u32, u32), Unit<'a>>, barbarian_units: &mut HashMap<(u32, u32), Unit<'a>>, game_map: &mut GameMap, turn_banner: &mut TurnBanner, current_player: &mut Team, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) {
    if !turn_banner.banner_visible {
        evaluate_current_position(p2_units, game_map, p2_castle, p1_castle, camp_coords);
        //End turn
        *current_player = Team::Barbarians;

        //Start displaying the barbarians' banner
        turn_banner.current_banner_transparency = 250;
        turn_banner.banner_colors = Color::RGBA(163,96,30, turn_banner.current_banner_transparency);
        turn_banner.banner_key = "b_banner";
        turn_banner.banner_visible = true;
    }
}

pub fn evaluate_current_position<'a> (p2_units: &HashMap<(u32, u32), Unit<'a>>, game_map: &GameMap, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) -> u32 {
    let mut total_value: u32 = 0;
    let mut units_defending: u32 = 0; //Units near own castle 
    let mut units_sieging: u32 = 0; //Units near enemy castle
    let mut units_near_camp: u32 = 0;
    let mut units_able_to_attack: u32 = 0;

    for unit in p2_units.values() {
        let result = current_unit_value(unit, game_map, p2_castle, p1_castle, camp_coords);
        total_value += result.0;
        units_defending += result.1;
        units_sieging += result.2;
        units_near_camp += result.3;
        units_able_to_attack += result.4;
    }

    total_value
}

// Order of values in return 
// 0: value of state
// 1: near_own_castle
// 2: near_enemy_castle
// 3: near_camp
// 4: able_to_attack
pub fn current_unit_value<'b> (unit: &Unit<'b>, game_map: &GameMap, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) -> (u32, u32, u32, u32, u32) {
    let mut value: u32 = 0;
    let mut defending: u32 = 0; //Units near own castle 
    let mut sieging: u32 = 0; //Units near enemy castle
    let mut near_camp: u32 = 0;
    let mut able_to_attack: u32 = 0;

    (value, defending, sieging, near_camp, able_to_attack)
}