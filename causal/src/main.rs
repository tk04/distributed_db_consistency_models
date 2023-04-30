mod master_node;
use master_node::MasterNode;
use redis_store::Protocol;
use std::{
    io::{self, BufRead},
    time::Duration,
};
mod replica_node;
use replica_node::ReplicaNode;
use std::{io::Write, net::TcpStream, thread};
fn main() {
    let pub_addr = "tcp://localhost:5555";
    let master_addr = "localhost:5557";
    let mut master = MasterNode::new("localhost:6000", master_addr, "tcp://*:5555", 2);
    let mut replica1 = ReplicaNode::new("localhost:6553", "localhost:6001", master_addr);
    let mut replica2 = ReplicaNode::new("localhost:6554", "localhost:6002", master_addr);
    thread::spawn(move || replica1.listen(pub_addr));
    thread::spawn(move || replica2.listen("tcp://localhost:5555"));
    thread::scope(|t| {
        t.spawn(|| master.listen());
        t.spawn(|| client_test());
    });
}
fn client_test() {
    thread::sleep(Duration::from_millis(200));
    let mut stream = TcpStream::connect("localhost:6553").unwrap();
    println!("-----------SEND TEST------------");
    stream.write("SET KEY\nVALUE\n".as_bytes()).unwrap();
    let response = read_reply(stream);
    stream = response.1;
    println!("received: {}", response.0);
    thread::scope(|t| {
        t.spawn(|| {
            thread::sleep(Duration::from_secs(5));
            let mut stream = TcpStream::connect("localhost:6554").unwrap();
            stream.write("SET KEY 1\nVALUE\n".as_bytes()).unwrap();
            let response = read_reply(stream);
            println!("received: {}", response.0);
        });
        t.spawn(|| {
            println!("-----------GET TEST------------");
            stream.write("GET KEY 2\n".as_bytes()).unwrap();
            let response = read_reply(stream);
            println!("received: {}", response.0);
        });
    });
}
fn read_reply(stream: TcpStream) -> (String, TcpStream) {
    let mut reader = io::BufReader::new(stream);

    let rec: Vec<u8> = reader.fill_buf().unwrap().to_vec();
    reader.consume(rec.len());
    let n_stream = reader.into_inner();
    return (String::from_utf8(rec).unwrap(), n_stream);
}
