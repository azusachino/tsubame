# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tsubame is a Rust practice project exploring various network protocols and load balancing technologies. The name comes from YOASOBI. The project uses Rust 2024 edition and is structured as a Cargo workspace with multiple binary and library crates.

## Architecture

### Workspace Structure

The project is a Cargo workspace with five member crates:

1. **commons** - Shared library crate containing:
   - gRPC protocol buffer definitions (compiled from `protos/push_service.proto`)
   - Common data structures (tree structures, config types)
   - TOML configuration utilities
   - Builds protobuf definitions via `tonic-prost-build` in build.rs

2. **application** - Main application server featuring:
   - Dual-server architecture: HTTP (Axum on port 3000) + gRPC (Tonic on port 50002)
   - JWT authentication middleware (`interceptor/jwt_auth.rs`)
   - Rate limiting middleware (`interceptor/rate_limit.rs`)
   - User service with PostgreSQL integration via SQLx
   - Environment-based configuration loading (looks for `config.{ENVIRONMENT}.toml`)
   - Uses jemalloc as global allocator for better performance

3. **gateway** - HTTP gateway using Actix-web (port 8081, minimal implementation)

4. **balancer** - Load balancer using Cloudflare's Pingora framework

5. **cake** - QUIC/HTTP3 experimentation using the quiche library

### Configuration System

The application uses TOML-based configuration with environment-aware loading:
- Configuration files: `config.dev.toml`, `config.prod.toml`
- Environment variable `ENVIRONMENT` determines which config to load (defaults to "dev")
- Config structure in `application/src/config.rs` includes app settings and PostgreSQL connection details
- The `commons` crate provides utilities for working with TOML configurations

### gRPC Architecture

Protocol buffers are defined in `commons/protos/push_service.proto` and compiled during build:
- Service: `PushService` with a `v1` RPC endpoint
- The `application` binary implements the gRPC server alongside the HTTP server
- Both servers run concurrently using tokio::spawn

## Development Commands

### Building

```bash
# Build all workspace members
cargo build

# Build specific crate
cargo build -p tsubame_application
cargo build -p tsubame_gateway
cargo build -p tsubame_balancer
cargo build -p cake

# Release build with LTO optimization
cargo build --release
```

### Running

```bash
# Run the main application (HTTP + gRPC servers)
cargo run -p tsubame_application

# Run specific examples
cargo run -p tsubame_application --example pingpong
cargo run -p tsubame_application --example pg_practice
cargo run -p tsubame_application --example tokio_

# Run gateway
cargo run -p tsubame_gateway

# Run balancer
cargo run -p tsubame_balancer

# Run QUIC/HTTP3 server
cargo run -p cake
```

### Testing

```bash
# Run all tests
cargo test

# Test specific crate
cargo test -p tsubame_commons
cargo test -p tsubame_application
```

### Code Quality

```bash
# Format code (configured in rustfmt.toml: max_width=98, 4 spaces)
cargo fmt

# Lint with Clippy (cognitive-complexity-threshold set to 100 in clippy.toml)
cargo clippy

# Check without building
cargo check
```

### Environment Setup

The application requires:
- PostgreSQL connection (configured via config.{env}.toml)
- Environment variables can be loaded from .env files (using dotenv crate)
- Set `ENVIRONMENT` variable to switch between dev/prod configs
- Set `CONFIG_PATH` to specify config directory (defaults to ".")

### Cross-Compilation

For Linux targets from macOS:

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Build for Linux
cargo build --target x86_64-unknown-linux-gnu --release
```

For crates with *-sys dependencies, use Docker (see `docker/Dockerfile.x64` and `docker/Dockerfile.arm64`).

## Important Implementation Details

### Protocol Buffer Changes

When modifying `commons/protos/push_service.proto`:
1. The build.rs script automatically recompiles protos
2. Generated code is included via `tonic::include_proto!("push_commons")` in `commons/src/lib.rs`
3. Dependent crates need to be rebuilt to see changes

### Middleware Execution Order

In the application server, middleware executes in this order:
1. Header interceptor (logs all headers)
2. JWT authentication (for /users route)
3. Rate limiting (currently commented out but available via AppState)

### Database Connections

PostgreSQL configuration in config files uses:
- SQLx with async runtime-tokio-rustls
- Connection string is built from config via `PostgresqlConfig::to_url()`
- See `application/examples/pg_practice.rs` for usage examples

### Allocator

The application binary uses tikv-jemalloc as the global allocator for improved memory performance in server workloads.

## References

Framework documentation referenced in the codebase:
- Actix-web: https://actix.rs/docs/getting-started/
- Axum: https://github.com/tokio-rs/axum, https://docs.rs/axum
- Pingora: https://github.com/cloudflare/pingora/blob/main/docs/quick_start.md
- gRPC with Tonic: https://www.thorsten-hans.com/grpc-services-in-rust-with-tonic/
