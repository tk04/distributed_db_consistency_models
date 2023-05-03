mod master_node;
mod replica_node;
pub use master_node::MasterNode;
pub use replica_node::ReplicaNode;
#[allow(unused)]
use std::{io::Write, net::TcpStream, thread};
use std::{
    io::{self, BufRead},
    time::Duration,
};

#[allow(dead_code)]
fn start_cluster() {
    static mut CLUSTER_INITIALIZED: bool = false;
    unsafe {
        if !CLUSTER_INITIALIZED {
            let pub_addr = "tcp://localhost:5555";
            let master_addr = "localhost:5557";
            let master = MasterNode::new("localhost:6000", master_addr, "tcp://*:5555", 2);
            let mut replica1 = ReplicaNode::new("localhost:6553", "localhost:6001", master_addr);
            let mut replica2 = ReplicaNode::new("localhost:6554", "localhost:6002", master_addr);
            thread::spawn(move || master.listen());
            thread::spawn(move || replica1.listen(pub_addr));
            thread::spawn(move || replica2.listen("tcp://localhost:5555"));
            CLUSTER_INITIALIZED = true;
            thread::sleep(Duration::from_millis(100));
        }
    }
}
#[allow(dead_code)]
fn read_reply(stream: &mut TcpStream) -> String {
    let mut reader = io::BufReader::new(stream);

    let rec: Vec<u8> = reader.fill_buf().unwrap().to_vec();
    reader.consume(rec.len());
    return String::from_utf8(rec).unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_replica_write_read() {
        start_cluster();
        let mut stream = TcpStream::connect("localhost:6553").unwrap();

        stream.write("FLUSH_ALL\n".as_bytes()).unwrap();
        read_reply(&mut stream);

        stream.write("SET KEY\nVALUE\n".as_bytes()).unwrap();
        let set_response = read_reply(&mut stream);
        assert_eq!(set_response, "OK");
        stream.write("GET KEY\n".as_bytes()).unwrap();
        let get_response = read_reply(&mut stream);
        assert_eq!(get_response, "INVALID_KEY\n");
    }
    #[test]
    fn test_convergence() {
        start_cluster();
        let mut write_replica = TcpStream::connect("localhost:6553").unwrap();
        let mut read_replica = TcpStream::connect("localhost:6554").unwrap();
        //reset db
        read_replica.write("FLUSH_ALL\n".as_bytes()).unwrap();
        write_replica.write("FLUSH_ALL\n".as_bytes()).unwrap();
        read_reply(&mut read_replica);
        read_reply(&mut write_replica);

        write_replica.write("SET KEY1\nVALUE\n".as_bytes()).unwrap();
        let set_response = read_reply(&mut write_replica);
        assert_eq!(set_response, "OK");

        // wait for write to propagate
        thread::sleep(Duration::from_secs(1));
        read_replica.write("GET KEY1\n".as_bytes()).unwrap();
        let get_response = read_reply(&mut read_replica);
        assert_eq!(get_response, "VALUE");

        write_replica.write("GET KEY1\n".as_bytes()).unwrap();
        let set_response2 = read_reply(&mut write_replica);
        assert_eq!(set_response2, "VALUE");
    }
}
