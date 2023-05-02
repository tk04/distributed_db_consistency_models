mod client;
mod publisher;
mod server;
mod sub;

pub use client::Client;
pub use publisher::Publisher;
pub use server::Server;
pub use sub::Sub;
pub fn init_server(addr: &str) -> Server {
    Server::new(addr)
}
pub fn init_client(master_addr: &str) -> Client {
    Client::new(master_addr)
}

pub fn init_pub(addr: &str) -> Publisher {
    Publisher::new(addr)
}
pub fn init_sub(pub_addr: &str) -> Sub {
    Sub::new(pub_addr)
}
