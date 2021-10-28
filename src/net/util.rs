pub const MSG_CREATE: u8 = 0; // create a new room
pub const MSG_JOIN: u8 = 1;   // join an existing room
pub const MSG_EVENT: u8 = 2;  // push an event to a room
pub const MSG_POLL: u8 = 3;   // poll for new events in a room

pub const EVENT_NONE: u8 = 0; // there are no events to poll
pub const EVENT_JOIN: u8 = 1; // a player has joined the room

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

pub struct Event {
	pub action: u8,
	pub id: u8,
	pub from_pos: (u32, u32),
	pub to_pos: (u32, u32),
}

impl Event {
	pub fn new(action: u8) -> Event {
		Event {
			action,
			id: 0,
			from_pos: (0, 0),
			to_pos: (0, 0),
		}
	}

	pub fn from_bytes(arr: &[u8; 18]) -> Event {
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
		}
	}

	pub fn to_bytes(&self) -> [u8; 18] {
		let mut arr = [0; 18];
		arr[0] = self.action;
		arr[1] = self.id;

		set_range!(arr[2..6] = to_u32_bytes(self.from_pos.0));
		set_range!(arr[6..10] = to_u32_bytes(self.from_pos.1));

		set_range!(arr[10..14] = to_u32_bytes(self.to_pos.0));
		set_range!(arr[14..18] = to_u32_bytes(self.to_pos.1));

		arr
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