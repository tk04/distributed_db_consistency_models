use zmq::{Context, Socket, SUB};
pub struct Sub {
    socket: Socket,
}

impl Sub {
    pub fn new(pub_host: &str) -> Self {
        let context = Context::new();
        let subscriber = context.socket(SUB).unwrap();
        subscriber
            .connect(pub_host)
            .expect("failed connecting subscriber");
        subscriber
            .set_subscribe(b"event")
            .expect("failed subscribing");
        return Self { socket: subscriber };
    }
    pub fn recv(&self) -> (String, String) {
        let event = self
            .socket
            .recv_string(0)
            .expect("failed receiving envelope")
            .unwrap();
        let msg = self
            .socket
            .recv_string(0)
            .expect("failed receiving message")
            .unwrap();
        return (event, msg);
    }
}
