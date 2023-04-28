use com;
use redis_store::Protocol;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
pub struct Master {
    store: redis_store::Store,
    com: com::Server,
    producer: com::Publisher,
    num_replicas: u16,
    msg_q: Arc<Mutex<Vec<Protocol>>>,
}
impl Master {
    pub fn new(db_host: &str, master_host: &str, producer_addr: &str, num_replicas: u16) -> Self {
        let server = com::init_server(master_host);
        let store = redis_store::init(db_host);
        let producer = com::init_pub(producer_addr);
        return Self {
            store,
            com: server,
            producer,
            num_replicas,
            msg_q: Arc::new(Mutex::new(Vec::new())),
        };
    }
    fn recieve(&self) -> Protocol {
        let val = self.com.receive();
        return Protocol::parse(val).unwrap();
    }

    fn send(&self, msg: &str) {
        self.com.send(msg);
    }
    pub fn publish(&self, msg: &str) {
        self.producer.publish(msg);
    }

    pub fn listen(&mut self) {
        loop {
            let msg = self.recieve();
            match &msg {
                Protocol::Get(key) => {
                    println!("recieved get {key}.. sending back");
                    thread::sleep(Duration::from_secs(2));
                    self.send(&msg.to_string());
                }
                Protocol::Set(_, _) => {
                    self.store.set(msg.clone());
                    self.send("recieved value");
                    self.publish(&msg.to_string());
                }
                _ => {}
            }
        }
    }
}
