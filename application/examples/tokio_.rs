/// example to use tokio by raw operations
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { println!("hello tokio async world") });
}
