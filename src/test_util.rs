#![allow(dependency_on_unit_never_type_fallback)]

use anyhow::Result;
use libipld::{cbor::DagCborCodec, multihash::Code, Block, Cid, DagCbor, DefaultParams};
use rand::RngCore;

#[derive(Debug, DagCbor)]
pub struct Node {
    pub links: Vec<Cid>,
    pub depth: u64,
    pub payload: Box<[u8]>,
}

fn create_block(links: Vec<Cid>, depth: u64) -> Result<Block<DefaultParams>> {
    let payload = if links.is_empty() {
        let mut payload = [0u8; 1024 * 16];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut payload);
        payload.to_vec().into_boxed_slice()
    } else {
        let mut payload = [0u8; 512];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut payload);
        payload.to_vec().into_boxed_slice()
    };
    let node = Node {
        links,
        depth,
        payload,
    };
    Block::encode(DagCborCodec, Code::Blake3_256, &node)
}

pub fn build_tree(width: u64, depth: u64) -> Result<(Cid, Vec<Block<DefaultParams>>)> {
    let mut blocks = Vec::new();

    // Iteratively build the tree bottom-up. Keeping this iterative avoids deep
    // recursion, which previously overflowed the stack for tall chains (width=1).
    let leaf_count = width.pow(depth as u32) as usize;
    let mut current = Vec::with_capacity(leaf_count);
    for _ in 0..leaf_count {
        let block = create_block(Vec::new(), 0)?;
        let cid = *block.cid();
        blocks.push(block);
        current.push(cid);
    }

    let mut current_depth = 0u64;
    while current_depth < depth {
        let parent_count = current.len() / width as usize;
        let mut next = Vec::with_capacity(parent_count);
        for chunk in current.chunks(width as usize) {
            let block = create_block(chunk.to_vec(), current_depth + 1)?;
            let cid = *block.cid();
            blocks.push(block);
            next.push(cid);
        }
        current = next;
        current_depth += 1;
    }

    Ok((current[0], blocks))
}
