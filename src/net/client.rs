use std::io::prelude::*;
use std::net::{TcpStream};
use std::time::{Instant, Duration};

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
	pub code: u32,
	token: u32,
	pub is_host: bool,
	pub is_joined: bool,
	addr: String,
	last_poll: Instant,
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
		let mut client = Client { code, token: 0, is_host: code == 0, is_joined: false, addr, last_poll: Instant::now() };
		let mut stream = client.connect(if code == 0 { MSG_CREATE } else { MSG_JOIN })?;

		let mut buffer = [0; 8];
		stream.read(&mut buffer).map_err(|_e| "Could not read connection response")?;

		// check if the returned room code matches the intended join code in send_bytes (i.e. whether the room was actually joined)
		let new_code = from_u32_bytes(&buffer[0..4]);
		let new_token = from_u32_bytes(&buffer[4..8]);

		if !client.is_host && code != new_code {
			return Err(String::from("Invalid room code returned"))
		} else {
			println!("Entered a room with code {:?}", new_code);
			client.code = new_code;
			client.token = new_token;
		}

		// successfully joined a room & constructed a client
		Ok(client)
	}

	// creates a new TcpStream connection & sends/validates the request header
	fn connect(&self, action: u8) -> Result<TcpStream, String> {
		let mut stream = TcpStream::connect(&self.addr).map_err(|_e| "Could not initialize TCP stream")?;
		stream.set_read_timeout(Some(Duration::from_secs(1))).map_err(|_e| "Could set read timeout")?;
		stream.set_write_timeout(Some(Duration::from_secs(1))).map_err(|_e| "Could set write timeout")?;

		let mut send_bytes = [0; 10];
		send_bytes[0] = action;
		send_bytes[1] = if self.is_host { 1 } else { 0 };
		set_range!(send_bytes[2..6] = to_u32_bytes(self.code));
		set_range!(send_bytes[6..10] = to_u32_bytes(self.token));

		stream.write_all(&send_bytes).map_err(|_e| "Could not send connection info")?;

		Ok(stream)
	}

	pub fn send(&self, event: Event) -> Result<(), String> {
		let mut stream = self.connect(MSG_EVENT)?;

		let buffer = event.to_bytes();
		stream.write_all(&buffer).map_err(|_e| "Could not write event buffer")?;

		let mut response_buffer = [0; 1];
		stream.read(&mut response_buffer).map_err(|_e| "Could not read event response")?;

		// check that event was pushed successfully
		if response_buffer[0] == 1 {
			Ok(())
		} else {
			Err(String::from("Event did not receive a successful response from the server"))
		}
	}

	pub fn poll(&mut self) -> Result<Option<Event>, String> {
		// don't send polls if <1s since last (empty) call
		if Instant::now().duration_since(self.last_poll).as_secs() < 1 {
			return Ok(None);
		}

		let mut stream = self.connect(MSG_POLL)?;

		let mut buffer = [0; 19];
		stream.read(&mut buffer).map_err(|_e| "Could not read poll response")?;

		let ret = match Event::from_bytes(&buffer) {
			Event{action: EVENT_NONE, ..} => Ok(None),
			Event{action: EVENT_JOIN, ..} => {
				self.is_joined = true;
				Ok(None)
			},
			e => Ok(Some(e))
		};

		// only update last_poll once an empty poll is received
		if let Ok(Some(e)) = ret {
			if e.action == EVENT_NONE {
				self.last_poll = Instant::now();
			}
		}

		ret
	}
}