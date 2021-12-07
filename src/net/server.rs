use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, IpAddr};
use std::collections::HashMap;
use std::time::{Instant, Duration};

use rand::Rng;
use rand::prelude::*;

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
				Err(e) => {
					println!("Incoming stream error: {}", e.to_string());
					None
				}
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
		stream.set_read_timeout(Some(Duration::from_secs(1))).map_err(|_e| "Could set read timeout")?;
		stream.set_write_timeout(Some(Duration::from_secs(1))).map_err(|_e| "Could set write timeout")?;

		let mut buffer = [0; 13]; // parse request header: 1 byte (MSG_ type) + 4 bytes (u32 user token) + 4 bytes (u32 room code) + 4 bytes (u32 room token)
		stream.read(&mut buffer).map_err(|_e| "Could not read request stream.")?;

		// parse variables from request header: [addr, user_token, code, token]
		let addr = stream.peer_addr().map_err(|_e| "Could not read request address.")?.ip();
		let user_token = from_u32_bytes(&buffer[1..5]);
		let code = from_u32_bytes(&buffer[5..9]);
		let token = from_u32_bytes(&buffer[9..13]);

		// ensure that room code is within the expected range
		if code <= 0 || code > 9999 || (code == 0 && buffer[0] != MSG_CREATE) {
			return Err(String::from("Invalid request: code not valid"));
		}

		if buffer[0] == MSG_CREATE {
			// creating a room
			let mut code: u32;
			let token: u32;
			let host_token: u32;
			loop {
				new_code = self.rand.gen_range(1..10000);

				// overwrite room entry if older than 24h
				if let Some(room) = self.rooms.get(&code) {
					if Instant::now().duration_since(room.last_poll).as_secs() > 60*60*24 {
						self.rooms.remove(&code);
					} else {
						continue;
					}
				}

				// once an unused room code is found, create the room
				println!("{} is creating a room with code {:?}", addr.to_string(), code);
				let room = Room::new(addr);
				token = room.token;
				host_token = room.host_token;
				self.rooms.insert(code, room);
				break;
			}

			// respond with new code + token of created room
			let mut send_buffer = [0; 12];
			set_range!(send_buffer[0..4] = to_u32_bytes(host_token))
			set_range!(send_buffer[4..8] = to_u32_bytes(code));
			set_range!(send_buffer[8..12] = to_u32_bytes(token));
			stream.write_all(&send_buffer).map_err(|_e| "Could not write code response to stream")?;
			stream.flush().map_err(|_e| "Could not flush stream")?;
			return Ok(());
		}

		// not creating a room: get existing room from HashMap
		let room = self.rooms.get_mut(&code).ok_or("Could not find a matching room")?;

		if buffer[0] == MSG_JOIN {
			// joining a room
			println!("{} is joining room {:?}", addr.to_string(), code);
			room.try_join(addr)?;

			// respond with joined room code + token to indicate success
			let mut send_buffer = [0; 12];
			set_range!(send_buffer[0..4] = to_u32_bytes(room.peer_token));
			set_range!(send_buffer[4..8] = to_u32_bytes(code));
			set_range!(send_buffer[8..12] = to_u32_bytes(room.token));
			stream.write_all(&send_buffer).map_err(|_e| "Could not write join response to stream")?;
			stream.flush().map_err(|_e| "Could not flush stream")?;
			return Ok(());
		}

		// performing an operation on an already-joined room: ensure that token is valid
		if token != room.token {
			return Err(String::from("Invalid request: incorrect token"));
		}

		if user_token != room.host_token && user_token != room.peer_token {
			return Err(String::from("Invalid request: invalid user token"));
		}
		let is_host = user_token == room.host_token;

		if buffer[0] == MSG_EVENT {
			// sending an event
			let mut event_buffer = [0; 19];
			stream.read(&mut event_buffer).map_err(|_e| "Could not read event stream.")?;

			// push event into room
			let event = Event::from_bytes(&event_buffer);
			room.push_event(is_host, addr, event)?;

			// respond with 1 byte to indicate success
			stream.write_all(&[1]).map_err(|_e| "Could not write event response to stream")?;
		} else if buffer[0] == MSG_POLL {
			// polling for events
			let event = room.pop_event(is_host, addr)?;
			let event_buffer = event.to_bytes();

			// respond with event contents
			stream.write_all(&event_buffer).map_err(|_e| "Could not write poll response to stream")?;
		}

		stream.flush().map_err(|_e| "Could not flush stream")?;
		Ok(())
	}
}

struct Room {
	token: u32,
	host_addr: IpAddr,
	peer_addr: Option<IpAddr>,
	host_token: u32,
	peer_token: u32,
	host_events: Vec<Event>,
	peer_events: Vec<Event>,
	last_poll: Instant,
}

impl Room {

	fn new(addr: IpAddr) -> Room {
		Room {
			token: random(),
			host_addr: addr,
			peer_addr: None,
			host_token: random(),
			peer_token: random(),
			host_events: Vec::new(),
			peer_events: vec![Event::new(EVENT_JOIN)], // initial join event for host -> peer
			last_poll: Instant::now(),
		}
	}

	fn try_join(&mut self, addr: IpAddr) -> Result<(), String> {
		if self.peer_addr == None {
			self.peer_addr = Some(addr);
			self.push_event(false, addr, Event::new(EVENT_JOIN))?; // new join event for peer -> host
			Ok(())
		} else {
			Err(String::from("Room already full"))
		}
	}

	fn push_event(&mut self, is_host: bool, addr: IpAddr, event: Event) -> Result<(), String> {
		if is_host && self.host_addr == addr {
			// as host, push an event to the peer
			self.peer_events.push(event);
		} else if !is_host && self.peer_addr == Some(addr) {
			// as peer, push an event to the host
			self.host_events.push(event);
		} else {
			return Err(String::from("Cannot push_event: Peer has not joined the room"));
		}

		Ok(())
	}

	fn pop_event(&mut self, is_host: bool, addr: IpAddr) -> Result<Event, String> {
		self.last_poll = Instant::now();

		let events = if is_host && self.host_addr == addr {
			&mut self.host_events
		} else if !is_host && self.peer_addr == Some(addr) {
			&mut self.peer_events
		} else {
			return Err(String::from("Cannot pop_event: Peer has not joined the room"));
		};

		if events.is_empty() {
			return Ok(Event::new(EVENT_NONE))
		}

		Ok(events.remove(0))
	}
}

pub fn run() {
	let addr = unsafe {
		String::from(SERVER_ADDR)
	};

	let port = *(addr.split(":").collect::<Vec<&str>>().last().unwrap());
	let mut server = Server::new(format!("0.0.0.0:{}", port).as_ref());

	println!("Listening at {}", &addr);
	server.listen();
}
