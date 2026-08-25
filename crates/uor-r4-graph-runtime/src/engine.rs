use crate::runtime_state::RuntimeState;
use crate::runtime_state::SemanticStateSlot;
use crate::status::ResolutionStatus;
use crate::vp_tree::{MIN_ROUTE_INDEX_NODES, VpTree};
use uor_r4_graph_format::{CODE_OP_HALT, OP_CLEAR_SLOT, OP_SHIFT_SLOTS, OP_UPDATE_SLOT};
use uor_r4_graph_format::{FormatError, PsiBagTable, ScoreQ, SkipmixTable};
use uor_r4_graph_format::{GraphView, NotAProduct, SectionId};

/// Fixed width of the normative served-candidate shortlist.
pub const SERVED_CANDIDATE_CAPACITY: usize = 8;

/// Compiler-pinned context window consumed by the learned serving lane.
/// Kept distinct from shortlist capacity even though both are eight today.
const SERVED_CONTEXT_WINDOW: usize = 8;

/// Compiler-pinned maximum entries consumed from one SKMX or PSIB row.
const SERVED_SKIPMIX_MAX_ENTRIES: u16 = 64;

/// Compiler-pinned maximum open-addressing probes consumed by one SKMX lookup.
const SERVED_SKIPMIX_MAX_PROBE: u16 = 32;

/// Score domain responsible for a served candidate's rank.
///
/// `Skipmix` sorts ahead of `Base`; scores are compared only within the same
/// source. This keeps learned skip-count residuals separate from graph scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServedCandidateSource {
    Base,
    Skipmix,
}

/// One entry in the fixed-capacity normative serving shortlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServedCandidate {
    pub token: u32,
    pub score: ScoreQ,
    pub source: ServedCandidateSource,
    /// A nonzero SKMX primary-row entry contributed to this candidate score.
    pub skmx_contributed: bool,
    /// A nonzero PSIB fallback-row entry contributed to this candidate score.
    pub psib_contributed: bool,
}

impl ServedCandidate {
    const EMPTY: Self = Self {
        token: 0,
        score: ScoreQ::MIN,
        source: ServedCandidateSource::Base,
        skmx_contributed: false,
        psib_contributed: false,
    };
}

/// Evidence that the skip-mix lane changed the base scorer's winner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipmixAttribution {
    pub base_token: u32,
    pub promoted_token: u32,
    pub contribution: ScoreQ,
    /// At least one nonzero SKMX primary-row entry contributed.
    pub skmx_contributed: bool,
    /// At least one nonzero PSIB fallback-row entry contributed.
    pub psib_contributed: bool,
}

/// Allocation-free fixed-capacity result of normative served selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServedCandidates {
    ranked: [ServedCandidate; SERVED_CANDIDATE_CAPACITY],
    len: u8,
    attribution: Option<SkipmixAttribution>,
}

/// Bounded attribution for the signature-routing stage of one normative
/// runtime selection.
///
/// The trace carries no candidate authority. It reports which mutually
/// exclusive route source supplied the active graph nodes, plus whether the
/// primary context probe missed and the secondary session probe admitted a
/// calibrated node. This is sufficient for a product canary to distinguish a
/// genuinely exercised session lane from mere signature-byte inequality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignatureRoutingTrace {
    pub context_row_hit: bool,
    pub suffix_dfa_nodes: u8,
    pub context_probe_attempted: bool,
    pub context_admitted_nodes: u8,
    pub session_probe_attempted: bool,
    pub session_admitted_nodes: u8,
    pub selected_source: SignatureRoutingSource,
}

impl Default for SignatureRoutingTrace {
    fn default() -> Self {
        Self {
            context_row_hit: false,
            suffix_dfa_nodes: 0,
            context_probe_attempted: false,
            context_admitted_nodes: 0,
            session_probe_attempted: false,
            session_admitted_nodes: 0,
            selected_source: SignatureRoutingSource::None,
        }
    }
}

/// Active-node source selected before emission scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureRoutingSource {
    None,
    ContextRow,
    SuffixDfa,
    ContextSignature,
    SessionSignature,
    NearestContextSignature,
    NearestSessionSignature,
    DefaultNode,
}

impl ServedCandidates {
    /// Canonically ranked candidates: source, descending score, ascending token.
    pub fn ranked(&self) -> &[ServedCandidate] {
        &self.ranked[..usize::from(self.len)]
    }

    /// The normative served token candidate, when any candidate exists.
    pub fn winner(&self) -> Option<ServedCandidate> {
        self.ranked().first().copied()
    }

    /// Skip-mix promotion evidence; absent when the base winner was retained.
    pub fn attribution(&self) -> Option<SkipmixAttribution> {
        self.attribution
    }

    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for ServedCandidates {
    fn default() -> Self {
        Self {
            ranked: [ServedCandidate::EMPTY; SERVED_CANDIDATE_CAPACITY],
            len: 0,
            attribution: None,
        }
    }
}

/// Multiplication-free zero-allocation prediction runtime wrapping an R4G1 borrowed `PatchChain`.
///
/// This is the sole normative production candidate/token selector (ADR-0001).
/// `uor-r4-api::engine::R4Engine` supplies token-free D4 policy resolution; it
/// may permit, widen, or abstain but cannot replace the candidate selected
/// here. Historical `convert_r4g1` container-demo artifacts remain research
/// evidence: their 0.0% held-out result does not authorize production.
#[derive(Debug, Clone)]
pub struct R4G1Runtime<'a> {
    chain: crate::patch_chain::PatchChain<'a>,
    route_index: Option<VpTree>,
    skipmix: Option<SkipmixTable<'a>>,
    psi_bag: Option<PsiBagTable<'a>>,
}

fn signature_affinity_bonus(prototype: &[u8], mask: &[u8], signature: &[u8]) -> i32 {
    let mut distance = 0u32;
    for ((&prototype_byte, &mask_byte), &signature_byte) in
        prototype.iter().zip(mask).zip(signature)
    {
        distance += ((signature_byte ^ prototype_byte) & mask_byte).count_ones();
    }
    let x = 288i32.saturating_sub(distance as i32);
    // x * 10 == (x << 3) + (x << 1), preserving the integer-only kernel.
    (x << 3).saturating_add(x << 1)
}

