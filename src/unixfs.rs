use crate::{Block, Cid, Ipfs};
use libipld::multihash::Code;
use libipld::raw::RawCodec;
use libipld::codec::Codec;
use libipld::codec::References;
use libipld::pb::DagPbCodec;
use libipld::{Ipld, store::StoreParams};
use std::collections::BTreeMap;
use std::io::Read;

const CHUNK_SIZE: usize = 256 * 1024;

fn put_varint(buf: &mut Vec<u8>, mut x: u64) {
    while x >= 0x80 {
        buf.push((x as u8) | 0x80);
        x >>= 7;
    }
    buf.push(x as u8);
}

fn get_varint(buf: &[u8], i: &mut usize) -> u64 {
    let mut res = 0u64;
    let mut shift = 0;
    loop {
        let b = buf[*i];
        *i += 1;
        res |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    res
}

fn encode_unixfs_file(data: Option<&[u8]>, filesize: u64, blocksizes: &[u64]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push((1 << 3) | 0);
    put_varint(&mut out, 2);
    if let Some(data) = data {
        out.push((2 << 3) | 2);
        put_varint(&mut out, data.len() as u64);
        out.extend_from_slice(data);
    }
    if filesize > 0 {
        out.push((3 << 3) | 0);
        put_varint(&mut out, filesize);
    }
    for bs in blocksizes {
        out.push((4 << 3) | 0);
        put_varint(&mut out, *bs);
    }
    out
}

struct UnixFsMeta {
    _type: u64,
    data: Option<Vec<u8>>,
    filesize: u64,
}

fn parse_unixfs(buf: &[u8]) -> UnixFsMeta {
    let mut i = 0;
    let mut typ = 0u64;
    let mut data = None;
    let mut filesize = 0u64;
    while i < buf.len() {
        let tag = get_varint(buf, &mut i);
        let field = tag >> 3;
        let wire = tag & 7;
        match (field, wire) {
            (1, 0) => typ = get_varint(buf, &mut i),
            (2, 2) => {
                let len = get_varint(buf, &mut i) as usize;
                data = Some(buf[i..i + len].to_vec());
                i += len;
            }
            (3, 0) => filesize = get_varint(buf, &mut i),
            (_, 0) => {
                let _ = get_varint(buf, &mut i);
            }
            (_, 2) => {
                let len = get_varint(buf, &mut i) as usize;
                i += len;
            }
            _ => break,
        }
    }
    UnixFsMeta {
        _type: typ,
        data,
        filesize,
    }
}

impl<P> Ipfs<P>
where
    P: StoreParams<Hashes = Code>,
    P::Codecs: From<RawCodec> + From<DagPbCodec>,
    Ipld: References<P::Codecs>,
{
    /// Ingest a reader as a UnixFS v1 file, returning the root CID.
    pub fn add_unixfs<R: Read>(&self, mut reader: R) -> anyhow::Result<Cid> {
        self.add_unixfs_with_chunk_size(reader, 0)
    }

    pub fn add_unixfs_with_chunk_size<R: Read>(&self, mut reader: R, chunk_size: usize) -> anyhow::Result<Cid> {
        let chunk_size = if chunk_size == 0 { CHUNK_SIZE } else { chunk_size };
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let chunks: Vec<&[u8]> = if data.is_empty() {
            vec![&[] as &[u8]]
        } else {
            data.chunks(chunk_size).collect()
        };

        if chunks.len() <= 1 {
            let unixfs = encode_unixfs_file(Some(&data), data.len() as u64, &[]);
            let mut node = BTreeMap::new();
            node.insert("Data".to_string(), Ipld::Bytes(unixfs));
            node.insert("Links".to_string(), Ipld::List(vec![]));
            let ipld = Ipld::Map(node);
            let block = Block::encode(DagPbCodec, Code::Sha2_256, &ipld)?;
            let cid = *block.cid();
            self.insert(block)?;
            return Ok(cid);
        }

        let mut link_cids = Vec::new();
        let mut sizes = Vec::new();
        for chunk in &chunks {
            let leaf = Block::encode(RawCodec, Code::Sha2_256, *chunk)?;
            let leaf_cid = *leaf.cid();
            self.insert(leaf)?;
            link_cids.push(leaf_cid);
            sizes.push(chunk.len() as u64);
        }

        let unixfs = encode_unixfs_file(None, data.len() as u64, &sizes);
        let links: Vec<Ipld> = link_cids
            .iter()
            .zip(sizes.iter())
            .map(|(cid, size)| {
                let mut link = BTreeMap::new();
                link.insert("Hash".to_string(), Ipld::Link(cid.clone()));
                link.insert("Name".to_string(), Ipld::String(String::new()));
                link.insert("Tsize".to_string(), Ipld::Integer(*size as i128));
                Ipld::Map(link)
            })
            .collect();
        let mut node = BTreeMap::new();
        node.insert("Data".to_string(), Ipld::Bytes(unixfs));
        node.insert("Links".to_string(), Ipld::List(links));
        let ipld = Ipld::Map(node);
        let block = Block::encode(DagPbCodec, Code::Sha2_256, &ipld)?;
        let cid = *block.cid();
        self.insert(block)?;
        Ok(cid)
    }

    /// Returns the concatenated bytes of a UnixFS file DAG.
    pub fn cat_unixfs(&self, cid: &Cid) -> anyhow::Result<Vec<u8>> {
        let block = self.get(cid)?;
        if cid.codec() == u64::from(RawCodec) {
            return Ok(block.data().to_vec());
        }

        let ipld: Ipld = DagPbCodec.decode(block.data())?;
        let node = match ipld {
            Ipld::Map(m) => m,
            _ => return Err(anyhow::anyhow!("not a dag-pb node")),
        };

        let data_bytes = node
            .get("Data")
            .and_then(|v| match v {
                Ipld::Bytes(b) => Some(b.as_slice()),
                _ => None,
            })
            .unwrap_or(&[]);

        let meta = parse_unixfs(data_bytes);

        let links = node
            .get("Links")
            .and_then(|v| match v {
                Ipld::List(l) => Some(l.as_slice()),
                _ => None,
            })
            .unwrap_or(&[]);

        if links.is_empty() {
            if let Some(d) = meta.data {
                return Ok(d);
            }
            return Ok(data_bytes.to_vec());
        }

        let mut out = Vec::with_capacity(meta.filesize as usize);
        for link in links {
            if let Ipld::Map(m) = link {
                if let Some(Ipld::Link(child_cid)) = m.get("Hash") {
                    let child = self.get(child_cid)?;
                    out.extend_from_slice(child.data());
                }
            }
        }
        if meta.filesize > 0 {
            out.truncate(meta.filesize as usize);
        }
        Ok(out)
    }
}
