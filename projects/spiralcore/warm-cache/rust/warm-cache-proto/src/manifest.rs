//! Page-group manifest over a 4 KiB page Merkle tree.
use crate::hash_parts;
use crate::PageTree;

pub const GROUP_PAGES: usize = 64;

fn build_levels(leaves: &[[u8; 32]]) -> Vec<Vec<[u8; 32]>> {
    if leaves.is_empty() { return vec![vec![[0u8; 32]]]; }
    let mut levels = vec![leaves.to_vec()];
    while levels.last().unwrap().len() > 1 {
        let prev = levels.last().unwrap().clone();
        let mut next = Vec::with_capacity((prev.len() + 1) / 2);
        let mut i = 0;
        while i < prev.len() {
            let left = prev[i];
            let right = if i + 1 < prev.len() { prev[i + 1] } else { prev[i] };
            next.push(hash_parts(&[b"group-node", &left, &right]));
            i += 2;
        }
        levels.push(next);
    }
    levels
}

fn proof_at(levels: &[Vec<[u8; 32]>], index: usize) -> Vec<(bool, [u8; 32])> {
    let mut idx = index;
    let mut out = Vec::new();
    for level in 0..levels.len().saturating_sub(1) {
        let nodes = &levels[level];
        let sib = if idx % 2 == 0 { (idx + 1).min(nodes.len() - 1) } else { idx - 1 };
        out.push((idx % 2 == 0, nodes[sib]));
        idx /= 2;
    }
    out
}

fn verify_proof(leaf: [u8; 32], proof: &[(bool, [u8; 32])], root: &[u8; 32]) -> bool {
    let mut h = leaf;
    for (sibling_right, sib) in proof {
        h = if *sibling_right { hash_parts(&[b"group-node", &h, sib]) } else { hash_parts(&[b"group-node", sib, &h]) };
    }
    h == *root
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedManifest {
    pub version: u32,
    pub page_count: usize,
    pub data_len: usize,
    pub page_roots: Vec<[u8; 32]>,
    pub group_roots: Vec<[u8; 32]>,
    pub root: [u8; 32],
}

impl SeedManifest {
    pub fn from_tree(tree: &PageTree) -> Self {
        let page_roots: Vec<[u8; 32]> = tree.levels[0].clone();
        let mut group_roots = Vec::new();
        for (gi, chunk) in page_roots.chunks(GROUP_PAGES).enumerate() {
            let gi_bytes = (gi as u32).to_le_bytes();
            let mut parts: Vec<&[u8]> = vec![b"page-group", gi_bytes.as_slice()];
            for r in chunk { parts.push(r); }
            group_roots.push(hash_parts(&parts));
        }
        let levels = build_levels(&group_roots);
        let root = *levels.last().unwrap().first().unwrap();
        Self { version: tree.version, page_count: tree.page_count(), data_len: tree.data_len, page_roots, group_roots, root }
    }

    pub fn page_group(&self, page_id: usize) -> usize { page_id / GROUP_PAGES }
    pub fn page_root(&self, page_id: usize) -> Option<[u8; 32]> { self.page_roots.get(page_id).copied() }
    pub fn group_root(&self, group: usize) -> Option<[u8; 32]> { self.group_roots.get(group).copied() }

    pub fn prove_group(&self, group: usize) -> Option<([u8; 32], Vec<(bool, [u8; 32])>)> {
        let root = self.group_root(group)?;
        let levels = build_levels(&self.group_roots);
        Some((root, proof_at(&levels, group)))
    }

    pub fn verify_group(&self, group: usize, group_root: &[u8; 32], proof: &[(bool, [u8; 32])]) -> bool {
        verify_proof(*group_root, proof, &self.root) && self.group_root(group) == Some(*group_root)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"WCM1");
        v.push(1);
        v.extend_from_slice(&self.version.to_le_bytes());
        v.extend_from_slice(&(self.page_count as u32).to_le_bytes());
        v.extend_from_slice(&(self.data_len as u64).to_le_bytes());
        v.extend_from_slice(&(self.group_roots.len() as u32).to_le_bytes());
        for r in &self.page_roots { v.extend_from_slice(r); }
        for r in &self.group_roots { v.extend_from_slice(r); }
        v.extend_from_slice(&self.root);
        v
    }

    pub fn wire_len(&self) -> u64 { self.encode().len() as u64 }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 4 + 1 + 4 + 4 + 8 + 4 + 32 { return Err("short".into()); }
        if &bytes[..4] != b"WCM1" { return Err("magic".into()); }
        if bytes[4] != 1 { return Err("version".into()); }
        let mut i = 5;
        let version = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()); i += 4;
        let page_count = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()) as usize; i += 4;
        let data_len = u64::from_le_bytes(bytes[i..i+8].try_into().unwrap()) as usize; i += 8;
        let groups = u32::from_le_bytes(bytes[i..i+4].try_into().unwrap()) as usize; i += 4;
        let mut page_roots = Vec::with_capacity(page_count);
        for _ in 0..page_count { page_roots.push(bytes[i..i+32].try_into().unwrap()); i += 32; }
        let mut group_roots = Vec::with_capacity(groups);
        for _ in 0..groups { group_roots.push(bytes[i..i+32].try_into().unwrap()); i += 32; }
        let root: [u8; 32] = bytes[i..i+32].try_into().unwrap(); i += 32;
        if i != bytes.len() { return Err("trailing".into()); }
        Ok(Self { version, page_count, data_len, page_roots, group_roots, root })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip_and_groups() {
        let data: Vec<u8> = (0..(3 * 64 * 4096 + 123)).map(|i| (i * 7) as u8).collect();
        let tree = PageTree::new(&data, 4);
        let m = SeedManifest::from_tree(&tree);
        assert_eq!(m.page_count, tree.page_count());
        assert_eq!(m.group_roots.len(), (m.page_count + GROUP_PAGES - 1) / GROUP_PAGES);
        let enc = m.encode();
        assert_eq!(m, SeedManifest::decode(&enc).unwrap());
        for g in 0..m.group_roots.len() {
            let (root, proof) = m.prove_group(g).unwrap();
            assert!(m.verify_group(g, &root, &proof));
            let mut bad = root; bad[0] ^= 1;
            assert!(!m.verify_group(g, &bad, &proof));
        }
    }
}
