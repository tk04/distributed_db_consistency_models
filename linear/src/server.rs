use net;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
pub struct Server {
    socket: TcpListener,
    msg_q: Arc<Mutex<Vec<String>>>,
}

// protocol format
impl Server {
    pub fn new(addr: &str) -> Self {
        let socket = TcpListener::bind(addr).unwrap();
        return Self {
            socket,
            msg_q: Arc::new(Mutex::new(Vec::new())),
        };
    }
    pub fn receive(&self) {
        loop {
            match self.socket.accept().unwrap() {
                (socket, addr) => {
                    let msg_q = Arc::clone(&self.msg_q);
                    thread::spawn(move || {
                        handle_msg(
                            msg_q,
                            net::Conn::new_with_socket(&addr.to_string(), socket).unwrap(),
                        );
                    });
                }
            }
        }
    }
    pub fn sort_q(&self, msg_q: Vec<(String, net::Conn)>) {}
}

pub fn handle_msg(msg_q: Arc<Mutex<Vec<String>>>, conn: net::Conn) {
    loop {
        let msg = conn.read_msg();
        if msg == "" {
            break;
        }
        let mut q = msg_q.lock().unwrap();
        q.push(msg);
        println!("{:?}", q);
    }
}
