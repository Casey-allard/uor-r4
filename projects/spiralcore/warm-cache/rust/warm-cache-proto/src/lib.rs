//! Minimal four-oscillator Kuramoto time + warm-cache network prototype.
//!
//! This is a local research prototype, not a repository delivery.

use sha2::{Digest, Sha256};

pub mod manifest;
pub mod replay;
pub mod wire;
pub mod calibration;
pub mod mux;
pub mod session;
pub mod certs;

pub use session::{WarmSession, WarmStats};

pub const N: usize = 4;
pub const PHASE_BITS: u8 = 10;

pub fn hash_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update((p.len() as u64).to_le_bytes());
        h.update(p);
    }
    h.finalize().into()
}

fn wrap(x: f64) -> f64 {
    let mut y = x % (2.0 * std::f64::consts::PI);
    if y < 0.0 { y += 2.0 * std::f64::consts::PI; }
    y
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub phases: [f64; N],
    pub r: f64,
    pub psi: f64,
    pub epoch: u64,
    pub phase_q: u32,
}

pub struct KuramotoCell {
    pub phases: [f64; N],
    pub omega: [f64; N],
    pub k: f64,
    pub phase_bits: u8,
    last_psi: f64,
    turns: f64,
}

impl KuramotoCell {
    pub fn new(seed: u64, phase_bits: u8) -> Self {
        let mut x = seed | 1;
        let mut phases = [0.0f64; N];
        for p in phases.iter_mut() {
            x ^= x << 13; x ^= x >> 7; x ^= x << 17;
            *p = (x as f64 / u64::MAX as f64) * 2.0 * std::f64::consts::PI;
        }
        Self {
            phases,
            omega: [
                2.0 * std::f64::consts::PI * 1.0,
                2.0 * std::f64::consts::PI * 1.05,
                2.0 * std::f64::consts::PI * 0.95,
                2.0 * std::f64::consts::PI * 1.02,
            ],
            k: 20.0,
            phase_bits,
            last_psi: 0.0,
            turns: 0.0,
        }
    }

    fn mean(&self) -> (f64, f64) {
        let mut c = 0.0;
        let mut s = 0.0;
        for &t in &self.phases { c += t.cos(); s += t.sin(); }
        let n = N as f64;
        (c / n, s / n)
    }

    fn deriv(p: &[f64; N], omega: &[f64; N], k: f64) -> [f64; N] {
        let mut out = [0.0; N];
        for i in 0..N {
            let mut inter = 0.0;
            for j in 0..N { inter += (p[j] - p[i]).sin(); }
            out[i] = omega[i] + (k / N as f64) * inter;
        }
        out
    }

    pub fn step(&mut self, dt: f64) {
        let k1 = Self::deriv(&self.phases, &self.omega, self.k);
        let mut p2 = [0.0; N];
        for i in 0..N { p2[i] = self.phases[i] + 0.5 * dt * k1[i]; }
        let k2 = Self::deriv(&p2, &self.omega, self.k);
        let mut p3 = [0.0; N];
        for i in 0..N { p3[i] = self.phases[i] + 0.5 * dt * k2[i]; }
        let k3 = Self::deriv(&p3, &self.omega, self.k);
        let mut p4 = [0.0; N];
        for i in 0..N { p4[i] = self.phases[i] + dt * k3[i]; }
        let k4 = Self::deriv(&p4, &self.omega, self.k);
        for i in 0..N {
            self.phases[i] = (self.phases[i] + dt * (k1[i] + 2.0*k2[i] + 2.0*k3[i] + k4[i]) / 6.0)
                % (2.0 * std::f64::consts::PI);
        }
        let (c, s) = self.mean();
        let psi = s.atan2(c);
        let delta = (psi - self.last_psi + std::f64::consts::PI)
            .rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI;
        self.turns += delta / (2.0 * std::f64::consts::PI);
        self.last_psi = psi;
    }

