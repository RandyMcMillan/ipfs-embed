use anyhow::Result;
use ipfs_embed::{Block, Config, DefaultParams, Ipfs};
use libipld::{multihash::Code, raw::RawCodec};
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    // In-memory store (path None).
    let node = Ipfs::<DefaultParams>::new(Config::default()).await?;

    let a = Block::encode(RawCodec, Code::Blake3_256, b"hello".as_slice())?;
    let b = Block::encode(RawCodec, Code::Blake3_256, b"world".as_slice())?;
    let a_cid = *a.cid();
    let b_cid = *b.cid();

    node.insert(a.clone())?;
    node.insert(b.clone())?;

    println!("contains a: {}", node.contains(&a_cid)?);
    println!("contains b: {}", node.contains(&b_cid)?);
    assert_eq!(node.get(&a_cid)?.data(), a.data());

    // Iterate stored CIDs.
    let stored: Vec<_> = node.iter()?.collect();
    println!("stored blocks: {}", stored.len());

    // Batched insert is atomic-ish per callback.
    let c = Block::encode(RawCodec, Code::Blake3_256, b"batched".as_slice())?;
    node.batch_ops(|db| {
        db.insert(c.clone())?;
        Ok(())
    })?;
    assert!(node.contains(c.cid())?);

    // Temp pin keeps a block during eviction pressure.
    {
        let mut pin = node.create_temp_pin()?;
        node.temp_pin(&mut pin, &b_cid)?;
        let _ = node.evict().await;
        assert!(node.contains(&b_cid)?);
    }
    println!("evict after pin released");

    // Flush persistence.
    node.flush().await?;
    println!("flushed");
    let _ = Duration::from_secs(0);
    Ok(())
}