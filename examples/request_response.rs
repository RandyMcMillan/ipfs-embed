use anyhow::Result;
use futures::stream::StreamExt;
use ipfs_embed::{Config, DefaultParams, Ipfs};
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    let mut a = Ipfs::<DefaultParams>::new(Config::default()).await?;
    let mut b = Ipfs::<DefaultParams>::new(Config::default()).await?;
    a.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();
    b.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();
    b.dial_address(a.local_peer_id(), a.listeners()[0].clone());
    async_std::task::sleep(Duration::from_secs(2)).await;

    let resp = a.send_request(b.local_peer_id(), b"hello".to_vec()).await?;
    println!("response: {:?}", String::from_utf8_lossy(&resp));
    let _ = a.flush().await;
    let _ = b.flush().await;
    Ok(())
}