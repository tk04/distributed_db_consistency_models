use net;
use redis_store::Protocol;
use std::collections::{HashMap, VecDeque};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use Protocol::{Ack, Get, Set};
pub struct Server {
    store: Arc<Mutex<redis_store::Store>>,
    socket: TcpListener,
    msg_q: Arc<Mutex<VecDeque<(Protocol, String, Sender<()>)>>>,
    num_nodes: u32,
    connections: Arc<Mutex<HashMap<String, Arc<Mutex<net::Conn>>>>>,
    producer: Arc<Mutex<com::Publisher>>,
}

// protocol format
impl Server {
    pub fn new(db_host: &str, addr: &str, producer_addr: &str, num_nodes: u32) -> Self {
        let socket = TcpListener::bind(addr).unwrap();
        let producer = Arc::new(Mutex::new(com::init_pub(producer_addr)));
        let store = Arc::new(Mutex::new(redis_store::init(db_host)));
        return Self {
            store,
            socket,
            msg_q: Arc::new(Mutex::new(VecDeque::new())),
            num_nodes,
            connections: Arc::new(Mutex::new(HashMap::new())),
            producer,
        };
    }

    pub fn publish(&self, msg: &str) {
        let p = self.producer.lock().unwrap();
        p.publish(msg);
    }
    pub fn receive(&self) {
        loop {
            match self.socket.accept().unwrap() {
                (socket, addr) => {
                    let msg_q = Arc::clone(&self.msg_q);
                    let conn = Arc::new(Mutex::new(
                        net::Conn::new_with_socket(&addr.to_string(), socket.try_clone().unwrap())
                            .unwrap(),
                    ));
                    let mut connections = self.connections.lock().unwrap();
                    connections.insert(addr.to_string(), Arc::clone(&conn));
                    drop(connections);

                    thread::spawn(move || {
                        handle_msg(msg_q, Arc::clone(&conn));
                    });
                }
            }
        }
    }
    pub fn listen(&self) {
        let mut store = self.store.lock().unwrap();
        loop {
            let mut q = self.msg_q.lock().unwrap();
            if q.len() > 0 {
                if q.get(0).unwrap().0 != Ack {
                    let msg = q.pop_front().unwrap();
                    drop(q);
                    let conn = self.connections.lock().unwrap();
                    let mut socket = conn.get(&msg.1).unwrap().lock().unwrap();
                    let mut num_waits = 0;
                    match msg.0 {
                        Get(key) => {
                            println!("GOT GET {key}");
                            socket.send_message(Protocol::Get(key).to_string());
                            // wait for acks
                            num_waits = 1;
                        }
                        Set(k, v) => {
                            println!("SET VALUE {k} to {v}");
                            let p = Protocol::Set(k, v);
                            store.set(p.clone());
                            self.publish(&p.to_string());
                            socket.send_message(p.to_string());
                            num_waits = self.num_nodes;
                        }
                        _ => (),
                    }
                    drop(socket);
                    drop(conn);
                    msg.2.send(()).unwrap();
                    if num_waits > 0 {
                        println!("waiting for acks");
                        self.wait_for_acks(num_waits);
                    }

                    println!("MESSAGE PROCESSED SUCCESFFULY, SEND CONRTOL BACK");

                    //handle msg
                }
            } else {
                drop(q);
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
    fn wait_for_acks(&self, num_acks: u32) {
        let mut curr_acks = 0;
        while curr_acks < num_acks {
            let mut q = self.msg_q.lock().unwrap();
            if q.len() > 0 {
                for i in 0..q.len() {
                    let item = q.get(i).unwrap();
                    if item.0 == Protocol::Ack {
                        curr_acks += 1;
                        item.2.send(()).unwrap();
                        drop(item);
                        q.remove(i);
                        break;
                    };
                }
                // for (i, item) in q.clone().iter().enumerate() {
                //     if item.0 == Protocol::Ack {
                //         curr_acks += 1;
                //         q.remove(i);
                //         item.2.send(()).unwrap();
                //     };
                // }
                drop(q);
            } else {
                drop(q);
            }
            thread::sleep(Duration::from_millis(200));
        }
        println!("finished acks");
    }
}

pub fn handle_msg(
    msg_q: Arc<Mutex<VecDeque<(Protocol, String, Sender<()>)>>>,
    conn: Arc<Mutex<net::Conn>>,
) {
    loop {
        let connection = conn.lock().unwrap();
        let msg = connection.read_msg();
        if msg == "" {
            // disconnect
            drop(connection);
            break;
        }
        let mut q = msg_q.lock().unwrap();

        let (sender, receiver) = channel::<()>();
        match Protocol::parse(msg) {
            Ok(val) => {
                q.push_back((val, connection.addr.clone(), sender));
            }
            Err(err) => {
                println!("error parsing: {err}")
            }
        }
        println!("{:?}", q);
        drop(connection);
        drop(q);
        println!("WAITING FOR MSG TO BE PROCESSED");
        receiver.recv();
        // dont read from this connection again until everything has been processed
    }
}
