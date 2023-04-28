mod master;
// mod net;
mod replica;
use master::Master;
use replica::Replica;
use std::thread;
fn main() {
    let pub_addr = "tcp://localhost:5555";
    let master_addr = "tcp://localhost:5556";
    let mut master = Master::new("localhost:6000", "tcp://*:5556", "tcp://*:5555");
    let mut replica1 = Replica::new("localhost:6553", "localhost:6001", master_addr);

    thread::spawn(move || replica1.listen(pub_addr));
    master.listen();
    println!("Press enter to wake up the child thread");
}
