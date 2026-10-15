use crate::{Block, Cid, Ipfs};
use libp2p::identity::Keypair;
use libp2p::identity::PublicKey;
use libipld::cbor::DagCborCodec;
use libipld::codec::{Codec, References};
use libipld::{Ipld, store::StoreParams};
use std::collections::BTreeMap;

impl<P> Ipfs<P>
where
    P: StoreParams<Hashes = libipld::multihash::Code>,
    P::Codecs: From<DagCborCodec>,
    Ipld: References<P::Codecs>,
{
    /// Publish (locally) a signed IPNS-style record pointing `keypair`'s name at `value`.
    /// Returns the CID of the stored record block.
    pub fn ipns_publish(
        &self,
        keypair: &Keypair,
        sequence: u64,
        value: &Cid,
    ) -> anyhow::Result<Cid> {
        let pubkey = keypair.public();
        let peer_id = pubkey.to_peer_id();
        let name = peer_id.to_bytes();
        let mut payload = Vec::new();
        payload.extend_from_slice(&name);
        payload.extend_from_slice(value.to_string().as_bytes());
        payload.extend_from_slice(&sequence.to_be_bytes());
        let sig = keypair
            .sign(&payload)
            .map_err(|e| anyhow::anyhow!("sign failed: {e}"))?;

        let mut record = BTreeMap::new();
        record.insert("name".to_string(), Ipld::Bytes(name));
        record.insert("value".to_string(), Ipld::String(value.to_string()));
        record.insert("pubkey".to_string(), Ipld::Bytes(pubkey.to_protobuf_encoding()));
        record.insert("sequence".to_string(), Ipld::Integer(sequence as i128));
        record.insert("signature".to_string(), Ipld::Bytes(sig));
        let ipld = Ipld::Map(record);

        let block = Block::encode(DagCborCodec, libipld::multihash::Code::Sha2_256, &ipld)?;
        let cid = *block.cid();
        self.insert(block)?;
        Ok(cid)
    }

    /// Resolve a previously published record block, verifying its signature.
    pub fn ipns_resolve(&self, record_cid: &Cid) -> anyhow::Result<Cid> {
        let block = self.get(record_cid)?;
        let ipld: Ipld = DagCborCodec.decode(block.data())?;
        let map = match ipld {
            Ipld::Map(m) => m,
            _ => return Err(anyhow::anyhow!("invalid record")),
        };

        let name = match map.get("name") {
            Some(Ipld::Bytes(b)) => b.clone(),
            _ => return Err(anyhow::anyhow!("missing name")),
        };
        let value_str = match map.get("value") {
            Some(Ipld::String(s)) => s.clone(),
            _ => return Err(anyhow::anyhow!("missing value")),
        };
        let pubkey_bytes = match map.get("pubkey") {
            Some(Ipld::Bytes(b)) => b.clone(),
            _ => return Err(anyhow::anyhow!("missing pubkey")),
        };
        let sequence = match map.get("sequence") {
            Some(Ipld::Integer(n)) => *n as u64,
            _ => return Err(anyhow::anyhow!("missing sequence")),
        };
        let signature = match map.get("signature") {
            Some(Ipld::Bytes(b)) => b.clone(),
            _ => return Err(anyhow::anyhow!("missing signature")),
        };

        let pubkey = PublicKey::from_protobuf_encoding(&pubkey_bytes)
            .map_err(|e| anyhow::anyhow!("bad pubkey: {e}"))?;

        let mut payload = Vec::new();
        payload.extend_from_slice(&name);
        payload.extend_from_slice(value_str.as_bytes());
        payload.extend_from_slice(&sequence.to_be_bytes());
        if !pubkey.verify(&payload, &signature) {
            return Err(anyhow::anyhow!("invalid signature"));
        }

        let peer = pubkey.to_peer_id();
        if peer.to_bytes() != name {
            return Err(anyhow::anyhow!("name/pubkey mismatch"));
        }

        value_str.parse::<Cid>().map_err(|e| anyhow::anyhow!("bad cid: {e}"))
    }
}