/// Resolve the explicit lexical context rows before geometric graph scoring.
/// The first non-empty row wins: trigram, then bigram, then the EMIT
/// unigram path (represented by `None`).
fn context_backoff(view: &GraphView<'_>, context_tokens: &[u32]) -> Option<(u32, ScoreQ)> {
    let mut best: Option<(u32, ScoreQ)> = None;
    with_context_row_entries(view, context_tokens, |entry| {
        let candidate = (entry.token, entry.score_q);
        if best.is_none_or(|(token, score): (u32, ScoreQ)| {
            candidate.1.raw() > score.raw()
                || (candidate.1.raw() == score.raw() && candidate.0 < token)
        }) {
            best = Some(candidate);
        }
    });
    best
}

/// Visit every entry of the winning explicit context row for
/// `context_tokens` — the identical trigram-then-bigram resolution and
/// leading-BOS skip `context_backoff` applies — so single-winner backoff
/// and the top-k candidate walk read the same row instead of drifting.
/// Returns true when a non-empty row was found.
fn with_context_row_entries<F: FnMut(uor_r4_graph_format::NgramEntry)>(
    view: &GraphView<'_>,
    context_tokens: &[u32],
    mut visit: F,
) -> bool {
    let Some(table) = view.ngram_table().ok().flatten() else {
        return false;
    };
    let tokens = if context_tokens.first().is_some_and(|&token| token <= 1) {
        &context_tokens[1..]
    } else {
        context_tokens
    };
    if tokens.len() >= 2
        && let Some(row) = table.find(2, tokens[tokens.len() - 2], tokens[tokens.len() - 1])
    {
        let mut any = false;
        for entry in row.entries() {
            any = true;
            visit(entry);
        }
        if any {
            return true;
        }
    }
    if let Some(&previous) = tokens.last()
        && let Some(row) = table.find(1, previous, 0)
    {
        let mut any = false;
        for entry in row.entries() {
            any = true;
            visit(entry);
        }
        return any;
    }
    false
}

fn validate_serving_table_bounds(
    skipmix: Option<&SkipmixTable<'_>>,
    psi_bag: Option<&PsiBagTable<'_>>,
) -> Result<(), NotAProduct> {
    if skipmix.is_some_and(|table| table.max_entries() > SERVED_SKIPMIX_MAX_ENTRIES) {
        return Err(FormatError::SkipmixInvalidRow.into());
    }
    if skipmix.is_some_and(|table| table.max_probe() > SERVED_SKIPMIX_MAX_PROBE) {
        return Err(FormatError::SkipmixProbeExceeded.into());
    }
    if psi_bag.is_some_and(|table| table.max_entries() > SERVED_SKIPMIX_MAX_ENTRIES) {
        return Err(FormatError::PsiBagInvalidRow.into());
    }
    Ok(())
}

