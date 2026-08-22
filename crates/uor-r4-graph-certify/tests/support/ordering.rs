//! The #845 reference search skeleton — the deployed bounded layered search
//! (`uor_r4_graph_runtime::plan`) ported faithfully with exactly one seam
//! opened: the frontier-retention score. Certifier-instrument code.
//!
//! Parity is a proven-by-test property, not an aspiration: with the incumbent
//! goal-distance scorer this skeleton must reproduce the deployed planner's
//! outcome, plan, and every counter field, episode by episode (breadth-first
//! and beam alike; asserted in `w33_ordering_harness_845.rs`). Counter
//! accounting therefore mirrors the deployed `Episode` call-for-call,
//! including the post-search unwind and considered-record reads.
//!
//! Scoring work an ordering performs is deliberately *not* part of
//! `PlanCounters` (the deployed scorer's goal-distance reads are uncounted
//! there too); each scorer reports its auxiliary lookups and table bytes
//! separately for the equal-bytes/equal-operations audit.
#![allow(dead_code)]

use uor_r4_graph_format::plan::{
    EffectDelta, SlotVec, PLAN_ACTIONS_MAX, PLAN_FRONTIER_MAX, PLAN_HORIZON_MAX, PLAN_SLOTS_MAX,
    PLAN_VISITED_MAX,
};
use uor_r4_graph_format::plan_sections::{PackedDecline, PlanSchema, PredicateSet, RuleTable};
use uor_r4_graph_runtime::plan::PlanCounters;

const VISITED_MAX_PROBE: u8 = 16;
const VISITED_INDEX_SLOTS: usize = PLAN_VISITED_MAX * 2;
const VISITED_INDEX_MASK: u32 = (VISITED_INDEX_SLOTS - 1) as u32;
const EMPTY: u16 = u16::MAX;

/// A frontier-retention scorer: higher survives the frontier bound. The
/// incumbent (deployed-parity) scorer returns the negated goal distance.
pub trait Scorer {
    /// Retention score of an admitted successor against the episode's goal.
    fn score(&mut self, successor: &SlotVec) -> i32;
    /// Auxiliary table lookups spent scoring so far (reported, not budgeted).
    fn lookups(&self) -> u64;
    /// Bytes of auxiliary tables this ordering consults (byte-parity audit).
    fn table_bytes(&self) -> usize;
    /// Stable arm label.
    fn name(&self) -> &'static str;
}

/// The reference episode outcome: a plan as (effect, resulting state,
/// operator, rule row) steps, or the typed decline the deployed planner would
/// report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefOutcome {
    Plan(Vec<(EffectDelta, SlotVec, u16, u16)>),
    Declined(PackedDecline),
}

/// One reference episode result: outcome plus deployed-equivalent counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefResult {
    pub outcome: RefOutcome,
    pub counters: PlanCounters,
}

/// The query, mirroring the deployed `PlanQuery` field for field.
pub struct RefQuery<'a, 'b> {
    pub schema: &'a PlanSchema<'b>,
    pub rules: &'a RuleTable<'b>,
    pub predicates: &'a PredicateSet<'b>,
    pub initial: SlotVec,
    pub available: u64,
    pub horizon: u8,
    pub frontier: u16,
    pub max_expansions: u32,
    pub max_candidates: u32,
    pub max_table_reads: u32,
}

/// The deployed multiply-free state hash, ported unchanged (Jenkins
/// one-at-a-time over the slot bytes).
fn hash_state(state: &SlotVec) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for value in state.as_slice() {
        let bytes = value.to_le_bytes();
        for byte in bytes {
            hash = hash.wrapping_add(u32::from(byte));
            hash = hash.wrapping_add(hash << 10);
            hash ^= hash >> 6;
        }
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash.wrapping_add(hash << 15)
}

/// The deployed saturating goal distance, ported unchanged.
pub fn goal_distance_for(predicates: &PredicateSet<'_>, state: &SlotVec) -> i32 {
    let Some(goal) = predicates.goal(0) else {
        return 0;
    };
    let mut total: i32 = 0;
    for slot in 0..PLAN_SLOTS_MAX {
        if goal.read_mask() & (1u8 << slot) == 0 {
            continue;
        }
        let Some(value) = state.get(slot) else {
            continue;
        };
        let bound = goal.bound(slot);
        let gap = if value >= bound {
            value.saturating_sub(bound)
        } else {
            bound.saturating_sub(value)
        };
        total = total.saturating_add(i32::from(gap));
    }
    total
}

