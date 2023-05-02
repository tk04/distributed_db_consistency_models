use redis_store::Protocol::{Get, Set};
use std::io::{self, BufRead};
use std::{io::Write, net::TcpStream};
#[derive(Debug)]
pub struct Conn {
    pub addr: String,
    pub stream: TcpStream,
}
impl Conn {
    pub fn new(addr: &str) -> std::io::Result<Conn> {
        let stream = TcpStream::connect(addr)?;
        return Ok(Self {
            addr: addr.to_string(),
            stream,
        });
    }
    pub fn new_with_socket(addr: &str, socket: TcpStream) -> std::io::Result<Conn> {
        return Ok(Self {
            addr: addr.to_string(),
            stream: socket,
        });
    }
    pub fn send_message(&mut self, msg: String) {
        self.stream.write(msg.as_bytes()).unwrap();
    }
    pub fn read_msg(&self) -> String {
        let mut reader = io::BufReader::new(&self.stream);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        match redis_store::Protocol::parse(buf.clone()) {
            Ok(Get(_, _)) => return buf,
            Ok(Set(_, _, _)) => {
                let mut line2 = String::new();
                reader.read_line(&mut line2).unwrap();
                buf += line2.as_str();
                return buf;
            }
            _ => (),
        }
        return buf;
    }
}
