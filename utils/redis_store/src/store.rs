use crate::protocol::Protocol;
use redis::Commands;

pub struct Store {
    pub store: redis::Connection,
}
impl Store {
    pub fn new(host: &str) -> Self {
        let client = redis::Client::open(format!("redis://{}/", host)).unwrap();
        let con = client.get_connection().unwrap();
        return Self { store: con };
    }
    pub fn set(&mut self, value: Protocol) {
        if let Protocol::Set(key, value, _) = value {
            self.store
                .set::<String, String, String>(key, value)
                .unwrap();
        } else {
            panic!("Recieved invalid command");
        }
    }

    pub fn get(&mut self, key: String) -> Result<String, String> {
        match self.store.get(key) {
            Ok(value) => Ok(value),
            Err(_) => Err("Key not Found".to_string()),
        }
    }
    pub fn flush_db(&mut self) {
        let _: () = redis::cmd("FLUSHALL").query(&mut self.store).unwrap();
        println!("------------------finished flushing-------------");
        return;
    }
}
