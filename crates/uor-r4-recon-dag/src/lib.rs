//! Production reader for the reconstruction byte-DAG.
//!
//! **What this replaces.** The experiment at
//! `archive/labs/spiralcore-native-attention-review-2026-10-14/evidence/cs/range_reader.rs` reads the whole
//! records file into a `HashMap<Id, Node>` before answering any range query, and its own README says so:
//! `whole_object_sha256_in_rust=NOT_CHECKED`. Two things follow, and they are the two halves of this crate:
//!
//! 1. **An on-demand backend.** The records file is a fixed-stride sequence of `1-byte scale || 12-byte id
//!    || 32-or-36-byte data`. The on-disk header is 13 bytes per node, not the resident index cost:
//!    a `HashMap<Id, offset>` also stores its keys, offsets, buckets and allocator overhead.
//!    `open()` transiently reads the whole file to index those headers; subsequent traversals fetch and
//!    authenticate only visited records. No bounded-memory indexing claim is made.
//! 2. **Two digest claims that are NOT the same claim.** A node id authenticates **the nodes a range
//!    touched** ([`Verification::RangeVerified`]); it says nothing about the nodes outside the range, and
//!    nothing about the whole object. Only a whole-object digest ([`Verification::WholeObject`]) can carry
//!    integrity for the reconstruction as a whole.
//!
//! **The owner's rule shapes the API: on the main track a read must be whole-object verified.** So a
//! selective read returns a [`Read`] whose [`Verification`] states which claim it actually supports, and
//! [`Read::whole_object_verified`] is the only thing a main-track consumer may treat as integrity. A
//! range-verified read reports its **unverified remainder** rather than letting the weaker claim pass as the
//! stronger one.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A node id: the crate's own 12-byte digest over `scale || data`.
pub type Id = [u8; 12];

/// One stored node: its scale (0 is a leaf) and its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub scale: u8,
    pub data: Vec<u8>,
}

/// Every failure this reader reports. A malformed file is never a panic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Format,
    Allocation,
    Range,
    Missing(Id),
    Scale,
    /// A node's id does not match its bytes: the DAG's own integrity check.
    Integrity,
    Padding,
    /// The reconstructed bytes do not match the declared WHOLE-OBJECT digest. This is the claim a node id
    /// cannot make.
    WholeObject,
    Io(String),
}

/// The span, in bytes, that a node at `scale` covers: `32 * 3^scale`.
pub fn span(scale: u8) -> Result<usize, Error> {
    32usize
        .checked_mul(3usize.checked_pow(scale as u32).ok_or(Error::Format)?)
        .ok_or(Error::Format)
}

// --- the node digest, ported verbatim so the ids are the same ids -------------------------------

const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];
const SIG: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

