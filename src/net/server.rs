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
			match self.handle_request(&mut stream) {
				Err(e) => println!("Request error: {}", e),
				_ => {},
			}
		}
	}

	fn handle_request<'s>(&mut self, stream: &'s mut TcpStream) -> Result<(), String> {
		let mut buffer = [0; 6]; // parse request header: 1 byte (MSG_ type) + 4 bytes (u32 room code)
		stream.read(&mut buffer).map_err(|_e| "Could not read request stream.")?;

		let addr = stream.peer_addr().map_err(|_e| "Could not read request address.")?.to_string();
		let is_host = if buffer[1] == 0 { false } else if buffer[1] == 1 { true } else {
			return Err(String::from("Invalid request: is_host not valid"));
		};
		let code = from_u32_bytes(
			buffer[2..6].try_into().map_err(|_e| "Could not convert room code integer")?
		);

		if (code <= 0 && !is_host) || code > 9999 {
			return Err(String::from("Invalid request: code not valid"));
		}

		if buffer[0] == MSG_CREATE {
			// creating a room
			let mut code_new: u32;
			loop {
				code_new = self.rand.gen_range(1..10000);

				// TODO: overwrite room entry if older than 24h

				// once an unused room code is found, create the room
				if !self.rooms.contains_key(&code_new) {
					println!("{} is creating a room with code {:?}", addr, code_new);
					let room = Room::new(code_new);
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
				if is_host {
					return Err(String::from("Cannot join a room as the host"));
				}

				println!("{} is joining room {:?}", addr, code);
				room.try_join()?;

				// respond with joined room code to indicate success
				stream.write(&buffer[2..]).map_err(|_e| "Could not write join response to stream")?;
			} else if buffer[0] == MSG_EVENT {
				// sending an event
				let mut event_buffer = [0; 18];
				stream.read(&mut event_buffer).map_err(|_e| "Could not read event stream.")?;

				// push event into room
				let event = Event::from_bytes(&event_buffer);
				room.push_event(is_host, event)?;

				// respond with 1 byte to indicate success
				stream.write(&[1]).map_err(|_e| "Could not write event response to stream")?;
			} else if buffer[0] == MSG_POLL {
				// polling for events
				let event = room.pop_event(is_host)?;
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
	peer_joined: bool,
	host_events: Vec<Event>,
	peer_events: Vec<Event>,
}

impl Room {

	fn new(code: u32) -> Room {
		Room {
			code,
			peer_joined: false,
			host_events: Vec::new(),
			peer_events: vec![Event::new(EVENT_JOIN)], // initial join event for host -> peer
		}
	}

	fn try_join(&mut self) -> Result<(), String> {
		if !self.peer_joined {
			self.peer_joined = true;
			self.push_event(false, Event::new(EVENT_JOIN))?; // new join event for peer -> host
			Ok(())
		} else {
			Err(String::from("Room already full"))
		}
	}

	fn push_event(&mut self, is_host: bool, event: Event) -> Result<(), String> {
		if is_host {
			// as host, push an event to the peer
			self.peer_events.push(event);
		} else if !is_host && self.peer_joined {
			// as peer, push an event to the host
			self.host_events.push(event);
		} else {
			return Err(String::from("Cannot push_event: Peer has not joined the room"));
		}

		Ok(())
	}

	fn pop_event(&mut self, is_host: bool) -> Result<Event, String> {
		if is_host {
			Ok(self.host_events.pop().unwrap_or(Event::new(EVENT_NONE)))
		} else if !is_host && self.peer_joined {
			Ok(self.peer_events.pop().unwrap_or(Event::new(EVENT_NONE)))
		} else {
			Err(String::from("Cannot pop_event: Peer has not joined the room"))
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
