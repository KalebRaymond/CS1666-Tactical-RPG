use std::collections::HashMap;

use crate::player_action::PlayerAction;
use crate::unit::Unit;

pub struct PlayerState<'a> {
    //Matrix coordinates of the currently selected unit. When these are both equal to -1, no unit is selected
    pub active_unit_i: i32,
    pub active_unit_j: i32,

    //Player action to handle inputs differently based on context
    pub current_player_action: PlayerAction,

    pub p1_units: HashMap<(u32, u32), Unit<'a>>,
}

impl PlayerState<'_> {
    pub fn new<'a>() -> PlayerState<'a> {
        PlayerState {
            active_unit_i: -1,
            active_unit_j: -1,
            current_player_action: PlayerAction::Default,
            p1_units: HashMap::new(),
        }
    }
}