use std::io::prelude::*;
use std::io::{Read, Result, Write};
use std::net::TcpStream;

/// A connection to a GDB server over the GDB Remote Serial Protocol. This is the client.
///
/// See https://sourceware.org/gdb/current/onlinedocs/gdb.html/Remote-Protocol.html
pub struct Client {
    stream: TcpStream,
}

// For an example of a GDB server, see https://github.com/ares-emulator/ares/tree/master/nall/gdb

impl Client {
    pub fn new(address: &str) -> Result<Self> {
        let stream = TcpStream::connect(address)?;
        Ok(Self { stream })
    }

    // https://sourceware.org/gdb/current/onlinedocs/gdb.html/Packets.html
    pub fn write_packet(&mut self, packet: &[u8]) -> Result<()> {
        let checksum = packet
            .iter()
            .fold(0, |acc: u8, &byte| acc.wrapping_add(byte));

        self.stream.write_all(b"$")?;
        self.stream.write_all(packet)?;
        self.stream.write_all(b"#")?;
        self.stream
            .write_all(format!("{:02}", checksum).as_bytes())?;

        Ok(())
    }
}
