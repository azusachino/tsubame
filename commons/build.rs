use std::io::Result as IoResult;

fn main() -> IoResult<()> {
    tonic_prost_build::configure()
        .compile_protos(&["protos/push_service.proto"], &["protos"])
        .unwrap();
    Ok(())
}
