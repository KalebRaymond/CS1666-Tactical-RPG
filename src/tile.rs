use sdl2::render::Texture;
use std::fmt;
use crate::unit::{Team};

pub enum Structure {
	Camp,
	P-Castle,
	E-Castle,
}
impl PartialEq for Structure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Structure::Camp, Structure::Camp) => true,
            (Structure::P-Castle, Structure::P-Castle) => true,
            (Structure::E-Castle, Structure::E-Castle) => true,
            _ => false,
        }
    }
}

pub struct Tile<'a> {
    pub x: u32,
    pub y: u32,
    pub is_traversable: bool,
    pub can_attack_through: bool, // e.x. archers and mages can attack over rivers and through trees
    pub contained_unit_team: Option<Team>, // Storing a unit causes some pains with lifetimes and references, so store an enum that is better than a boolean
    pub contained_structure: Option<Structure>,
    pub texture: &'a Texture<'a>,
}

impl Tile <'_>{
    pub fn new<'a> (x:u32, y:u32, is_traversable: bool, can_attack_through: bool, contained_unit_team: Option<Team>, texture: &'a Texture) -> Tile<'a> {
        Tile {
            x,
            y,
            is_traversable,
            can_attack_through,
            contained_unit_team,
            texture,
        }
    }
    pub fn update_team(&mut self, new_team: Option<Team>) {
        self.contained_unit_team = new_team;
    }
    pub fn unit_can_move_here(&self) -> bool {
        match &self.contained_unit_team {
            Some(_) => false,
            None => self.is_traversable,
        }
    }
}

impl fmt::Display for Tile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tile(x:{}, y:{}, is_traversable:{})", self.x, self.y, self.is_traversable)
    }
}