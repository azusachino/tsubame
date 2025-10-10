//! HTTP/3 POST request client example with request body

use anyhow::Result;
use quiche::h3::NameValue;
use ring::rand::SecureRandom;
use std::net::ToSocketAddrs;

const MAX_DATAGRAM_SIZE: usize = 1350;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let url = url::Url::parse("https://127.0.0.1:4433/echo")?;
    let request_body = b"Hello from HTTP/3 client! This is a POST request.";

    // Resolve server address
    let peer_addr = format!("{}:{}", url.host_str().unwrap(), url.port().unwrap_or(4433))
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve address"))?;

    log::info!("Connecting to {}", peer_addr);

    // Create QUIC config
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.verify_peer(false);
    config.set_application_protos(quiche::h3::APPLICATION_PROTOCOL)?;
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
    config.enable_early_data();

    // Create socket
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(peer_addr).await?;

    // Generate connection ID
    let mut scid = [0; quiche::MAX_CONN_ID_LEN];
    ring::rand::SystemRandom::new()
        .fill(&mut scid)
        .map_err(|_| anyhow::anyhow!("Failed to generate scid"))?;
    let scid = quiche::ConnectionId::from_ref(&scid);

    // Create QUIC connection
    let local_addr = socket.local_addr()?;
    let mut conn = quiche::connect(url.host_str(), &scid, local_addr, peer_addr, &mut config)?;

    log::info!("QUIC connection created");

    let mut buf = vec![0; 65535];
    let mut out = vec![0; MAX_DATAGRAM_SIZE];

    // Perform handshake
    loop {
        match conn.send(&mut out) {
            Ok((write, _)) => {
                socket.send(&out[..write]).await?;
                log::debug!("Sent {} bytes", write);
            }
            Err(quiche::Error::Done) => break,
            Err(e) => return Err(e.into()),
        }
    }

    // Wait for handshake completion
    let h3_conn;
    loop {
        let len = socket.recv(&mut buf).await?;
        let recv_info = quiche::RecvInfo {
            from: peer_addr,
            to: local_addr,
        };

        conn.recv(&mut buf[..len], recv_info)?;

        if conn.is_established() {
            log::info!("QUIC connection established");

            let h3_config = quiche::h3::Config::new()?;
            h3_conn = Some(quiche::h3::Connection::with_transport(
                &mut conn, &h3_config,
            )?);
            break;
        }

        loop {
            match conn.send(&mut out) {
                Ok((write, _)) => {
                    socket.send(&out[..write]).await?;
                }
                Err(quiche::Error::Done) => break,
                Err(e) => return Err(e.into()),
            }
        }
    }

    let mut h3_conn = h3_conn.unwrap();

    // Send HTTP/3 POST request with body
    let req = vec![
        quiche::h3::Header::new(b":method", b"POST"),
        quiche::h3::Header::new(b":scheme", b"https"),
        quiche::h3::Header::new(b":authority", url.host_str().unwrap().as_bytes()),
        quiche::h3::Header::new(b":path", url.path().as_bytes()),
        quiche::h3::Header::new(b"user-agent", b"cake-h3-client"),
        quiche::h3::Header::new(b"content-type", b"text/plain"),
        quiche::h3::Header::new(b"content-length", request_body.len().to_string().as_bytes()),
    ];

    let stream_id = h3_conn.send_request(&mut conn, &req, false)?;
    log::info!("Sent HTTP/3 POST request on stream {}", stream_id);

    // Send request body
    h3_conn.send_body(&mut conn, stream_id, request_body, true)?;
    log::info!("Sent {} bytes of request body", request_body.len());

    // Send QUIC packets
    loop {
        match conn.send(&mut out) {
            Ok((write, _)) => {
                socket.send(&out[..write]).await?;
            }
            Err(quiche::Error::Done) => break,
            Err(e) => return Err(e.into()),
        }
    }

    // Receive response
    let mut response_body = Vec::new();
    let mut response_headers = None;
    let mut finished = false;

    while !finished {
        tokio::select! {
            result = socket.recv(&mut buf) => {
                let len = result?;
                let recv_info = quiche::RecvInfo {
                    from: peer_addr,
                    to: local_addr,
                };
                conn.recv(&mut buf[..len], recv_info)?;

                // Process HTTP/3 events
                loop {
                    match h3_conn.poll(&mut conn) {
                        Ok((s, quiche::h3::Event::Headers { list, .. })) if s == stream_id => {
                            log::info!("Got response headers:");
                            for hdr in &list {
                                log::info!("  {}: {}",
                                    String::from_utf8_lossy(hdr.name()),
                                    String::from_utf8_lossy(hdr.value())
                                );
                            }
                            response_headers = Some(list);
                        }

                        Ok((s, quiche::h3::Event::Data)) if s == stream_id => {
                            let mut body_buf = vec![0; 4096];
                            match h3_conn.recv_body(&mut conn, stream_id, &mut body_buf) {
                                Ok(read) => {
                                    response_body.extend_from_slice(&body_buf[..read]);
                                    log::debug!("Received {} bytes of body", read);
                                }
                                Err(quiche::h3::Error::Done) => {}
                                Err(e) => return Err(e.into()),
                            }
                        }

                        Ok((s, quiche::h3::Event::Finished)) if s == stream_id => {
                            log::info!("Response finished");
                            finished = true;
                            break;
                        }

                        Ok(_) => {}

                        Err(quiche::h3::Error::Done) => break,

                        Err(e) => return Err(e.into()),
                    }
                }

                // Send any pending data
                loop {
                    match conn.send(&mut out) {
                        Ok((write, _)) => {
                            socket.send(&out[..write]).await?;
                        }
                        Err(quiche::Error::Done) => break,
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }
    }

    // Print response
    println!("\n=== POST Request ===");
    println!("Sent: {}", String::from_utf8_lossy(request_body));
    println!("\n=== Response ===");
    if let Some(headers) = response_headers {
        if let Some(status) = headers.iter().find(|h| h.name() == b":status") {
            println!("Status: {}", String::from_utf8_lossy(status.value()));
        }
    }
    println!(
        "\nEcho Response:\n{}",
        String::from_utf8_lossy(&response_body)
    );

    conn.close(true, 0x00, b"done")?;

    Ok(())
}
