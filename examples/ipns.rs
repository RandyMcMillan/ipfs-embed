use anyhow::Result;
use ipfs_embed::{identity::Keypair, Block, Config, DefaultParams, Ipfs};
use libipld::{multihash::Code, raw::RawCodec};

#[async_std::main]
async fn main() -> Result<()> {
    let node = Ipfs::<DefaultParams>::new(Config::default()).await?;
    let keypair = Keypair::generate_ed25519();

    let block = Block::encode(RawCodec, Code::Blake3_256, b"hello ipns".as_slice())?;
    node.insert(block.clone())?;
    let cid = *block.cid();

    let record_cid = node.ipns_publish(&keypair, 1, &cid)?;
    println!("published record: {}", record_cid);
    println!("name peer id: {}", keypair.public().to_peer_id());

    let resolved = node.ipns_resolve(&record_cid)?;
    println!("resolved: {}", resolved);
    assert_eq!(resolved, cid);
    println!("ok");
    Ok(())
}