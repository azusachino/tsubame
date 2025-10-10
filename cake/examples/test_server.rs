//! Minimal test to check if we can reach the server

use std::net::UdpSocket;

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("127.0.0.1:4433")?;

    // Send a simple UDP packet
    socket.send(b"test")?;

    println!("Sent test packet to 127.0.0.1:4433");

    // Try to receive
    let mut buf = [0; 1024];
    socket.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;

    match socket.recv(&mut buf) {
        Ok(len) => println!("Received {} bytes from server", len),
        Err(e) => println!("No response from server: {}", e),
    }

    Ok(())
}
