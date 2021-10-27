use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use crate::net::SERVER_ADDR;
use crate::net::util::*;

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
	code: u32,
	addr: String
}

impl Client {
	pub fn new() -> Result<Client, String> {
		let addr = unsafe {
			String::from(SERVER_ADDR)
		};

		let code = match unsafe { CODE } {
			Some(c) => c,
			_ => 0,
		};

		// construct the client and either create/join the room
		let mut client = Client { code, addr };
		let mut stream = client.connect(if code == 0 { MSG_CREATE } else { MSG_JOIN })?;

		let mut buffer = [0; 4];
		stream.read(&mut buffer).map_err(|_e| "Could not read connection response")?;

		// check if the returned room code matches the intended join code in send_bytes (i.e. whether the room was actually joined)
		let new_code = from_u32_bytes(&buffer);
		if code != 0 && code != new_code {
			return Err(String::from("Invalid room code returned"))
		} else {
			println!("Entered a room with code {:?}", new_code);
			client.code = new_code;
		}

		// successfully joined a room & constructed a client
		Ok(client)
	}

	// creates a new TcpStream connection & sends/validates the request header
	fn connect(&self, action: u8) -> Result<TcpStream, String> {
		let mut stream = TcpStream::connect(&self.addr).map_err(|_e| "Could not initialize TCP stream")?;

		let code_bytes = to_u32_bytes(self.code);
		let send_bytes = [action, code_bytes[0], code_bytes[1], code_bytes[2], code_bytes[3]]; // there has better way to do this... (I couldn't find any docs on a spread operator at all)

		stream.write(&send_bytes).map_err(|_e| "Could not send connection info")?;

		Ok(stream)
	}

	// TODO: pub fn poll(&self) -> Vec<Event>

	// TODO: pub fn send(&self, action: Event)
}