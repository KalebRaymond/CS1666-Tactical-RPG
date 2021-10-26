use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

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