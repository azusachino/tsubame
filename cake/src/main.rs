//! HTTP/3 Server using Cloudflare's quiche

use anyhow::Result;
use quiche::h3::NameValue;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

const MAX_DATAGRAM_SIZE: usize = 1350;

struct ClientConnection {
    conn: quiche::Connection,
    h3_conn: Option<quiche::h3::Connection>,
    partial_requests: HashMap<u64, cake::PartialRequest>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = "127.0.0.1:4433".parse::<SocketAddr>()?;

    // Generate or use existing certificates
    let cert_path = PathBuf::from("cert.pem");
    let key_path = PathBuf::from("key.pem");

    if !cert_path.exists() || !key_path.exists() {
        log::info!("Generating self-signed certificate...");
        cake::generate_self_signed_cert(&cert_path, &key_path)?;
    }

    // Create QUIC config
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.load_cert_chain_from_pem_file(cert_path.to_str().unwrap())?;
    config.load_priv_key_from_pem_file(key_path.to_str().unwrap())?;

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

    // Store cert/key paths for creating new configs
    let cert_path_str = cert_path.to_str().unwrap().to_string();
    let key_path_str = key_path.to_str().unwrap().to_string();

    // Bind UDP socket
    let socket = tokio::net::UdpSocket::bind(addr).await?;
    log::info!("HTTP/3 server listening on {}", addr);

    let mut clients: HashMap<Vec<u8>, ClientConnection> = HashMap::new();
    let mut buf = [0; 65535];
    let mut out = [0; MAX_DATAGRAM_SIZE];

