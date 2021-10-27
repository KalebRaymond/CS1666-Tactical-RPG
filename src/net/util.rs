pub const MSG_CREATE: u8 = 0;
pub const MSG_JOIN: u8 = 1;
pub const MSG_EVENT: u8 = 2;
pub const MSG_POLL: u8 = 3;

pub fn to_u32_bytes(num: u32) -> [u8; 4] {
	[
		(num >> 24) as u8,
		(num >> 16) as u8,
		(num >> 8) as u8,
		(num >> 0) as u8,
	]
}

pub fn from_u32_bytes(arr: &[u8; 4]) -> u32 {
	(arr[0] as u32) << 24
	| (arr[1] as u32) << 16
	| (arr[2] as u32) << 8
	| (arr[3] as u32)
}
