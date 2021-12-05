use rand::{seq::IteratorRandom, thread_rng};

use crate::game_map::GameMap;
use crate::SDLCore;
use crate::net::util::*;

pub fn handle_barbarian_turn<'a>(_core: &SDLCore<'a>, game_map: &mut GameMap<'a>) -> Result<(), String> {
	if game_map.banner.banner_visible {
		return Ok(())
	}

	//RNG for making unaggroed barbarians roam
	let mut rng_thread = thread_rng();

	let barbarian = if let Some((_, b)) = game_map.barbarian_units.iter().find(|(_, u)| u.has_moved == false) {
		b
	} else {
		// no more units to move: end turn
		game_map.event_list.push(Event::create(EVENT_END_TURN, EVENT_ID_BARBARIAN, (0,0), (0,0), 0));
		return Ok(());
	};

	let (original_x, original_y) = (barbarian.x, barbarian.y);
	let possible_moves: Vec<(u32, u32)> = barbarian.get_tiles_in_movement_range(&mut game_map.map_tiles);

	for (mov_x, mov_y) in &possible_moves {
		let attacks = barbarian.get_tiles_can_attack_from_pos((*mov_x, *mov_y), &mut game_map.map_tiles);
		if attacks.is_empty() {
			continue;
		}

		let (atk_x, atk_y) = attacks[0];
		let attacked_unit = game_map.get_unit(&(atk_x, atk_y))?;
		let damage_done = barbarian.get_attack_damage(attacked_unit);

		game_map.event_list.push(Event::create(EVENT_MOVE, 0, (original_x, original_y), (*mov_x, *mov_y), 0));
		game_map.event_list.push(Event::create(EVENT_ATTACK, 0, (*mov_x, *mov_y), (atk_x, atk_y), damage_done as u8));
		return Ok(());
	}

	//If the barbarian did not find a unit to attack, make it move randomly by 1 tile in an available direction
	let mut directions = vec![0, 1, 2, 3, 4];
	let mut potential_move = (original_x, original_y);

	while directions.len() > 0 {
		//Pick and remove a random direction from the vector of directions
		let index = (0..directions.len()).choose(&mut rng_thread).unwrap();
		let direction_to_move = directions.swap_remove(index);

		match direction_to_move {
			0 => {
				//Move up
				potential_move.1 -= 1;
			},
			1 => {
				//Move right
				potential_move.0 += 1;
			},
			2 => {
				//Move down
				potential_move.1 += 1;
			},
			3 => {
				//Move left
				potential_move.0 -= 1;
			},
			_ => {
				//Do nothing
			}
		};

		//Make sure the barbarian does not roam outside of a certain manhattan distance from its starting point
		//Radius = 3 tiles
		let dist_from_start_point = ((potential_move.0 as i32) - (barbarian.starting_x as i32)).abs() + ((potential_move.1 as i32) - (barbarian.starting_y as i32)).abs();

		if dist_from_start_point <= 3 && possible_moves.contains(&potential_move) {
			//Move the barbarian
			game_map.event_list.push(Event::create(EVENT_MOVE, 0, (original_x, original_y), potential_move, 0));
			break;
		}

		//Reset the potential move to the barbarian's starting position and try a different direction
		potential_move = (original_x, original_y);
	}

	Ok(())
}
