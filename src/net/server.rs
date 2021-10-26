use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;
use std::convert::TryInto;

use rand::Rng;
use rand::prelude::ThreadRng;

use crate::net::util;

struct Server {
	addr: String,
	rand: ThreadRng,

	// rooms: code -> (host, participant)
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
		let mut buffer = [0; 5];
		stream.read(&mut buffer).map_err(|_e| "Could not read request stream.")?;

		let addr = stream.peer_addr().map_err(|_e| "Could not read request address.")?.to_string();
		let code = util::from_u32_bytes(
			buffer[1..5].try_into().map_err(|_e| "Could not convert room code integer")?
		);

		if code > 9999 {
			return Err(String::from("Invalid request"));
		}

		if buffer[0] == 0 {
			// creating a room
			let mut code_new: u32;
			loop {
				code_new = self.rand.gen_range(0..10000);

				// TODO: overwrite room entry if older than 24h

				// once an unused room code is found, create the room
				if !self.rooms.contains_key(&code_new) {
					let room = Room::new(code_new, &addr);
					self.rooms.insert(code_new, room);
					break;
				}
			}

			let send_buffer = util::to_u32_bytes(code_new);
			stream.write(&send_buffer).map_err(|_e| "Could not write code response to stream")?;
		} else {
			println!("Locating room {:?}", code);
			let room = self.rooms.get_mut(&code).ok_or("Could not find a matching room")?;

			if buffer[0] == 1 {
				// joining a room

				println!("Joining room");
				room.try_join(&addr)?;
				stream.write(&[1]).map_err(|_e| "Could not write join response to stream")?;
			} else if buffer[0] == 2 {
				// sending an event

				stream.write(&[1]).map_err(|_e| "Could not write event response to stream")?;
			} else if buffer[0] == 3 {
				// polling for events
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
	// TODO: Vec<Event>,
}

impl Room {

	fn new(code: u32, addr: &str) -> Room {
		Room {
			code,
			host_addr: String::from(addr),
			peer_addr: None,
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

}

pub fn run(addr: &str) {
	let mut server = Server::new(addr);

	println!("Listening at {}", addr);
	server.listen();
}
