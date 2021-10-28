use sdl2::pixels::Color;
use std::collections::HashMap;

use crate::game_map::GameMap;
use crate::pixel_coordinates::PixelCoordinates;
use crate::unit::{Team, Unit};
use crate::turn_banner::TurnBanner;

pub fn handle_barbarian_turn<'a>(barb_units: &mut HashMap<(u32, u32), Unit<'a>>, game_map: &mut GameMap, turn_banner: &mut TurnBanner, current_player: &mut Team) {
    if !turn_banner.banner_visible {
        //End turn
        *current_player = Team::Player;

        //Start displaying Player 1's banner
        turn_banner.current_banner_transparency = 250;
        turn_banner.banner_colors = Color::RGBA(0, 89, 178, turn_banner.current_banner_transparency);
        turn_banner.banner_key = "p1_banner";
        turn_banner.banner_visible = true;
    }
}