//! HTTP/3 concurrent requests example - demonstrates multiple parallel requests

use anyhow::Result;
use quiche::h3::NameValue;
use ring::rand::SecureRandom;
use std::net::ToSocketAddrs;

const MAX_DATAGRAM_SIZE: usize = 1350;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let url = url::Url::parse("https://127.0.0.1:4433")?;

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
    config.set_max_idle_timeout(10000);
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

    // Send multiple concurrent requests
    let requests = vec![
        ("/", "GET"),
        ("/health", "GET"),
        ("/json", "GET"),
        ("/data", "POST"),
    ];

    let mut stream_ids = Vec::new();

    println!("\n=== Sending {} concurrent requests ===\n", requests.len());

    for (path, method) in &requests {
        let req_headers = if *method == "POST" {
            let body = b"Test data for concurrent POST";
            let req = vec![
                quiche::h3::Header::new(b":method", method.as_bytes()),
                quiche::h3::Header::new(b":scheme", b"https"),
                quiche::h3::Header::new(b":authority", url.host_str().unwrap().as_bytes()),
                quiche::h3::Header::new(b":path", path.as_bytes()),
                quiche::h3::Header::new(b"user-agent", b"cake-h3-client"),
                quiche::h3::Header::new(b"content-type", b"application/octet-stream"),
                quiche::h3::Header::new(b"content-length", body.len().to_string().as_bytes()),
            ];

            let stream_id = h3_conn.send_request(&mut conn, &req, false)?;
            h3_conn.send_body(&mut conn, stream_id, body, true)?;
            log::info!("Sent {} {} (stream {})", method, path, stream_id);
            stream_id
        } else {
            let req = vec![
                quiche::h3::Header::new(b":method", method.as_bytes()),
                quiche::h3::Header::new(b":scheme", b"https"),
                quiche::h3::Header::new(b":authority", url.host_str().unwrap().as_bytes()),
                quiche::h3::Header::new(b":path", path.as_bytes()),
                quiche::h3::Header::new(b"user-agent", b"cake-h3-client"),
            ];

            let stream_id = h3_conn.send_request(&mut conn, &req, true)?;
            log::info!("Sent {} {} (stream {})", method, path, stream_id);
            stream_id
        };

        stream_ids.push(req_headers);
    }

    // Send all requests
    loop {
        match conn.send(&mut out) {
            Ok((write, _)) => {
                socket.send(&out[..write]).await?;
            }
            Err(quiche::Error::Done) => break,
            Err(e) => return Err(e.into()),
        }
    }

    // Track responses
    use std::collections::HashMap;
    let mut responses: HashMap<u64, (Vec<u8>, Option<Vec<quiche::h3::Header>>)> = HashMap::new();
    let mut finished_streams = std::collections::HashSet::new();

    // Receive all responses
    while finished_streams.len() < stream_ids.len() {
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
                        Ok((s, quiche::h3::Event::Headers { list, .. })) => {
                            log::info!("Got response headers for stream {}", s);
                            let entry = responses.entry(s).or_insert_with(|| (Vec::new(), None));
                            entry.1 = Some(list);
                        }

                        Ok((s, quiche::h3::Event::Data)) => {
                            let entry = responses.entry(s).or_insert_with(|| (Vec::new(), None));
                            let mut body_buf = vec![0; 4096];
                            match h3_conn.recv_body(&mut conn, s, &mut body_buf) {
                                Ok(read) => {
                                    entry.0.extend_from_slice(&body_buf[..read]);
                                    log::debug!("Received {} bytes on stream {}", read, s);
                                }
                                Err(quiche::h3::Error::Done) => {}
                                Err(e) => log::error!("recv_body error: {:?}", e),
                            }
                        }

                        Ok((s, quiche::h3::Event::Finished)) => {
                            log::info!("Stream {} finished", s);
                            finished_streams.insert(s);
                        }

                        Ok(_) => {}

                        Err(quiche::h3::Error::Done) => break,

                        Err(e) => {
                            log::error!("HTTP/3 error: {:?}", e);
                            break;
                        }
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

    // Print all responses
    println!("\n=== Responses ===\n");
    for (idx, stream_id) in stream_ids.iter().enumerate() {
        if let Some((body, headers)) = responses.get(stream_id) {
            let (path, method) = requests[idx];
            println!("Request {}: {} {}", idx + 1, method, path);
            println!("Stream ID: {}", stream_id);

            if let Some(hdrs) = headers {
                if let Some(status) = hdrs.iter().find(|h| h.name() == b":status") {
                    println!("Status: {}", String::from_utf8_lossy(status.value()));
                }
            }

            let body_str = String::from_utf8_lossy(body);
            if body_str.len() > 100 {
                println!("Body: {}... ({} bytes)", &body_str[..100], body.len());
            } else {
                println!("Body: {}", body_str);
            }
            println!();
        }
    }

    println!(
        "Successfully completed {} concurrent requests!",
        stream_ids.len()
    );

    conn.close(true, 0x00, b"done")?;

    Ok(())
}