/// The incumbent retention scorer: exactly the deployed beam's signal.
pub struct GoalDistanceScorer<'a, 'b> {
    predicates: &'a PredicateSet<'b>,
}

impl<'a, 'b> GoalDistanceScorer<'a, 'b> {
    pub fn new(predicates: &'a PredicateSet<'b>) -> Self {
        Self { predicates }
    }
}

impl Scorer for GoalDistanceScorer<'_, '_> {
    fn score(&mut self, successor: &SlotVec) -> i32 {
        -goal_distance_for(self.predicates, successor)
    }
    fn lookups(&self) -> u64 {
        0
    }
    fn table_bytes(&self) -> usize {
        0
    }
    fn name(&self) -> &'static str {
        "table-guided-beam"
    }
}

#[derive(Clone, Copy)]
struct Node {
    state: SlotVec,
    parent: u16,
    rule_row: u16,
    operator: u16,
    depth: u8,
}

#[derive(Clone, Copy)]
struct Frame {
    state: SlotVec,
    node: u16,
    depth: u8,
}

/// Reference scratch: same capacities as the deployed scratch, Vec-backed
/// (reference scope allows allocation).
pub struct RefScratch {
    visited: Vec<Node>,
    index: Vec<u16>,
    frontier: Vec<Frame>,
    next: Vec<Frame>,
    scores: Vec<i32>,
}

impl Default for RefScratch {
    fn default() -> Self {
        Self::new()
    }
}

impl RefScratch {
    pub fn new() -> Self {
        Self {
            visited: Vec::with_capacity(PLAN_VISITED_MAX),
            index: vec![EMPTY; VISITED_INDEX_SLOTS],
            frontier: Vec::with_capacity(PLAN_FRONTIER_MAX),
            next: Vec::with_capacity(PLAN_FRONTIER_MAX),
            scores: Vec::with_capacity(PLAN_FRONTIER_MAX),
        }
    }

    fn reset(&mut self) {
        self.visited.clear();
        self.frontier.clear();
        self.next.clear();
        self.scores.clear();
        for slot in self.index.iter_mut() {
            *slot = EMPTY;
        }
    }
}

enum Search {
    Found(u16),
    Exhausted,
    Declined(PackedDecline),
}

enum Step {
    Taken(u16),
    Blocked,
    Capacity,
    Unknown,
}

struct Episode<'q, 'a, 'b> {
    query: &'q RefQuery<'a, 'b>,
    counters: PlanCounters,
}

