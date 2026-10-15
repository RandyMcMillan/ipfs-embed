use anyhow::Result;
use futures::stream::StreamExt;
use ipfs_embed::{Config, DefaultParams, Ipfs, ListenerEvent};
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    let mut node = Ipfs::<DefaultParams>::new(Config::default()).await?;

    // Listen on a fixed port and inspect listener events.
    let mut events = node.listen_on("/ip4/127.0.0.1/tcp/0".parse()?);

    println!("peer id: {}", node.local_peer_id());

    // Collect a listener event (NewListenAddr typically).
    if let Some(ev) = events.next().await {
        match ev {
            ListenerEvent::NewListenAddr(addr) => println!("listening on {}", addr),
            ListenerEvent::ExpiredListenAddr(addr) => println!("listener expired {}", addr),
            ListenerEvent::ListenFailed(addr, err) => println!("listener failed {} {}", addr, err),
        }
    }

    // Listening addresses are now visible.
    println!("active listeners: {:?}", node.listeners());

    // Keep the process alive briefly so inbound peers could connect.
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("done");
    Ok(())
}