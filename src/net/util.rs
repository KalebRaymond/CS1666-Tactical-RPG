pub const MSG_CREATE: u8 = 0; // create a new room
pub const MSG_JOIN: u8 = 1;   // join an existing room
pub const MSG_EVENT: u8 = 2;  // push an event to a room
pub const MSG_POLL: u8 = 3;   // poll for new events in a room

pub const EVENT_NONE: u8 = 0; // there are no events to poll
pub const EVENT_JOIN: u8 = 1; // a player has joined the room
pub const EVENT_MOVE: u8 = 2;
pub const EVENT_ATTACK: u8 = 3;
pub const EVENT_END_TURN: u8 = 4;
pub const EVENT_END_GAME: u8 = 5;
pub const EVENT_SPAWN_UNIT: u8 = 6;

pub const EVENT_ID_ENEMY: u8 = 0;
pub const EVENT_ID_PLAYER: u8 = 1;
pub const EVENT_ID_BARBARIAN: u8 = 2;

pub const EVENT_UNIT_ARCHER: u8 = 0;
pub const _EVENT_UNIT_GUARD: u8 = 1;
pub const EVENT_UNIT_MAGE: u8 = 2;
pub const EVENT_UNIT_MELEE: u8 = 3;
pub const _EVENT_UNIT_SCOUT: u8 = 4;

// allows a range of indeces in an array to be set with one expression
// e.g. set_range!(arr[4..6] = [4, 5, 6, 7, 8]); will set arr[4] = 4 and arr[5] = 5
macro_rules! set_range {
	($to:ident[$range:expr] = $from:expr) => {
		{
			let from = $from;
			for i in $range {
				$to[i] = from[i - $range.start];
			}
		}
	};
}

#[derive(Copy, Clone)]
pub struct Event {
	pub action: u8,
	pub id: u8,
	pub from_pos: (u32, u32),
	pub to_pos: (u32, u32),
	pub value: u8,
	pub from_self: bool,
}

impl Event {
	pub fn new(action: u8) -> Event {
		Event {
			action,
			id: 0,
			from_pos: (0, 0),
			to_pos: (0, 0),
			value: 0,
			from_self: true,
		}
	}

	pub fn create(action: u8, id: u8, from_pos: (u32, u32), to_pos: (u32, u32), value: u8) -> Event {
		Event {
			action,
			id,
			from_pos,
			to_pos,
			value,
			from_self: true,
		}
	}

	pub fn from_bytes(arr: &[u8; 19]) -> Event {
		Event {
			action: arr[0],
			id: arr[1],
			from_pos: (
				from_u32_bytes(&arr[2..6]),
				from_u32_bytes(&arr[6..10]),
			),
			to_pos: (
				from_u32_bytes(&arr[10..14]),
				from_u32_bytes(&arr[14..18]),
			),
			value: arr[18],
			from_self: false,
		}
	}

	pub fn to_bytes(&self) -> [u8; 19] {
		let mut arr = [0; 19];
		arr[0] = self.action;
		arr[1] = self.id;

		set_range!(arr[2..6] = to_u32_bytes(self.from_pos.0));
		set_range!(arr[6..10] = to_u32_bytes(self.from_pos.1));

		set_range!(arr[10..14] = to_u32_bytes(self.to_pos.0));
		set_range!(arr[14..18] = to_u32_bytes(self.to_pos.1));

		arr[18] = self.value;
		arr
	}
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let action = match self.action {
			EVENT_NONE => "none",
			EVENT_JOIN => "join",
			EVENT_MOVE => "move",
			EVENT_ATTACK => "attack",
			EVENT_END_TURN => "end turn",
			EVENT_END_GAME => "end game",
			EVENT_SPAWN_UNIT => "spawn unit",
			_ => "unknown",
		};

        write!(f, "Event(action:{}, id:{}, from:{:?}, to:{:?}, value:{})", action, self.id, self.from_pos, self.to_pos, self.value)
    }
}

pub fn to_u32_bytes(num: u32) -> [u8; 4] {
	[
		(num >> 24) as u8,
		(num >> 16) as u8,
		(num >> 8) as u8,
		(num >> 0) as u8,
	]
}

pub fn from_u32_bytes(arr: &[u8]) -> u32 {
	(arr[0] as u32) << 24
	| (arr[1] as u32) << 16
	| (arr[2] as u32) << 8
	| (arr[3] as u32)
}
