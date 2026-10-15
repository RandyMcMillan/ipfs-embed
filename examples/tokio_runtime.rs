#[cfg(feature = "tokio")]
fn main() {
    use ipfs_embed::{Config, DefaultParams, Ipfs};
    use tokio_crate as tokio;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    rt.block_on(async {
        let node = Ipfs::<DefaultParams>::new(Config::default()).await.unwrap();
        println!("tokio runtime node peer id: {}", node.local_peer_id());
    });
}

#[cfg(not(feature = "tokio"))]
fn main() {
    println!("run with: cargo run --example tokio_runtime --no-default-features --features tokio");
}