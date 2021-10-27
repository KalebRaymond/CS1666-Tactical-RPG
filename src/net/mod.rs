pub static mut SERVER_ADDR: &str = "127.0.0.1:5776";

#[macro_use] pub mod util;
pub mod client;
pub mod server;
