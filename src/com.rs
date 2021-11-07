
use std::net::{TcpStream};
use std::io::{Read, Write};

use bincode::{serialize, deserialize};
use serde::ser::Serialize;
use serde::de::{DeserializeOwned};

pub struct TcpWrapper {
    pub stream: TcpStream,
}

impl TcpWrapper {
    pub fn send(&mut self, data: &impl Serialize) {
        let data = serialize(data).expect("Cannot serialize data");
        self.stream.write(&serialize(&data.len()).expect("Cannot serialize len")).expect("Cannot write len");
        self.stream.write(&data).expect("Cannot write data");
    }
    pub fn recv<'de, T: DeserializeOwned>(&mut self) -> T {
        let mut buf = [0u8; 8];
        self.stream.read_exact(&mut buf).expect("Cannot read len");
        let len: usize = deserialize(&buf).expect("cannot deserialize length");
        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).expect("Cannot read data");
        deserialize(&buf).expect("Cannot deserialize data")
    }
    pub fn ask<'de, T: DeserializeOwned>(&mut self, data: &impl Serialize) -> T {
        self.send(data);
        self.recv()
    }
}
