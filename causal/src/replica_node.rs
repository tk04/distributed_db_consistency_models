use nodes::Replica;

use com;
use com::Sub;
use net;
use redis_store::Protocol::{Ack, Get, Set};
use redis_store::{self, Protocol};
use std::cell::RefCell;
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ReplicaNode {
    replica: Replica,
    ts: usize,
}
impl ReplicaNode {
    pub fn new(endpoint: &str, db_host: &str, master_host: &str) -> Self {
        let replica = Replica::new(endpoint, db_host, master_host);
        return Self { replica, ts: 0 };
    }

    pub fn listen(&mut self, pub_addr: &str) {
        let endpoint = self.replica.endpoint.clone();
        let mut shared_val = Arc::new(Mutex::new(self));
        let cpy_val = shared_val.clone();

        let sub = Sub::new(pub_addr);
        let (tx, rx) = mpsc::channel();
        thread::scope(|t| {
            t.spawn(move || loop {
                let msg = sub.recv().1;
                let mut val = shared_val.lock().unwrap();
                let parsed_val = Protocol::parse(msg).unwrap();
                val.ts = parsed_val.get_ts();
                val.replica.store.set(parsed_val);
                // thread::sleep(Duration::from_secs(2));
                println!("Replica: SEND ACK TO MASTER");
                val.replica.send_master(&Ack.to_string());
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
                                let mut ts = val.ts;
                                let read_msg = conn.read_msg();

                                match Protocol::parse(read_msg.clone()) {
                                    Ok(Get(value, ts)) => {
                                        while ts > val.ts {
                                            drop(val);
                                            rx.recv().unwrap();
                                            val = new_val.lock().unwrap();
                                        }
                                        conn.send_message(val.replica.get(value));
                                    }
                                    Ok(Set(k, v, _)) => {
                                        ts += 1;
                                        let setCmd = Protocol::Set(k, v, ts);
                                        val.replica.send_master(&read_msg);
                                        val.replica.recieve();
                                        drop(val);
                                        rx.recv().unwrap();
                                        val = new_val.lock().unwrap();
                                        conn.send_message(format!("OK {}", val.ts));
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
