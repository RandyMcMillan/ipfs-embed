use anyhow::Result;
use ipfs_embed::{Block, Config, DefaultParams, Ipfs, Key};
use libipld::{multihash::Code, raw::RawCodec};
use libp2p::kad::{record::Record, Quorum};
use futures::stream::StreamExt;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    // bootstrap node
    let mut bootstrap = Ipfs::<DefaultParams>::new(Config::default()).await?;
    bootstrap.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();

    // two clients
    let mut a = Ipfs::<DefaultParams>::new(Config::default()).await?;
    let mut b = Ipfs::<DefaultParams>::new(Config::default()).await?;
    a.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();
    b.listen_on("/ip4/127.0.0.1/tcp/0".parse()?).next().await.unwrap();

    let addr = bootstrap.listeners()[0].clone();
    let peer = bootstrap.local_peer_id();
    a.bootstrap(vec![(peer, addr.clone())]).await?;
    b.bootstrap(vec![(peer, addr)]).await?;
    async_std::task::sleep(Duration::from_secs(2)).await;
    println!("bootstrapped: a={}, b={}", a.is_bootstrapped(), b.is_bootstrapped());

    // provide a block cid via DHT
    let block = Block::encode(RawCodec, Code::Blake3_256, b"dht payload".as_slice())?;
    let cid = *block.cid();
    a.insert(block.clone())?;
    let key = Key::new(&cid.to_bytes());
    a.provide(key.clone()).await?;
    println!("a providing {}", cid);

    // query providers
    let providers = b.providers(key.clone()).await?;
    println!("b sees providers: {:?}", providers);

    let _fetched = b.fetch(&cid, providers.into_iter().collect()).await?;
    println!("b fetched block");

    // put/get record
    let record_key = Key::new(b"my-record");
    a.put_record(Record::new(record_key.clone(), b"value".to_vec()), Quorum::One).await?;
    let records = b.get_record(record_key.clone(), Quorum::One).await?;
    println!("records: {:?}", records);
    b.remove_record(record_key.clone())?;

    // unprovide
    a.unprovide(key)?;
    println!("done");
    Ok(())
}