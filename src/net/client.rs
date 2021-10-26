use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

static mut CODE: Option<u32> = None;

// sets a static CODE variable representing the multiplayer room to join
//   if Some(code) -> joins a room with code
//   if None       -> creates a new room
pub fn set_code(code: Option<u32>) {
	unsafe {
		CODE = code;
	}
}

pub struct Client {
	stream: TcpStream,
}

impl Client {
	pub fn new(addr: &str) -> Result<Client, String> {
		let stream = TcpStream::connect(addr).map_err(|e| "Could not initialize TCP stream")?;

		Ok(Client {
			stream,
		})
	}

	// TODO: pub fn poll(&self) -> Vec<Event>

	// TODO: pub fn send(&self, action: Event)
}