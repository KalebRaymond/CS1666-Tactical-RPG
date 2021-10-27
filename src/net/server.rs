use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;
use std::convert::TryInto;

use rand::Rng;
use rand::prelude::ThreadRng;

use crate::net::SERVER_ADDR;
use crate::net::util::*;

struct Server {
	addr: String,
	rand: ThreadRng,

	// rooms: map code -> Room info
	rooms: HashMap<u32, Room>,
}

impl Server {
	fn new(addr: &str) -> Server {
		Server {
			addr: String::from(addr),
			rand: rand::thread_rng(),
			rooms: HashMap::new(),
		}
	}

	fn listen(&mut self) {
		let listener = TcpListener::bind(String::from(&self.addr)).unwrap();
		let incoming = listener.incoming()
			.filter_map(|s| match s {
				Ok(stream) => Some(stream),
				_ => None,
			});

		// listen for any new connections
		for mut stream in incoming {
			println!("Incoming request!");
			match self.handle_request(&mut stream) {
				Err(e) => println!("Request error: {}", e),
				_ => {},
			}
		}
	}

	fn handle_request<'s>(&mut self, stream: &'s mut TcpStream) -> Result<(), String> {
		let mut buffer = [0; 5]; // parse request header: 1 byte (MSG_ type) + 4 bytes (u32 room code)
		stream.read(&mut buffer).map_err(|_e| "Could not read request stream.")?;

		let addr = stream.peer_addr().map_err(|_e| "Could not read request address.")?.to_string();
		let code = from_u32_bytes(
			buffer[1..5].try_into().map_err(|_e| "Could not convert room code integer")?
		);

		if code > 9999 {
			return Err(String::from("Invalid request"));
		}

		if buffer[0] == MSG_CREATE {
			// creating a room
			let mut code_new: u32;
			loop {
				code_new = self.rand.gen_range(0..10000);

				// TODO: overwrite room entry if older than 24h

				// once an unused room code is found, create the room
				if !self.rooms.contains_key(&code_new) {
					println!("{} is creating a room with code {:?}", addr, code_new);
					let room = Room::new(code_new, &addr);
					self.rooms.insert(code_new, room);
					break;
				}
			}

			// respond with new code of created room
			let send_buffer = to_u32_bytes(code_new);
			stream.write(&send_buffer).map_err(|_e| "Could not write code response to stream")?;
		} else {
			let room = self.rooms.get_mut(&code).ok_or("Could not find a matching room")?;

			if buffer[0] == MSG_JOIN {
				// joining a room

				println!("{} is joining room {:?}", addr, code);
				room.try_join(&addr)?;

				// respond with joined room code to indicate success
				stream.write(&buffer[1..]).map_err(|_e| "Could not write join response to stream")?;
			} else if buffer[0] == MSG_EVENT {
				// sending an event
				let mut event_buffer = [0; 18];
				stream.read(&mut event_buffer).map_err(|_e| "Could not read event stream.")?;

				// push event into room
				let event = Event::from_bytes(&event_buffer);
				room.push_event(&addr, event)?;

				// respond with 1 byte to indicate success
				stream.write(&[1]).map_err(|_e| "Could not write event response to stream")?;
			} else if buffer[0] == MSG_POLL {
				// polling for events
				let event = room.pop_event(&addr)?;
				let event_buffer = event.to_bytes();

				// respond with event contents
				stream.write(&event_buffer).map_err(|_e| "Could not write poll response to stream")?;
			}
		}

		stream.flush().map_err(|_e| "Could not flush stream")?;
		Ok(())
	}
}

struct Room {
	code: u32,
	host_addr: String,
	peer_addr: Option<String>,
	host_events: Vec<Event>,
	peer_events: Vec<Event>,
}

impl Room {

	fn new(code: u32, addr: &str) -> Room {
		Room {
			code,
			host_addr: String::from(addr),
			peer_addr: None,
			host_events: Vec::new(),
			peer_events: Vec::new(),
		}
	}

	fn try_join(&mut self, addr: &str) -> Result<(), String> {
		if self.peer_addr == None {
			self.peer_addr = Some(String::from(addr));
			Ok(())
		} else {
			Err(String::from("Room already full"))
		}
	}

	fn push_event(&mut self, addr: &str, event: Event) -> Result<(), String> {
		if addr == self.host_addr {
			self.peer_events.push(event);
		} else if Some(String::from(addr)) == self.peer_addr {
			self.host_events.push(event);
		} else {
			return Err(String::from("Peer has not joined the room"));
		}

		Ok(())
	}

	fn pop_event(&mut self, addr: &str) -> Result<Event, String> {
		if addr == self.host_addr {
			Ok(self.host_events.pop().unwrap_or(Event::new()))
		} else if Some(String::from(addr)) == self.peer_addr {
			Ok(self.peer_events.pop().unwrap_or(Event::new()))
		} else {
			Err(String::from("Err"))
		}
	}

}

pub fn run() {
	let addr = unsafe {
		String::from(SERVER_ADDR)
	};

	let mut server = Server::new(&addr);

	println!("Listening at {}", &addr);
	server.listen();
}