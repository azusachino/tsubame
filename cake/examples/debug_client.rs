//! Debug HTTP/3 client to troubleshoot connection issues

use anyhow::Result;
use ring::rand::SecureRandom;

const MAX_DATAGRAM_SIZE: usize = 1350;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let server_name = "localhost";
    let peer_addr = "127.0.0.1:4433".parse()?;

    log::info!("Connecting to {}", peer_addr);

    // Create QUIC config
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.verify_peer(false);

    // Try setting application protocol manually
    config.set_application_protos(&[b"h3", b"h3-29", b"h3-28", b"h3-27"])?;

    config.set_max_idle_timeout(5000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_stream_data_uni(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_initial_max_streams_uni(100);
    config.set_disable_active_migration(true);

    log::info!("Config created successfully");

    // Create socket
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(peer_addr).await?;
    let local_addr = socket.local_addr()?;

    log::info!("Socket bound to {}", local_addr);

    // Generate connection ID
    let mut scid = [0; quiche::MAX_CONN_ID_LEN];
    ring::rand::SystemRandom::new()
        .fill(&mut scid)
        .map_err(|_| anyhow::anyhow!("Failed to generate scid"))?;
    let scid = quiche::ConnectionId::from_ref(&scid);

    log::info!("Connection ID: {}", hex::encode(scid.as_ref()));

    // Create QUIC connection
    let mut conn = quiche::connect(Some(server_name), &scid, local_addr, peer_addr, &mut config)?;

    log::info!("QUIC connection object created");

    let mut buf = vec![0; 65535];
    let mut out = vec![0; MAX_DATAGRAM_SIZE];

    // Send initial packet
    log::info!("Sending initial handshake packet...");
    loop {
        match conn.send(&mut out) {
            Ok((write, send_info)) => {
                socket.send(&out[..write]).await?;
                log::info!("Sent {} bytes to {}", write, send_info.to);
            }
            Err(quiche::Error::Done) => {
                log::info!("Initial send complete");
                break;
            }
            Err(e) => {
                log::error!("Send error: {:?}", e);
                return Err(e.into());
            }
        }
    }

    // Wait for server response
    log::info!("Waiting for server response...");

    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let len = socket.recv(&mut buf).await?;
        log::info!("Received {} bytes from server", len);

        let recv_info = quiche::RecvInfo {
            from: peer_addr,
            to: local_addr,
        };

        log::info!("Processing received packet...");
        match conn.recv(&mut buf[..len], recv_info) {
            Ok(recv_len) => {
                log::info!("Successfully processed {} bytes", recv_len);
                log::info!("Connection established: {}", conn.is_established());
                log::info!("Connection closed: {}", conn.is_closed());
                Ok::<(), anyhow::Error>(())
            }
            Err(e) => {
                log::error!("Error processing packet: {:?}", e);
                log::error!("First 32 bytes of packet: {:02x?}", &buf[..32.min(len)]);
                Err(e.into())
            }
        }
    })
    .await??;

    println!("\nConnection test completed successfully!");

    Ok(())
}