impl<'a> R4G1Runtime<'a> {
    /// Create a new R4G1 runtime by running two-stage validation over `bytes`.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, NotAProduct> {
        let view = GraphView::parse(bytes)?;
        let skipmix = view.skipmix_table()?;
        let psi_bag = view.psi_bag_table()?;
        validate_serving_table_bounds(skipmix.as_ref(), psi_bag.as_ref())?;
        let route_index = (view.node_count().unwrap_or(0) >= MIN_ROUTE_INDEX_NODES)
            .then(|| VpTree::from_graph(&view))
            .flatten();
        Ok(Self {
            chain: crate::patch_chain::PatchChain::new(view),
            route_index,
            skipmix,
            psi_bag,
        })
    }

    /// Appends a patch epoch to the runtime's chain.
    ///
    /// Total verdict: `None` on success, `Some(reason)` when the patch is
    /// rejected — either because `patch_bytes` is not a valid R4G1 artifact
    /// or because the chain-precedence rules reject it (see
    /// [`crate::patch_chain::PatchChain::try_push_patch`]).
    pub fn try_push_patch(&mut self, patch_bytes: &'a [u8]) -> Option<&'static str> {
        let Ok(view) = GraphView::parse(patch_bytes) else {
            return Some("patch bytes are not a valid R4G1 artifact");
        };
        let Ok(skipmix) = view.skipmix_table() else {
            return Some("patch SKMX section is not a product of these bytes");
        };
        let Ok(psi_bag) = view.psi_bag_table() else {
            return Some("patch PSIB section is not a product of these bytes");
        };
        if validate_serving_table_bounds(skipmix.as_ref(), psi_bag.as_ref()).is_err() {
            return Some("patch skip-mix table exceeds the serving work bound");
        }
        let verdict = self.chain.try_push_patch(view);
        if verdict.is_none() {
            if skipmix.is_some() {
                self.skipmix = skipmix;
            }
            if psi_bag.is_some() {
                self.psi_bag = psi_bag;
            }
        }
        verdict
    }

    /// One ROUT probe: query `sig` against the per-node prototypes/masks
    /// (VP-tree when built, linear scan otherwise), filling `active_nodes`
    /// with the within-radius matches (<= 8) and returning
    /// `(nearest_node, within_radius_count)`. Extracted from the fallback
    /// block so the #247 session-lane admission reuses the identical scan.
    fn rout_probe(
        &self,
        base_graph: &uor_r4_graph_format::GraphView<'_>,
        sig: &[u8],
        active_nodes: &mut [u32],
    ) -> (u32, usize) {
        if let Some(index) = self.route_index.as_ref() {
            let mut matched_nodes = [0u32; 8];
            let (best_node, _best_dist, active_count) = index.query(sig, &mut matched_nodes);
            active_nodes[..active_count].copy_from_slice(&matched_nodes[..active_count]);
            (best_node, active_count)
        } else {
            let num_nodes = base_graph.node_count().unwrap_or(0);
            let mut best_node = 0;
            let mut best_dist = u32::MAX;
            let mut active_count = 0usize;
            let rout_bytes = base_graph.section(SectionId::ROUT).unwrap_or(&[]);

            for n in 1..num_nodes {
                if let Some(node) = base_graph.node(n) {
                    let proto_offset = (node.prototype_word_start as usize) << 3;
                    let mask_offset = (node.mask_word_start as usize) << 3;

                    if proto_offset + sig.len() <= rout_bytes.len()
                        && mask_offset + sig.len() <= rout_bytes.len()
                    {
                        let mut dist = 0u32;
                        for (i, &s) in sig.iter().enumerate() {
                            let p = rout_bytes[proto_offset + i];
                            let m = rout_bytes[mask_offset + i];
                            dist += ((s ^ p) & m).count_ones();
                        }

                        if dist < best_dist {
                            best_dist = dist;
                            best_node = n;
                        }

                        // Collect Quantum MoE ensemble nodes matching distance threshold
                        let rad = u32::from(node.radius.0).max(120);
                        if dist <= rad
                            && active_count < 8
                            && !active_nodes[..active_count].contains(&n)
                        {
                            active_nodes[active_count] = n;
                            active_count += 1;
                        }
                    }
                }
            }
            (best_node, active_count)
        }
    }

    pub fn view(&self) -> &GraphView<'a> {
        self.chain.base_graph()
    }

    /// Whether the effective patch chain supplies `(SKMX, PSIB)` tables.
    pub fn skipmix_tables_present(&self) -> (bool, bool) {
        (self.skipmix.is_some(), self.psi_bag.is_some())
    }

    pub fn node_count(&self) -> u32 {
        self.chain.base_graph().node_count().unwrap_or(0)
    }

    /// Run one bounded planning episode against this artifact's packed
    /// planning sections (#843).
    ///
    /// `Ok(None)` when the artifact carries no `PSCH`, `PTRN` or `PGOL`
    /// section — in which case serving is identical to the pre-#843 baseline,
    /// which is what absent-section identity means. `Err` when a planning
    /// section is present but is not a product of these bytes; the episode
    /// fails closed rather than planning on a partially-read table.
    ///
    /// Deterministic and allocation-free: all state lives in the caller's
    /// [`crate::plan::PlanScratch`].
    pub fn plan_bounded(
        &self,
        request: &crate::plan::PlanRequest,
        scratch: &mut crate::plan::PlanScratch,
    ) -> Result<Option<crate::plan::PlanResult>, NotAProduct> {
        let view = self.view();
        let Some(schema) = view.plan_schema()? else {
            return Ok(None);
        };
        let (Some(rules), Some(predicates)) = (view.plan_rule_table()?, view.plan_predicates()?)
        else {
            return Ok(None);
        };
        Ok(Some(crate::plan::plan(
            &crate::plan::PlanQuery {
                strategy: request.strategy,
                schema: &schema,
                rules: &rules,
                predicates: &predicates,
                initial: request.initial,
                available: request.available,
                budget: request.budget,
            },
            scratch,
        )))
    }

    pub fn edge_count(&self) -> u32 {
        self.chain.base_graph().edge_count().unwrap_or(0)
    }

    /// Single deterministic, allocation-free step of the graph runtime.
    ///
    /// Total: every input yields a `(token, status)` pair — an empty graph
    /// resolves to `(0, BackedOff)` rather than an error.
    pub fn step(
        &self,
        state: &mut RuntimeState,
        token: u32,
        _witness: &mut [u8],
    ) -> (u32, ResolutionStatus) {
        state.record_token(token);

        let num_nodes = self.node_count();
        if num_nodes == 0 {
            return (0, ResolutionStatus::BackedOff);
        }

        let context = state.token().as_slice();
        let mut node_scores = [ScoreQ::MIN; 64];
        let (pred_token, score) = self.predict_distribution(context, None, &mut node_scores);

        let status = if score.raw() > 50_000 {
            ResolutionStatus::Supported
        } else if score.raw() > 0 {
            ResolutionStatus::Boundary
        } else {
            ResolutionStatus::BackedOff
        };

        // Phase 8: Execute CODE section state update programs.
        if let Some(code_bytes) = self.view().section(SectionId::CODE) {
            Self::execute_state_updates(state, code_bytes);
        }

        (pred_token, status)
    }

    /// Execute the bytecode program in the CODE section to update semantic states.
    fn execute_state_updates(state: &mut RuntimeState, code_bytes: &[u8]) {
        let mut cursor = 0;
        while cursor < code_bytes.len() {
            let opcode = code_bytes[cursor];
            match opcode {
                CODE_OP_HALT => break,
                OP_UPDATE_SLOT => {
                    if cursor + 16 > code_bytes.len() {
                        break;
                    }
                    let level = code_bytes[cursor + 1];
                    let region_id = u32::from_le_bytes([
                        code_bytes[cursor + 2],
                        code_bytes[cursor + 3],
                        code_bytes[cursor + 4],
                        code_bytes[cursor + 5],
                    ]);
                    let token = u32::from_le_bytes([
                        code_bytes[cursor + 6],
                        code_bytes[cursor + 7],
                        code_bytes[cursor + 8],
                        code_bytes[cursor + 9],
                    ]);
                    let score_q = ScoreQ::from_raw(i32::from_le_bytes([
                        code_bytes[cursor + 10],
                        code_bytes[cursor + 11],
                        code_bytes[cursor + 12],
                        code_bytes[cursor + 13],
                    ]));
                    let age =
                        u16::from_le_bytes([code_bytes[cursor + 14], code_bytes[cursor + 15]]);

                    let slot = SemanticStateSlot {
                        region_id,
                        token,
                        score_q,
                        age,
                    };
                    match level {
                        0 => state.local_mut().update_slot(slot),
                        1 => state.segment_mut().update_slot(slot),
                        2 => state.session_mut().update_slot(slot),
                        _ => {}
                    }
                    cursor += 16;
                }
                OP_CLEAR_SLOT => {
                    if cursor + 2 > code_bytes.len() {
                        break;
                    }
                    let level = code_bytes[cursor + 1];
                    match level {
                        0 => state.local_mut().clear(),
                        1 => state.segment_mut().clear(),
                        2 => state.session_mut().clear(),
                        _ => {}
                    }
                    cursor += 2;
                }
                OP_SHIFT_SLOTS => {
                    if cursor + 2 > code_bytes.len() {
                        break;
                    }
                    let level = code_bytes[cursor + 1];
                    match level {
                        0 => state.local_mut().shift_slots(),
                        1 => state.segment_mut().shift_slots(),
                        2 => state.session_mut().shift_slots(),
                        _ => {}
                    }
                    cursor += 2;
                }
                _ => break, // Unknown opcode, halt execution
            }
        }
    }
}

