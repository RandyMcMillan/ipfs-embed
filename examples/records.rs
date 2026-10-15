use anyhow::Result;
use ipfs_embed::{Config, DefaultParams, Ipfs, Key};
use libp2p::kad::{record::Record, Quorum};
use futures::stream::StreamExt;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    let mut node = Ipfs::<DefaultParams>::new(Config::default()).await?;
    node.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();

    let key = Key::new(b"hello");
    node.put_record(Record::new(key.clone(), b"world".to_vec()), Quorum::One).await?;
    println!("put record");

    let records = node.get_record(key.clone(), Quorum::One).await?;
    println!("fetched {} record(s)", records.len());
    for r in &records {
        println!("  value={:?}", r.record.value);
    }

    node.remove_record(key)?;
    println!("removed");

    // Iterate isn't exposed for records; re-fetch should yield nothing worthwhile.
    let again = node.get_record(Key::new(b"hello"), Quorum::One).await.unwrap_or_default();
    println!("remaining: {}", again.len());
    let _ = Duration::from_secs(0);
    Ok(())
}