    pub fn turns(&self) -> f64 { self.turns }

    pub fn phase_spread(&self) -> f64 {
        let mut spread = 0.0f64;
        for i in 0..N {
            for j in (i + 1)..N {
                let mut d = (self.phases[i] - self.phases[j]) % (2.0 * std::f64::consts::PI);
                if d > std::f64::consts::PI { d -= 2.0 * std::f64::consts::PI; }
                if d < -std::f64::consts::PI { d += 2.0 * std::f64::consts::PI; }
                spread = spread.max(d.abs());
            }
        }
        spread
    }

    pub fn snapshot(&self) -> Snapshot {
        let (c, s) = self.mean();
        let r = (c*c + s*s).sqrt();
        let psi = s.atan2(c);
        let mask = (1u32 << self.phase_bits) - 1;
        let phase_q = (((wrap(psi) / (2.0 * std::f64::consts::PI)) * (1u32 << self.phase_bits) as f64)
            as u32) & mask;
        Snapshot {
            phases: self.phases,
            r,
            psi,
            epoch: self.turns.floor().max(0.0) as u64,
            phase_q,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeHeader {
    pub level: u8,
    pub pass: u8,
    pub epoch: u64,
    pub phase_q: u32,
    pub coherence_q: u8,
    pub parent_epoch: u64,
    pub phase_bits: u8,
}

impl TimeHeader {
    pub fn from_snapshot(level: u8, pass: u8, parent_epoch: u64, s: &Snapshot) -> Self {
        Self {
            level, pass, epoch: s.epoch,
            phase_q: s.phase_q,
            coherence_q: (s.r * 255.0).round().clamp(0.0, 255.0) as u8,
            parent_epoch, phase_bits: PHASE_BITS,
        }
    }

    pub fn from_calibrated(level: u8, pass: u8, parent_epoch: u64, s: &Snapshot,
                             cal: &crate::calibration::TimeCalibration) -> Self {
        let bins = 1u32 << cal.phase_bits;
        let q = (((wrap(s.psi) / (2.0 * std::f64::consts::PI)) * bins as f64) as u32) & (bins - 1);
        Self {
            level, pass, epoch: s.epoch, phase_q: q,
            coherence_q: (s.r * 255.0).round().clamp(0.0, 255.0) as u8,
            parent_epoch, phase_bits: cal.phase_bits,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(24);
        v.push(self.level);
        v.push(self.pass);
        v.extend_from_slice(&self.epoch.to_le_bytes());
        v.extend_from_slice(&self.phase_q.to_le_bytes());
        v.push(self.coherence_q);
        v.extend_from_slice(&self.parent_epoch.to_le_bytes());
        v.push(self.phase_bits);
        v
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 24 { return Err("header length".into()); }
        Ok(Self {
            level: bytes[0],
            pass: bytes[1],
            epoch: u64::from_le_bytes(bytes[2..10].try_into().unwrap()),
            phase_q: u32::from_le_bytes(bytes[10..14].try_into().unwrap()),
            coherence_q: bytes[14],
            parent_epoch: u64::from_le_bytes(bytes[15..23].try_into().unwrap()),
            phase_bits: bytes[23],
        })
    }

    pub fn logical_tick(&self) -> u64 {
        self.epoch * (1u64 << self.phase_bits) + self.phase_q as u64
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roots {
    pub content: [u8; 32],
    pub temporal: [u8; 32],
    pub witness: [u8; 32],
}

pub fn content_root(page_id: u32, version: u32, bytes: &[u8]) -> [u8; 32] {
    hash_parts(&[b"content", &page_id.to_le_bytes(), &version.to_le_bytes(), bytes])
}

pub fn temporal_root(content: &[u8; 32], header: &TimeHeader, parent_time_id: &[u8; 32]) -> [u8; 32] {
    hash_parts(&[b"temporal", &header.encode(), content, parent_time_id])
}

pub fn witness(temporal: &[u8; 32], header: &TimeHeader, parent_time_id: &[u8; 32]) -> [u8; 32] {
    hash_parts(&[b"witness", temporal, &header.encode(), parent_time_id])
}

pub fn paired_witness(w_forward: &[u8; 32], w_reverse: &[u8; 32], super_epoch: u64) -> [u8; 32] {
    hash_parts(&[b"pair", w_forward, w_reverse, &super_epoch.to_le_bytes()])
}

pub fn roots(page_id: u32, version: u32, bytes: &[u8], header: &TimeHeader) -> Roots {
    let parent = [0u8; 32];
    let c = content_root(page_id, version, bytes);
    let t = temporal_root(&c, header, &parent);
    let w = witness(&t, header, &parent);
    Roots { content: c, temporal: t, witness: w }
}

pub fn replay_error(forward: &[[f64; N]], reverse: &[[f64; N]]) -> f64 {
    assert_eq!(forward.len(), reverse.len());
    let mut worst = 0.0f64;
    let n = forward.len();
    for i in 0..n {
        let f = &forward[n - 1 - i];
        let r = &reverse[i];
        for k in 0..N {
            let mut d = (f[k] - r[k]) % (2.0 * std::f64::consts::PI);
            if d > std::f64::consts::PI { d -= 2.0 * std::f64::consts::PI; }
            if d < -std::f64::consts::PI { d += 2.0 * std::f64::consts::PI; }
            worst = worst.max(d.abs());
        }
    }
    worst
}

pub const PAGE: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageTree {
    pub version: u32,
    pub data_len: usize,
    pub pages: Vec<Vec<u8>>,
    pub levels: Vec<Vec<[u8; 32]>>,
}

impl PageTree {
    fn leaf_hash(page_id: u32, version: u32, page: &[u8]) -> [u8; 32] {
        hash_parts(&[b"page", &page_id.to_le_bytes(), &version.to_le_bytes(), page])
    }

    pub fn new(data: &[u8], version: u32) -> Self {
        let mut pages = Vec::new();
        if data.is_empty() {
            pages.push(vec![0u8; PAGE]);
        } else {
            for chunk in data.chunks(PAGE) {
                let mut p = chunk.to_vec();
                p.resize(PAGE, 0);
                pages.push(p);
            }
        }
        let leaves: Vec<[u8; 32]> = pages.iter().enumerate()
            .map(|(i, p)| Self::leaf_hash(i as u32, version, p))
            .collect();
        let mut levels = vec![leaves];
        while levels.last().unwrap().len() > 1 {
            let prev = levels.last().unwrap().clone();
            let mut next = Vec::with_capacity((prev.len() + 1) / 2);
            let mut i = 0;
            while i < prev.len() {
                let left = prev[i];
                let right = if i + 1 < prev.len() { prev[i + 1] } else { prev[i] };
                next.push(hash_parts(&[b"node", &left, &right]));
                i += 2;
            }
            levels.push(next);
        }
        Self { version, data_len: data.len(), pages, levels }
    }

    pub fn root(&self) -> [u8; 32] { *self.levels.last().unwrap().first().unwrap() }

    pub fn page_count(&self) -> usize { self.pages.len() }

    pub fn page(&self, i: usize) -> &[u8] { &self.pages[i] }

    pub fn proof(&self, index: usize) -> Vec<(bool, [u8; 32])> {
        let mut idx = index.min(self.pages.len() - 1);
        let mut out = Vec::new();
        for level in 0..self.levels.len().saturating_sub(1) {
            let nodes = &self.levels[level];
            let sib = if idx % 2 == 0 { (idx + 1).min(nodes.len() - 1) } else { idx - 1 };
            out.push((idx % 2 == 0, nodes[sib]));
            idx /= 2;
        }
        out
    }

    pub fn verify_page(&self, index: usize, page: &[u8], proof: &[(bool, [u8; 32])], root: &[u8; 32]) -> bool {
        let mut h = Self::leaf_hash(index as u32, self.version, page);
        for (sibling_right, sib) in proof {
            h = if *sibling_right {
                hash_parts(&[b"node", &h, sib])
            } else {
                hash_parts(&[b"node", sib, &h])
            };
        }
        h == *root
    }

    pub fn diff(&self, other: &PageTree) -> Vec<usize> {
        let n = self.page_count().max(other.page_count());
        let mut out = Vec::new();
        for i in 0..n {
            let a = self.pages.get(i).map(|p| &p[..]).unwrap_or(&[]);
            let b = other.pages.get(i).map(|p| &p[..]).unwrap_or(&[]);
            if a != b { out.push(i); }
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seed {
    pub id: [u8; 32],
    pub version: u32,
    pub bytes: Vec<u8>,
    pub tree: PageTree,
}

impl Seed {
    pub fn new(bytes: Vec<u8>, version: u32) -> Self {
        let tree = PageTree::new(&bytes, version);
        let id = tree.root();
        Self { id, version, bytes, tree }
    }
    pub fn cold_wire_len(&self) -> u64 {
        4 + 32 + self.bytes.len() as u64
    }
    pub fn page_count(&self) -> usize { self.tree.page_count() }
    pub fn page_root(&self, i: usize) -> [u8; 32] { self.tree.levels[0][i] }
    pub fn page_proof(&self, i: usize) -> Vec<(bool, [u8; 32])> { self.tree.proof(i) }
    pub fn verify_page(&self, i: usize, page: &[u8], proof: &[(bool, [u8; 32])]) -> bool {
        self.tree.verify_page(i, page, proof, &self.tree.root())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plan {
    Identity,
    Patch(Vec<(usize, u8)>),
    PageRepair(Vec<(u32, Vec<u8>)>),
}

impl Plan {
    pub fn apply(&self, seed: &[u8]) -> Vec<u8> {
        let mut out = seed.to_vec();
        match self {
            Plan::Identity => {}
            Plan::Patch(p) => {
                for &(i, b) in p { out[i] = b; }
            }
            Plan::PageRepair(p) => {
                for (page_id, page) in p {
                    let start = *page_id as usize * PAGE;
                    if start >= out.len() { continue; }
                    let end = (start + PAGE).min(out.len());
                    let n = (end - start).min(page.len());
                    out[start..start + n].copy_from_slice(&page[..n]);
                }
            }
        }
        out
    }
    pub fn payload(&self) -> Vec<u8> {
        match self {
            Plan::Identity => Vec::new(),
            Plan::Patch(p) => {
                let mut out = Vec::with_capacity(p.len() * 5);
                for &(i, b) in p {
                    out.extend_from_slice(&(i as u32).to_le_bytes());
                    out.push(b);
                }
                out
            }
            Plan::PageRepair(p) => {
                let mut out = Vec::new();
                for (page_id, page) in p {
                    out.extend_from_slice(&page_id.to_le_bytes());
                    out.extend_from_slice(&(page.len() as u32).to_le_bytes());
                    out.extend_from_slice(page);
                }
                out
            }
        }
    }
    pub fn payload_len(&self) -> u64 { self.payload().len() as u64 }
    pub fn apply_from_wire(&self, seed: &[u8]) -> Vec<u8> { self.apply(seed) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarmFrame {
    pub seed_id: [u8; 32],
    pub version: u32,
    pub plan: Plan,
    pub header: TimeHeader,
    pub content: [u8; 32],
    pub witness: [u8; 32],
}

impl WarmFrame {
    pub fn wire_len(&self) -> u64 {
        32 + 4 + self.plan.payload_len() + self.header.encode().len() as u64 + 32 + 32
    }
}

pub fn make_warm(seed: &Seed, plan: Plan, header: TimeHeader) -> WarmFrame {
    let target = plan.apply(&seed.bytes);
    let c = PageTree::new(&target, seed.version).root();
    let t = temporal_root(&c, &header, &[0u8; 32]);
    let w = witness(&t, &header, &[0u8; 32]);
    WarmFrame { seed_id: seed.id, version: seed.version, plan, header, content: c, witness: w }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WarmResult {
    Committed,
    SeedMismatch,
    ClockMismatch,
    ContentMismatch,
    WitnessMismatch,
}

pub fn receive_warm(seed: &Seed, local_header: &TimeHeader, frame: &WarmFrame) -> WarmResult {
    if seed.id != frame.seed_id || seed.version != frame.version { return WarmResult::SeedMismatch; }
    if local_header != &frame.header { return WarmResult::ClockMismatch; }
    let target = frame.plan.apply(&seed.bytes);
    let c = PageTree::new(&target, seed.version).root();
    if c != frame.content { return WarmResult::ContentMismatch; }
    let t = temporal_root(&c, &frame.header, &[0u8; 32]);
    let w = witness(&t, &frame.header, &[0u8; 32]);
    if w != frame.witness { return WarmResult::WitnessMismatch; }
    WarmResult::Committed
}

pub fn record_forward(mut cell: KuramotoCell, seconds: f64, dt: f64) -> (KuramotoCell, Vec<[f64; N]>) {
    let steps = (seconds / dt).round() as usize;
    let mut frames = Vec::with_capacity(steps);
    for _ in 0..steps {
        cell.step(dt);
        frames.push(cell.phases);
    }
    (cell, frames)
}

pub fn replay(frames: &[[f64; N]]) -> Vec<[f64; N]> {
    frames.iter().rev().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_oscillators_synchronize() {
        let mut cell = KuramotoCell::new(7, PHASE_BITS);
        for _ in 0..5000 { cell.step(0.001); }
        let s = cell.snapshot();
        assert!(s.r > 0.95, "R = {}", s.r);
        assert!(s.epoch >= 1, "epoch = {}", s.epoch);
        assert!(s.phase_q < (1u32 << PHASE_BITS), "phase_q = {}", s.phase_q);
    }

    #[test]
    fn replay_is_exact() {
        let cell = KuramotoCell::new(11, PHASE_BITS);
        let (_after, fwd) = record_forward(cell, 2.0, 0.001);
        let rev = replay(&fwd);
        let err = replay_error(&fwd, &rev);
        assert!(err < 1e-15, "replay error {}", err);
    }

    #[test]
    fn headers_are_forward_monotonic() {
        let mut cell = KuramotoCell::new(13, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let h1 = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
        assert_eq!(h1.encode().len(), 24);
        for _ in 0..1000 { cell.step(0.001); }
        let h2 = TimeHeader::from_snapshot(0, 1, 0, &cell.snapshot());
        assert!(h2.logical_tick() > h1.logical_tick());
    }

    #[test]
    fn content_and_temporal_roots_split() {
        let mut cell = KuramotoCell::new(17, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let h1 = TimeHeader::from_snapshot(0, 0, 0, &s);
        let h2 = TimeHeader::from_snapshot(0, 1, 0, &s);
        let a = roots(0, 1, b"payload", &h1);
        let b = roots(0, 1, b"payload", &h2);
        assert_eq!(a.content, b.content);
        assert_ne!(a.temporal, b.temporal);
        assert_ne!(a.witness, b.witness);
    }

    #[test]
    fn identical_warm_is_zero_payload() {
        let seed = Seed::new(vec![7u8; 4096], 1);
        let mut cell = KuramotoCell::new(19, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let h = TimeHeader::from_snapshot(0, 0, 0, &s);
        let frame = make_warm(&seed, Plan::Identity, h.clone());
        assert_eq!(frame.plan.payload_len(), 0);
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
        assert!(frame.wire_len() < 200, "warm wire {}", frame.wire_len());
    }

    #[test]
    fn patch_warm_moves_only_corrections() {
        let seed = Seed::new(vec![3u8; 4096], 1);
        let mut cell = KuramotoCell::new(23, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let h = TimeHeader::from_snapshot(0, 0, 0, &s);
        let patches = vec![(100usize, 9u8), (200usize, 8u8), (300usize, 7u8)];
        let frame = make_warm(&seed, Plan::Patch(patches), h.clone());
        assert_eq!(frame.plan.payload_len(), 15);
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
        assert!(frame.wire_len() < 200, "warm wire {}", frame.wire_len());
    }

    #[test]
    fn stale_clock_is_clock_mismatch_not_content() {
        let seed = Seed::new(vec![1u8; 4096], 1);
        let mut cell = KuramotoCell::new(29, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let h_sender = TimeHeader::from_snapshot(0, 0, 0, &s);
        let mut h_local = h_sender.clone();
        h_local.epoch += 1;
        let frame = make_warm(&seed, Plan::Identity, h_sender);
        assert_eq!(receive_warm(&seed, &h_local, &frame), WarmResult::ClockMismatch);
    }

    #[test]
    fn content_mismatch_is_detected() {
        let seed = Seed::new(vec![2u8; 4096], 1);
        let mut cell = KuramotoCell::new(31, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let h = TimeHeader::from_snapshot(0, 0, 0, &s);
        let mut frame = make_warm(&seed, Plan::Identity, h.clone());
        frame.content[0] ^= 1;
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::ContentMismatch);
    }

    #[test]
    fn one_cold_many_warm() {
        let seed = Seed::new(vec![5u8; 1 << 16], 1);
        let cold = seed.cold_wire_len();
        let mut cell = KuramotoCell::new(37, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let h = TimeHeader::from_snapshot(0, 0, 0, &s);
        let frame = make_warm(&seed, Plan::Identity, h.clone());
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
        assert_eq!(cold, 4 + 32 + (1 << 16));
        assert_eq!(frame.plan.payload_len(), 0);
    }

    #[test]
    fn paired_witness_separates_pass() {
        let mut cell = KuramotoCell::new(41, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let f = TimeHeader::from_snapshot(0, 0, 0, &s);
        let r = TimeHeader::from_snapshot(0, 1, 0, &s);
        let a = roots(0, 1, b"same-payload", &f);
        let b = roots(0, 1, b"same-payload", &r);
        assert_eq!(a.content, b.content);
        assert_ne!(a.witness, b.witness);
        assert_ne!(paired_witness(&a.witness, &b.witness, 0), a.witness);
    }

    #[test]
    fn seed_mismatch_is_detected() {
        let seed = Seed::new(vec![1u8; 64], 1);
        let other = Seed::new(vec![2u8; 64], 1);
        let mut cell = KuramotoCell::new(43, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
        let frame = make_warm(&seed, Plan::Identity, h.clone());
        assert_eq!(receive_warm(&other, &h, &frame), WarmResult::SeedMismatch);
    }

    #[test]
    fn forward_reverse_forward_reverse_replay_is_exact() {
        let cell = KuramotoCell::new(47, PHASE_BITS);
        let (_after, fwd) = record_forward(cell, 1.0, 0.001);
        let rev = replay(&fwd);
        let fwd2 = replay(&rev);
        assert!(replay_error(&fwd, &rev) < 1e-15);
        assert_eq!(fwd, fwd2);
    }

    #[test]
    fn one_cold_is_much_larger_than_two_warm_frames() {
        let seed = Seed::new(vec![5u8; 1 << 16], 1);
        let mut cell = KuramotoCell::new(53, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
        let frame = make_warm(&seed, Plan::Identity, h.clone());
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
        let warm2 = 2 * frame.wire_len();
        assert!(warm2 * 100 < seed.cold_wire_len(), "warm2={} cold={}", warm2, seed.cold_wire_len());
        assert_eq!(frame.plan.payload_len(), 0);
    }

    #[test]
    fn page_merkle_proofs_verify_and_tamper_fails() {
        let mut data = vec![0u8; 3 * PAGE + 123];
        for (i, b) in data.iter_mut().enumerate() { *b = (i * 31 + 7) as u8; }
        let seed = Seed::new(data, 1);
        assert_eq!(seed.page_count(), 4);
        for i in 0..seed.page_count() {
            let page = seed.tree.page(i).to_vec();
            let proof = seed.page_proof(i);
            assert!(seed.verify_page(i, &page, &proof));
            let mut bad = page.clone();
            bad[0] ^= 1;
            assert!(!seed.verify_page(i, &bad, &proof));
        }
    }

    #[test]
    fn page_repair_moves_one_page_not_the_seed() {
        let seed_bytes: Vec<u8> = (0..(1 << 20)).map(|i| (i * 13 + 5) as u8).collect();
        let seed = Seed::new(seed_bytes.clone(), 1);
        let mut cell = KuramotoCell::new(59, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
        let page_id = 7usize;
        let mut target = seed_bytes.clone();
        target[page_id * PAGE + 100] ^= 0xAA;
        let new_page = target[page_id * PAGE..(page_id + 1) * PAGE].to_vec();
        let frame = make_warm(&seed, Plan::PageRepair(vec![(page_id as u32, new_page)]), h.clone());
        assert_eq!(frame.plan.payload_len(), 4 + 4 + PAGE as u64);
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
        assert!(frame.wire_len() * 100 < seed.cold_wire_len());
    }

    #[test]
    fn warm_frame_size_is_constant_across_seed_sizes() {
        let mut cell = KuramotoCell::new(61, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
        let sizes = [4 * 1024usize, 64 * 1024, 256 * 1024, 1 << 20, 4 << 20];
        let mut warm_lens = Vec::new();
        for &size in &sizes {
            let bytes: Vec<u8> = (0..size).map(|i| (i * 17 + 3) as u8).collect();
            let seed = Seed::new(bytes, 1);
            let frame = make_warm(&seed, Plan::Identity, h.clone());
            assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
            assert_eq!(frame.plan.payload_len(), 0);
            warm_lens.push(frame.wire_len());
        }
        assert!(warm_lens.iter().all(|&x| x == warm_lens[0]), "warm lens {:?}", warm_lens);
    }

    #[test]
    fn multi_page_repair_moves_only_changed_pages() {
        let seed_bytes: Vec<u8> = (0..(1 << 20)).map(|i| (i * 19 + 11) as u8).collect();
        let seed = Seed::new(seed_bytes.clone(), 1);
        let mut cell = KuramotoCell::new(67, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
        let mut target = seed_bytes.clone();
        let mut pages = Vec::new();
        for page_id in [2usize, 5] {
            target[page_id * PAGE + 7] ^= 0x55;
            pages.push((page_id as u32, target[page_id * PAGE..(page_id + 1) * PAGE].to_vec()));
        }
        let frame = make_warm(&seed, Plan::PageRepair(pages), h.clone());
        assert_eq!(frame.plan.payload_len(), 2 * (8 + PAGE as u64));
        assert_eq!(receive_warm(&seed, &h, &frame), WarmResult::Committed);
        assert!(frame.wire_len() < seed.cold_wire_len() / 10);
    }

    #[test]
    fn calibrated_header_uses_derived_phase_bits() {
        let mut cell = KuramotoCell::new(73, PHASE_BITS);
        for _ in 0..2000 { cell.step(0.001); }
        let s = cell.snapshot();
        let cal = crate::calibration::TimeCalibration::default_bell(0.1259, 0.0);
        let h = TimeHeader::from_calibrated(0, 0, 0, &s, &cal);
        assert_eq!(h.phase_bits, cal.phase_bits);
        assert!(h.phase_q < (1u32 << cal.phase_bits));
        assert_eq!(h.encode().len(), 24);
    }
}