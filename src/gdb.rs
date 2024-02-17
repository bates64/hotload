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

    #[error("bad packet {0:?}")]
    BadPacket(String),
}

impl Client {
    /// Connects to a GDB server at the given address.
    pub fn new(address: &str) -> Result<Self> {
        let stream = TcpStream::connect(address)?;
        //stream.set_nonblocking(true)?;
        let mut client = Self { stream };

        // Acknowledge the connection
        // https://github.com/ares-emulator/ares/blob/dd9c728a1277f586a663e415234f7b6b1c6dea55/nall/tcptext/tcptext-server.cpp#L18
        client.ack_recv()?;

        Ok(client)
    }

    /// Blocks until a connection is established.
    pub fn new_blocking(address: &str) -> Result<Self> {
        loop {
            match Self::new(address) {
                Ok(client) => return Ok(client),
                Err(error) => {
                    if let Error::Io(error) = &error {
                        if error.kind() == std::io::ErrorKind::ConnectionRefused {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                    }

                    // Some other error
                    return Err(error);
                }
            }
        }
    }

    fn ack_recv(&mut self) -> Result<()> {
        self.stream.write_all(b"+")?;
        self.stream.flush()?;
        Ok(())
    }

    /// Handles a single packet from the GDB server.
    /// This function does not block. It will return Err if there is no packet to read.
    pub fn accept_packet(&mut self) -> Result<String> {
        let mut buffer = [0; 4096];
        let mut packet = Vec::new();

        loop {
            let bytes_read = self.stream.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            packet.extend_from_slice(&buffer[..bytes_read]);

            if packet[packet.len() - 3] == b'#' {
                break;
            }
        }

        //let mut packet = std::str::from_utf8(&packet)?.to_string();

        // Ignore ack
        if packet[0] == b'+' {
            packet.remove(0);
        }

        // Packets are in the form $...#..
        if packet[0] == b'$' {
            // Won't bother checking the checksum.

            // Remove the checksum
            let hash = packet.len() - 3; // #xx
            let packet = &packet[1..hash]; // 1 to skip the $

            let packet = std::str::from_utf8(packet)?.to_string();

            #[cfg(debug_assertions)]
            println!("(gdb) <- {}", &packet);

            self.ack_recv()?;

            Ok(packet.to_string())
        } else {
            let packet = std::str::from_utf8(&packet)?.to_string();
            Err(Error::BadPacket(packet))
        }
    }

    /// Writes a packet to the GDB server.
    // https://sourceware.org/gdb/current/onlinedocs/gdb.html/Packets.html
    fn send(&mut self, packet: &[u8]) -> Result<()> {
        let checksum = packet
            .iter()
            .fold(0, |acc: u8, &byte| acc.wrapping_add(byte));

        self.stream.write_all(b"$")?;
        self.stream.write_all(packet)?;
        self.stream.write_all(b"#")?;
        self.stream
            .write_all(format!("{:02}", checksum).as_bytes())?;
        self.stream.flush()?;

        #[cfg(debug_assertions)]
        println!("(gdb) -> {}", std::str::from_utf8(packet)?);

        Ok(())
    }

    fn send_str(&mut self, packet: &str) -> Result<()> {
        self.send(packet.as_bytes())
    }

    // M addr,length:XX…
    pub fn write_memory(&mut self, address: u64, data: &[u8]) -> Result<()> {
        let mut data_hex = String::with_capacity(data.len() * 2);
        for byte in data {
            data_hex.push_str(&format!("{:02X}", byte));
        }

        self.send_str(&format!("M{:X},{}:{}", address, data.len(), data_hex))?;

        let response = self.accept_packet()?;
        if response == "OK" {
            Ok(())
        } else {
            Err(Error::BadPacket(response))
        }
    }
}
