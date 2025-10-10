# Cake - HTTP/3 Server with Cloudflare Quiche

An HTTP/3 server implementation using Cloudflare's quiche library for QUIC protocol support.

## Features

- HTTP/3 server with automatic certificate generation
- Support for GET and POST requests
- Request body handling
- Multiple client example implementations

## Building

```bash
# Build the server
cargo build -p cake

# Build in release mode
cargo build -p cake --release
```

## Running the Server

```bash
# Start the HTTP/3 server on 127.0.0.1:4433
cargo run -p cake

# With logging
RUST_LOG=info cargo run -p cake
```

The server will automatically generate a self-signed certificate (`cert.pem` and `key.pem`) if they don't exist.

## Server Endpoints

- `GET /` - Returns a welcome message
- `GET /health` - Health check endpoint
- `GET /json` - Returns JSON response
- `POST /echo` - Echoes the request body back
- `POST /data` - Returns information about received data

## Client Examples

### Simple GET Request

```bash
# Run the simple GET client
RUST_LOG=info cargo run -p cake --example simple_get
```

Demonstrates:
- Establishing QUIC connection
- Sending HTTP/3 GET request
- Receiving and displaying response

### POST with Echo

```bash
# Run the POST echo client
RUST_LOG=info cargo run -p cake --example post_echo
```

Demonstrates:
- Sending HTTP/3 POST request with body
- Receiving echo response

### Concurrent Requests

```bash
# Run concurrent requests client
RUST_LOG=info cargo run -p cake --example concurrent_requests
```

Demonstrates:
- Sending multiple HTTP/3 requests in parallel
- Handling multiple streams over single QUIC connection
- Processing responses as they arrive

## Implementation Details

- Uses Cloudflare's quiche library for QUIC/HTTP3 support
- Implements token-based connection validation
- Handles multiple concurrent connections
- Automatic certificate generation using rcgen
- Tokio async runtime for async I/O

## Dependencies

- `quiche` (0.24) - QUIC protocol implementation
- `tokio` - Async runtime
- `rcgen` - Certificate generation
- `ring` - Cryptographic operations
- `anyhow` - Error handling
