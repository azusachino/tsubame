# tsubame

rust practice project

## cross compilation

```bash
# compile toolchain
rustup target add x86_64-unknown-linux-gnu


# modify Cargo.toml
tee -a Cargo.toml <<EOF
[target.x86_64-unknown-linux-gnu]
linker = "rust-lld" # Optional, but recommended for faster linking
rustflags = ["-C", "target-cpu=x86-64"] # Optional, optimize for x86-64

[build]
target = "x86_64-unknown-linux-gnu"
EOF

# build with specific toolchain
cargo build --target x86_64-unknown-linux-gnu --release
```

if dealing with `*-sys` dependencies, maybe try build with docker containers.

```Dockerfile
FROM rust:1.76-slim-bookworm as builder

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    llvm \
    clang \
    libssl-dev

RUN rustup target add x86_64-unknown-linux-gnu

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-gnu
```

## references

- https://actix.rs/docs/getting-started/
- https://github.com/tokio-rs/axum
- https://docs.rs/axum
- https://github.com/cloudflare/pingora/blob/main/docs/quick_start.md
