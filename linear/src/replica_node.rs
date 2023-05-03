use com;
use com::Sub;
use net;
use redis_store::Protocol::{Ack, Get, Set};
use redis_store::{self, Protocol};
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ReplicaNode {
    pub store: redis_store::Store,
    pub com: net::Conn,
    pub endpoint: String,
}
impl ReplicaNode {
    pub fn new(endpoint: &str, db_host: &str, master_host: &str) -> Self {
        let client = net::Conn::new(master_host).unwrap();
        let store = redis_store::init(db_host);
        return Self {
            store,
            com: client,
            endpoint: endpoint.to_string(),
        };
    }
    pub fn listen(&mut self, pub_addr: &str) {
        let endpoint = self.endpoint.clone();
        let store_val = Arc::new(Mutex::new(&mut self.store));
        let com_val = Arc::new(Mutex::new(&mut self.com));
        let store_cpy = store_val.clone();
        let com_cpy = com_val.clone();

        let sub = Sub::new(pub_addr);

        let (tx, rx) = mpsc::channel();
        thread::scope(|t| {
            t.spawn(move || loop {
                let msg = sub.recv().1;
                let mut val = store_val.lock().unwrap();
                let parsed_val = Protocol::parse(msg).unwrap();
                let mut com = com_cpy.lock().unwrap();
                val.set(parsed_val);
                com.send_message(Ack.to_string());
                tx.send(()).unwrap();
            });
            t.spawn(move || {
                let listener = TcpListener::bind(endpoint).unwrap();

                loop {
                    match listener.accept().unwrap() {
                        (socket, addr) => {
                            let new_val = store_cpy.clone();
                            let mut conn =
                                net::Conn::new_with_socket(&addr.to_string(), socket).unwrap();

                            loop {
                                let read_msg = conn.read_msg();

                                match Protocol::parse(read_msg.clone()) {
                                    Ok(Get(value, _)) => {
                                        let mut val = com_val.lock().unwrap();
                                        drop(val);
                                        thread::sleep(Duration::from_millis(500));
                                        val = com_val.lock().unwrap();
                                        val.send_message(read_msg);
                                        val.read_msg();

                                        let mut store = new_val.lock().unwrap();
                                        let key_val = match store.get(value) {
                                            Ok(value) => value,
                                            _ => "INVALID_KEY\n".to_string(),
                                        };
                                        conn.send_message(key_val);
                                        val.send_message(Ack.to_string());
                                    }
                                    Ok(Set(_, _, _)) => {
                                        let mut val = com_val.lock().unwrap();
                                        val.send_message(read_msg);
                                        val.read_msg();
                                        drop(val);
                                        rx.recv().unwrap();
                                        conn.send_message("OK".to_string());
                                    }
                                    Ok(Protocol::FlushAll) => {
                                        let mut val = new_val.lock().unwrap();
                                        val.flush_db();
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
