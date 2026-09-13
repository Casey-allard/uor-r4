//! Eight-phase hard addressed attention. Policy supplies decisions, never targets.
use super::artifact::BoundGeometry;
use super::circuit::CircuitError;
use super::inputs::{Active, CausalInputs, Phase, Selected};
use super::objects::{self, Action, ActiveLease, Lease, ObjectSession, Offer, Reference, Symbol};

#[derive(Debug)]
pub enum EngineError {
    Circuit(CircuitError),
    Object(objects::ObjectError),
    Invalid(&'static str),
    Host(String),
}
impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EngineError {}
impl From<CircuitError> for EngineError {
    fn from(e: CircuitError) -> Self {
        Self::Circuit(e)
    }
}
impl From<objects::ObjectError> for EngineError {
    fn from(e: objects::ObjectError) -> Self {
        Self::Object(e)
    }
}
pub type Result<T> = std::result::Result<T, EngineError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    Root(u8),
    Extent,
    Control,
    Emission,
    Null(u8),
}
pub trait Policy {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> Result<u16>;
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize>;
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Work {
    pub circuit_calls: u64,
    pub candidate_scores: u64,
    pub predictions: u64,
    pub observations: u64,
    pub scalar_decodes: u64,
    pub selected_operations: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForwardTrace {
    pub contexts: [Option<u16>; 8],
    pub selected: [Option<Reference>; 2],
    pub roots_before: [u16; 4],
    pub roots_after: [u16; 4],
    pub offer_id: u64,
    pub action: Action,
    pub output: Symbol,
    pub actual: Option<Symbol>,
    pub cursor: Option<u8>,
    pub candidate_scores: u16,
    pub numeric_valid: [bool; 2],
    pub scalar_decodes: u8,
    pub selected_operations: u8,
}
impl ForwardTrace {
    pub fn circuit_calls(&self) -> usize {
        self.contexts.iter().flatten().count()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForwardOffer {
    pub offer: Offer,
    pub trace: ForwardTrace,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSession {
    model_id: [u8; 32],
    geometry_id: [u8; 32],
    objects: ObjectSession,
    roots: [u16; 4],
    phases: [u16; 4],
    last_byte: Option<u8>,
    turn_start: u64,
    last_control: Action,
    ended: bool,
    pending: Option<ForwardOffer>,
    last_trace: Option<ForwardTrace>,
    work: Work,
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
fn action_index(a: Action) -> u8 {
    match a {
        Action::Hold => 0,
        Action::AcquireA => 1,
        Action::AcquireB => 2,
        Action::AddAB => 3,
        Action::SubAB => 4,
        Action::Advance => 5,
        Action::Clear => 6,
    }
}
fn numeric(mut input: CausalInputs, valid: [bool; 2]) -> CausalInputs {
    if let Some(a) = &mut input.selected_a {
        a.numeric_valid = valid[0];
    }
    if let Some(b) = &mut input.selected_b {
        b.numeric_valid = valid[1];
    }
    input
}
fn geo<T>(r: std::result::Result<T, impl std::fmt::Display>) -> Result<T> {
    r.map_err(|e| EngineError::Host(e.to_string()))
}
fn choose(policy: &mut impl Policy, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
    if context >= 512 || !legal.iter().any(|&b| b) {
        return Err(EngineError::Invalid("empty/domain head"));
    }
    let value = policy.choice(head, context, legal)?;
    if !legal.get(value).copied().unwrap_or(false) {
        return Err(EngineError::Invalid("policy selected illegal outcome"));
    }
    Ok(value)
}
fn call(
    policy: &mut impl Policy,
    phase: Phase,
    input: CausalInputs,
    trace: &mut ForwardTrace,
) -> Result<u16> {
    let context = policy.context(phase, &input.pack(phase)?)?;
    if context >= 512 || trace.contexts[phase as usize].is_some() {
        return Err(EngineError::Invalid("context/phase reuse"));
    }
    trace.contexts[phase as usize] = Some(context);
    Ok(context)
}
impl RuntimeSession {
    pub fn new(model_id: [u8; 32], geometry: &BoundGeometry, epoch: u64) -> Result<Self> {
        let identity = geometry.identity();
        if identity >= 120 {
            return Err(EngineError::Invalid("geometry identity"));
        }
        Ok(Self {
            model_id,
            geometry_id: geometry.identity_digest(),
            objects: ObjectSession::new(model_id, geometry.identity_digest(), epoch),
            roots: [identity; 4],
            phases: [0; 4],
            last_byte: None,
            turn_start: 0,
            last_control: Action::Hold,
            ended: false,
            pending: None,
            last_trace: None,
            work: Work::default(),
        })
    }
    pub fn objects(&self) -> &ObjectSession {
        &self.objects
    }
    pub fn roots(&self) -> [u16; 4] {
        self.roots
    }
    pub fn phases(&self) -> [u16; 4] {
        self.phases
    }
    pub fn work(&self) -> Work {
        self.work
    }
    pub fn last_trace(&self) -> Option<&ForwardTrace> {
        self.last_trace.as_ref()
    }
    pub fn pending(&self) -> Option<&ForwardOffer> {
        self.pending.as_ref()
    }
    fn binding(&self, g: &BoundGeometry) -> Result<()> {
        if g.identity_digest() != self.geometry_id {
            Err(EngineError::Invalid("geometry binding"))
        } else {
            Ok(())
        }
    }
    fn selected(&self, g: &BoundGeometry, lease: Lease, pre_extent: bool) -> Result<Selected> {
        let at = match lease.reference() {
            Reference::Occurrence { start, .. } => start,
            Reference::Result { epoch, id } => self.objects.result(epoch, id)?.published_at,
        };
        Ok(Selected {
            signature: geo(g.signature(lease.roots()[0]))?,
            first_byte: lease.payload()[0],
            length: if pre_extent {
                0
            } else {
                lease.payload().len() as u8
            },
            age: self.objects.frontier().saturating_sub(at).min(255) as u8,
            numeric_valid: false,
        })
    }
    fn inputs(
        &self,
        g: &BoundGeometry,
        a: Option<Lease>,
        b: Option<Lease>,
        active: Option<&ActiveLease>,
        pending: bool,
    ) -> Result<CausalInputs> {
        let mut signatures = [[0; 2]; 4];
        for (i, r) in self.roots.into_iter().enumerate() {
            signatures[i] = geo(g.signature(r))?;
        }
        Ok(CausalInputs {
            state_signatures: signatures,
            last_observed: self.last_byte,
            selected_a: a.map(|l| self.selected(g, l, false)).transpose()?,
            selected_b: b.map(|l| self.selected(g, l, false)).transpose()?,
            active: active.map(|a| Active {
                byte: a.byte(),
                kind: match a.lease().reference() {
                    Reference::Occurrence { .. } => 1,
                    Reference::Result { .. } => 2,
                },
                cursor: a.cursor(),
                length: a.lease().payload().len() as u8,
                acknowledged: a.acknowledged(),
                provisional: a.provisional().is_some(),
            }),
            turn_position: self
                .objects
                .frontier()
                .saturating_sub(self.turn_start)
                .min(255) as u8,
            zeta_bins: self.phases.map(|p| (p >> 12) as u8),
            last_control: action_index(self.last_control),
            publications: self.objects.publications_this_turn(),
            pending_offer: pending,
        })
    }
    fn read(
        &self,
        g: &BoundGeometry,
        p: &mut impl Policy,
        which: usize,
        a: Option<Lease>,
        trace: &mut ForwardTrace,
    ) -> Result<Option<Lease>> {
        let phase = if which == 0 {
            Phase::QueryA
        } else {
            Phase::QueryB
        };
        let context = call(
            p,
            phase,
            self.inputs(g, a, None, self.objects.active(), false)?,
            trace,
        )?;
        let query = [
            choose(p, Head::Root(0), context, &[true; 120])? as u16,
            choose(p, Head::Root(1), context, &[true; 120])? as u16,
        ];
        // Null roots are separate fresh head invocations for each read.
        let null = [
            choose(p, Head::Null(0), 0, &[true; 120])? as u16,
            choose(p, Head::Null(1), 0, &[true; 120])? as u16,
        ];
        let mut score = geo(g.score(query, null))?;
        let mut selected = None;
        trace.candidate_scores += 1;
        // Full typed ordering: Null, occurrence sequence, then result ID. Strict
        // improvement preserves the first member of every exact score tie.
        let epoch = self.objects.epoch();
        let frontier = self.objects.frontier();
        for sequence in frontier.saturating_sub(256)..frontier {
            let record = self.objects.occurrence(epoch, sequence)?;
            trace.candidate_scores += 1;
            let candidate = geo(g.score(query, record.keys))?;
            if candidate > score {
                score = candidate;
                selected = Some(Reference::Occurrence {
                    epoch,
                    turn: record.turn,
                    start: sequence,
                    end: sequence + 1,
                });
            }
        }
        for id in
            self.objects.result_frontier().saturating_sub(8).max(1)..self.objects.result_frontier()
        {
            let record = self.objects.result(epoch, id)?;
            trace.candidate_scores += 1;
            let candidate = geo(g.score(query, record.lease.keys()))?;
            if candidate > score {
                score = candidate;
                selected = Some(Reference::Result { epoch, id });
            }
        }
        let Some(reference) = selected else {
            return Ok(None);
        };
        let mut legal = [false; 64];
        let first = match reference {
            Reference::Occurrence { epoch, start, .. } => {
                let maximum = self.objects.maximum_extent(epoch, start)?;
                legal[..usize::from(maximum)].fill(true);
                self.objects.acquire_occurrence(epoch, start, 1)?
            }
            Reference::Result { epoch, id } => {
                let lease = self.objects.acquire_result(epoch, id)?;
                legal[lease.payload().len() - 1] = true;
                lease
            }
        };
        let phase = if which == 0 {
            Phase::ExtentA
        } else {
            Phase::ExtentB
        };
        let mut input = self.inputs(g, a, None, self.objects.active(), false)?;
        if which == 0 {
            input.selected_a = Some(self.selected(g, first, true)?);
        } else {
            input.selected_b = Some(self.selected(g, first, true)?);
        }
        let context = call(p, phase, input, trace)?;
        let length = choose(p, Head::Extent, context, &legal)? as u8 + 1;
        let lease = match reference {
            Reference::Occurrence { epoch, start, .. } => {
                self.objects.acquire_occurrence(epoch, start, length)?
            }
            Reference::Result { .. } => first,
        };
        trace.selected[which] = Some(lease.reference());
        Ok(Some(lease))
    }
    pub fn predict(&mut self, g: &BoundGeometry, p: &mut impl Policy) -> Result<ForwardOffer> {
        self.binding(g)?;
        if let Some(offer) = self.pending {
            return Ok(offer);
        }
        if self.ended {
            return Err(EngineError::Invalid("response ended"));
        }
        if self.objects.key_pending() || self.objects.pending().is_some() {
            return Err(EngineError::Invalid("incomplete object tick"));
        }
        let mut trace = ForwardTrace {
            contexts: [None; 8],
            selected: [None; 2],
            roots_before: self.roots,
            roots_after: self.roots,
            offer_id: 0,
            action: Action::Hold,
            output: Symbol::Eos,
            actual: None,
            cursor: None,
            candidate_scores: 0,
            numeric_valid: [false; 2],
            scalar_decodes: 0,
            selected_operations: 0,
        };
        let a = self.read(g, p, 0, None, &mut trace)?;
        let b = self.read(g, p, 1, a, &mut trace)?;
        let prepared = self.objects.prepare_operands(a, b)?;
        trace.numeric_valid = prepared.numeric_valid();
        trace.scalar_decodes = prepared.scalar_decodes();
        let input = numeric(
            self.inputs(g, a, b, self.objects.active(), false)?,
            trace.numeric_valid,
        );
        let context = call(p, Phase::Control, input, &mut trace)?;
        let legal = prepared.legal();
        let action = ACTIONS[choose(p, Head::Control, context, &legal)?];
        let prospective = self.objects.prepare_action(prepared, action)?;
        trace.selected_operations = u8::from(matches!(action, Action::AddAB | Action::SubAB));
        let input = numeric(
            self.inputs(g, a, b, prospective.active(), true)?,
            trace.numeric_valid,
        );
        let context = call(p, Phase::Emit, input, &mut trace)?;
        let symbol = choose(p, Head::Emission, context, &[true; 257])?;
        let symbol = if symbol == 256 {
            Symbol::Eos
        } else {
            Symbol::Byte(symbol as u8)
        };
        let calls = trace.circuit_calls() as u64;
        let work = Work {
            circuit_calls: self
                .work
                .circuit_calls
                .checked_add(calls)
                .ok_or(EngineError::Invalid("work counter exhausted"))?,
            candidate_scores: self
                .work
                .candidate_scores
                .checked_add(u64::from(trace.candidate_scores))
                .ok_or(EngineError::Invalid("work counter exhausted"))?,
            predictions: self
                .work
                .predictions
                .checked_add(1)
                .ok_or(EngineError::Invalid("work counter exhausted"))?,
            observations: self.work.observations,
            scalar_decodes: self
                .work
                .scalar_decodes
                .checked_add(u64::from(trace.scalar_decodes))
                .ok_or(EngineError::Invalid("work counter exhausted"))?,
            selected_operations: self
                .work
                .selected_operations
                .checked_add(u64::from(trace.selected_operations))
                .ok_or(EngineError::Invalid("work counter exhausted"))?,
        };
        let offer = self.objects.cache_prepared_offer(symbol, prospective)?;
        trace.offer_id = offer.id;
        trace.action = action;
        trace.output = symbol;
        trace.cursor = offer.active().map(ActiveLease::cursor);
        let forward = ForwardOffer { offer, trace };
        self.pending = Some(forward);
        self.work = work;
        Ok(forward)
    }
    /// Invalid/consumed IDs return before policy calls or any runtime mutation.
    /// Other policy errors preserve runtime state; caller-owned policy RNG/tapes
    /// cannot be rolled back through this trait and must be checkpointed by host.
    pub fn observe(
        &mut self,
        actual: Symbol,
        offer_id: u64,
        g: &BoundGeometry,
        p: &mut impl Policy,
    ) -> Result<ForwardTrace> {
        self.binding(g)?;
        let offered = self
            .pending
            .filter(|o| {
                o.offer.id == offer_id
                    && o.offer.epoch == self.objects.epoch()
                    && o.offer.turn == self.objects.turn()
                    && o.offer.frontier == self.objects.frontier()
            })
            .ok_or(objects::ObjectError::UnknownOffer)?;
        if self.objects.pending() != Some(&offered.offer) {
            return Err(EngineError::Invalid("pending offer binding"));
        }
        let mut next = self.clone();
        next.objects.acknowledge(actual, offer_id)?;
        next.pending = None;
        let mut trace = offered.trace;
        trace.actual = Some(actual);
        if let Symbol::Byte(byte) = actual {
            let operands = if actual == offered.offer.symbol {
                offered.offer.operands()
            } else {
                [None; 2]
            };
            if actual == offered.offer.symbol {
                next.last_control = offered.offer.action;
            }
            next.observe_phases(byte, operands, false, g, p, &mut trace)?;
        } else {
            next.ended = true;
        }
        trace.roots_after = next.roots;
        next.work.circuit_calls = next
            .work
            .circuit_calls
            .checked_add((trace.circuit_calls() - offered.trace.circuit_calls()) as u64)
            .ok_or(EngineError::Invalid("work counter exhausted"))?;
        next.work.observations = next
            .work
            .observations
            .checked_add(1)
            .ok_or(EngineError::Invalid("work counter exhausted"))?;
        next.last_trace = Some(trace);
        *self = next;
        Ok(trace)
    }
    fn observe_phases(
        &mut self,
        byte: u8,
        operands: [Option<Lease>; 2],
        input_origin: bool,
        g: &BoundGeometry,
        p: &mut impl Policy,
        trace: &mut ForwardTrace,
    ) -> Result<()> {
        self.last_byte = Some(byte);
        for (phase, delta) in self.phases.iter_mut().zip(g.byte_phases(byte)) {
            *phase = phase.wrapping_add(delta);
        }
        let active = if input_origin {
            None
        } else {
            self.objects.active()
        };
        let input = numeric(
            self.inputs(g, operands[0], operands[1], active, !input_origin)?,
            trace.numeric_valid,
        );
        let context = call(p, Phase::Observe, input, trace)?;
        for lane in 0..4 {
            let delta = choose(p, Head::Root(4 + lane as u8), context, &[true; 120])? as u16;
            self.roots[lane] = geo(g.product(self.roots[lane], delta))?;
        }
        let context = call(
            p,
            Phase::Key,
            self.inputs(
                g,
                None,
                None,
                if input_origin {
                    None
                } else {
                    self.objects.active()
                },
                false,
            )?,
            trace,
        )?;
        let keys = [
            choose(p, Head::Root(2), context, &[true; 120])? as u16,
            choose(p, Head::Root(3), context, &[true; 120])? as u16,
        ];
        if input_origin {
            self.objects.observe_input(byte, keys, self.roots)?;
        } else {
            self.objects.commit_key(keys, self.roots)?;
        }
        Ok(())
    }
    pub fn observe_input(
        &mut self,
        byte: u8,
        g: &BoundGeometry,
        p: &mut impl Policy,
    ) -> Result<ForwardTrace> {
        self.binding(g)?;
        if self.objects.key_pending() {
            return Err(EngineError::Invalid("incomplete object tick"));
        }
        let mut next = self.clone();
        next.pending = None;
        next.ended = false;
        next.last_control = Action::Hold;
        let mut trace = ForwardTrace {
            contexts: [None; 8],
            selected: [None; 2],
            roots_before: self.roots,
            roots_after: self.roots,
            offer_id: 0,
            action: Action::Hold,
            output: Symbol::Byte(byte),
            actual: Some(Symbol::Byte(byte)),
            cursor: None,
            candidate_scores: 0,
            numeric_valid: [false; 2],
            scalar_decodes: 0,
            selected_operations: 0,
        };
        next.observe_phases(byte, [None; 2], true, g, p, &mut trace)?;
        trace.roots_after = next.roots;
        next.work.circuit_calls = next
            .work
            .circuit_calls
            .checked_add(2)
            .ok_or(EngineError::Invalid("work counter exhausted"))?;
        next.work.observations = next
            .work
            .observations
            .checked_add(1)
            .ok_or(EngineError::Invalid("work counter exhausted"))?;
        next.last_trace = Some(trace);
        *self = next;
        Ok(trace)
    }
    pub fn begin_turn(&mut self) -> Result<()> {
        self.objects.begin_turn()?;
        self.turn_start = self.objects.frontier();
        self.pending = None;
        self.last_control = Action::Hold;
        self.ended = false;
        Ok(())
    }
    pub fn reset_session(&mut self, g: &BoundGeometry) -> Result<()> {
        self.binding(g)?;
        self.objects.reset_session()?;
        self.roots = [g.identity(); 4];
        self.phases = [0; 4];
        self.last_byte = None;
        self.turn_start = 0;
        self.last_control = Action::Hold;
        self.pending = None;
        self.last_trace = None;
        self.ended = false;
        self.work = Work::default();
        Ok(())
    }
}

// Host-only snapshot framing. The inner object codec checks exact lineage; this
// frame additionally binds roots/phases, cached trace and learned-model identity.
struct Writer(Vec<u8>);
impl Writer {
    fn b(&mut self, x: u8) {
        self.0.push(x)
    }
    fn n(&mut self, x: u64) {
        self.0.extend_from_slice(&x.to_le_bytes())
    }
    fn roots(&mut self, x: [u16; 4]) {
        for r in x {
            self.0.extend_from_slice(&r.to_le_bytes())
        }
    }
    fn symbol(&mut self, s: Symbol) {
        match s {
            Symbol::Byte(b) => {
                self.b(0);
                self.b(b)
            }
            Symbol::Eos => self.b(1),
        }
    }
    fn reference(&mut self, r: Option<Reference>) {
        match r {
            None => self.b(0),
            Some(Reference::Occurrence {
                epoch,
                turn,
                start,
                end,
            }) => {
                self.b(1);
                for x in [epoch, turn, start, end] {
                    self.n(x)
                }
            }
            Some(Reference::Result { epoch, id }) => {
                self.b(2);
                self.n(epoch);
                self.n(id)
            }
        }
    }
    fn trace(&mut self, t: Option<ForwardTrace>) {
        self.b(u8::from(t.is_some()));
        if let Some(t) = t {
            for c in t.contexts {
                self.0
                    .extend_from_slice(&c.unwrap_or(u16::MAX).to_le_bytes());
            }
            for r in t.selected {
                self.reference(r)
            }
            self.roots(t.roots_before);
            self.roots(t.roots_after);
            self.n(t.offer_id);
            self.b(action_index(t.action));
            self.symbol(t.output);
            self.b(u8::from(t.actual.is_some()));
            if let Some(a) = t.actual {
                self.symbol(a)
            }
            self.b(t.cursor.unwrap_or(255));
            self.0.extend_from_slice(&t.candidate_scores.to_le_bytes());
            for v in t.numeric_valid {
                self.b(u8::from(v))
            }
            self.b(t.scalar_decodes);
            self.b(t.selected_operations);
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn fixed<const N: usize>(&mut self) -> Result<[u8; N]> {
        let end = self
            .at
            .checked_add(N)
            .ok_or(EngineError::Invalid("snapshot size"))?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or(EngineError::Invalid("snapshot truncated"))?;
        let mut out = [0; N];
        out.copy_from_slice(bytes);
        self.at = end;
        Ok(out)
    }
    fn b(&mut self) -> Result<u8> {
        Ok(self.fixed::<1>()?[0])
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.b()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(EngineError::Invalid("snapshot boolean")),
        }
    }
    fn n(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.fixed()?))
    }
    fn roots(&mut self) -> Result<[u16; 4]> {
        Ok([self.u16()?, self.u16()?, self.u16()?, self.u16()?])
    }
    fn symbol(&mut self) -> Result<Symbol> {
        match self.b()? {
            0 => Ok(Symbol::Byte(self.b()?)),
            1 => Ok(Symbol::Eos),
            _ => Err(EngineError::Invalid("snapshot symbol")),
        }
    }
    fn reference(&mut self) -> Result<Option<Reference>> {
        match self.b()? {
            0 => Ok(None),
            1 => Ok(Some(Reference::Occurrence {
                epoch: self.n()?,
                turn: self.n()?,
                start: self.n()?,
                end: self.n()?,
            })),
            2 => Ok(Some(Reference::Result {
                epoch: self.n()?,
                id: self.n()?,
            })),
            _ => Err(EngineError::Invalid("snapshot reference")),
        }
    }
    fn trace(&mut self) -> Result<Option<ForwardTrace>> {
        if !self.boolean()? {
            return Ok(None);
        }
        let mut contexts = [None; 8];
        for c in &mut contexts {
            let v = self.u16()?;
            if v != u16::MAX {
                if v >= 512 {
                    return Err(EngineError::Invalid("snapshot context"));
                }
                *c = Some(v);
            }
        }
        let selected = [self.reference()?, self.reference()?];
        let roots_before = self.roots()?;
        let roots_after = self.roots()?;
        let offer_id = self.n()?;
        let action = *ACTIONS
            .get(usize::from(self.b()?))
            .ok_or(EngineError::Invalid("snapshot action"))?;
        let output = self.symbol()?;
        let actual = if self.boolean()? {
            Some(self.symbol()?)
        } else {
            None
        };
        let cursor = match self.b()? {
            255 => None,
            x if x < 64 => Some(x),
            _ => return Err(EngineError::Invalid("snapshot cursor")),
        };
        let candidate_scores = self.u16()?;
        let numeric_valid = [self.boolean()?, self.boolean()?];
        let scalar_decodes = self.b()?;
        let selected_operations = self.b()?;
        if candidate_scores > 530
            || scalar_decodes > 2
            || selected_operations > 1
            || roots_before
                .into_iter()
                .chain(roots_after)
                .any(|r| r >= 120)
        {
            return Err(EngineError::Invalid("snapshot trace domain"));
        }
        Ok(Some(ForwardTrace {
            contexts,
            selected,
            roots_before,
            roots_after,
            offer_id,
            action,
            output,
            actual,
            cursor,
            candidate_scores,
            numeric_valid,
            scalar_decodes,
            selected_operations,
        }))
    }
}
impl RuntimeSession {
    pub fn snapshot(&self) -> Result<Vec<u8>> {
        let object = self.objects.snapshot()?;
        let mut w = Writer(Vec::new());
        w.0.extend_from_slice(b"UORAE001");
        w.0.extend_from_slice(&self.model_id);
        w.0.extend_from_slice(&self.geometry_id);
        w.roots(self.roots);
        w.roots(self.phases);
        w.b(u8::from(self.last_byte.is_some()));
        if let Some(b) = self.last_byte {
            w.b(b)
        }
        w.n(self.turn_start);
        w.b(action_index(self.last_control));
        w.b(u8::from(self.ended));
        for x in [
            self.work.circuit_calls,
            self.work.candidate_scores,
            self.work.predictions,
            self.work.observations,
            self.work.scalar_decodes,
            self.work.selected_operations,
        ] {
            w.n(x)
        }
        w.trace(self.pending.map(|o| o.trace));
        w.trace(self.last_trace);
        w.0.extend_from_slice(&(object.len() as u32).to_le_bytes());
        w.0.extend_from_slice(&object);
        if w.0.len() + 32 > objects::SNAPSHOT_MAX {
            return Err(EngineError::Invalid("combined snapshot limit"));
        }
        let hash = *blake3::hash(&w.0).as_bytes();
        w.0.extend_from_slice(&hash);
        Ok(w.0)
    }
    pub fn restore(bytes: &[u8], model_id: [u8; 32], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > objects::SNAPSHOT_MAX {
            return Err(EngineError::Invalid("combined snapshot limit"));
        }
        let end = bytes
            .len()
            .checked_sub(32)
            .ok_or(EngineError::Invalid("snapshot truncated"))?;
        if blake3::hash(&bytes[..end]).as_bytes() != &bytes[end..] {
            return Err(EngineError::Invalid("snapshot checksum"));
        }
        let mut r = Reader {
            bytes: &bytes[..end],
            at: 0,
        };
        if r.fixed::<8>()? != *b"UORAE001"
            || r.fixed::<32>()? != model_id
            || r.fixed::<32>()? != g.identity_digest()
        {
            return Err(EngineError::Invalid("snapshot binding"));
        }
        let roots = r.roots()?;
        let phases = r.roots()?;
        if roots.iter().any(|&x| x >= 120) {
            return Err(EngineError::Invalid("snapshot roots"));
        }
        let last_byte = if r.boolean()? { Some(r.b()?) } else { None };
        let turn_start = r.n()?;
        let last_control = *ACTIONS
            .get(usize::from(r.b()?))
            .ok_or(EngineError::Invalid("snapshot control"))?;
        let ended = r.boolean()?;
        let work = Work {
            circuit_calls: r.n()?,
            candidate_scores: r.n()?,
            predictions: r.n()?,
            observations: r.n()?,
            scalar_decodes: r.n()?,
            selected_operations: r.n()?,
        };
        let pending_trace = r.trace()?;
        let last_trace = r.trace()?;
        let len = u32::from_le_bytes(r.fixed()?) as usize;
        let finish =
            r.at.checked_add(len)
                .ok_or(EngineError::Invalid("snapshot length"))?;
        if finish != end {
            return Err(EngineError::Invalid("snapshot trailing/length"));
        }
        let object = ObjectSession::restore(&r.bytes[r.at..finish], model_id, g.identity_digest())?;
        if object.key_pending()
            || turn_start > object.frontier()
            || (object.frontier() == 0) != last_byte.is_none()
        {
            return Err(EngineError::Invalid("snapshot runtime frontier"));
        }
        if let Some(byte) = last_byte {
            let last = object.occurrence(object.epoch(), object.frontier() - 1)?;
            if last.byte != byte || last.roots != roots {
                return Err(EngineError::Invalid("snapshot last byte/roots"));
            }
        } else if roots != [g.identity(); 4] || phases != [0; 4] {
            return Err(EngineError::Invalid("snapshot empty state"));
        }
        if object.frontier() <= 256 {
            let mut expected = [0_u16; 4];
            for sequence in 0..object.frontier() {
                let byte = object.occurrence(object.epoch(), sequence)?.byte;
                for (p, d) in expected.iter_mut().zip(g.byte_phases(byte)) {
                    *p = p.wrapping_add(d);
                }
            }
            if expected != phases {
                return Err(EngineError::Invalid("snapshot retained phase history"));
            }
        }
        let pending = match (pending_trace, object.pending().copied()) {
            (None, None) => None,
            (Some(trace), Some(offer)) => {
                if ended
                    || trace.offer_id != offer.id
                    || trace.action != offer.action
                    || trace.output != offer.symbol
                    || trace.actual.is_some()
                    || trace.roots_before != roots
                    || trace.roots_after != roots
                    || trace.cursor != offer.active().map(ActiveLease::cursor)
                    || trace.selected != offer.operands().map(|l| l.map(|l| l.reference()))
                    || trace.contexts[6..].iter().any(Option::is_some)
                    || [0, 2, 4, 5]
                        .into_iter()
                        .any(|i| trace.contexts[i].is_none())
                    || trace.contexts[1].is_some() != trace.selected[0].is_some()
                    || trace.contexts[3].is_some() != trace.selected[1].is_some()
                {
                    return Err(EngineError::Invalid("snapshot pending trace"));
                }
                if trace.numeric_valid
                    != offer
                        .operands()
                        .map(|l| l.is_some_and(|l| objects::decode_i64(l.payload()).is_ok()))
                    || usize::from(trace.scalar_decodes)
                        != offer.operands().iter().flatten().count()
                    || trace.selected_operations
                        != u8::from(matches!(offer.action, Action::AddAB | Action::SubAB))
                {
                    return Err(EngineError::Invalid("snapshot prepared scalar fields"));
                }
                Some(ForwardOffer { offer, trace })
            }
            _ => return Err(EngineError::Invalid("snapshot offer mismatch")),
        };
        if let Some(t) = last_trace {
            if t.actual.is_none() || t.roots_after != roots {
                return Err(EngineError::Invalid("snapshot last trace"));
            }
        }
        let s = Self {
            model_id,
            geometry_id: g.identity_digest(),
            objects: object,
            roots,
            phases,
            last_byte,
            turn_start,
            last_control,
            ended,
            pending,
            last_trace,
            work,
        };
        if s.snapshot()?.as_slice() != bytes {
            return Err(EngineError::Invalid("noncanonical snapshot"));
        }
        Ok(s)
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