impl Episode<'_, '_, '_> {
    fn read(&mut self, count: u32) {
        self.counters.table_reads = self.counters.table_reads.saturating_add(count);
    }

    fn op(&mut self, count: u32) {
        self.counters.integer_ops = self.counters.integer_ops.saturating_add(count);
    }

    fn over_budget(&self) -> bool {
        self.counters.expansions > self.query.max_expansions
            || self.counters.candidates > self.query.max_candidates
            || self.counters.table_reads > self.query.max_table_reads
    }

    fn admit(&mut self, scratch: &mut RefScratch, state: &SlotVec) -> Option<Option<u16>> {
        let mut slot = (hash_state(state) & VISITED_INDEX_MASK) as usize;
        for probe in 0..=VISITED_MAX_PROBE {
            if probe == VISITED_MAX_PROBE {
                return None;
            }
            self.counters.max_probe = self.counters.max_probe.max(probe.saturating_add(1));
            self.read(1);
            let occupant = scratch.index[slot];
            if occupant == EMPTY {
                if scratch.visited.len() >= PLAN_VISITED_MAX {
                    return None;
                }
                let node = scratch.visited.len() as u16;
                scratch.index[slot] = node;
                scratch.visited.push(Node {
                    state: *state,
                    parent: EMPTY,
                    rule_row: 0,
                    operator: 0,
                    depth: 0,
                });
                return Some(Some(node));
            }
            self.read(1);
            if scratch.visited[usize::from(occupant)].state == *state {
                return Some(None);
            }
            slot = (slot + 1) & (VISITED_INDEX_SLOTS - 1);
        }
        None
    }

    fn take(
        &mut self,
        scratch: &mut RefScratch,
        state: &SlotVec,
        parent: u16,
        depth: u8,
        operator: u16,
        rule_row: usize,
    ) -> Step {
        self.counters.candidates = self.counters.candidates.saturating_add(1);
        self.read(1);
        let Some(rule) = self.query.rules.rule(rule_row) else {
            return Step::Unknown;
        };
        self.op(1);
        if !rule.precondition.holds(state) {
            return Step::Blocked;
        }
        let Some(successor) = state.apply(&rule.effect) else {
            return Step::Unknown;
        };
        self.op(successor.len() as u32);
        self.read(1);
        if self.query.predicates.is_forbidden(&successor) {
            return Step::Blocked;
        }
        match self.admit(scratch, &successor) {
            None => Step::Capacity,
            Some(None) => Step::Blocked,
            Some(Some(node)) => {
                scratch.visited[usize::from(node)] = Node {
                    state: successor,
                    parent,
                    rule_row: rule_row as u16,
                    operator,
                    depth: depth.saturating_add(1),
                };
                Step::Taken(node)
            }
        }
    }

    fn unwind(
        &mut self,
        scratch: &RefScratch,
        node: u16,
    ) -> Option<Vec<(EffectDelta, SlotVec, u16, u16)>> {
        let mut depth = usize::from(scratch.visited[usize::from(node)].depth);
        if depth > PLAN_HORIZON_MAX {
            return None;
        }
        let mut path = vec![(EffectDelta::EMPTY, SlotVec::empty(), 0u16, 0u16); depth];
        let mut cursor = node;
        while depth > 0 {
            let current = scratch.visited[usize::from(cursor)];
            let parent = current.parent;
            if parent == EMPTY {
                return None;
            }
            let from = scratch.visited[usize::from(parent)].state;
            let rule = self.query.rules.rule(usize::from(current.rule_row))?;
            self.read(1);
            match from.apply(&rule.effect) {
                Some(next) if next == current.state => {}
                _ => return None,
            }
            depth -= 1;
            path[depth] = (
                rule.effect,
                current.state,
                current.operator,
                current.rule_row,
            );
            cursor = parent;
        }
        Some(path)
    }

    /// The deployed considered-record post-pass, ported for its counter
    /// effects (one table read per available ruled operator per path step);
    /// the record itself is informational and is not rebuilt here.
    fn record_considered_reads(&mut self, steps: usize) {
        let operators = self.query.schema.operator_count().min(PLAN_ACTIONS_MAX);
        for _step in 0..steps {
            for operator in 0..operators {
                if self.query.available & (1u64 << operator) == 0 {
                    continue;
                }
                let Some((first, end)) = self.query.rules.rules_for(operator) else {
                    continue;
                };
                if first >= end {
                    continue;
                }
                self.read(1);
                let _ = self.query.rules.rule(first);
            }
        }
    }
}

/// How the seam is opened.
///
/// - `Parity(false)` — FIFO retention, arrival-order expansion: the deployed
///   breadth-first, counter-exact.
/// - `Parity(true)` — score retention, arrival-order expansion: the deployed
///   table-guided beam, counter-exact (with the goal-distance scorer).
/// - `Arm` — score retention AND the retained layer expands in descending
///   score order (stable): the arm mode of spec §3/§4-A, where an ordering
///   can actually reduce expansions by reaching the goal-generating
///   expansion sooner. Same budget accounting as the deployed planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeamMode {
    Parity(bool),
    Arm,
}

impl SeamMode {
    fn ordered(self) -> bool {
        match self {
            SeamMode::Parity(ordered) => ordered,
            SeamMode::Arm => true,
        }
    }

    fn sorted_expansion(self) -> bool {
        matches!(self, SeamMode::Arm)
    }
}

/// Run one reference episode: the deployed layered search with the seam
/// opened per `mode`.
pub fn plan_reference(
    query: &RefQuery<'_, '_>,
    scratch: &mut RefScratch,
    mode: SeamMode,
    scorer: &mut dyn Scorer,
) -> RefResult {
    scratch.reset();
    let mut episode = Episode {
        query,
        counters: PlanCounters::default(),
    };

    if query.predicates.satisfies_goal(&query.initial) {
        return RefResult {
            outcome: RefOutcome::Plan(Vec::new()),
            counters: episode.counters,
        };
    }
    if query.predicates.is_forbidden(&query.initial) {
        return declined(PackedDecline::NoPlan, episode.counters);
    }
    if usize::from(query.horizon) > PLAN_HORIZON_MAX
        || usize::from(query.frontier) > PLAN_FRONTIER_MAX
    {
        return declined(PackedDecline::Capacity, episode.counters);
    }

    let Some(Some(root)) = episode.admit(scratch, &query.initial) else {
        return declined(PackedDecline::Capacity, episode.counters);
    };
    scratch.visited[usize::from(root)] = Node {
        state: query.initial,
        parent: EMPTY,
        rule_row: 0,
        operator: 0,
        depth: 0,
    };

    match search_layered(&mut episode, scratch, root, mode, scorer) {
        Search::Found(node) => match episode.unwind(scratch, node) {
            Some(path) => {
                episode.record_considered_reads(path.len());
                RefResult {
                    outcome: RefOutcome::Plan(path),
                    counters: episode.counters,
                }
            }
            None => declined(PackedDecline::Unknown, episode.counters),
        },
        Search::Exhausted => declined(PackedDecline::NoPlan, episode.counters),
        Search::Declined(reason) => declined(reason, episode.counters),
    }
}

