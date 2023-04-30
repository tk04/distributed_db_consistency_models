// use crate::net;
use com;
use com::Sub;
use net;
use redis_store::Protocol::{Get, Set};
use redis_store::{self, Protocol};
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
pub struct Replica {
    store: redis_store::Store,
    com: net::Conn,
    endpoint: String,
}
impl Replica {
    pub fn new(endpoint: &str, db_host: &str, master_host: &str) -> Self {
        let client = net::Conn::new(master_host).unwrap();
        let store = redis_store::init(db_host);
        return Self {
            store,
            com: client,
            endpoint: endpoint.to_string(),
        };
    }
    pub fn send_master(&mut self, msg: &str) {
        self.com.send_message(msg.to_string());
    }
    fn get(&mut self, key: String) -> String {
        match self.store.get(key) {
            Ok(value) => value,
            _ => "INVALID_KEY\n".to_string(),
        }
    }
    fn recieve(&self) -> String {
        // receive ack from master
        return self.com.read_msg();
    }

    pub fn listen(&mut self, pub_addr: &str) {
        let endpoint = self.endpoint.clone();
        let shared_val = Arc::new(Mutex::new(self));
        let cpy_val = shared_val.clone();

        let sub = Sub::new(pub_addr);
        let (tx, rx) = mpsc::channel();
        thread::scope(|t| {
            t.spawn(move || loop {
                let msg = sub.recv().1;
                let mut val = shared_val.lock().unwrap();
                let parsed_val = Protocol::parse(msg).unwrap();
                val.store.set(parsed_val);
                // thread::sleep(Duration::from_secs(2));
                println!("Replica: SEND ACK TO MASTER");
                val.send_master(&Protocol::Ack.to_string());
                tx.send(()).unwrap();
            });
            t.spawn(move || {
                let listener = TcpListener::bind(endpoint).unwrap();

                loop {
                    match listener.accept().unwrap() {
                        (socket, addr) => {
                            let new_val = cpy_val.clone();
                            let mut conn =
                                net::Conn::new_with_socket(&addr.to_string(), socket).unwrap();

                            loop {
                                let mut val = new_val.lock().unwrap();
                                let read_msg = conn.read_msg();

                                match Protocol::parse(read_msg.clone()) {
                                    Ok(Get(value)) => {
                                        println!("GET {value}");
                                        println!("SENDING MASTER");
                                        val.send_master(&read_msg);
                                        val.recieve();
                                        println!("GOT FROM MASTER");

                                        conn.send_message(val.get(value));
                                        val.send_master(&Protocol::Ack.to_string());
                                    }
                                    Ok(Set(_, _)) => {
                                        val.send_master(&read_msg);
                                        val.recieve();
                                        drop(val);
                                        rx.recv().unwrap();
                                        conn.send_message("OK".to_string());
                                    }
                                    _ => {
                                        if read_msg == "" {
                                            //disconnected client
                                            break;
                                        } else {
                                            conn.send_message("INVALID_REQUEST\n".to_string());
                                        }
                                    }
                                };
                            }
                        }
                    }
                }
            });
        });
    }
}
