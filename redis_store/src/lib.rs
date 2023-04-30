mod protocol;
mod store;
pub use protocol::Protocol;
pub use store::Store;

pub fn init(host: &str) -> store::Store {
    let client = redis::Client::open(format!("redis://{}/", host)).unwrap();
    let con = client.get_connection().unwrap();
    return store::Store { store: con };
}
