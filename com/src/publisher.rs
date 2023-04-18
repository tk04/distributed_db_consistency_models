use zmq::{Context, Socket, PUB};
pub struct Publisher {
    socket: Socket,
}

impl Publisher {
    pub fn new(host: &str) -> Self {
        let context = Context::new();
        let publisher = context.socket(PUB).unwrap();
        publisher.bind(host).expect("failed binding publisher");
        return Self { socket: publisher };
    }
    pub fn publish(&self, msg: &str) {
        self.socket
            .send("event", zmq::SNDMORE)
            .expect("failed sending second envelope");
        self.socket
            .send(msg, 0)
            .expect("failed sending second message");
    }
}
