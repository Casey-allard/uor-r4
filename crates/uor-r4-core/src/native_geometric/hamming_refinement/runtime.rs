//! A bounded sequence of query/read/update hops over a frozen committed frontier.
use super::metric::{Metric, MetricError};
use crate::native_geometric::addressed_attention::{
    artifact::BoundGeometry,
    objects::{Lease, ObjectError, ObjectSession, Reference},
};

pub const MAX_HOPS: usize = 4;
/// Owns a geometry-bound committed object store. Revision tracks actual input
/// bytes/roots/keys and turn changes, not merely the selected records.
#[derive(Clone, Debug)]
pub struct Memory {
    objects: ObjectSession,
    geometry_digest: [u8; 32],
    revision: [u8; 32],
}
impl Memory {
    pub fn new(model: [u8; 32], geometry: &BoundGeometry, epoch: u64) -> Self {
        let mut h = blake3::Hasher::new();
        h.update(b"uor-hamming-memory/1");
        h.update(&model);
        h.update(&geometry.identity_digest());
        h.update(&epoch.to_le_bytes());
        Self {
            objects: ObjectSession::new(model, geometry.identity_digest(), epoch),
            geometry_digest: geometry.identity_digest(),
            revision: *h.finalize().as_bytes(),
        }
    }
    pub fn import_object_snapshot(
        bytes: &[u8],
        model: [u8; 32],
        geometry: &BoundGeometry,
    ) -> Result<Self> {
        let objects = ObjectSession::restore(bytes, model, geometry.identity_digest())?;
        Ok(Self {
            objects,
            geometry_digest: geometry.identity_digest(),
            revision: *blake3::hash(bytes).as_bytes(),
        })
    }
    pub fn objects(&self) -> &ObjectSession {
        &self.objects
    }
    pub fn revision(&self) -> [u8; 32] {
        self.revision
    }
    pub fn observe_input(&mut self, byte: u8, keys: [u16; 2], roots: [u16; 4]) -> Result<()> {
        roots_valid(&keys)?;
        roots_valid(&roots)?;
        self.objects.observe_input(byte, keys, roots)?;
        let mut h = blake3::Hasher::new();
        h.update(b"uor-hamming-memory-observe/1");
        h.update(&self.revision);
        h.update(&[byte]);
        root_bytes(&mut h, &keys);
        root_bytes(&mut h, &roots);
        self.revision = *h.finalize().as_bytes();
        Ok(())
    }
    pub fn begin_turn(&mut self) -> Result<()> {
        self.objects.begin_turn()?;
        let mut h = blake3::Hasher::new();
        h.update(b"uor-hamming-memory-turn/1");
        h.update(&self.revision);
        h.update(&self.objects.turn().to_le_bytes());
        self.revision = *h.finalize().as_bytes();
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Metric(MetricError),
    Object(ObjectError),
    Invalid(&'static str),
    Policy(&'static str),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<MetricError> for Error {
    fn from(e: MetricError) -> Self {
        Self::Metric(e)
    }
}
impl From<ObjectError> for Error {
    fn from(e: ObjectError) -> Self {
        Self::Object(e)
    }
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Query {
    pub roots: [u16; 2],
    pub null: [u16; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hop {
    pub round: u8,
    pub query: Query,
    pub selected: Option<Lease>,
    pub distance: u16,
    pub candidate_scores: u16,
    pub before: [u16; 4],
    pub delta: [u16; 4],
    pub after: [u16; 4],
    pub parent_digest: [u8; 32],
    pub digest: [u8; 32],
}
/// No target or unobserved record is part of this interface. Previous selected
/// payloads remain exact owned leases, not one scalar or first-root summaries.
pub struct Context<'a> {
    pub round: u8,
    pub initial_roots: [u16; 4],
    pub roots: [u16; 4],
    pub history: &'a [Option<Hop>],
    pub epoch: u64,
    pub turn: u64,
    pub frontier: u64,
}
pub struct SelectedContext<'a> {
    pub context: Context<'a>,
    pub query: Query,
    pub selected: Lease,
}
/// A host can implement a learned shared policy here. The finite tests use an
/// explicitly authored policy and do not constitute fitted model behavior.
pub trait Policy {
    fn identity(&self) -> [u8; 32];
    fn query(&mut self, context: &Context<'_>) -> Result<Query>;
    /// Return a length in 1..=64 with legal[length-1] true.
    fn extent(&mut self, context: &SelectedContext<'_>, legal: &[bool; 64]) -> Result<u8>;
    /// All selected roots and exact bytes are accessible before this update.
    fn delta(&mut self, context: &SelectedContext<'_>) -> Result<[u16; 4]>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Refinement {
    pub initial_roots: [u16; 4],
    pub roots: [u16; 4],
    pub hops: [Option<Hop>; MAX_HOPS],
    pub hop_count: u8,
    pub candidate_scores: u16,
    pub geometry_digest: [u8; 32],
    pub policy_digest: [u8; 32],
    pub digest: [u8; 32],
    pub epoch: u64,
    pub turn: u64,
    pub frontier: u64,
}
fn roots_valid(roots: &[u16]) -> Result<()> {
    if roots.iter().any(|&r| r >= 120) {
        Err(Error::Invalid("root domain"))
    } else {
        Ok(())
    }
}
fn root_bytes(h: &mut blake3::Hasher, roots: &[u16]) {
    for r in roots {
        h.update(&r.to_le_bytes());
    }
}
fn reference_bytes(h: &mut blake3::Hasher, reference: Reference) {
    match reference {
        Reference::Occurrence {
            epoch,
            turn,
            start,
            end,
        } => {
            h.update(&[0]);
            for x in [epoch, turn, start, end] {
                h.update(&x.to_le_bytes());
            }
        }
        Reference::Result { epoch, id } => {
            h.update(&[1]);
            h.update(&epoch.to_le_bytes());
            h.update(&id.to_le_bytes());
        }
    }
}
fn hop_digest(hop: &Hop) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"uor-hamming-refinement-hop/1");
    h.update(&hop.parent_digest);
    h.update(&[hop.round]);
    root_bytes(&mut h, &hop.query.roots);
    root_bytes(&mut h, &hop.query.null);
    h.update(&hop.distance.to_le_bytes());
    h.update(&hop.candidate_scores.to_le_bytes());
    if let Some(lease) = hop.selected {
        h.update(&[1]);
        reference_bytes(&mut h, lease.reference());
        root_bytes(&mut h, &lease.keys());
        root_bytes(&mut h, &lease.roots());
        h.update(&[lease.payload().len() as u8]);
        h.update(lease.payload());
    } else {
        h.update(&[0]);
    }
    root_bytes(&mut h, &hop.before);
    root_bytes(&mut h, &hop.delta);
    root_bytes(&mut h, &hop.after);
    *h.finalize().as_bytes()
}
/// Read-only on committed objects. Fixed hop cap prevents a hidden unbounded
/// recursive scan; each hop scores at most 256+8+Null candidates. There is no
/// semantic shortlist/page index yet: candidate admission is this bounded store.
pub fn refine(
    memory: &Memory,
    geometry: &BoundGeometry,
    metric: &Metric,
    initial_roots: [u16; 4],
    hops: u8,
    policy: &mut impl Policy,
) -> Result<Refinement> {
    let objects = memory.objects();
    if memory.geometry_digest != geometry.identity_digest() {
        return Err(Error::Invalid("memory geometry binding"));
    }
    if hops == 0 || usize::from(hops) > MAX_HOPS {
        return Err(Error::Invalid("hop limit"));
    }
    roots_valid(&initial_roots)?;
    if metric.geometry_digest() != geometry.identity_digest() {
        return Err(Error::Invalid("metric geometry binding"));
    }
    if objects.key_pending() || objects.pending().is_some() {
        return Err(Error::Invalid("uncommitted frontier"));
    }
    let policy_digest = policy.identity();
    let mut h = blake3::Hasher::new();
    h.update(b"uor-hamming-refinement-start/1");
    h.update(&geometry.identity_digest());
    h.update(&policy_digest);
    h.update(&memory.revision());
    h.update(&[hops]);
    for x in [objects.epoch(), objects.turn(), objects.frontier()] {
        h.update(&x.to_le_bytes());
    }
    root_bytes(&mut h, &initial_roots);
    let mut result = Refinement {
        initial_roots,
        roots: initial_roots,
        hops: [None; MAX_HOPS],
        hop_count: hops,
        candidate_scores: 0,
        geometry_digest: geometry.identity_digest(),
        policy_digest,
        digest: *h.finalize().as_bytes(),
        epoch: objects.epoch(),
        turn: objects.turn(),
        frontier: objects.frontier(),
    };
    for round in 0..hops {
        let context = Context {
            round,
            initial_roots,
            roots: result.roots,
            history: &result.hops[..usize::from(round)],
            epoch: result.epoch,
            turn: result.turn,
            frontier: result.frontier,
        };
        let query = policy.query(&context)?;
        roots_valid(&query.roots)?;
        roots_valid(&query.null)?;
        let mut distance = metric.distance(query.roots, query.null)?;
        let mut candidate = None;
        let mut scores = 1u16;
        // Strict improvement gives Null first, then full occurrence order, then
        // result order. No digest bits or expected-answer labels enter scoring.
        for sequence in result.frontier.saturating_sub(256)..result.frontier {
            let r = objects.occurrence(result.epoch, sequence)?;
            let d = metric.distance(query.roots, r.keys)?;
            scores += 1;
            if d < distance {
                distance = d;
                candidate = Some(Reference::Occurrence {
                    epoch: r.epoch,
                    turn: r.turn,
                    start: r.sequence,
                    end: r.sequence + 1,
                });
            }
        }
        for id in objects.result_frontier().saturating_sub(8).max(1)..objects.result_frontier() {
            let r = objects.result(result.epoch, id)?;
            let d = metric.distance(query.roots, r.lease.keys())?;
            scores += 1;
            if d < distance {
                distance = d;
                candidate = Some(Reference::Result {
                    epoch: result.epoch,
                    id,
                });
            }
        }
        let mut delta = [geometry.identity(); 4];
        let mut selected = None;
        if let Some(reference) = candidate {
            let mut legal = [false; 64];
            let first = match reference {
                Reference::Occurrence { epoch, start, .. } => {
                    let max = objects.maximum_extent(epoch, start)?;
                    legal[..usize::from(max)].fill(true);
                    objects.acquire_occurrence(epoch, start, 1)?
                }
                Reference::Result { epoch, id } => {
                    let l = objects.acquire_result(epoch, id)?;
                    legal[l.payload().len() - 1] = true;
                    l
                }
            };
            let selected_context = SelectedContext {
                context,
                query,
                selected: first,
            };
            let length = policy.extent(&selected_context, &legal)?;
            if length == 0 || length > 64 || !legal[usize::from(length - 1)] {
                return Err(Error::Invalid("extent"));
            }
            let lease = match reference {
                Reference::Occurrence { epoch, start, .. } => {
                    objects.acquire_occurrence(epoch, start, length)?
                }
                Reference::Result { .. } => first,
            };
            let selected_context = SelectedContext {
                selected: lease,
                ..selected_context
            };
            delta = policy.delta(&selected_context)?;
            roots_valid(&delta)?;
            selected = Some(lease);
        }
        let before = result.roots;
        for lane in 0..4 {
            result.roots[lane] = geometry
                .product(before[lane], delta[lane])
                .map_err(|_| Error::Invalid("group composition"))?;
        }
        let mut hop = Hop {
            round,
            query,
            selected,
            distance,
            candidate_scores: scores,
            before,
            delta,
            after: result.roots,
            parent_digest: result.digest,
            digest: [0; 32],
        };
        hop.digest = hop_digest(&hop);
        result.digest = hop.digest;
        result.candidate_scores += scores;
        result.hops[usize::from(round)] = Some(hop);
    }
    Ok(result)
}
