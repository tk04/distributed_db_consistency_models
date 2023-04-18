mod protocol;
mod store;
pub use protocol::Protocol;
pub use store::Store;

pub fn init(host: &str) -> store::Store {
    let client = redis::Client::open(format!("redis://{}/", host)).unwrap();
    let con = client.get_connection().unwrap();
    return store::Store { store: con };
}
pub fn set_cmd(key: &str, value: &str) -> Protocol {
    return Protocol::Set(key.to_string(), value.to_string());
}

pub fn get_cmd(key: &str) -> Protocol {
    return Protocol::Get(key.to_string());
}

