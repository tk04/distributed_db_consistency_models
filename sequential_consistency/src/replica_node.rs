use nodes::Replica;

use com;
use com::Sub;
use net;
use redis_store::Protocol::{Ack, Get, Set};
use redis_store::{self, Protocol};
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ReplicaNode {
    replica: Replica,
}
impl ReplicaNode {
    pub fn new(endpoint: &str, db_host: &str, master_host: &str) -> Self {
        let replica = Replica::new(endpoint, db_host, master_host);
        return Self { replica };
    }
    pub fn flush_kv(&mut self) {
        self.replica.store.flush_db();
    }

    pub fn listen(&mut self, pub_addr: &str) {
        let endpoint = self.replica.endpoint.clone();
        let shared_val = Arc::new(Mutex::new(&mut self.replica));
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
                val.send_master(&Ack.to_string());
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
                                let read_msg = conn.read_msg();

                                match Protocol::parse(read_msg.clone()) {
                                    Ok(Get(value, _)) => {
                                        let mut val = new_val.lock().unwrap();
                                        conn.send_message(val.get(value));
                                    }
                                    Ok(Set(_, _, _)) => {
                                        let mut val = new_val.lock().unwrap();
                                        val.send_master(&read_msg);
                                        val.recieve();
                                        drop(val);
                                        rx.recv().unwrap();
                                        conn.send_message("OK".to_string());
                                    }
                                    Ok(Protocol::FlushAll) => {
                                        let mut val = new_val.lock().unwrap();
                                        val.store.flush_db();
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
