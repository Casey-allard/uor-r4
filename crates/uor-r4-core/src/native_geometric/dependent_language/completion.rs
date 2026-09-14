//! Completion-aware policy over the existing shared geometric transition.
//! Pending syntax is distinct from admitted routing. An unresolved route is not
//! evidence that a fact is absent from the world.
use super::{runtime as binding, scheduling};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime::RouteStatus,
    recurrent_text::runtime::Action,
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};
pub const ROWS: usize = 16;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: scheduling::Artifact,
    pub parent_digest: [u8; 32],
    pub actions: Vec<u8>,
    pub source_digest: [u8; 32],
    pub data_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        self.parent.validate(g)?;
        if self.schema != 1
            || self.actions.len() != ROWS
            || self.actions.iter().any(|&a| a > 3)
            || self.parent_digest != *blake3::hash(&self.parent.encode()?).as_bytes()
        {
            return Err(Error::Artifact);
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(bytes: &[u8], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(bytes).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    NoAdmittedUpdate,
    AmbiguousUpdate,
    NoCompatibleCandidate,
    Ambiguous,
    UnsupportedWordWindow,
    ReadDisabled,
    ScorerDisabled,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Answered,
    Unresolved { clause: usize, reason: Reason },
    Exhausted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Full,
    PendingHidden,
    UnresolvedDisabled,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub before: scheduling::Frame,
    pub row: usize,
    pub pending: bool,
    pub reason: Option<Reason>,
    pub selected: Option<[usize; 2]>,
    pub action: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generated {
    pub trace: scheduling::Generated,
    pub outcome: Outcome,
    pub decisions: Vec<Decision>,
}
#[derive(Clone, Debug)]
pub struct Observed {
    pub core: scheduling::Observed,
    pub row: usize,
    pub pending: bool,
    pub reason: Option<Reason>,
}
pub fn observe(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    qs: &[Vec<u8>],
    s: &scheduling::Frame,
    c: Control,
) -> Result<Observed> {
    observe_routed(
        a,
        g,
        m,
        records,
        qs,
        s,
        c,
        binding::PayloadWindow::Word,
        &|bytes, _| Ok(bytes.to_vec()),
        &mut |query, _| {
            reader::route(
                &a.parent.parent.parent,
                g,
                m,
                records,
                query,
                reader::Control::Full,
            )
        },
    )
}
pub(crate) fn observe_routed<F, P>(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    qs: &[Vec<u8>],
    s: &scheduling::Frame,
    c: Control,
    payload_window: binding::PayloadWindow,
    payload_fn: &P,
    route_fn: &mut F,
) -> Result<Observed>
where
    F: FnMut(&[u8], usize) -> Result<crate::native_geometric::language_relation::runtime::Route>,
    P: Fn(&[u8], usize) -> Result<Vec<u8>>,
{
    observe_routed_updates(
        a,
        g,
        m,
        records,
        qs,
        s,
        c,
        payload_window,
        payload_fn,
        route_fn,
        &|_, _| Ok(true),
    )
}
pub(crate) fn observe_routed_updates<F, P, U>(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    qs: &[Vec<u8>],
    s: &scheduling::Frame,
    c: Control,
    payload_window: binding::PayloadWindow,
    payload_fn: &P,
    route_fn: &mut F,
    update_filter: &U,
) -> Result<Observed>
where
    F: FnMut(&[u8], usize) -> Result<crate::native_geometric::language_relation::runtime::Route>,
    P: Fn(&[u8], usize) -> Result<Vec<u8>>,
    U: Fn(&[u8], &binding::UpdateCandidate) -> Result<bool>,
{
    let core = scheduling::observe_routed_updates(
        &a.parent,
        g,
        m,
        records,
        qs,
        s,
        scheduling::Control::Full,
        None,
        None,
        route_fn,
        payload_window,
        payload_fn,
        update_filter,
    )?;
    let pending = qs.get(s.clause + 1).is_some();
    let mut reason = None;
    if pending && !core.core.next_available {
        if core.update.is_none() {
            let admitted = if let (Some(next), Some(selected)) =
                (qs.get(s.clause + 1), core.route.selected.as_ref())
            {
                let updates = binding::updates_with_window(
                    &a.parent.parent,
                    g,
                    m,
                    records,
                    next,
                    &payload_fn(&selected.bytes, s.clause)?,
                    binding::Control::Full,
                    payload_window,
                )?;
                let mut count = 0;
                for u in updates {
                    if a.parent.parent.matches(u.features) && update_filter(next, &u)? {
                        count += 1;
                    }
                }
                count
            } else {
                0
            };
            reason = Some(if admitted > 1 {
                Reason::AmbiguousUpdate
            } else {
                Reason::NoAdmittedUpdate
            });
        } else {
            let next = route_fn(&core.core.next_query.bytes, s.clause + 1)?;
            reason = Some(match next.status {
                RouteStatus::NoCompatibleCandidate => Reason::NoCompatibleCandidate,
                RouteStatus::Ambiguous => Reason::Ambiguous,
                RouteStatus::UnsupportedWordWindow => Reason::UnsupportedWordWindow,
                RouteStatus::ReadDisabled => Reason::ReadDisabled,
                RouteStatus::ScorerDisabled => Reason::ScorerDisabled,
                RouteStatus::Selected => return Err(Error::State),
            });
        }
    }
    let row = core.core.row
        | if pending && c != Control::PendingHidden {
            8
        } else {
            0
        };
    Ok(Observed {
        core,
        row,
        pending,
        reason,
    })
}
/// Action3 has a typed terminal result. Other actions execute the existing
/// transition without changing byte emission, query transport or read counts.
pub fn unresolved(o: &Observed, s: &scheduling::Frame) -> Option<Outcome> {
    if !o.pending || o.core.route.selected.is_none() {
        return None;
    }
    o.reason.map(|reason| Outcome::Unresolved {
        clause: s.clause + 1,
        reason,
    })
}
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    c: Control,
) -> Result<Generated> {
    generate_routed(
        a,
        g,
        m,
        records,
        prompt,
        c,
        scheduling::Control::Full,
        |query, _| {
            reader::route(
                &a.parent.parent.parent,
                g,
                m,
                records,
                query,
                reader::Control::Full,
            )
        },
    )
}
/// Reuses completion actions, typed reasons and the recurrent executor. The
/// injected reader takes only current query bytes and clause index, never a target.
pub(crate) fn generate_routed<F>(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    c: Control,
    execution_control: scheduling::Control,
    route_fn: F,
) -> Result<Generated>
where
    F: FnMut(&[u8], usize) -> Result<crate::native_geometric::language_relation::runtime::Route>,
{
    generate_routed_payload(
        a,
        g,
        m,
        records,
        prompt,
        c,
        execution_control,
        binding::PayloadWindow::Word,
        |bytes, _| Ok(bytes.to_vec()),
        route_fn,
    )
}
/// Same transition loop with an explicit payload-admission window. The normal
/// phrase callback preserves selected bytes; diagnostic callbacks intervene only
/// on update input, leaving the exact routed occurrence and span intact.
pub(crate) fn generate_routed_payload<F, P>(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    c: Control,
    execution_control: scheduling::Control,
    payload_window: binding::PayloadWindow,
    payload_fn: P,
    route_fn: F,
) -> Result<Generated>
where
    F: FnMut(&[u8], usize) -> Result<crate::native_geometric::language_relation::runtime::Route>,
    P: Fn(&[u8], usize) -> Result<Vec<u8>>,
{
    generate_routed_payload_updates(
        a,
        g,
        m,
        records,
        prompt,
        c,
        execution_control,
        payload_window,
        payload_fn,
        route_fn,
        |_, _| Ok(true),
    )
}
pub(crate) fn generate_routed_payload_updates<F, P, U>(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    c: Control,
    execution_control: scheduling::Control,
    payload_window: binding::PayloadWindow,
    payload_fn: P,
    mut route_fn: F,
    update_filter: U,
) -> Result<Generated>
where
    F: FnMut(&[u8], usize) -> Result<crate::native_geometric::language_relation::runtime::Route>,
    P: Fn(&[u8], usize) -> Result<Vec<u8>>,
    U: Fn(&[u8], &binding::UpdateCandidate) -> Result<bool>,
{
    a.validate(g)?;
    let qs = scheduling::clauses(prompt)?;
    let mut state = scheduling::start(&a.parent, g, &qs)?;
    let mut out = Generated {
        trace: scheduling::Generated {
            tokens: Vec::new(),
            steps: Vec::new(),
            exhausted: false,
        },
        outcome: Outcome::Exhausted,
        decisions: Vec::new(),
    };
    for _ in 0..scheduling::MAX_STEPS {
        let o = observe_routed_updates(
            a,
            g,
            m,
            records,
            &qs,
            &state,
            c,
            payload_window,
            &payload_fn,
            &mut route_fn,
            &update_filter,
        )?;
        let mut choice = a.actions[o.row];
        if c == Control::UnresolvedDisabled && choice == 3 {
            choice = 2;
        }
        out.decisions.push(Decision {
            before: state.clone(),
            row: o.row,
            pending: o.pending,
            reason: o.reason,
            selected: o.core.route.selected.as_ref().map(|v| [v.source, v.word]),
            action: choice,
        });
        if choice == 3 {
            if let Some(outcome) = unresolved(&o, &state) {
                out.outcome = outcome;
                return Ok(out);
            }
            out.trace.exhausted = true;
            return Ok(out);
        }
        let action = Action::from_byte(choice)?;
        let before = state.clone();
        let token =
            scheduling::execute(&a.parent, g, &o.core, &mut state, action, execution_control)?;
        if let Some(t) = token {
            out.trace.tokens.push(t);
        }
        out.trace.steps.push(scheduling::Step {
            before,
            observation: o.core,
            action,
            token,
            after: state.clone(),
        });
        if state.core.done || state.core.exhausted {
            out.trace.exhausted = state.core.exhausted;
            out.outcome = if state.core.exhausted {
                Outcome::Exhausted
            } else {
                Outcome::Answered
            };
            return Ok(out);
        }
    }
    out.trace.exhausted = true;
    Ok(out)
}
