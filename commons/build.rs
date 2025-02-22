use std::io::Result as IoResult;

fn main() -> IoResult<()> {
    tonic_build::compile_protos("protos/push_service.proto")?;
    Ok(())
}