fn declined(reason: PackedDecline, counters: PlanCounters) -> RefResult {
    RefResult {
        outcome: RefOutcome::Declined(reason),
        counters,
    }
}

fn search_layered(
    episode: &mut Episode<'_, '_, '_>,
    scratch: &mut RefScratch,
    root: u16,
    mode: SeamMode,
    scorer: &mut dyn Scorer,
) -> Search {
    let ordered = mode.ordered();
    scratch.frontier.clear();
    scratch.frontier.push(Frame {
        state: scratch.visited[usize::from(root)].state,
        node: root,
        depth: 0,
    });

    for _ in 0..episode.query.horizon {
        scratch.next.clear();
        scratch.scores.clear();
        let width = scratch.frontier.len();
        for slot in 0..width {
            if episode.over_budget() {
                return Search::Declined(PackedDecline::Capacity);
            }
            let frame = scratch.frontier[slot];
            episode.counters.expansions = episode.counters.expansions.saturating_add(1);
            let operators = episode.query.schema.operator_count().min(PLAN_ACTIONS_MAX);
            for operator in 0..operators {
                if episode.query.available & (1u64 << operator) == 0 {
                    continue;
                }
                let Some((first, end)) = episode.query.rules.rules_for(operator) else {
                    continue;
                };
                for row in first..end {
                    match episode.take(
                        scratch,
                        &frame.state,
                        frame.node,
                        frame.depth,
                        operator as u16,
                        row,
                    ) {
                        Step::Taken(node) => {
                            let successor = scratch.visited[usize::from(node)].state;
                            if episode.query.predicates.satisfies_goal(&successor) {
                                return Search::Found(node);
                            }
                            if scratch.next.len() >= usize::from(episode.query.frontier) {
                                if ordered {
                                    replace_weakest(scratch, scorer, node, successor, frame.depth);
                                }
                                continue;
                            }
                            scratch.next.push(Frame {
                                state: successor,
                                node,
                                depth: frame.depth.saturating_add(1),
                            });
                            scratch.scores.push(scorer.score(&successor));
                        }
                        Step::Blocked => {}
                        Step::Capacity => return Search::Declined(PackedDecline::Capacity),
                        Step::Unknown => return Search::Declined(PackedDecline::Unknown),
                    }
                }
            }
        }
        if scratch.next.is_empty() {
            return Search::Exhausted;
        }
        if mode.sorted_expansion() {
            // Arm mode: expand the retained layer in descending score order.
            // The sort is stable, so equal scores keep the canonical arrival
            // order and determinism is preserved.
            let mut order: Vec<usize> = (0..scratch.next.len()).collect();
            order.sort_by_key(|slot| std::cmp::Reverse(scratch.scores[*slot]));
            let reordered: Vec<Frame> = order.iter().map(|slot| scratch.next[*slot]).collect();
            let rescored: Vec<i32> = order.iter().map(|slot| scratch.scores[*slot]).collect();
            scratch.next.clear();
            scratch.next.extend_from_slice(&reordered);
            scratch.scores.clear();
            scratch.scores.extend_from_slice(&rescored);
        }
        scratch.frontier.clear();
        scratch.frontier.extend_from_slice(&scratch.next);
    }
    Search::Exhausted
}

fn replace_weakest(
    scratch: &mut RefScratch,
    scorer: &mut dyn Scorer,
    node: u16,
    successor: SlotVec,
    depth: u8,
) {
    let score = scorer.score(&successor);
    let mut weakest = 0usize;
    for slot in 1..scratch.next.len() {
        if scratch.scores[slot] < scratch.scores[weakest] {
            weakest = slot;
        }
    }
    if scratch.next.is_empty() || score <= scratch.scores[weakest] {
        return;
    }
    scratch.scores[weakest] = score;
    scratch.next[weakest] = Frame {
        state: successor,
        node,
        depth: depth.saturating_add(1),
    };
}
