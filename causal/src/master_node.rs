use nodes::Master;
use redis_store::Protocol;
use std::{thread, time::Duration};
use Protocol::{Ack, Set};
pub struct MasterNode {
    master: Master,
    ts: usize,
}
impl MasterNode {
    pub fn new(db_host: &str, master_host: &str, producer_addr: &str, num_nodes: u32) -> Self {
        let master = Master::new(db_host, master_host, producer_addr, num_nodes);
        return Self { master, ts: 0 };
    }

    pub fn listen(&mut self) {
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
                            let mut num_waits = 0;
                            match msg.0 {
                                Set(k, v, _) => {
                                    println!("SET VALUE {k} to {v}");
                                    self.ts += 1;
                                    let p = Protocol::Set(k, v, self.ts);
                                    store.set(p.clone());
                                    self.master.publish(&p.to_string());
                                    socket.send_message(p.to_string());
                                    num_waits = self.master.num_nodes;
                                }
                                _ => (),
                            }
                            drop(socket);
                            drop(conn);
                            msg.2.send(()).unwrap();
                            if num_waits > 0 {
                                println!("waiting for acks");
                                self.master.wait_for_acks(num_waits);
                            }

                            println!("MESSAGE PROCESSED SUCCESFFULY, SEND CONRTOL BACK");
                            //handle msg
                        }
                    } else {
                        drop(q);
                        thread::sleep(Duration::from_millis(500));
                    }
                }
            });
        });
    }
}
