use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::unit::{Team, Unit};
use crate::turn_banner::TurnBanner;

pub fn handle_enemy_turn<'a>(p2_units: &mut HashMap<(u32, u32), Unit<'a>>, p1_units: &mut HashMap<(u32, u32), Unit<'a>>, barbarian_units: &mut HashMap<(u32, u32), Unit<'a>>, game_map: &mut GameMap, turn_banner: &mut TurnBanner, current_player: &mut Team, p2_castle: &(u32, u32), p1_castle: &(u32, u32), camp_coords: &Vec<(u32, u32)>) {
    if !turn_banner.banner_visible {
        //End turn
        *current_player = Team::Barbarians;

        //Start displaying the barbarians' banner
        turn_banner.current_banner_transparency = 250;
        turn_banner.banner_colors = Color::RGBA(163,96,30, turn_banner.current_banner_transparency);
        turn_banner.banner_key = "b_banner";
        turn_banner.banner_visible = true;
    }
}