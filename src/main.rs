use anyhow::Result;
use tsubame::CURRENT_VERSION;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Our hope is a little tsubame, current {}", CURRENT_VERSION);
    Ok(())
}
