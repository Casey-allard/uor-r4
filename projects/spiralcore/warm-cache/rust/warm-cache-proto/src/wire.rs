//! Versioned warm-plan wire frame for the local prototype.
use crate::{Plan, TimeHeader, WarmFrame};

pub const MAGIC: &[u8; 4] = b"WCF1";
pub const VERSION: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarmWire {
    pub seed_id: [u8; 32],
    pub seed_version: u32,
    pub plan_kind: u8,
    pub header: TimeHeader,
    pub content: [u8; 32],
    pub witness: [u8; 32],
    pub payload: Vec<u8>,
}

impl WarmWire {
    pub fn from_frame(frame: &WarmFrame) -> Self {
        let plan_kind = match frame.plan {
            Plan::Identity => 0,
            Plan::Patch(_) => 1,
            Plan::PageRepair(_) => 2,
        };
        Self {
            seed_id: frame.seed_id,
            seed_version: frame.version,
            plan_kind,
            header: frame.header.clone(),
            content: frame.content,
            witness: frame.witness,
            payload: frame.plan.payload(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(MAGIC);
        v.push(VERSION);
        v.extend_from_slice(&self.seed_id);
        v.extend_from_slice(&self.seed_version.to_le_bytes());
        v.push(self.plan_kind);
        v.extend_from_slice(&self.header.encode());
        v.extend_from_slice(&self.content);
        v.extend_from_slice(&self.witness);
        v.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        v.extend_from_slice(&self.payload);
        v
    }

    pub fn wire_len(&self) -> u64 { self.encode().len() as u64 }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let fixed = 4 + 1 + 32 + 4 + 1 + 24 + 32 + 32 + 4;
        if bytes.len() < fixed { return Err("short".into()); }
        if &bytes[..4] != MAGIC { return Err("magic".into()); }
        if bytes[4] != VERSION { return Err("version".into()); }
        let mut i = 5;
        let seed_id = bytes[i..i+32].try_into().unwrap(); i += 32;
        let seed_version = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()); i += 4;
        let plan_kind = bytes[i]; i += 1;
        let header = TimeHeader::decode(&bytes[i..i+24])?; i += 24;
        let content = bytes[i..i+32].try_into().unwrap(); i += 32;
        let witness = bytes[i..i+32].try_into().unwrap(); i += 32;
        let plen = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()) as usize; i += 4;
        if bytes.len() != i + plen { return Err("payload length".into()); }
        let payload = bytes[i..].to_vec();
        Ok(Self { seed_id, seed_version, plan_kind, header, content, witness, payload })
    }

    pub fn to_frame(&self) -> Result<WarmFrame, String> {
        let plan = match self.plan_kind {
            0 => {
                if !self.payload.is_empty() { return Err("identity payload".into()); }
                Plan::Identity
            }
            1 => {
                if self.payload.len() % 5 != 0 { return Err("patch length".into()); }
                let mut p = Vec::new();
                let mut i = 0;
                while i < self.payload.len() {
                    let idx = u32::from_le_bytes(self.payload[i..i+4].try_into().unwrap()) as usize;
                    p.push((idx, self.payload[i+4]));
                    i += 5;
                }
                Plan::Patch(p)
            }
            2 => {
                let mut p = Vec::new();
                let mut i = 0;
                while i < self.payload.len() {
                    if i + 8 > self.payload.len() { return Err("page header".into()); }
                    let page_id = u32::from_le_bytes(self.payload[i..i+4].try_into().unwrap());
                    let len = u32::from_le_bytes(self.payload[i+4..i+8].try_into().unwrap()) as usize;
                    i += 8;
                    if i + len > self.payload.len() { return Err("page body".into()); }
                    p.push((page_id, self.payload[i..i+len].to_vec()));
                    i += len;
                }
                Plan::PageRepair(p)
            }
            _ => return Err("plan kind".into()),
        };
        Ok(WarmFrame {
            seed_id: self.seed_id,
            version: self.seed_version,
            plan,
            header: self.header.clone(),
            content: self.content,
            witness: self.witness,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupRepairWire {
    pub seed_id: [u8; 32],
    pub seed_version: u32,
    pub group_id: u32,
    pub group_root: [u8; 32],
    pub group_proof: Vec<(bool, [u8; 32])>,
    pub header: TimeHeader,
    pub content: [u8; 32],
    pub witness: [u8; 32],
    pub pages: Vec<(u32, Vec<u8>)>,
}

impl GroupRepairWire {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"WGR1");
        v.push(1);
        v.extend_from_slice(&self.seed_id);
        v.extend_from_slice(&self.seed_version.to_le_bytes());
        v.extend_from_slice(&self.group_id.to_le_bytes());
        v.extend_from_slice(&self.group_root);
        v.extend_from_slice(&(self.group_proof.len() as u32).to_le_bytes());
        for (right, h) in &self.group_proof {
            v.push(if *right { 1 } else { 0 });
            v.extend_from_slice(h);
        }
        v.extend_from_slice(&self.header.encode());
        v.extend_from_slice(&self.content);
        v.extend_from_slice(&self.witness);
        v.extend_from_slice(&(self.pages.len() as u32).to_le_bytes());
        for (page_id, page) in &self.pages {
            v.extend_from_slice(&page_id.to_le_bytes());
            v.extend_from_slice(&(page.len() as u32).to_le_bytes());
            v.extend_from_slice(page);
        }
        v
    }

    pub fn wire_len(&self) -> u64 { self.encode().len() as u64 }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 4 + 1 + 32 + 4 + 4 + 32 + 4 + 24 + 32 + 32 + 4 { return Err("short".into()); }
        if &bytes[..4] != b"WGR1" { return Err("magic".into()); }
        if bytes[4] != 1 { return Err("version".into()); }
        let mut i = 5;
        let seed_id = bytes[i..i+32].try_into().unwrap(); i += 32;
        let seed_version = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()); i += 4;
        let group_id = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()); i += 4;
        let group_root = bytes[i..i+32].try_into().unwrap(); i += 32;
        let plen = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()) as usize; i += 4;
        let mut group_proof = Vec::with_capacity(plen);
        for _ in 0..plen {
            let right = bytes[i] != 0; i += 1;
            group_proof.push((right, bytes[i..i+32].try_into().unwrap())); i += 32;
        }
        let header = TimeHeader::decode(&bytes[i..i+24])?; i += 24;
        let content = bytes[i..i+32].try_into().unwrap(); i += 32;
        let witness = bytes[i..i+32].try_into().unwrap(); i += 32;
        let page_count = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()) as usize; i += 4;
        let mut pages = Vec::with_capacity(page_count);
        for _ in 0..page_count {
            if i + 8 > bytes.len() { return Err("page header".into()); }
            let page_id = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()); i += 4;
            let len = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()) as usize; i += 4;
            if i + len > bytes.len() { return Err("page body".into()); }
            pages.push((page_id, bytes[i..i+len].to_vec())); i += len;
        }
        if i != bytes.len() { return Err("trailing".into()); }
        Ok(Self { seed_id, seed_version, group_id, group_root, group_proof, header, content, witness, pages })
    }

    pub fn to_frame(&self) -> WarmFrame {
        WarmFrame {
            seed_id: self.seed_id,
            version: self.seed_version,
            plan: Plan::PageRepair(self.pages.clone()),
            header: self.header.clone(),
            content: self.content,
            witness: self.witness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{make_warm, receive_warm, KuramotoCell, PHASE_BITS, Seed, TimeHeader, WarmResult};

    fn header() -> TimeHeader {
        let mut cell = KuramotoCell::new(211, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot())
    }

    #[test]
    fn identity_wire_roundtrip() {
        let seed = Seed::new(vec![3u8; 1 << 16], 1);
        let frame = make_warm(&seed, Plan::Identity, header());
        let wire = WarmWire::from_frame(&frame);
        let enc = wire.encode();
        let back = WarmWire::decode(&enc).unwrap();
        assert_eq!(wire, back);
        assert_eq!(back.to_frame().unwrap(), frame);
        assert_eq!(wire.wire_len(), 134);
    }

    #[test]
    fn patch_and_page_repair_wire_roundtrip() {
        let seed = Seed::new(vec![4u8; 1 << 16], 1);
        let frame = make_warm(&seed, Plan::Patch(vec![(10, 1), (20, 2)]), header());
        let wire = WarmWire::from_frame(&frame);
        let back = WarmWire::decode(&wire.encode()).unwrap();
        assert_eq!(back.to_frame().unwrap(), frame);

        let page = vec![9u8; crate::PAGE];
        let frame = make_warm(&seed, Plan::PageRepair(vec![(1, page)]), header());
        let wire = WarmWire::from_frame(&frame);
        let back = WarmWire::decode(&wire.encode()).unwrap();
        assert_eq!(back.to_frame().unwrap(), frame);
    }

    #[test]
    fn bad_magic_and_payload_length_are_rejected() {
        let seed = Seed::new(vec![5u8; 4096], 1);
        let frame = make_warm(&seed, Plan::Identity, header());
        let mut enc = WarmWire::from_frame(&frame).encode();
        enc[0] = b'X';
        assert!(WarmWire::decode(&enc).is_err());
        let mut enc = WarmWire::from_frame(&frame).encode();
        enc[5 + 32 + 4 + 1 + 24 + 32 + 32] ^= 1; // payload length byte
        assert!(WarmWire::decode(&enc).is_err());
    }

    #[test]
    fn group_repair_frame_contains_and_verifies_group_proof() {
        use crate::manifest::SeedManifest;
        let data: Vec<u8> = (0..(1 << 20)).map(|i| (i * 23 + 9) as u8).collect();
        let seed = Seed::new(data.clone(), 1);
        let manifest = SeedManifest::from_tree(&seed.tree);
        let group = 1u32;
        let (group_root, proof) = manifest.prove_group(group as usize).unwrap();
        assert!(manifest.verify_group(group as usize, &group_root, &proof));
        let page_id = 70u32;
        let mut target_page = data[page_id as usize * crate::PAGE..(page_id as usize + 1) * crate::PAGE].to_vec();
        target_page[0] ^= 0xAA;
        let h = header();
        let frame = make_warm(&seed, Plan::PageRepair(vec![(page_id, target_page.clone())]), h.clone());
        let wire = GroupRepairWire {
            seed_id: seed.id, seed_version: seed.version, group_id: group,
            group_root, group_proof: proof, header: h.clone(),
            content: frame.content, witness: frame.witness,
            pages: vec![(page_id, target_page)],
        };
        let enc = wire.encode();
        let back = GroupRepairWire::decode(&enc).unwrap();
        assert_eq!(wire, back);
        assert!(manifest.verify_group(group as usize, &back.group_root, &back.group_proof));
        let frame2 = back.to_frame();
        assert_eq!(receive_warm(&seed, &h, &frame2), WarmResult::Committed);
    }
}
