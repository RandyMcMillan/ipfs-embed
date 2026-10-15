use anyhow::Result;
use ipfs_embed::{Config, DefaultParams, Ipfs};
use std::io::Cursor;

#[async_std::main]
async fn main() -> Result<()> {
    let node = Ipfs::<DefaultParams>::new(Config::default()).await?;

    // Small file
    let cid_small = node.add_unixfs(Cursor::new(b"hello unixfs"))?;
    println!("small root: {}", cid_small);
    let bytes = node.cat_unixfs(&cid_small)?;
    println!("small cat: {}", String::from_utf8_lossy(&bytes));
    assert_eq!(bytes, b"hello unixfs");

    // Large file spanning chunks
    let big = vec![b'a'; 300 * 1024];
    let cid_big = node.add_unixfs(Cursor::new(big.clone()))?;
    println!("big root: {}", cid_big);
    let got = node.cat_unixfs(&cid_big)?;
    assert_eq!(got, big);
    println!("big file roundtripped ({} bytes)", got.len());

    // empty file
    let cid_empty = node.add_unixfs(Cursor::new(Vec::<u8>::new()))?;
    assert_eq!(node.cat_unixfs(&cid_empty)?, Vec::<u8>::new());
    println!("empty ok");

    // Persist
    node.flush().await?;
    Ok(())
}