    loop {
        // Read incoming packet
        let (len, from) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("recv_from failed: {:?}", e);
                continue;
            }
        };

        let pkt_buf = &mut buf[..len];

        // Parse QUIC packet header
        let hdr = match quiche::Header::from_slice(pkt_buf, quiche::MAX_CONN_ID_LEN) {
            Ok(v) => v,
            Err(e) => {
                log::error!("Parsing packet header failed: {:?}", e);
                continue;
            }
        };

        // Check if this is an existing connection
        let conn_id = hdr.dcid.to_vec();

        let client = if !clients.contains_key(&conn_id) {
            if hdr.ty != quiche::Type::Initial {
                log::warn!("Packet is not Initial");
                continue;
            }

            // Generate source connection ID
            let mut scid_buf = [0; quiche::MAX_CONN_ID_LEN];
            use ring::rand::SecureRandom;
            ring::rand::SystemRandom::new()
                .fill(&mut scid_buf)
                .map_err(|_| anyhow::anyhow!("Failed to generate scid"))?;
            let scid = quiche::ConnectionId::from_vec(scid_buf.to_vec());

            // Get token from header
            let token: &[u8] = match &hdr.token {
                Some(t) => t,
                None => &[],
            };

            // Perform version negotiation
            if token.is_empty() {
                log::info!("Sending version negotiation for new connection");
                let new_token = mint_token(&hdr, &from);

                let len = quiche::retry(
                    &hdr.scid,
                    &hdr.dcid,
                    &scid,
                    &new_token,
                    hdr.version,
                    &mut out,
                )?;

                socket.send_to(&out[..len], from).await?;
                continue;
            }

            let odcid = validate_token(&from, token);
            if odcid.is_none() {
                log::warn!("Invalid token");
                continue;
            }

            // Create new QUIC connection - need a new config for each connection
            let mut conn_config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
            conn_config.load_cert_chain_from_pem_file(&cert_path_str)?;
            conn_config.load_priv_key_from_pem_file(&key_path_str)?;
            conn_config.set_application_protos(quiche::h3::APPLICATION_PROTOCOL)?;
            conn_config.set_max_idle_timeout(5000);
            conn_config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
            conn_config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
            conn_config.set_initial_max_data(10_000_000);
            conn_config.set_initial_max_stream_data_bidi_local(1_000_000);
            conn_config.set_initial_max_stream_data_bidi_remote(1_000_000);
            conn_config.set_initial_max_stream_data_uni(1_000_000);
            conn_config.set_initial_max_streams_bidi(100);
            conn_config.set_initial_max_streams_uni(100);
            conn_config.set_disable_active_migration(true);
            conn_config.enable_early_data();

            let conn = quiche::accept(&scid, odcid.as_ref(), addr, from, &mut conn_config)?;

            log::info!(
                "New connection: dcid={} scid={}",
                hex::encode(&conn_id),
                hex::encode(scid.as_ref())
            );

            let client = ClientConnection {
                conn,
                h3_conn: None,
                partial_requests: HashMap::new(),
            };

            clients.insert(conn_id.clone(), client);
            clients.get_mut(&conn_id).unwrap()
        } else {
            clients.get_mut(&conn_id).unwrap()
        };

        // Process packet
        let recv_info = quiche::RecvInfo { from, to: addr };

        let read = match client.conn.recv(pkt_buf, recv_info) {
            Ok(v) => v,
            Err(e) => {
                log::error!("recv failed: {:?}", e);
                continue;
            }
        };

        log::debug!("Received {} bytes", read);

        // Create HTTP/3 connection if needed
        if client.conn.is_established() && client.h3_conn.is_none() {
            log::info!("QUIC connection established, creating HTTP/3 connection");

            let h3_config = quiche::h3::Config::new()?;
            client.h3_conn = Some(quiche::h3::Connection::with_transport(
                &mut client.conn,
                &h3_config,
            )?);
        }

        // Handle HTTP/3 events
        if let Some(ref mut h3_conn) = client.h3_conn {
            loop {
                match h3_conn.poll(&mut client.conn) {
                    Ok((stream_id, quiche::h3::Event::Headers { list, more_frames })) => {
                        log::info!("Got request on stream {}", stream_id);

                        let mut path = String::new();
                        let mut method = String::new();

                        for hdr in &list {
                            let name = String::from_utf8_lossy(hdr.name());
                            let value = String::from_utf8_lossy(hdr.value());
                            log::debug!("  {}: {}", name, value);

                            if name == ":path" {
                                path = value.to_string();
                            } else if name == ":method" {
                                method = value.to_string();
                            }
                        }

                        let req = client
                            .partial_requests
                            .entry(stream_id)
                            .or_insert_with(cake::PartialRequest::default);
                        req.headers = list;

                        if !more_frames {
                            // Send response immediately if no body
                            handle_request(
                                h3_conn,
                                &mut client.conn,
                                stream_id,
                                &method,
                                &path,
                                &[],
                            )?;
                            req.finished = true;
                        }
                    }

                    Ok((stream_id, quiche::h3::Event::Data)) => {
                        if let Some(req) = client.partial_requests.get_mut(&stream_id) {
                            let mut buf = vec![0; 4096];

                            match h3_conn.recv_body(&mut client.conn, stream_id, &mut buf) {
                                Ok(read) => {
                                    req.body.extend_from_slice(&buf[..read]);
                                    log::debug!(
                                        "Got {} bytes of request body on stream {}",
                                        read,
                                        stream_id
                                    );
                                }
                                Err(quiche::h3::Error::Done) => {
                                    // Request body complete
                                    let method = req
                                        .headers
                                        .iter()
                                        .find(|h| h.name() == b":method")
                                        .map(|h| String::from_utf8_lossy(h.value()).to_string())
                                        .unwrap_or_default();

                                    let path = req
                                        .headers
                                        .iter()
                                        .find(|h| h.name() == b":path")
                                        .map(|h| String::from_utf8_lossy(h.value()).to_string())
                                        .unwrap_or_default();

                                    handle_request(
                                        h3_conn,
                                        &mut client.conn,
                                        stream_id,
                                        &method,
                                        &path,
                                        &req.body,
                                    )?;
                                    req.finished = true;
                                }
                                Err(e) => {
                                    log::error!("recv_body failed: {:?}", e);
                                }
                            }
                        }
                    }

                    Ok((stream_id, quiche::h3::Event::Finished)) => {
                        log::debug!("Stream {} finished", stream_id);
                    }

                    Ok((_goaway_id, quiche::h3::Event::GoAway)) => {
                        log::info!("Got GOAWAY");
                    }

                    Ok((_stream_id, quiche::h3::Event::Reset(error_code))) => {
                        log::info!("Stream reset with error code: {}", error_code);
                    }

                    Ok((_stream_id, quiche::h3::Event::PriorityUpdate)) => {
                        log::debug!("Priority update received");
                    }

                    Err(quiche::h3::Error::Done) => {
                        break;
                    }

                    Err(e) => {
                        log::error!("HTTP/3 error: {:?}", e);
                        break;
                    }
                }
            }
        }

        // Send outgoing packets
        loop {
            let (write, send_info) = match client.conn.send(&mut out) {
                Ok(v) => v,
                Err(quiche::Error::Done) => break,
                Err(e) => {
                    log::error!("send failed: {:?}", e);
                    break;
                }
            };

            if let Err(e) = socket.send_to(&out[..write], send_info.to).await {
                log::error!("send_to failed: {:?}", e);
                break;
            }

            log::debug!("Sent {} bytes", write);
        }

        // Clean up closed connections
        if client.conn.is_closed() {
            log::info!("Connection closed");
            clients.remove(&conn_id);
        }
    }
}

