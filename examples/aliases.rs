use anyhow::Result;
use ipfs_embed::{Block, Config, DefaultParams, Ipfs};
use libipld::{multihash::Code, raw::RawCodec};
use std::collections::HashSet;

#[async_std::main]
async fn main() -> Result<()> {
    let node = Ipfs::<DefaultParams>::new(Config::default()).await?;

    // Insert a couple of blocks and build a DAG root.
    let root_block = Block::encode(RawCodec, Code::Blake3_256, b"root".as_slice())?;
    let root_cid = *root_block.cid();
    node.insert(root_block)?;
    let other = Block::encode(RawCodec, Code::Blake3_256, b"other".as_slice())?;
    let other_cid = *other.cid();
    node.insert(other)?;

    // Create an alias pointing at root.
    let name = b"my-root".to_vec();
    node.alias(name.clone(), Some(&root_cid))?;
    println!("resolve my-root -> {:?}", node.resolve(&name)?);

    // Multiple aliases can point at the same root.
    node.alias(b"alt".to_vec(), Some(&root_cid))?;
    println!("aliases: {:?}", node.aliases());

    // reverse_alias shows which names pin a given CID.
    let pinned_by: Option<HashSet<Vec<u8>>> = node.reverse_alias(&root_cid)?;
    println!("reverse alias for root: {:?}", pinned_by);

    // Repoint an alias.
    node.alias(name.clone(), Some(&other_cid))?;
    println!("repointed my-root -> {:?}", node.resolve(&name)?);

    // Remove an alias entirely.
    node.alias(b"alt".to_vec(), None)?;
    node.alias(name, None)?;
    assert!(node.resolve(b"my-root")?.is_none());
    println!("done");
    Ok(())
}