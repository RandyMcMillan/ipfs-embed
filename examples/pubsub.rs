use anyhow::Result;
use futures::stream::StreamExt;
use ipfs_embed::{Config, DefaultParams, Ipfs};
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    let mut node1 = Ipfs::<DefaultParams>::new(Config::default()).await?;
    let mut node2 = Ipfs::<DefaultParams>::new(Config::default()).await?;
    node1.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();
    node2.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();
    node2.dial_address(node1.local_peer_id(), node1.listeners()[0].clone());
    async_std::task::sleep(Duration::from_secs(2)).await;

    let topic = "news".to_string();
    let mut sub1 = node1.subscribe(topic.clone()).await?;
    let mut sub2 = node2.subscribe(topic.clone()).await?;

    // give subscriptions time to propagate
    async_std::task::sleep(Duration::from_millis(500)).await;

    node1.publish(topic.clone(), b"hello pubsub".to_vec()).await?;
    node2.broadcast(topic.clone(), b"broadcast msg".to_vec()).await?;

    for _ in 0..2 {
        async_std::task::sleep(Duration::from_millis(500)).await;
        if let Some(ev) = sub1.next().await {
            println!("node1 received: {:?}", ev);
        }
        if let Some(ev) = sub2.next().await {
            println!("node2 received: {:?}", ev);
        }
    }
    println!("done");
    Ok(())
}