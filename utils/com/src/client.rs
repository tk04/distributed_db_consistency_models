use zmq;
pub struct Client {
    socket: zmq::Socket,
}

impl Client {
    pub fn new(master_addr: &str) -> Self {
        let context = zmq::Context::new();
        let requester = context.socket(zmq::REQ).unwrap();
        assert!(requester.connect(&master_addr).is_ok());
        return Self { socket: requester };
    }
    pub fn send(&self, msg: &str) {
        self.socket.send(msg, 0).unwrap();
    }
    pub fn receive(&self) -> String {
        let mut msg = zmq::Message::new();
        self.socket.recv(&mut msg, 0).unwrap();
        let message = msg.as_str().unwrap();
        return message.to_string();
    }
}
