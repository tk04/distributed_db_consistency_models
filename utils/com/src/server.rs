use zmq;
pub struct Server {
    socket: zmq::Socket,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        let context = zmq::Context::new();
        let responder = context.socket(zmq::REP).unwrap();
        assert!(responder.bind(addr).is_ok());
        return Self { socket: responder };
    }
    pub fn receive(&self) -> String {
        let mut msg = zmq::Message::new();
        self.socket.recv(&mut msg, 0).unwrap();
        let message = msg.as_str().unwrap();
        return message.to_string();
    }
    pub fn send(&self, msg: &str) {
        self.socket.send(msg, 0).unwrap();
    }
}
