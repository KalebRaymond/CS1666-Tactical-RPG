use crate::player_action::PlayerAction;
use crate::unit_interface::UnitInterface;
use crate::unit::Team;

pub struct PlayerState {
    pub team: Team,
    pub current_turn: Team,

    //Matrix coordinates of the currently selected unit. When these are both equal to -1, no unit is selected
    pub active_unit_i: i32,
    pub active_unit_j: i32,

    //Player action to handle inputs differently based on context
    pub current_player_action: PlayerAction
}

impl PlayerState {
    pub fn new(team: Team) -> PlayerState {
        PlayerState {
            team,
            current_turn: Team::Player,
            active_unit_i: -1,
            active_unit_j: -1,
            current_player_action: PlayerAction::Default,
        }
    }

    pub fn advance_turn(&mut self) -> Team {
        match self.current_turn {
            Team::Player => self.current_turn = Team::Enemy,
            Team::Enemy => self.current_turn = Team::Barbarians,
            Team::Barbarians => self.current_turn = Team::Player,
        }

        self.current_turn
    }

    pub fn is_turn(&self) -> bool {
        return self.current_turn == self.team;
    }
}
