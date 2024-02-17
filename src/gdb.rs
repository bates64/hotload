use paris::error;
use std::io::{Read, Write};
use std::net::TcpStream;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// A connection to a GDB server over the GDB Remote Serial Protocol. This is the client.
///
/// See https://sourceware.org/gdb/current/onlinedocs/gdb.html/Remote-Protocol.html
pub struct Client {
    stream: TcpStream,
}

// For an example of a GDB server, see https://github.com/ares-emulator/ares/tree/master/nall/gdb

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

impl Client {
    pub fn new(address: &str) -> Result<Self> {
        let stream = TcpStream::connect(address)?;

        Ok(Self { stream })
    }

    pub fn handle_recieve(&mut self) -> Result<()> {
        let mut buffer = [0; 4096];
        let mut packet = Vec::new();

        loop {
            let bytes_read = self.stream.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            packet.extend_from_slice(&buffer[..bytes_read]);

            if packet.ends_with(b"$") {
                break;
            }
        }

        println!("(gdb) received packet: {:?}", packet);

        let packet = packet;

        if packet.len() < 4 {
            return Ok(());
        }

        // Won't bother checking the checksum.

        let packet = std::str::from_utf8(&packet)?;

        println!("(gdb) received packet: {}", packet);

        if packet.starts_with("qSupported") {
            self.write_packet(b"QStartNoAckMode")?;
        }

        Ok(())
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

    pub fn write_memory(&mut self, address: u64, data: &[u8]) -> Result<()> {
        let mut packet = Vec::new();
        packet.extend_from_slice(b"M");
        packet.extend_from_slice(&address.to_be_bytes());
        packet.extend_from_slice(&data.len().to_be_bytes());
        packet.extend_from_slice(data);
        self.write_packet(&packet)
    }
}
