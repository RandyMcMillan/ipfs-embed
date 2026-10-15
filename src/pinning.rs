use crate::{Cid, Ipfs};
use libipld::codec::References;
use libipld::store::StoreParams;
use libipld::Ipld;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinMode {
    Direct,
    Recursive,
    Indirect,
}

impl<P: StoreParams> Ipfs<P>
where
    Ipld: References<P::Codecs>,
{
    fn pin_alias_name(cid: &Cid, mode: PinMode) -> Vec<u8> {
        let prefix = match mode {
            PinMode::Direct => "pin-direct:",
            PinMode::Recursive => "pin-recursive:",
            PinMode::Indirect => "pin-indirect:",
        };
        format!("{}{}", prefix, cid).into_bytes()
    }

    /// Pins a CID. Aliases are always recursive in the underlying store, so
    /// `mode` is recorded in the alias name for bookkeeping.
    pub fn pin(&self, cid: &Cid, mode: PinMode) -> anyhow::Result<()> {
        self.alias(Self::pin_alias_name(cid, mode), Some(cid))
    }

    /// Removes all pins for a CID.
    pub fn unpin(&self, cid: &Cid) -> anyhow::Result<()> {
        for mode in [PinMode::Direct, PinMode::Recursive, PinMode::Indirect] {
            let _ = self.alias(Self::pin_alias_name(cid, mode), None);
        }
        Ok(())
    }

    /// Returns true if the CID has at least one alias-derived pin.
    pub fn is_pinned(&self, cid: &Cid) -> anyhow::Result<bool> {
        Ok(self.reverse_alias(cid)?.map(|s| !s.is_empty()).unwrap_or(false))
    }

    /// Lists (cid, mode) for alias-derived pins.
    pub fn list_pins(&self) -> anyhow::Result<Vec<(Cid, PinMode)>> {
        let mut out = Vec::new();
        for (name, cid) in self.aliases()? {
            if let Some(rest) = String::from_utf8_lossy(&name).strip_prefix("pin-") {
                let mode = if rest.starts_with("direct:") {
                    PinMode::Direct
                } else if rest.starts_with("indirect:") {
                    PinMode::Indirect
                } else {
                    PinMode::Recursive
                };
                out.push((cid, mode));
            }
        }
        // de-duplicate by cid keeping first
        let mut seen = std::collections::HashSet::new();
        out.retain(|(c, _)| seen.insert(*c));
        Ok(out)
    }

    /// Run garbage collection over unpinned blocks.
    pub fn gc(&self) -> impl std::future::Future<Output = anyhow::Result<()>> {
        self.evict()
    }
}