fn handle_request(
    h3_conn: &mut quiche::h3::Connection,
    conn: &mut quiche::Connection,
    stream_id: u64,
    method: &str,
    path: &str,
    body: &[u8],
) -> Result<()> {
    log::info!("Handling {} request for {}", method, path);

    let (status, response_body) = match (method, path) {
        ("GET", "/") => (200, b"Hello from HTTP/3 server!".to_vec()),
        ("GET", "/health") => (200, b"OK".to_vec()),
        ("GET", "/json") => (
            200,
            br#"{"message":"Hello HTTP/3","protocol":"h3"}"#.to_vec(),
        ),
        ("POST", "/echo") => {
            log::info!("Echo body: {}", String::from_utf8_lossy(body));
            (200, body.to_vec())
        }
        ("POST", "/data") => {
            let response = format!(
                r#"{{"received_bytes":{},"message":"Data received"}}"#,
                body.len()
            );
            (200, response.into_bytes())
        }
        _ => (404, b"Not Found".to_vec()),
    };

    // Send response headers
    let headers = vec![
        quiche::h3::Header::new(b":status", status.to_string().as_bytes()),
        quiche::h3::Header::new(b"server", b"cake-h3"),
        quiche::h3::Header::new(
            b"content-length",
            response_body.len().to_string().as_bytes(),
        ),
    ];

    h3_conn.send_response(conn, stream_id, &headers, false)?;

    // Send response body
    h3_conn.send_body(conn, stream_id, &response_body, true)?;

    log::info!("Response sent: {} bytes", response_body.len());
    Ok(())
}

fn mint_token(hdr: &quiche::Header, src: &SocketAddr) -> Vec<u8> {
    let mut token = Vec::new();
    token.extend_from_slice(b"quiche");
    token.extend_from_slice(hdr.dcid.as_ref());
    token.extend_from_slice(src.ip().to_string().as_bytes());
    token
}

fn validate_token(src: &SocketAddr, token: &[u8]) -> Option<quiche::ConnectionId<'static>> {
    if token.len() < 6 {
        return None;
    }

    if &token[..6] != b"quiche" {
        return None;
    }

    let token = &token[6..];
    let addr_len = src.ip().to_string().as_bytes().len();

    if token.len() < addr_len {
        return None;
    }

    let dcid_len = token.len() - addr_len;
    Some(quiche::ConnectionId::from_vec(token[..dcid_len].to_vec()))
}
