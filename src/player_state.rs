use crate::player_action::PlayerAction;

pub struct PlayerState {
    //Matrix coordinates of the currently selected unit. When these are both equal to -1, no unit is selected
    pub active_unit_i: i32,
    pub active_unit_j: i32,

    //Player action to handle inputs differently based on context
    pub current_player_action: PlayerAction,
}

impl PlayerState {
    pub fn new() -> PlayerState {
        PlayerState {
            active_unit_i: -1,
            active_unit_j: -1,
            current_player_action: PlayerAction::Default,
        }
    }
}
