use anyhow::Result;
use futures::stream::StreamExt;
use ipfs_embed::{identity::ed25519::Keypair, Block, Config, DefaultParams, Ipfs};
use libipld::{multihash::Code, raw::RawCodec};
use prometheus::Registry;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // ---- 1. Create a node backed by an on-disk store ----
    let dir = std::env::temp_dir().join("ipfs-embed-overview");
    std::fs::create_dir_all(&dir)?;
    let mut config = Config::new(&dir, Keypair::generate());
    config.network.mdns = None; // keep the example quiet
    let mut ipfs = Ipfs::<DefaultParams>::new(config).await?;

    let listener = ipfs.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();
    println!("listening on {:?}", listener);
    println!("peer id: {}", ipfs.local_peer_id());
    println!("node name: {}", ipfs.local_node_name());
    println!("public key: {:?}", ipfs.local_public_key());

    // ---- 2. Insert and read blocks ----
    let block = Block::encode(RawCodec, Code::Blake3_256, b"hello ipfs-embed".as_slice())?;
    let cid = *block.cid();
    ipfs.insert(block.clone())?;
    println!("inserted {}", cid);
    assert!(ipfs.contains(&cid)?);
    let got = ipfs.get(&cid)?;
    assert_eq!(got.data(), block.data());
    println!("read back {} bytes", got.data().len());

    // ---- 3. Temp pins keep blocks alive within a scope ----
    let other = Block::encode(RawCodec, Code::Blake3_256, b"to be pinned".as_slice())?;
    let other_cid = *other.cid();
    ipfs.insert(other.clone())?;
    {
        let mut pin = ipfs.create_temp_pin()?;
        ipfs.temp_pin(&mut pin, &other_cid)?;
        // block stays alive while `pin` is in scope
        assert!(ipfs.contains(&other_cid)?);
    }
    println!("temp pin released");

    // ---- 4. Aliases (named roots) and resolution ----
    let root_alias = b"my-root".as_slice().to_vec();
    ipfs.alias(root_alias.clone(), Some(&cid))?;
    assert_eq!(ipfs.resolve(&root_alias)?, Some(cid));
    let rev = ipfs.reverse_alias(&cid)?;
    println!("reverse alias: {:?}", rev);
    let all = ipfs.aliases()?;
    println!("known aliases: {}", all.len());
    ipfs.alias(root_alias.clone(), None)?; // remove

    // ---- 5. Iterate over stored blocks ----
    let mut n = 0;
    for c in ipfs.iter()? {
        n += 1;
        let _ = c;
    }
    println!("stored blocks: {}", n);

    // ---- 6. Batched writes ----
    let blk_a = Block::encode(RawCodec, Code::Blake3_256, b"batch-a".as_slice())?;
    let blk_b = Block::encode(RawCodec, Code::Blake3_256, b"batch-b".as_slice())?;
    ipfs.batch_ops(|db| {
        db.insert(blk_a.clone())?;
        db.insert(blk_b.clone())?;
        Ok(())
    })?;
    assert!(ipfs.contains(blk_a.cid())? && ipfs.contains(blk_b.cid())?);
    println!("batch insert ok");

    // ---- 7. Eviction / gc trigger (best effort, size bound by config) ----
    let _ = ipfs.evict().await;

    // ---- 8. Prometheus metrics ----
    let registry = Registry::new();
    ipfs.register_metrics(&registry)?;
    let families = registry.gather();
    println!("metrics families: {}", families.len());

    // ---- 9. Swarm events stream (drain a couple, non-blocking) ----
    let mut events = ipfs.swarm_events().await?;
    if let Some(ev) = events.next().await {
        println!("swarm event: {:?}", ev);
    }

    // ---- 10. Connections / peers snapshots ----
    println!("peers: {:?}", ipfs.peers());
    println!("connections: {:?}", ipfs.connections());
    for p in ipfs.peers() {
        println!("peer info present: {}", ipfs.peer_info(&p).is_some());
    }

    // ---- 11. External addresses & prune ----
    ipfs.add_external_address("/ip4/127.0.0.1/tcp/9999".parse()?);
    println!("external addresses: {:?}", ipfs.external_addresses());
    ipfs.prune_peers(Duration::from_secs(0));

    // ---- 12. Flush & drop ----
    ipfs.flush().await?;
    println!("flushed");

    drop(ipfs);
    println!("done");
    Ok(())
}