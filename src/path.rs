use crate::{Block, Cid, Ipfs};
use libipld::cbor::DagCborCodec;
use libipld::codec::{Codec, References};
use libipld::{Ipld, store::StoreParams};
use std::str::FromStr;

impl<P> Ipfs<P>
where
    P: StoreParams<Hashes = libipld::multihash::Code>,
    P::Codecs: From<libipld::raw::RawCodec>
        + From<libipld::pb::DagPbCodec>
        + From<DagCborCodec>,
    Ipld: References<P::Codecs>,
{
    /// Store an IPLD value (DAG-CBOR) and return its root CID.
    pub fn put_ipld(&self, value: &Ipld) -> anyhow::Result<Cid> {
        let block = Block::encode(DagCborCodec, libipld::multihash::Code::Sha2_256, value)?;
        let cid = *block.cid();
        self.insert(block)?;
        Ok(cid)
    }

    /// Read an IPLD value back.
    pub fn get_ipld(&self, cid: &Cid) -> anyhow::Result<Ipld> {
        let block = self.get(cid)?;
        let ipld: Ipld = DagCborCodec.decode(block.data())?;
        Ok(ipld)
    }

    /// Resolve an `/ipfs/<cid>` (or `/ipfs/<cid>/...` for UnixFS files) path.
    /// Only the root segment and full-file cat are supported in this minimal
    /// implementation; sub-path traversal is returned as an error.
    pub fn get_path(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();
        if parts.is_empty() || (parts[0] != "ipfs" && parts[0] != "ipld") {
            return Err(anyhow::anyhow!("path must start with /ipfs or /ipld"));
        }
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("missing cid"));
        }
        let cid: Cid = parts[1].parse().map_err(|e| anyhow::anyhow!("bad cid: {e}"))?;
        if parts.len() == 2 {
            // Try UnixFS cat first, fall back to raw block.
            match self.cat_unixfs(&cid) {
                Ok(b) => Ok(b),
                Err(_) => Ok(self.get(&cid)?.data().to_vec()),
            }
        } else {
            Err(anyhow::anyhow!("sub-path traversal not implemented"))
        }
    }
}
