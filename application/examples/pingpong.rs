use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:9001";
    let socket = UdpSocket::bind(addr).await?;
    println!("Listening on: {}", addr);

    // Spawn a task to handle incoming messages
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];

        loop {
            // Receive data
            let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
            let received = &buf[..len];

            // Handle "ping" and "pong"
            let message = String::from_utf8_lossy(received);
            println!("Received {} from {}", message, addr);

            if message.trim() == "ping" {
                // Respond with "pong"
                println!("Sending pong to {}", addr);
                socket.send_to(b"pong", addr).await.unwrap();
            } else if message.trim() == "pong" {
                println!("Received pong from {}", addr);
            }
        }
    });

    let peer_addr = "127.0.0.1:10001";
    let peer_socket = UdpSocket::bind(peer_addr).await?;

    println!("Peer listening on: {}", peer_addr);
    peer_socket.send_to(b"ping", addr).await?;

    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    Ok(())
}