#[allow(clippy::too_many_arguments)]
fn mix(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

/// The node id: a 12-byte digest over `scale || data`, identical to the experiment's `id_for`.
///
/// **This is not the whole-object digest.** It authenticates one node's bytes; the object those nodes
/// reconstruct is checked separately, by [`whole_object_digest`].
/// Refuses malformed payloads/scales before accessing the fixed digest buffer.
pub fn id_for(node: &Node) -> Result<Id, Error> {
    span(node.scale)?;
    if node.data.len() != data_size(node.scale) {
        return Err(Error::Format);
    }
    let mut bytes = [0u8; 128];
    bytes[0] = node.scale;
    bytes[1..1 + node.data.len()].copy_from_slice(&node.data);
    let mut m = [0u64; 16];
    for (i, c) in bytes.chunks_exact(8).enumerate() {
        m[i] = u64::from_le_bytes(c.try_into().unwrap_or([0u8; 8]));
    }
    let mut h = IV;
    h[0] ^= 0x0101000c;
    let mut v = [0u64; 16];
    v[..8].copy_from_slice(&h);
    v[8..].copy_from_slice(&IV);
    v[12] ^= (node.data.len() + 1) as u64;
    v[14] = !v[14];
    for r in 0..12 {
        let s = &SIG[r % 10];
        mix(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        mix(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        mix(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        mix(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);
        mix(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        mix(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        mix(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        mix(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }
    for i in 0..8 {
        h[i] ^= v[i] ^ v[i + 8];
    }
    let mut out = [0u8; 12];
    out[..8].copy_from_slice(&h[0].to_le_bytes());
    out[8..].copy_from_slice(&h[1].to_le_bytes()[..4]);
    Ok(out)
}

/// The whole-object digest of reconstructed bytes, or of the source object they came from.
///
/// **The claim a node id cannot make.** `id_for` authenticates the nodes a traversal visited; this
/// authenticates the object. A main-track consumer needs this one.
pub fn whole_object_digest(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}
// --- the manifest -------------------------------------------------------------------------------

/// A parsed manifest: the object's logical length and its root nodes, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub len: usize,
    pub roots: Vec<(Id, u8)>,
}

fn parse_id(s: &str) -> Result<Id, Error> {
    if s.len() != 24 || !s.is_ascii() {
        return Err(Error::Format);
    }
    let mut id = [0u8; 12];
    for (i, x) in id.iter_mut().enumerate() {
        *x = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| Error::Format)?;
    }
    Ok(id)
}

/// Parse a manifest's text. The scale ladder is DERIVED from the atom count and every root must match it,
/// so a manifest cannot declare a shape the records could not hold.
pub fn parse_manifest(text: &str) -> Result<Manifest, Error> {
    let mut it = text.split_whitespace();
    let len = it
        .next()
        .ok_or(Error::Format)?
        .parse::<usize>()
        .map_err(|_| Error::Format)?;
    let atoms = it
        .next()
        .ok_or(Error::Format)?
        .parse::<usize>()
        .map_err(|_| Error::Format)?;
    let nr = it
        .next()
        .ok_or(Error::Format)?
        .parse::<usize>()
        .map_err(|_| Error::Format)?;
    if atoms != len.div_ceil(32) {
        return Err(Error::Format);
    }
    let mut scales = vec![];
    let mut n = atoms;
    let mut s = 0u8;
    while n > 0 {
        for _ in 0..n % 3 {
            scales.push(s)
        }
        n /= 3;
        s += 1;
    }
    scales.reverse();
    if nr != scales.len() {
        return Err(Error::Format);
    }
    let mut roots = vec![];
    for expected in scales {
        let id = parse_id(it.next().ok_or(Error::Format)?)?;
        let scale = it
            .next()
            .ok_or(Error::Format)?
            .parse::<u8>()
            .map_err(|_| Error::Format)?;
        if scale != expected {
            return Err(Error::Format);
        }
        span(scale)?;
        roots.push((id, scale));
    }
    if it.next().is_some() {
        return Err(Error::Format);
    }
    Ok(Manifest { len, roots })
}

// --- backends: where node bytes come from -------------------------------------------------------

/// The size of a record's data, given its scale byte: a leaf carries 32 bytes, a parent 3 child ids.
pub fn data_size(scale: u8) -> usize {
    if scale == 0 {
        32
    } else {
        36
    }
}

/// Where a traversal gets node bytes.
///
/// **The trait is the point.** The eager implementation is the reference the experiment used; the indexed
/// one is the production path. Both answer the same question, so a range read can be compared between them
/// byte for byte rather than trusted.
pub trait Backend {
    fn node(&self, id: Id) -> Result<Node, Error>;
    /// How many nodes the backend could serve at all - the denominator for an unverified remainder.
    fn stored(&self) -> usize;
}

/// The experiment's backend: every node resident. Kept as the reference for comparison.
#[derive(Clone, Debug, Default)]
pub struct EagerBackend {
    pub nodes: HashMap<Id, Node>,
}

impl Backend for EagerBackend {
    fn node(&self, id: Id) -> Result<Node, Error> {
        self.nodes.get(&id).cloned().ok_or(Error::Missing(id))
    }
    fn stored(&self) -> usize {
        self.nodes.len()
    }
}

/// The production backend retains a header-derived index and fetches node data during traversal.
///
/// The on-disk header is 13 bytes; resident index cost includes HashMap entries and overhead.
/// Opening the backend transiently reads the whole records file. It does not retain node payloads,
/// unlike [`EagerBackend`], but no low peak-memory or 13-byte resident-index claim follows.
pub struct IndexedBackend {
    path: PathBuf,
    index: HashMap<Id, u64>,
}

impl IndexedBackend {
    /// Read the file transiently, then retain only the header-derived index.
    pub fn open(records_path: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(records_path).map_err(|e| Error::Io(e.to_string()))?;
        let mut index = HashMap::new();
        let mut at = 0usize;
        while at < bytes.len() {
            let scale = bytes[at];
            if scale > 40 {
                return Err(Error::Format);
            }
            let size = data_size(scale);
            if bytes.len() - at < 13 + size {
                return Err(Error::Format);
            }
            let id: Id = bytes[at + 1..at + 13]
                .try_into()
                .map_err(|_| Error::Format)?;
            // A duplicate id is a malformed store: two records claim the same identity.
            if index.insert(id, (at + 13) as u64).is_some() {
                return Err(Error::Format);
            }
            at += 13 + size;
        }
        Ok(Self {
            path: records_path.to_path_buf(),
            index,
        })
    }

    /// How many nodes are indexed.
    pub fn indexed(&self) -> usize {
        self.index.len()
    }
}

impl Backend for IndexedBackend {
    fn node(&self, id: Id) -> Result<Node, Error> {
        use std::io::{Read as _, Seek, SeekFrom};
        let at = *self.index.get(&id).ok_or(Error::Missing(id))?;
        let mut f = std::fs::File::open(&self.path).map_err(|e| Error::Io(e.to_string()))?;
        f.seek(SeekFrom::Start(at - 13))
            .map_err(|e| Error::Io(e.to_string()))?;
        let mut head = [0u8; 13];
        f.read_exact(&mut head)
            .map_err(|e| Error::Io(e.to_string()))?;
        let scale = head[0];
        let mut data = vec![0u8; data_size(scale)];
        f.read_exact(&mut data)
            .map_err(|e| Error::Io(e.to_string()))?;
        Ok(Node { scale, data })
    }
    fn stored(&self) -> usize {
        self.index.len()
    }
}
// --- the traversal, and what it proves ----------------------------------------------------------

/// What a read actually establishes.
///
/// **These are different claims, and the type keeps them apart.** A node id authenticates the nodes a
/// traversal touched. It says nothing about the nodes outside the requested range, and nothing about the
/// object. Only a whole-object digest can carry integrity for the reconstruction as a whole.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verification {
    /// Every node the traversal TOUCHED matched its own id. Nodes outside the range were not authenticated and are
    /// NOT covered - `unverified_nodes` counts them. **Not object integrity.**
    RangeVerified {
        nodes: usize,
        unverified_nodes: usize,
    },
    /// The reconstructed object matched a caller-declared whole-object digest. The only claim a main-track
    /// consumer may treat as integrity.
    WholeObject { digest: [u8; 32], nodes: usize },
}

impl Verification {
    /// Whether this supports an integrity claim about the OBJECT rather than about the nodes read.
    pub fn is_whole_object(&self) -> bool {
        matches!(self, Verification::WholeObject { .. })
    }

    /// One line naming the claim and, when it is the weaker one, what is left unverified.
    pub fn line(&self) -> String {
        match self {
            Verification::WholeObject { nodes, .. } => format!(
                "whole-object verified against the declared digest ({nodes} node(s) authenticated)"
            ),
            Verification::RangeVerified {
                nodes,
                unverified_nodes,
            } => format!(
                "range-verified only: {nodes} node(s) authenticated, {unverified_nodes} node(s) in the store were NOT authenticated by this traversal and are not covered"
            ),
        }
    }
}

/// One traversal's result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    pub bytes: Vec<u8>,
    pub verification: Verification,
    pub visits: usize,
    pub leaves: usize,
}

impl Read {
    /// **The only thing a main-track consumer may treat as integrity.**
    ///
    /// The owner's rule: on the main track a read must always be verified. A `RangeVerified` read is a
    /// property of the bytes fetched, so it cannot stand in for the object; this method is how a caller
    /// asks the question the rule actually requires.
    pub fn whole_object_verified(&self) -> bool {
        self.verification.is_whole_object()
    }
}

#[derive(Default)]
struct Traversal {
    visits: usize,
    leaves: usize,
    verified: std::collections::HashSet<Id>,
}

#[allow(clippy::too_many_arguments)]
fn visit(
    id: Id,
    scale: u8,
    start: usize,
    a: usize,
    b: usize,
    len: usize,
    backend: &dyn Backend,
    out: &mut Vec<u8>,
    t: &mut Traversal,
) -> Result<(), Error> {
    let end = start.checked_add(span(scale)?).ok_or(Error::Format)?;
    if b <= start || a >= end {
        return Ok(());
    }
    let node = backend.node(id)?;
    if node.scale != scale {
        return Err(Error::Scale);
    }
    // Backend is public: malformed nodes must refuse, including on repeated visits.
    if node.data.len() != data_size(scale) {
        return Err(Error::Format);
    }
    t.visits += 1;
    if id_for(&node)? != id {
        return Err(Error::Integrity);
    }
    t.verified.insert(id);
    if scale == 0 {
        t.leaves += 1;
        let lo = a.saturating_sub(start);
        let hi = (b - start).min(32);
        if end > len && node.data[len - start..].iter().any(|&x| x != 0) {
            return Err(Error::Padding);
        }
        out.extend_from_slice(&node.data[lo..hi]);
    } else {
        let step = span(scale - 1)?;
        for j in 0..3 {
            let child: Id = node.data[j * 12..j * 12 + 12]
                .try_into()
                .map_err(|_| Error::Format)?;
            visit(
                child,
                scale - 1,
                start + j * step,
                a,
                b,
                len,
                backend,
                out,
                t,
            )?;
        }
    }
    Ok(())
}

fn walk(
    manifest: &Manifest,
    backend: &dyn Backend,
    a: usize,
    b: usize,
) -> Result<(Vec<u8>, Traversal), Error> {
    if a > b || b > manifest.len {
        return Err(Error::Range);
    }
    let mut out = Vec::new();
    out.try_reserve_exact(b - a)
        .map_err(|_| Error::Allocation)?;
    let mut t = Traversal::default();
    if a == b {
        return Ok((out, t));
    }
    let mut at = 0usize;
    for &(id, scale) in &manifest.roots {
        visit(id, scale, at, a, b, manifest.len, backend, &mut out, &mut t)?;
        at = at.checked_add(span(scale)?).ok_or(Error::Format)?;
    }
    if out.len() != b - a {
        return Err(Error::Format);
    }
    Ok((out, t))
}

/// Read `a..b`, reporting the **weaker** claim: the nodes this range touched, and how many it did not.
pub fn read_range(
    manifest: &Manifest,
    backend: &dyn Backend,
    a: usize,
    b: usize,
) -> Result<Read, Error> {
    let (bytes, t) = walk(manifest, backend, a, b)?;
    let verified = t.verified.len();
    Ok(Read {
        bytes,
        verification: Verification::RangeVerified {
            nodes: verified,
            unverified_nodes: backend.stored().saturating_sub(verified),
        },
        visits: t.visits,
        leaves: t.leaves,
    })
}

/// Read the whole object and check it against `expected`.
///
/// This is the claim the experiment printed as `NOT_CHECKED`. It is the one a main-track consumer needs,
/// and it is deliberately a separate entry point rather than a flag on [`read_range`]: a caller cannot reach
/// the strong claim by forgetting to ask for the weak one.
pub fn read_whole_object(
    manifest: &Manifest,
    backend: &dyn Backend,
    expected: [u8; 32],
) -> Result<Read, Error> {
    let (bytes, t) = walk(manifest, backend, 0, manifest.len)?;
    let digest = whole_object_digest(&bytes);
    if digest != expected {
        return Err(Error::WholeObject);
    }
    Ok(Read {
        bytes,
        verification: Verification::WholeObject {
            digest,
            nodes: t.verified.len(),
        },
        visits: t.visits,
        leaves: t.leaves,
    })
}
