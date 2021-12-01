struct Objective {
    location
    height
    width
    current_team
    turns_occupied
}

impl Objective {
    pub fn new() -> Objective {

    }

    pub fn contains_location(location: u32, u32) -> bool {

    }
}

pub struct ObjectiveManager {
    p1 castle
    p2 castle
    camps
}

impl ObjectiveManager {
    pub fn new() -> ObjectiveManager {

    }

    pub fn check_objectives(team: Team, team_units) {
        //Check if enemy is occupying player castle
        if team == enemy && team has a unit on p1 castle


        //Check if player is occupying enemy castle
        if team == player && team has a unit on p2 castle

        //Check barbarian camps
        if team != barbarian
            for each camp in camps
                if team has a unit in this camp
    }
}