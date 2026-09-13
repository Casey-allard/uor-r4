//! Two causal refinement hops feed typed control and byte/EOS emission.
//! Snapshot imports and transactional clones allocate bounded host memory. This
//! integration does not claim the frozen R4G1 allocation-free serving contract.
use super::{
    artifact::Artifact,
    policy::{Environment, Fields, Phase, Policy},
};
use crate::native_geometric::{
    addressed_attention::{
        artifact::BoundGeometry,
        objects::{Action, Lease, ObjectSession, Offer, Symbol},
    },
    hamming_refinement::{
        metric::Metric,
        runtime::{self, Memory, Refinement},
    },
};
#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Host(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn host<T>(r: std::result::Result<T, impl std::fmt::Display>) -> Result<T> {
    r.map_err(|e| Error::Host(e.to_string()))
}
const ACTIONS: [Action; 7] = [
    Action::Hold,
    Action::AcquireA,
    Action::AcquireB,
    Action::AddAB,
    Action::SubAB,
    Action::Advance,
    Action::Clear,
];
pub fn argmax(scores: &[u8], legal: &[bool]) -> Result<usize> {
    if scores.len() != legal.len() {
        return Err(Error::Invalid("score domain"));
    }
    let mut best = None;
    for (i, (&s, &l)) in scores.iter().zip(legal).enumerate() {
        if l && best.is_none_or(|b| s > scores[b]) {
            best = Some(i);
        }
    }
    best.ok_or(Error::Invalid("empty legal domain"))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prediction {
    pub offer: Offer,
    pub refinement: Refinement,
    pub scores: [u8; 257],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub refinement: Refinement,
    pub roots: [u16; 4],
    pub keys: Option<[u16; 2]>,
    pub actual: Symbol,
    pub result_frontier: u64,
}
#[derive(Clone, Debug)]
pub struct Session {
    policy_id: Option<[u8; 32]>,
    model_id: [u8; 32],
    parameter_id: [u8; 32],
    geometry_id: [u8; 32],
    objects: ObjectSession,
    memory: Memory,
    roots: [u16; 4],
    phases: [u16; 4],
    last_byte: Option<u8>,
    turn_start: u64,
    ended: bool,
    pending: Option<Prediction>,
    pub snapshot_imports: u64,
    pub snapshot_bytes: u64,
}
impl Session {
    pub fn new(a: &Artifact, g: &BoundGeometry, epoch: u64) -> Result<Self> {
        if a.geometry_digest() != g.identity_digest() {
            return Err(Error::Invalid("artifact geometry binding"));
        }
        Ok(Self {
            policy_id: None,
            model_id: a.id(),
            parameter_id: a.params().digest(),
            geometry_id: g.identity_digest(),
            objects: ObjectSession::new(a.id(), g.identity_digest(), epoch),
            memory: Memory::new(a.id(), g, epoch),
            roots: [g.identity(); 4],
            phases: [0; 4],
            last_byte: None,
            turn_start: 0,
            ended: false,
            pending: None,
            snapshot_imports: 0,
            snapshot_bytes: 0,
        })
    }
    pub fn objects(&self) -> &ObjectSession {
        &self.objects
    }
    pub fn roots(&self) -> [u16; 4] {
        self.roots
    }
    pub fn ended(&self) -> bool {
        self.ended
    }
    fn binding(&mut self, g: &BoundGeometry, p: &Policy<'_>) -> Result<()> {
        if self.geometry_id != g.identity_digest()
            || self.parameter_id != p.params().digest()
            || p.geometry_digest() != self.geometry_id
            || self.policy_id.is_some_and(|id| id != p.identity())
        {
            return Err(Error::Invalid("model/geometry/parameter binding"));
        }
        self.policy_id = Some(p.identity());
        Ok(())
    }
    fn environment(&self) -> Environment {
        Environment {
            last_observed: self.last_byte,
            zeta_bins: self.phases.map(|x| (x >> 12) as u8),
            position: self
                .objects
                .frontier()
                .saturating_sub(self.turn_start)
                .min(255) as u8,
            initial_roots: self.roots,
        }
    }
    fn fields(&self, r: &Refinement, selected: [Option<Lease>; 2]) -> Fields {
        Fields {
            initial_roots: r.initial_roots,
            roots: r.roots,
            selected: selected[0],
            previous: selected[1],
            active: self.objects.active().copied(),
            last_observed: self.last_byte,
            zeta_bins: self.phases.map(|x| (x >> 12) as u8),
            position: self
                .objects
                .frontier()
                .saturating_sub(self.turn_start)
                .min(255) as u8,
            round: 2,
            numeric_valid: [false; 2],
        }
    }
    fn refresh(&mut self, g: &BoundGeometry) -> Result<()> {
        let bytes = host(self.objects.snapshot())?;
        self.memory = host(Memory::import_object_snapshot(&bytes, self.model_id, g))?;
        self.snapshot_imports = self
            .snapshot_imports
            .checked_add(1)
            .ok_or(Error::Invalid("snapshot counter exhausted"))?;
        self.snapshot_bytes = self
            .snapshot_bytes
            .checked_add(bytes.len() as u64)
            .ok_or(Error::Invalid("snapshot counter exhausted"))?;
        Ok(())
    }
    fn refinement(&self, g: &BoundGeometry, m: &Metric, p: &mut Policy<'_>) -> Result<Refinement> {
        p.environment = self.environment();
        host(runtime::refine(&self.memory, g, m, self.roots, 2, p))
    }
    pub fn predict(
        &mut self,
        g: &BoundGeometry,
        m: &Metric,
        p: &mut Policy<'_>,
    ) -> Result<Prediction> {
        self.binding(g, p)?;
        if let Some(x) = &self.pending {
            return Ok(x.clone());
        }
        if self.ended {
            return Err(Error::Invalid("response ended"));
        }
        let r = self.refinement(g, m, p)?;
        let selected = operands(&r);
        let prepared = host(self.objects.prepare_operands(selected[0], selected[1]))?;
        let mut f = self.fields(&r, selected);
        f.numeric_valid = prepared.numeric_valid();
        let control = host(p.evaluate(&f, Phase::Control))?;
        let action = ACTIONS[argmax(&control.control_scores, &prepared.legal())?];
        let prospective = host(self.objects.prepare_action(prepared, action))?;
        f.active = prospective.active().copied();
        let emission = host(p.evaluate(&f, Phase::Emit))?;
        let i = argmax(&emission.emission_scores, &[true; 257])?;
        let symbol = if i == 256 {
            Symbol::Eos
        } else {
            Symbol::Byte(i as u8)
        };
        let offer = host(self.objects.cache_prepared_offer(symbol, prospective))?;
        let prediction = Prediction {
            offer,
            refinement: r,
            scores: emission.emission_scores,
        };
        self.pending = Some(prediction.clone());
        Ok(prediction)
    }
    // Refinement depends only on the committed prefix and is retained even when
    // a teacher observes a different byte. Speculative actions/leases still obey
    // ObjectSession acknowledgement; no wrong offered result is published.
    pub fn observe(
        &mut self,
        actual: Symbol,
        id: u64,
        g: &BoundGeometry,
        p: &mut Policy<'_>,
    ) -> Result<Observation> {
        self.binding(g, p)?;
        let offered = self
            .pending
            .as_ref()
            .filter(|x| x.offer.id == id && self.objects.pending() == Some(&x.offer))
            .ok_or(Error::Invalid("unknown offer"))?;
        let r = offered.refinement;
        let mut next = self.clone();
        host(next.objects.acknowledge(actual, id))?;
        next.pending = None;
        next.roots = r.roots;
        let keys = match actual {
            Symbol::Byte(b) => Some(next.commit_byte(b, false, &r, g, p)?),
            Symbol::Eos => {
                next.ended = true;
                None
            }
        };
        let out = Observation {
            refinement: r,
            roots: next.roots,
            keys,
            actual,
            result_frontier: next.objects.result_frontier(),
        };
        // EOS acknowledges without appending a byte, but may clear an active lease.
        if keys.is_none() {
            next.refresh(g)?;
        }
        *self = next;
        Ok(out)
    }
    fn commit_byte(
        &mut self,
        byte: u8,
        input: bool,
        r: &Refinement,
        g: &BoundGeometry,
        p: &mut Policy<'_>,
    ) -> Result<[u16; 2]> {
        self.last_byte = Some(byte);
        for (phase, delta) in self.phases.iter_mut().zip(g.byte_phases(byte)) {
            *phase = phase.wrapping_add(delta);
        }
        let mut f = self.fields(r, operands(r));
        f.roots = self.roots;
        let delta = host(p.evaluate(&f, Phase::Observe))?.roots;
        for (root, d) in self.roots.iter_mut().zip(delta) {
            *root = host(g.product(*root, d))?;
        }
        f.roots = self.roots;
        let decision = host(p.evaluate(&f, Phase::Key))?;
        let keys = [decision.roots[0], decision.roots[1]];
        if input {
            host(self.objects.observe_input(byte, keys, self.roots))?;
        } else {
            host(self.objects.commit_key(keys, self.roots))?;
        }
        self.refresh(g)?;
        Ok(keys)
    }
    /// Current input byte is observed before querying prior committed objects;
    /// it is published only after its contextual update and key decision.
    pub fn observe_input(
        &mut self,
        byte: u8,
        g: &BoundGeometry,
        m: &Metric,
        p: &mut Policy<'_>,
    ) -> Result<Observation> {
        self.binding(g, p)?;
        if self.pending.is_some() || self.ended {
            return Err(Error::Invalid("pending/ended input"));
        }
        let mut next = self.clone();
        next.last_byte = Some(byte);
        let r = next.refinement(g, m, p)?;
        next.roots = r.roots;
        let keys = next.commit_byte(byte, true, &r, g, p)?;
        let out = Observation {
            refinement: r,
            roots: next.roots,
            keys: Some(keys),
            actual: Symbol::Byte(byte),
            result_frontier: next.objects.result_frontier(),
        };
        *self = next;
        Ok(out)
    }
    pub fn begin_turn(&mut self, g: &BoundGeometry) -> Result<()> {
        if self.geometry_id != g.identity_digest() {
            return Err(Error::Invalid("geometry binding"));
        }
        let mut next = self.clone();
        host(next.objects.begin_turn())?;
        next.pending = None;
        next.ended = false;
        next.turn_start = next.objects.frontier();
        next.refresh(g)?;
        *self = next;
        Ok(())
    }
}
fn operands(r: &Refinement) -> [Option<Lease>; 2] {
    [
        r.hops[0].and_then(|h| h.selected),
        r.hops[1].and_then(|h| h.selected),
    ]
}
