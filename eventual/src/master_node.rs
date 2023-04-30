use nodes::Master;
use redis_store::Protocol;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use Protocol::{Ack, Get, Set};
pub struct MasterNode {
    master: Master,
}
impl MasterNode {
    pub fn new(db_host: &str, master_host: &str, producer_addr: &str, num_nodes: u32) -> Self {
        let master = Master::new(db_host, master_host, producer_addr, num_nodes);
        return Self { master };
    }

    pub fn listen(&self) {
        thread::scope(|t| {
            t.spawn(|| self.master.receive());
            t.spawn(|| {
                let mut store = self.master.store.lock().unwrap();
                loop {
                    let mut q = self.master.msg_q.lock().unwrap();
                    if q.len() > 0 {
                        if q.get(0).unwrap().0 != Ack {
                            let msg = q.pop_front().unwrap();
                            drop(q);
                            let conn = self.master.connections.lock().unwrap();
                            let mut socket = conn.get(&msg.1).unwrap().lock().unwrap();
                            match msg.0 {
                                Set(k, v, _) => {
                                    println!("SET VALUE {k} to {v}");
                                    let p = Protocol::Set(k, v, 0);
                                    store.set(p.clone());
                                    self.master.publish(&p.to_string());
                                    socket.send_message(p.to_string());
                                }
                                _ => (),
                            }
                            drop(socket);
                            drop(conn);
                            msg.2.send(()).unwrap();
                            println!("MESSAGE PROCESSED SUCCESFFULY, SEND CONRTOL BACK");
                            //handle msg
                        }
                    } else {
                        drop(q);
                        thread::sleep(Duration::from_millis(200));
                    }
                }
            });
        });
    }
}
