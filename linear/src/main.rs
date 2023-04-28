mod master;
mod server;
// mod net;
mod replica;
use master::Master;
use replica::Replica;
use server::Server;
use std::thread;
fn main() {
    let pub_addr = "tcp://localhost:5555";
    // let master_addr = "tcp://localhost:5556";
    // let mut master = Master::new("localhost:6000", "tcp://*:5556", "tcp://*:5555", 2);
    // let mut replica1 = Replica::new("localhost:6553", "localhost:6001", master_addr);
    // let mut replica2 = Replica::new("localhost:6554", "localhost:6002", master_addr);
    //
    // thread::spawn(move || replica1.listen(pub_addr));
    // thread::spawn(move || replica2.listen("tcp://localhost:5556"));
    // master.listen();
    // println!("Press enter to wake up the child thread");
    let s1 = Server::new("localhost:5555");
    s1.receive();
}