/// Whether any node in `base_graph` carries a per-node emission list.
///
/// The two graph writers differ here: the certify score writer wires
/// per-node `emission_start`/`emission_len` ranges over the EMIT
/// remainder, while the converter carryover writes every node with an
/// empty range and stores a single global root-prior (token, count)
/// table as the whole EMIT remainder. The fallback readers below must
/// know which flavor they are looking at before treating a section
/// remainder as a pair list (#785).
fn has_per_node_emissions(base_graph: &GraphView<'_>) -> bool {
    let count = base_graph.node_count().unwrap_or(0);
    for n in 0..count {
        if let Some(node) = base_graph.node(n)
            && node.emission_len > 0
        {
            return true;
        }
    }
    false
}

fn check_node_emits(
    base_graph: &uor_r4_graph_format::GraphView,
    node_id: u32,
    target_token: u32,
    emit_remainder: Option<&[u8]>,
) -> (bool, ScoreQ) {
    let mut stack = [0u32; 128];
    let mut visited = [0u32; 128];
    let mut stack_len = 1usize;
    let mut visited_len = 1usize;
    stack[0] = node_id;
    visited[0] = node_id;

    while stack_len > 0 {
        stack_len -= 1;
        let current_id = stack[stack_len];
        let node = match base_graph.node(current_id) {
            Some(n) => n,
            None => continue,
        };

        if let Some(emit_bytes) = emit_remainder {
            let start = node.emission_start as usize;
            let len = node.emission_len as usize;
            if len > 0 && start + len <= emit_bytes.len() {
                let sl = &emit_bytes[start..start + len];
                for i in 0..(sl.len() >> 3) {
                    let offset = i << 3;
                    let cand = u32::from_le_bytes([
                        sl[offset],
                        sl[offset + 1],
                        sl[offset + 2],
                        sl[offset + 3],
                    ]);
                    if cand == target_token {
                        let raw = i32::from_le_bytes([
                            sl[offset + 4],
                            sl[offset + 5],
                            sl[offset + 6],
                            sl[offset + 7],
                        ]);
                        return (
                            true,
                            if raw > 0 {
                                ScoreQ::from_raw(raw)
                            } else {
                                ScoreQ::from_raw(1)
                            },
                        );
                    }
                }
            }
        }

        if node.child_len > 0 {
            let start = node.child_start as usize;
            let count = (node.child_len as usize).min(16);
            for i in (0..count).rev() {
                if let Some(edge) = base_graph.edge((start + i) as u32) {
                    let dst = edge.dst.0;
                    if stack_len < stack.len()
                        && visited_len < visited.len()
                        && !visited[..visited_len].contains(&dst)
                    {
                        stack[stack_len] = dst;
                        stack_len += 1;
                        visited[visited_len] = dst;
                        visited_len += 1;
                    }
                }
            }
        }
    }

    // #785 C1c: no EXCT fallback here. EXCT is a storage descriptor
    // followed by a raw store container (TLS1 carryover from the
    // converter, RX1 residual tables from the score writer), never a
    // (token, score_q) pair list — scanning it as pairs manufactured
    // garbage matches at garbage scores and paid a multi-megabyte
    // linear walk per query doing it.
    (false, ScoreQ::ZERO)
}

fn collect_target_leaf_nodes<'a>(
    base_graph: &GraphView<'a>,
    start_id: u32,
    out: &mut [u32; 128],
    out_len: &mut usize,
) {
    if *out_len >= out.len() {
        return;
    }

    let mut stack = [0u32; 256];
    let mut visited = [0u32; 256];
    let mut stack_len = 1usize;
    let mut visited_len = 1usize;
    stack[0] = start_id;
    visited[0] = start_id;

    while stack_len > 0 && *out_len < out.len() {
        stack_len -= 1;
        let node_id = stack[stack_len];
        let Some(node) = base_graph.node(node_id) else {
            continue;
        };

        if (node.emission_len > 0 || node_id == 0) && !out[..*out_len].contains(&node_id) {
            out[*out_len] = node_id;
            *out_len += 1;
            if *out_len >= out.len() {
                break;
            }
        }

        if node.child_len > 0 {
            let start = node.child_start as usize;
            let count = node.child_len as usize;
            for i in (0..count).rev() {
                if let Some(edge) = base_graph.edge((start + i) as u32) {
                    let dst = edge.dst.0;
                    if stack_len < stack.len()
                        && visited_len < visited.len()
                        && !visited[..visited_len].contains(&dst)
                    {
                        stack[stack_len] = dst;
                        stack_len += 1;
                        visited[visited_len] = dst;
                        visited_len += 1;
                    }
                }
            }
        }
    }
}

