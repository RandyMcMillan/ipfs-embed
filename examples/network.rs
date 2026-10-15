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

    println!("node a peer={} name={}", a.local_peer_id(), a.local_node_name());
    println!("node b peer={} name={}", b.local_peer_id(), b.local_node_name());
    println!("a public key: {:?}", a.local_public_key());

    b.dial_address(a.local_peer_id(), a.listeners()[0].clone());
    async_std::task::sleep(Duration::from_secs(2)).await;
    println!("a connected to b? {}", a.is_connected(&b.local_peer_id()));
    println!("b connected to a? {}", b.is_connected(&a.local_peer_id()));

    for (peer, addr, since, dir) in a.connections() {
        println!("a connection: {} {} {:?} {:?}", peer, addr, since, dir);
    }
    if let Some(info) = a.peer_info(&b.local_peer_id()) {
        println!("a knows b agent: {:?}", info.agent_version());
        println!("a knows b protocols: {:?}", info.protocols().collect::<Vec<_>>());
        println!("a knows b listeners: {:?}", info.listen_addresses().collect::<Vec<_>>());
    }

    a.add_external_address(b.listeners()[0].clone());
    a.add_address(b.local_peer_id(), b.listeners()[0].clone());
    a.add_addresses(vec![(b.local_peer_id(), b.listeners()[0].clone())]);
    println!("a knows b addresses");
    println!("a peers: {:?}", a.peers());
    println!("a external addresses: {:?}", a.external_addresses());
    a.remove_address(b.local_peer_id(), b.listeners()[0].clone());

    a.dial(b.local_peer_id());
    a.prune_peers(Duration::from_secs(0));

    let _ = a.ban(b.local_peer_id());
    println!("banned b, now unban");
    a.unban(b.local_peer_id());

    let _ = a.swarm_events().await;
    println!("done");
    Ok(())
}