fn unique_tokens_in_newest_compiler_window(
    context_tokens: &[u32],
) -> ([u32; SERVED_CONTEXT_WINDOW], usize) {
    let mut unique = [0u32; SERVED_CONTEXT_WINDOW];
    let mut len = 0usize;
    let start = context_tokens.len().saturating_sub(SERVED_CONTEXT_WINDOW);
    for &token in &context_tokens[start..] {
        let mut at = 0usize;
        while at < len && unique[at] < token {
            at += 1;
        }
        if at < len && unique[at] == token {
            continue;
        }
        let mut shift = len;
        while shift > at {
            unique[shift] = unique[shift - 1];
            shift -= 1;
        }
        unique[at] = token;
        len += 1;
    }
    (unique, len)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SkipmixContribution {
    raw: i32,
    skmx_contributed: bool,
    psib_contributed: bool,
}

fn skipmix_contribution(
    skipmix: Option<&SkipmixTable<'_>>,
    psi_bag: Option<&PsiBagTable<'_>>,
    unique_tokens: &[u32],
    last_token: u32,
    candidate: u32,
) -> SkipmixContribution {
    let mut contribution = SkipmixContribution::default();
    for &content_token in unique_tokens {
        let (score, from_skmx) =
            if let Some(row) = skipmix.and_then(|table| table.find(content_token, last_token)) {
                (row.entries().find(candidate), true)
            } else {
                (
                    psi_bag
                        .and_then(|table| table.find(content_token))
                        .and_then(|row| row.entries().find(candidate)),
                    false,
                )
            };
        if let Some(score) = score {
            let raw = score.raw();
            contribution.raw = contribution.raw.saturating_add(raw);
            if raw != 0 {
                if from_skmx {
                    contribution.skmx_contributed = true;
                } else {
                    contribution.psib_contributed = true;
                }
            }
        }
    }
    contribution
}

fn base_candidate_score(
    base: &[(u32, ScoreQ); SERVED_CANDIDATE_CAPACITY],
    base_len: usize,
    token: u32,
) -> Option<ScoreQ> {
    base[..base_len]
        .iter()
        .find_map(|&(base_token, score)| (base_token == token).then_some(score))
}

fn served_candidate_precedes(left: ServedCandidate, right: ServedCandidate) -> bool {
    match (left.source, right.source) {
        (ServedCandidateSource::Skipmix, ServedCandidateSource::Base) => true,
        (ServedCandidateSource::Base, ServedCandidateSource::Skipmix) => false,
        _ => {
            left.score.raw() > right.score.raw()
                || (left.score.raw() == right.score.raw() && left.token < right.token)
        }
    }
}

fn insert_served_candidate(result: &mut ServedCandidates, candidate: ServedCandidate) {
    let len = result.len();
    if result.ranked[..len]
        .iter()
        .any(|entry| entry.token == candidate.token)
    {
        return;
    }

    let mut at = if len < SERVED_CANDIDATE_CAPACITY {
        result.len = result.len.saturating_add(1);
        len
    } else {
        let last = SERVED_CANDIDATE_CAPACITY - 1;
        if !served_candidate_precedes(candidate, result.ranked[last]) {
            return;
        }
        last
    };
    result.ranked[at] = candidate;
    while at > 0 && served_candidate_precedes(result.ranked[at], result.ranked[at - 1]) {
        result.ranked.swap(at, at - 1);
        at -= 1;
    }
}

fn make_served_candidate(
    token: u32,
    base_score: Option<ScoreQ>,
    skipmix: Option<&SkipmixTable<'_>>,
    psi_bag: Option<&PsiBagTable<'_>>,
    unique_tokens: &[u32],
    last_token: u32,
) -> Option<ServedCandidate> {
    let contribution = skipmix_contribution(skipmix, psi_bag, unique_tokens, last_token, token);
    if contribution.raw > 0 {
        Some(ServedCandidate {
            token,
            score: ScoreQ::from_raw(contribution.raw),
            source: ServedCandidateSource::Skipmix,
            skmx_contributed: contribution.skmx_contributed,
            psib_contributed: contribution.psib_contributed,
        })
    } else {
        base_score.map(|score| ServedCandidate {
            token,
            score,
            source: ServedCandidateSource::Base,
            skmx_contributed: false,
            psib_contributed: false,
        })
    }
}

impl<'a> R4G1Runtime<'a> {
    pub fn predict_token(
        &self,
        context_tokens: &[u32],
        signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
    ) -> u32 {
        let (token, _) = self.predict_distribution(context_tokens, signature, node_scores);
        token
    }

    pub fn predict_distribution(
        &self,
        context_tokens: &[u32],
        signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
    ) -> (u32, ScoreQ) {
        self.predict_distribution_with_signature_lanes(context_tokens, signature, None, node_scores)
    }

    /// Predict with separate context and session signatures.
    ///
    /// The context signature keeps primacy in ROUT fallback. Since the
    /// #247 calibration (recorded on the issue: pinned multi-turn fixture
    /// signatures through the shipped quantizer land 24/24 within ROUT
    /// radii; declared admission criterion met), the session signature is
    /// consulted as the SECONDARY fallback probe — only when the context
    /// probe admits nothing within any calibrated radius. The session
    /// signature also participates in the emission affinity bonus below,
    /// unchanged.
    ///
    /// `node_scores` (#785 C1): the caller provides a buffer it has reset to
    /// `ScoreQ::MIN`; when the node-emission path executes, every target
    /// node whose emissions were scored receives its best final score
    /// (bounds-checked, so a short buffer simply records fewer nodes). An
    /// n-gram context-row hit returns before any node is consulted and
    /// leaves the buffer untouched — absence of node evidence stays
    /// visible, never fabricated. `predict_candidates_with_signature_lanes`
    /// reads exactly these entries to expand beyond the single distribution
    /// winner; before this contract existed the buffer was never written and
    /// the candidate walk could not return more than one token.
    pub fn predict_distribution_with_signature_lanes(
        &self,
        context_tokens: &[u32],
        context_signature: Option<&[u8]>,
        session_signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
    ) -> (u32, ScoreQ) {
        self.predict_distribution_with_signature_lanes_traced(
            context_tokens,
            context_signature,
            session_signature,
            node_scores,
        )
        .0
    }

    /// Diagnostic counterpart of
    /// [`Self::predict_distribution_with_signature_lanes`]. The prediction
    /// is identical; the additional fixed-size trace only reports which
    /// routing source supplied active nodes.
    pub fn predict_distribution_with_signature_lanes_traced(
        &self,
        context_tokens: &[u32],
        context_signature: Option<&[u8]>,
        session_signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
    ) -> ((u32, ScoreQ), SignatureRoutingTrace) {
        let mut routing = SignatureRoutingTrace::default();
        let num_nodes = self.node_count();
        if num_nodes == 0 || context_tokens.is_empty() {
            return ((0, ScoreQ::ZERO), routing);
        }

        if let Some(prediction) = context_backoff(self.chain.base_graph(), context_tokens) {
            routing.context_row_hit = true;
            routing.selected_source = SignatureRoutingSource::ContextRow;
            return (prediction, routing);
        }

        let base_graph = self.chain.base_graph();
        let emit_remainder = base_graph
            .section(SectionId::EMIT)
            .and_then(|b| if b.len() >= 4 { Some(&b[4..]) } else { None });
        // #785 C1c: EXCT is deliberately not read anywhere in this
        // engine. Both writers emit it as a storage descriptor followed
        // by a raw store container (TLS1 carryover / RX1 residual
        // tables); no artifact carries a pair-list EXCT, so the old
        // pair-scan fallbacks over it only ever produced near-saturated
        // garbage scores that drowned every real tier.
        let per_node_emissions = has_per_node_emissions(base_graph);

        let mut active_nodes = [0u32; 64];
        let mut active_len = 0usize;

        let tokens_slice = if !context_tokens.is_empty() && context_tokens[0] <= 1 {
            &context_tokens[1..]
        } else {
            context_tokens
        };
        if tokens_slice.is_empty() {
            return ((0, ScoreQ::ZERO), routing);
        }

        let max_suffix = core::cmp::min(10, tokens_slice.len());

        for suffix_len in (1..=max_suffix).rev() {
            let suffix = &tokens_slice[tokens_slice.len() - suffix_len..];

            let mut current = [0u32; 64];
            let mut current_len = 0usize;
            for n in 0..num_nodes {
                if check_node_emits(base_graph, n, suffix[0], emit_remainder).0 && current_len < 64
                {
                    current[current_len] = n;
                    current_len += 1;
                }
            }

            let mut failed = false;
            for &t in &suffix[1..] {
                let mut next_current = [0u32; 64];
                let mut next_len = 0usize;
                for &node_id in &current[..current_len] {
                    if let Some(node) = base_graph.node(node_id) {
                        let start = node.forward_start;
                        let len = node.forward_len as u32;
                        for i in 0..len {
                            let idx = start + i;
                            if let Some(edge_id) = base_graph.reverse_edge_id(idx) {
                                if self.chain.is_edge_tombstoned(edge_id) {
                                    continue;
                                }
                                if let Some(edge) = base_graph.edge(edge_id) {
                                    if edge.kind != 2 {
                                        continue;
                                    } // EDGE_KIND_TRANSITION

                                    let dst = edge.dst.0;
                                    if check_node_emits(base_graph, dst, t, emit_remainder).0
                                        && !next_current[..next_len].contains(&dst)
                                        && next_len < 64
                                    {
                                        next_current[next_len] = dst;
                                        next_len += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                current = next_current;
                current_len = next_len;
                if current_len == 0 {
                    failed = true;
                    break;
                }
            }

            if !failed && current_len > 0 {
                active_nodes[..current_len].copy_from_slice(&current[..current_len]);
                active_len = current_len;
                routing.suffix_dfa_nodes = current_len.min(usize::from(u8::MAX)) as u8;
                routing.selected_source = SignatureRoutingSource::SuffixDfa;
                break;
            }
        }

        // Geometric Routing Fallback (Phase 6 & 8)
        // If the suffix DFA fell off the manifold, use the continuous 288-bit VSA signature
        // to find the top-M semantic regions N_best (N_best <= 8) to jump back onto the graph!
        //
        // #247 admission (calibration recorded on the issue: fixture session
        // signatures land 24/24 within ROUT radii through the shipped
        // quantizer): the context signature keeps primacy, and the SESSION
        // signature is consulted as the secondary probe only when the context
        // probe admits nothing within any calibrated radius (previously such
        // positions routed via the nearest out-of-radius prototype or fell
        // through to Novel). Within-radius session routing beats
        // outside-radius context routing by the radius rule's own semantics.
        if active_len == 0 || (active_len == 1 && active_nodes[0] == 0) {
            let mut best_node = 0u32;
            let mut active_count = 0usize;
            if let Some(sig) = context_signature {
                routing.context_probe_attempted = true;
                (best_node, active_count) = self.rout_probe(base_graph, sig, &mut active_nodes);
                routing.context_admitted_nodes = active_count.min(usize::from(u8::MAX)) as u8;
                if active_count > 0 {
                    routing.selected_source = SignatureRoutingSource::ContextSignature;
                }
            }
            if active_count == 0
                && let Some(sig) = session_signature
            {
                routing.session_probe_attempted = true;
                let mut session_nodes = [0u32; 64];
                let (session_best, session_count) =
                    self.rout_probe(base_graph, sig, &mut session_nodes);
                routing.session_admitted_nodes = session_count.min(usize::from(u8::MAX)) as u8;
                if session_count > 0 {
                    active_nodes[..session_count].copy_from_slice(&session_nodes[..session_count]);
                    best_node = session_best;
                    active_count = session_count;
                    routing.selected_source = SignatureRoutingSource::SessionSignature;
                } else if best_node == 0 {
                    best_node = session_best;
                    if best_node != 0 {
                        routing.selected_source = SignatureRoutingSource::NearestSessionSignature;
                    }
                }
            }

            if active_count > 0 {
                active_len = active_count;
            } else if best_node != 0 {
                active_nodes[0] = best_node;
                active_len = 1;
                if routing.selected_source == SignatureRoutingSource::SuffixDfa
                    || routing.selected_source == SignatureRoutingSource::None
                {
                    routing.selected_source = SignatureRoutingSource::NearestContextSignature;
                }
            }
        }

        // Expand active regions with outbound transition edge neighbors
        let mut expanded_nodes = [0u32; 32];
        let mut expanded_len = 0usize;
        for &node_id in &active_nodes[..active_len] {
            if expanded_len < 32 && !expanded_nodes[..expanded_len].contains(&node_id) {
                expanded_nodes[expanded_len] = node_id;
                expanded_len += 1;
            }
            if let Some(node) = base_graph.node(node_id)
                && node.forward_len > 0
            {
                let start = node.forward_start as usize;
                let count = (node.forward_len as usize).min(4);
                for i in 0..count {
                    if let Some(rev_id) = base_graph.reverse_edge_id((start + i) as u32)
                        && let Some(edge) = base_graph.edge(rev_id)
                        && expanded_len < 32
                        && !expanded_nodes[..expanded_len].contains(&edge.dst.0)
                    {
                        expanded_nodes[expanded_len] = edge.dst.0;
                        expanded_len += 1;
                    }
                }
            }
        }

        if expanded_len > 0 {
            active_nodes[..expanded_len].copy_from_slice(&expanded_nodes[..expanded_len]);
            active_len = expanded_len;
        }

        if active_len == 0 {
            active_nodes[0] = 0;
            active_len = 1;
            routing.selected_source = SignatureRoutingSource::DefaultNode;
        }

        let mut best_token = 0;
        let mut best_score = ScoreQ::MIN;

        // Read predicted tokens directly from active node emission lists (and child refinement lists)
        for &node_id in &active_nodes[..active_len] {
            if let Some(_node) = base_graph.node(node_id) {
                let mut target_nodes = [0u32; 128];
                let mut num_targets = 0usize;
                collect_target_leaf_nodes(base_graph, node_id, &mut target_nodes, &mut num_targets);

                for &target_id in &target_nodes[..num_targets] {
                    let target_node = match base_graph.node(target_id) {
                        Some(tn) => tn,
                        None => continue,
                    };

                    let sl = if target_node.emission_len == 0 {
                        if target_id == 0 && !per_node_emissions {
                            // Converter-flavor graph: the whole EMIT
                            // remainder is the root prior (token, count)
                            // table, served as node 0's list. On a scored
                            // graph the remainder is a root-prior block
                            // plus per-region lists and must only be read
                            // through per-node ranges (#785 C1c).
                            emit_remainder.unwrap_or(&[])
                        } else {
                            &[][..]
                        }
                    } else if let Some(remainder) = emit_remainder {
                        let start = target_node.emission_start as usize;
                        let len = target_node.emission_len as usize;
                        if start + len <= remainder.len() {
                            &remainder[start..start + len]
                        } else {
                            &[][..]
                        }
                    } else {
                        &[][..]
                    };
                    for i in 0..(sl.len() >> 3) {
                        let offset = i << 3;
                        let cand = u32::from_le_bytes([
                            sl[offset],
                            sl[offset + 1],
                            sl[offset + 2],
                            sl[offset + 3],
                        ]);
                        if cand == 0 || cand >= 49152 {
                            continue;
                        }
                        let raw = i32::from_le_bytes([
                            sl[offset + 4],
                            sl[offset + 5],
                            sl[offset + 6],
                            sl[offset + 7],
                        ]);

                        let mut emit_score = if raw > 0 {
                            ScoreQ::from_raw(raw)
                        } else {
                            ScoreQ::from_raw(1)
                        };

                        let sig_bonus = if let Some(sig) = session_signature.or(context_signature) {
                            let rout_bytes = base_graph.section(SectionId::ROUT).unwrap_or(&[]);
                            let proto_offset = (target_node.prototype_word_start as usize) << 3;
                            let mask_offset = (target_node.mask_word_start as usize) << 3;
                            if proto_offset + sig.len() <= rout_bytes.len()
                                && mask_offset + sig.len() <= rout_bytes.len()
                            {
                                signature_affinity_bonus(
                                    &rout_bytes[proto_offset..proto_offset + sig.len()],
                                    &rout_bytes[mask_offset..mask_offset + sig.len()],
                                    sig,
                                )
                            } else {
                                0
                            }
                        } else {
                            0
                        };

                        let mut penalty = 0i32;
                        let recent_window = 48;
                        let start_pos = context_tokens.len().saturating_sub(recent_window);
                        for (idx, &recent_tok) in context_tokens[start_pos..].iter().enumerate() {
                            if cand == recent_tok {
                                let age = context_tokens.len() - (start_pos + idx);
                                // x * 350 == (x<<8)+(x<<6)+(x<<4)+(x<<3)+(x<<2)+(x<<1) (shift/add only).
                                let x = 48 - age as i32;
                                penalty += (x << 8)
                                    .saturating_add(x << 6)
                                    .saturating_add(x << 4)
                                    .saturating_add(x << 3)
                                    .saturating_add(x << 2)
                                    .saturating_add(x << 1);
                            }
                        }

                        // #400: the Cayley-Dickson morphism term and its
                        // punctuation gating were removed as measured-dead
                        // code — this node-candidate path executes only when
                        // context_backoff misses, which was 0/1998 sampled
                        // positions on a rows-on bundle (issue #400 record).
                        let final_score = emit_score
                            .raw()
                            .saturating_add(sig_bonus)
                            .saturating_sub(penalty);
                        emit_score = ScoreQ::from_raw(final_score);

                        // #785 C1: publish this node's best final score so
                        // the top-k candidate walk can expand genuinely
                        // active nodes instead of gating on a never-written
                        // buffer.
                        if let Some(slot) = node_scores.get_mut(target_id as usize)
                            && emit_score.raw() > slot.raw()
                        {
                            *slot = emit_score;
                        }

                        if emit_score.raw() > best_score.raw()
                            || (best_token != 0
                                && emit_score.raw() == best_score.raw()
                                && cand < best_token)
                        {
                            best_score = emit_score;
                            best_token = cand;
                        }
                    }
                }
            }
        }

        if best_token == 0 && !per_node_emissions {
            // Converter-flavor last resort: emit the first usable token
            // from the global root-prior table. A scored graph's EMIT
            // remainder starts with the root-prior block header, which is
            // not a pair list — an unmatched scored-graph query returns
            // (0, ...) honestly instead (#785 C1c).
            if let Some(remainder) = emit_remainder {
                for i in 0..(remainder.len() >> 3) {
                    let offset = i << 3;
                    let cand = u32::from_le_bytes([
                        remainder[offset],
                        remainder[offset + 1],
                        remainder[offset + 2],
                        remainder[offset + 3],
                    ]);
                    if cand > 2 && cand < 49152 {
                        best_token = cand;
                        break;
                    }
                }
            }
        }

        ((best_token, best_score), routing)
    }

    /// Predict top-k candidate tokens with their scores for Beam Search decoding.
    pub fn predict_candidates(
        &self,
        context_tokens: &[u32],
        signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
        out_candidates: &mut [(u32, ScoreQ); 8],
    ) -> usize {
        self.predict_candidates_with_signature_lanes(
            context_tokens,
            signature,
            None,
            node_scores,
            out_candidates,
        )
    }

    /// Top-k counterpart of
    /// [`Self::predict_distribution_with_signature_lanes`].
    pub fn predict_candidates_with_signature_lanes(
        &self,
        context_tokens: &[u32],
        context_signature: Option<&[u8]>,
        session_signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
        out_candidates: &mut [(u32, ScoreQ); 8],
    ) -> usize {
        let (top_tok, top_score) = self.predict_distribution_with_signature_lanes(
            context_tokens,
            context_signature,
            session_signature,
            node_scores,
        );

        let mut count = 0usize;
        if top_tok != 0 {
            out_candidates[0] = (top_tok, top_score);
            count = 1;
        }

        // #785 C1: a context-row hit resolves before any node is consulted,
        // so `node_scores` stays empty on those steps — but the winning row
        // itself carries the alternative continuations. Surface them here so
        // the candidate walk is real on the n-gram tier too, not only the
        // node path. Same reserved-token guard and dedup as the node walk.
        with_context_row_entries(self.chain.base_graph(), context_tokens, |entry| {
            if count >= 8 {
                return;
            }
            let cand = entry.token;
            if cand > 2 && cand < 49152 && !out_candidates[..count].iter().any(|(c, _)| *c == cand)
            {
                out_candidates[count] = (cand, entry.score_q);
                count += 1;
            }
        });

        // Iterate over the top active nodes in node_scores to collect candidate tokens
        let emit_remainder = self.view().section(SectionId::EMIT);
        if let Some(remainder) = emit_remainder {
            let view = self.view();
            let max_nodes = core::cmp::min(self.node_count() as usize, node_scores.len());
            #[allow(clippy::needless_range_loop)]
            for node_idx in 0..max_nodes {
                if count >= 8 {
                    break;
                }
                let n_score = node_scores[node_idx];
                if n_score.raw() == ScoreQ::MIN.raw() {
                    continue;
                }
                if let Some(target_node) = view.node(node_idx as u32) {
                    let start = target_node.emission_start as usize;
                    let len = target_node.emission_len as usize;
                    if start + len <= remainder.len() {
                        let sl = &remainder[start..start + len];
                        for i in 0..(sl.len() >> 3) {
                            if count >= 8 {
                                break;
                            }
                            let offset = i << 3;
                            let cand = u32::from_le_bytes([
                                sl[offset],
                                sl[offset + 1],
                                sl[offset + 2],
                                sl[offset + 3],
                            ]);
                            let raw = i32::from_le_bytes([
                                sl[offset + 4],
                                sl[offset + 5],
                                sl[offset + 6],
                                sl[offset + 7],
                            ]);

                            if cand > 2
                                && cand < 49152
                                && !out_candidates[..count].iter().any(|(c, _)| *c == cand)
                            {
                                out_candidates[count] = (cand, ScoreQ::from_raw(raw));
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        out_candidates[..count].sort_by(|left, right| {
            right
                .1
                .raw()
                .cmp(&left.1.raw())
                .then_with(|| left.0.cmp(&right.0))
        });
        count
    }

    /// Normative fixed-capacity served candidates with optional skip-mix
    /// candidate injection and attribution.
    pub fn predict_served_candidates(
        &self,
        context_tokens: &[u32],
        signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
    ) -> ServedCandidates {
        self.predict_served_candidates_with_signature_lanes(
            context_tokens,
            signature,
            None,
            node_scores,
        )
    }

    /// Signature-lane counterpart of [`Self::predict_served_candidates`].
    ///
    /// The base graph shortlist is retained exactly when both learned tables
    /// are absent. When either is present, candidates learned from the
    /// distinct tokens in the newest eight-token compiler window join the
    /// same fixed shortlist. A present joint `(content, last)` row always
    /// gates the PSIB fallback for that content, even when the joint row lacks
    /// a particular candidate.
    pub fn predict_served_candidates_with_signature_lanes(
        &self,
        context_tokens: &[u32],
        context_signature: Option<&[u8]>,
        session_signature: Option<&[u8]>,
        node_scores: &mut [ScoreQ],
    ) -> ServedCandidates {
        let mut base = [(0u32, ScoreQ::MIN); SERVED_CANDIDATE_CAPACITY];
        let base_len = self.predict_candidates_with_signature_lanes(
            context_tokens,
            context_signature,
            session_signature,
            node_scores,
            &mut base,
        );

        let mut result = ServedCandidates::default();
        if base_len == 0 {
            return result;
        }
        if self.skipmix.is_none() && self.psi_bag.is_none() {
            for &(token, score) in &base[..base_len] {
                insert_served_candidate(
                    &mut result,
                    ServedCandidate {
                        token,
                        score,
                        source: ServedCandidateSource::Base,
                        skmx_contributed: false,
                        psib_contributed: false,
                    },
                );
            }
            return result;
        }

        let Some(&last_token) = context_tokens.last() else {
            return result;
        };
        let (unique, unique_len) = unique_tokens_in_newest_compiler_window(context_tokens);
        let unique = &unique[..unique_len];

        for &(token, score) in &base[..base_len] {
            if let Some(candidate) = make_served_candidate(
                token,
                Some(score),
                self.skipmix.as_ref(),
                self.psi_bag.as_ref(),
                unique,
                last_token,
            ) {
                insert_served_candidate(&mut result, candidate);
            }
        }

        let vocab_size = self.view().head().map_or(49152, |head| head.vocab_size());
        for &content_token in unique {
            let entries = if let Some(row) = self
                .skipmix
                .as_ref()
                .and_then(|table| table.find(content_token, last_token))
            {
                Some(row.entries())
            } else {
                self.psi_bag
                    .as_ref()
                    .and_then(|table| table.find(content_token))
                    .map(|row| row.entries())
            };
            let Some(entries) = entries else {
                continue;
            };
            for entry in entries.iter() {
                // Teacher argmax rows may legitimately select BOS/EOS or
                // another reserved token. They remain candidates here; the
                // decode policy decides whether a selected special token
                // terminates generation. Only an out-of-vocabulary id is
                // structurally invalid for this graph.
                if entry.token >= vocab_size {
                    continue;
                }
                let base_score = base_candidate_score(&base, base_len, entry.token);
                if let Some(candidate) = make_served_candidate(
                    entry.token,
                    base_score,
                    self.skipmix.as_ref(),
                    self.psi_bag.as_ref(),
                    unique,
                    last_token,
                ) {
                    insert_served_candidate(&mut result, candidate);
                }
            }
        }

        let base_token = base[0].0;
        if let Some(winner) = result.winner()
            && winner.source == ServedCandidateSource::Skipmix
            && winner.token != base_token
        {
            let contribution = skipmix_contribution(
                self.skipmix.as_ref(),
                self.psi_bag.as_ref(),
                unique,
                last_token,
                winner.token,
            );
            debug_assert_eq!(contribution.raw, winner.score.raw());
            debug_assert!(contribution.skmx_contributed || contribution.psib_contributed);
            result.attribution = Some(SkipmixAttribution {
                base_token,
                promoted_token: winner.token,
                contribution: winner.score,
                skmx_contributed: contribution.skmx_contributed,
                psib_contributed: contribution.psib_contributed,
            });
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::signature_affinity_bonus;

    #[test]
    fn session_lane_bonus_is_integer_hamming_affinity() {
        let prototype = [0u8; 36];
        let mask = [0xffu8; 36];
        let near = [0u8; 36];
        let far = [0xffu8; 36];
        assert!(
            signature_affinity_bonus(&prototype, &mask, &near)
                > signature_affinity_bonus(&prototype, &mask, &far)
        );
    }
}
