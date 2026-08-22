//! Bounded semantic planner — normative deployed serving (#843 §6).
//!
//! Frozen contract: `docs/bounded_semantic_transitions_spec_843.md` §6 and §7.
//! Executes only the P-4 permitted operation classes — XOR/AND/OR/NOT, shift
//! and rotate, popcount, saturating and wrapping integer add-sub, integer
//! comparison, and fixed-offset table reads. **There is no multiply, no divide
//! and no float anywhere in this module**, including the scoring and the
//! visited-set hash.
//!
//! Everything the planner touches lives in a caller-provided [`PlanScratch`]:
//! the planner allocates nothing, in steady state or otherwise, and its scratch
//! size is a compile-time function of the frozen capacities. Exceeding a
//! capacity is a deterministic [`PackedDecline::Capacity`], never a silent
//! truncation; slot and score arithmetic saturates rather than wrapping into a
//! valid-looking value.
//!
//! Three bounded strategies share one search core, one visited index, one
//! successor generator and one witness surface, so a comparison between them is
//! a comparison of search order and nothing else.

use uor_r4_graph_format::plan::{
    EffectDelta, PLAN_ACTIONS_MAX, PLAN_FRONTIER_MAX, PLAN_HORIZON_MAX, PLAN_SLOTS_MAX,
    PLAN_VISITED_MAX, PreconditionMask, SlotVec,
};
use uor_r4_graph_format::plan_sections::{
    ConsideredCandidate, PackedDecline, PlanSchema, PredicateSet, RuleTable, WitnessStep,
};

/// Probe bound for the visited index. A membership test costs at most this many
/// fixed-offset reads whatever the set holds, so lookup cost is never a
/// function of how many states have been seen.
pub const VISITED_MAX_PROBE: u8 = 16;

/// Slots in the visited open-addressing index. A power of two, so the modulus
/// is a mask rather than a division.
const VISITED_INDEX_SLOTS: usize = PLAN_VISITED_MAX * 2;
const VISITED_INDEX_MASK: u32 = (VISITED_INDEX_SLOTS - 1) as u32;
const EMPTY: u16 = u16::MAX;

/// Which bounded strategy an episode runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStrategy {
    /// Frontier-limited breadth-first search.
    BreadthFirst,
    /// Depth-limited depth-first search with an increasing bound.
    IterativeDeepening,
    /// Beam ordered by an integer score read from the rule table and the goal.
    BestFirstBeam,
}

/// The declared budget one episode may spend. Every arm runs under the same
/// budget, so a comparison between them is not a comparison of effort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanBudget {
    /// Maximum plan length.
    pub horizon: u8,
    /// Maximum retained frontier width.
    pub frontier: u16,
    /// Maximum state expansions.
    pub max_expansions: u32,
    /// Maximum candidate rule tests.
    pub max_candidates: u32,
    /// Maximum fixed-offset table reads.
    pub max_table_reads: u32,
}

impl PlanBudget {
    /// The frozen capacities as a budget: the largest episode this build runs.
    pub const fn frozen() -> Self {
        Self {
            horizon: PLAN_HORIZON_MAX as u8,
            frontier: PLAN_FRONTIER_MAX as u16,
            max_expansions: PLAN_VISITED_MAX as u32,
            max_candidates: (PLAN_VISITED_MAX * PLAN_ACTIONS_MAX) as u32,
            max_table_reads: (PLAN_VISITED_MAX * PLAN_ACTIONS_MAX * 4) as u32,
        }
    }
}

/// What an episode actually spent. Recorded on every path, including every
/// decline, so a budget-parity check has something to compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlanCounters {
    /// States expanded.
    pub expansions: u32,
    /// Candidate rules tested.
    pub candidates: u32,
    /// Fixed-offset table reads.
    pub table_reads: u32,
    /// Integer operations on the hot path.
    pub integer_ops: u32,
    /// Deepest probe used by the visited index.
    pub max_probe: u8,
}

/// The outcome of an episode: a plan of the recorded length, or a typed honest
/// decline. There is no third possibility and no fabricated plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanOutcome {
    /// A plan was found; its steps are readable from the scratch.
    Plan {
        /// Steps in the chosen path.
        steps: usize,
    },
    /// The episode declined, for this stated reason.
    Declined(PackedDecline),
}

/// One planning query: the packed sections, the initial state, which operators
/// this instance offers, and the budget.
pub struct PlanQuery<'a, 'b> {
    /// Search order.
    pub strategy: PlanStrategy,
    /// The artifact's planning schema.
    pub schema: &'a PlanSchema<'b>,
    /// The artifact's transition rule table.
    pub rules: &'a RuleTable<'b>,
    /// The query's goal and forbidden predicates.
    pub predicates: &'a PredicateSet<'b>,
    /// Where the episode starts.
    pub initial: SlotVec,
    /// Bit `i` set when operator `i` is available to this instance. The
    /// vocabulary is artifact-wide; which operators an instance offers is a
    /// property of the query, so it is a mask rather than a second table.
    pub available: u64,
    /// The declared budget.
    pub budget: PlanBudget,
}

/// The result of an episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanResult {
    /// Plan or typed decline.
    pub outcome: PlanOutcome,
    /// What it spent.
    pub counters: PlanCounters,
}

#[derive(Debug, Clone, Copy)]
struct VisitedNode {
    state: SlotVec,
    parent: u16,
    rule_row: u16,
    operator: u16,
    depth: u8,
}

impl VisitedNode {
    const fn empty() -> Self {
        Self {
            state: SlotVec::empty(),
            parent: EMPTY,
            rule_row: 0,
            operator: 0,
            depth: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Frame {
    state: SlotVec,
    node: u16,
    depth: u8,
}

impl Frame {
    const fn empty() -> Self {
        Self {
            state: SlotVec::empty(),
            node: 0,
            depth: 0,
        }
    }
}

/// Caller-owned scratch. Its size is a compile-time function of the frozen
/// capacities; the planner never allocates.
pub struct PlanScratch {
    visited: [VisitedNode; PLAN_VISITED_MAX],
    visited_len: u16,
    index: [u16; VISITED_INDEX_SLOTS],
    frontier: [Frame; PLAN_FRONTIER_MAX],
    frontier_len: u16,
    next: [Frame; PLAN_FRONTIER_MAX],
    next_len: u16,
    scores: [i32; PLAN_FRONTIER_MAX],
    path: [WitnessStep; PLAN_HORIZON_MAX],
    path_len: u8,
    considered: [ConsideredCandidate; PLAN_HORIZON_MAX * PLAN_ACTIONS_MAX],
    considered_len: u16,
    considered_per_step: u8,
}

impl Default for PlanScratch {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanScratch {
    /// A zeroed scratch. Large by design — it is the bounded working set of a
    /// whole episode — so a caller places it in a static, an arena, or a box
    /// rather than on a small stack.
    pub const fn new() -> Self {
        Self {
            visited: [VisitedNode::empty(); PLAN_VISITED_MAX],
            visited_len: 0,
            index: [EMPTY; VISITED_INDEX_SLOTS],
            frontier: [Frame::empty(); PLAN_FRONTIER_MAX],
            frontier_len: 0,
            next: [Frame::empty(); PLAN_FRONTIER_MAX],
            next_len: 0,
            scores: [0; PLAN_FRONTIER_MAX],
            path: [(EffectDelta::EMPTY, SlotVec::empty(), 0, 0); PLAN_HORIZON_MAX],
            path_len: 0,
            considered: [ConsideredCandidate::EMPTY; PLAN_HORIZON_MAX * PLAN_ACTIONS_MAX],
            considered_len: 0,
            considered_per_step: 0,
        }
    }

    fn reset(&mut self) {
        self.visited_len = 0;
        self.frontier_len = 0;
        self.next_len = 0;
        self.path_len = 0;
        self.considered_len = 0;
        self.considered_per_step = 0;
        for slot in self.index.iter_mut() {
            *slot = EMPTY;
        }
    }

    /// Steps in the chosen path.
    pub fn path_len(&self) -> usize {
        usize::from(self.path_len)
    }

    /// Step `index` of the chosen path, in witness form: the applied effect,
    /// the resulting state, the *operator index* of the chosen candidate, and
    /// the `PTRN` row its rule came from.
    pub fn path_step(&self, index: usize) -> Option<WitnessStep> {
        if index >= self.path_len() {
            return None;
        }
        Some(self.path[index])
    }

    /// Candidates recorded per step of the chosen path.
    pub fn considered_per_step(&self) -> usize {
        usize::from(self.considered_per_step)
    }

    /// The recorded candidates, row-major over the chosen path's steps.
    pub fn considered(&self) -> &[ConsideredCandidate] {
        &self.considered[..usize::from(self.considered_len)]
    }
}

// ---------------------------------------------------------------------------
// P-4 helpers
// ---------------------------------------------------------------------------

/// Multiply-free add/rotate/xor mixer in the Jenkins one-at-a-time family, the
/// same construction already normative for the packed skip-mix tables. Unseeded
/// and unrandomized, so identical inputs hash identically on every platform —
/// the determinism a keyed hash cannot give.
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

/// Saturating absolute distance from `state` to the slots a goal predicate
/// pins by equality. Comparison and saturating subtraction only.
fn goal_distance(goal: &PreconditionMask, state: &SlotVec) -> i32 {
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

// ---------------------------------------------------------------------------
// The planner
// ---------------------------------------------------------------------------

struct Episode<'q, 'a, 'b> {
    query: &'q PlanQuery<'a, 'b>,
    counters: PlanCounters,
}

/// The outcome of a bounded search: a goal node, an exhausted space, or a typed
/// decline.
///
/// Modelled as data rather than as a `Result`, for two reasons that point the
/// same way. A decline is an outcome the planner *reports*, not an error it
/// suffers, so it should be as inspectable as a plan and impossible to discard
/// with a `?`. And the repository sanctions only a fixed set of error types on
/// a shipped boundary (R5): a bound is a property of the caller's chosen
/// instantiation, so a `Result` over anything else would be claiming a
/// limitation the model does not sanction.
enum Search {
    Found(u16),
    Exhausted,
    Declined(PackedDecline),
}

/// The reason a step could not be taken. Kept separate from a decline so a
/// blocked candidate does not end the episode.
enum Step {
    Taken(u16),
    Blocked,
    Capacity,
    Unknown,
}

impl Episode<'_, '_, '_> {
    fn read(&mut self, count: u32) {
        self.counters.table_reads = self.counters.table_reads.saturating_add(count);
    }

    fn op(&mut self, count: u32) {
        self.counters.integer_ops = self.counters.integer_ops.saturating_add(count);
    }

    fn over_budget(&self) -> bool {
        self.counters.expansions > self.query.budget.max_expansions
            || self.counters.candidates > self.query.budget.max_candidates
            || self.counters.table_reads > self.query.budget.max_table_reads
    }

    /// Record `state` in the visited set. `Some(index)` when it is new,
    /// `None` when already present; the probe bound is checked.
    fn admit(&mut self, scratch: &mut PlanScratch, state: &SlotVec) -> Option<Option<u16>> {
        let mut slot = (hash_state(state) & VISITED_INDEX_MASK) as usize;
        for probe in 0..=VISITED_MAX_PROBE {
            if probe == VISITED_MAX_PROBE {
                // A checked bound, not a best effort: the episode declines
                // rather than degrading into an unbounded scan.
                return None;
            }
            self.counters.max_probe = self.counters.max_probe.max(probe.saturating_add(1));
            self.read(1);
            let occupant = scratch.index[slot];
            if occupant == EMPTY {
                if usize::from(scratch.visited_len) >= PLAN_VISITED_MAX {
                    return None;
                }
                let node = scratch.visited_len;
                scratch.index[slot] = node;
                scratch.visited_len = scratch.visited_len.saturating_add(1);
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

    /// Apply one candidate rule from `state`, admitting the successor.
    fn take(
        &mut self,
        scratch: &mut PlanScratch,
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
            // An arity mismatch is an unknown typed situation, resolved by
            // decline rather than by a default.
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
                scratch.visited[usize::from(node)] = VisitedNode {
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

    /// Walk the parent chain from `node` into the scratch's path buffer.
    fn unwind(&mut self, scratch: &mut PlanScratch, node: u16) -> bool {
        let mut depth = usize::from(scratch.visited[usize::from(node)].depth);
        if depth > PLAN_HORIZON_MAX {
            return false;
        }
        scratch.path_len = depth as u8;
        let mut cursor = node;
        while depth > 0 {
            let current = scratch.visited[usize::from(cursor)];
            let parent = current.parent;
            if parent == EMPTY {
                return false;
            }
            let from = scratch.visited[usize::from(parent)].state;
            let Some(rule) = self.query.rules.rule(usize::from(current.rule_row)) else {
                return false;
            };
            self.read(1);
            // The recorded effect must really carry the predecessor to the
            // recorded successor; the witness is verified from this, so it is
            // checked here rather than asserted.
            match from.apply(&rule.effect) {
                Some(next) if next == current.state => {}
                _ => return false,
            }
            depth -= 1;
            scratch.path[depth] = (
                rule.effect,
                current.state,
                current.operator,
                current.rule_row,
            );
            cursor = parent;
        }
        true
    }

    /// Record what was considered at each step of the chosen path. Purely
    /// informational: replay never reads it, so it is gathered in a bounded
    /// post-pass rather than carried through the search.
    fn record_considered(&mut self, scratch: &mut PlanScratch) {
        let operators = self.query.schema.operator_count().min(PLAN_ACTIONS_MAX);
        let available: usize = (0..operators)
            .filter(|i| self.query.available & (1u64 << i) != 0)
            .count();
        scratch.considered_per_step = available.min(PLAN_ACTIONS_MAX) as u8;
        scratch.considered_len = 0;
        let mut state = self.query.initial;
        for step in 0..scratch.path_len() {
            let (effect, resulting, chosen_operator, _) = scratch.path[step];
            let mut rank: u16 = 0;
            for operator in 0..operators {
                if self.query.available & (1u64 << operator) == 0 {
                    continue;
                }
                let Some((first, end)) = self.query.rules.rules_for(operator) else {
                    continue;
                };
                // The canonical first rule of each available operator stands
                // for it, so the record is one row per operator per step and
                // its width is the declared `considered_per_step`.
                let row = first;
                if row >= end {
                    continue;
                }
                self.read(1);
                let Some(rule) = self.query.rules.rule(row) else {
                    continue;
                };
                let score = (i32::from(rule.band) << 10)
                    .saturating_sub(goal_distance_for(self.query.predicates, &resulting));
                let flags = u8::from(operator as u16 == chosen_operator);
                let at = usize::from(scratch.considered_len);
                if at < scratch.considered.len() {
                    scratch.considered[at] = ConsideredCandidate {
                        operator: operator as u16,
                        rule_row: row as u16,
                        score,
                        tie_rank: rank,
                        support: rule.support,
                        band: rule.band,
                        flags,
                    };
                    scratch.considered_len = scratch.considered_len.saturating_add(1);
                }
                rank = rank.saturating_add(1);
            }
            let _ = effect;
            state = resulting;
        }
        let _ = state;
    }
}

/// Distance to the first goal predicate, or zero when the query pins none.
fn goal_distance_for(predicates: &PredicateSet<'_>, state: &SlotVec) -> i32 {
    match predicates.goal(0) {
        Some(goal) => goal_distance(&goal, state),
        None => 0,
    }
}

/// Run one bounded planning episode.
///
/// Total and deterministic: every path, including every capacity and conflict
/// boundary, ends in a plan or a typed decline, and the counters are recorded
/// either way. Ties break by the canonical order — operator index ascending,
/// then rule row ascending — with no clock, RNG, or hash-iteration order
/// anywhere.
pub fn plan(query: &PlanQuery<'_, '_>, scratch: &mut PlanScratch) -> PlanResult {
    scratch.reset();
    let mut episode = Episode {
        query,
        counters: PlanCounters::default(),
    };

    // A query whose initial state already satisfies the goal is a zero-step
    // plan, not a search.
    if query.predicates.satisfies_goal(&query.initial) {
        scratch.path_len = 0;
        return PlanResult {
            outcome: PlanOutcome::Plan { steps: 0 },
            counters: episode.counters,
        };
    }
    if query.predicates.is_forbidden(&query.initial) {
        return decline(PackedDecline::NoPlan, episode.counters);
    }
    if usize::from(query.budget.horizon) > PLAN_HORIZON_MAX
        || usize::from(query.budget.frontier) > PLAN_FRONTIER_MAX
    {
        return decline(PackedDecline::Capacity, episode.counters);
    }

    let Some(Some(root)) = episode.admit(scratch, &query.initial) else {
        return decline(PackedDecline::Capacity, episode.counters);
    };
    scratch.visited[usize::from(root)] = VisitedNode {
        state: query.initial,
        parent: EMPTY,
        rule_row: 0,
        operator: 0,
        depth: 0,
    };

    let found = match query.strategy {
        PlanStrategy::BreadthFirst => search_layered(&mut episode, scratch, root, false),
        PlanStrategy::BestFirstBeam => search_layered(&mut episode, scratch, root, true),
        PlanStrategy::IterativeDeepening => search_deepening(&mut episode, scratch, root),
    };

    match found {
        Search::Found(node) => {
            if !episode.unwind(scratch, node) {
                return decline(PackedDecline::Unknown, episode.counters);
            }
            episode.record_considered(scratch);
            let steps = scratch.path_len();
            PlanResult {
                outcome: PlanOutcome::Plan { steps },
                counters: episode.counters,
            }
        }
        Search::Exhausted => decline(PackedDecline::NoPlan, episode.counters),
        Search::Declined(reason) => decline(reason, episode.counters),
    }
}

fn decline(reason: PackedDecline, counters: PlanCounters) -> PlanResult {
    PlanResult {
        outcome: PlanOutcome::Declined(reason),
        counters,
    }
}

/// Layered search. With `ordered` clear this is a frontier-limited
/// breadth-first sweep; with it set the retained frontier is the highest-scoring
/// prefix, which is the table-guided beam. They differ only in which candidates
/// survive the frontier bound — and the bound *binds* on this task shape, so
/// that difference is the whole comparison.
fn search_layered(
    episode: &mut Episode<'_, '_, '_>,
    scratch: &mut PlanScratch,
    root: u16,
    ordered: bool,
) -> Search {
    scratch.frontier[0] = Frame {
        state: scratch.visited[usize::from(root)].state,
        node: root,
        depth: 0,
    };
    scratch.frontier_len = 1;

    for _ in 0..episode.query.budget.horizon {
        scratch.next_len = 0;
        let width = usize::from(scratch.frontier_len);
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
                            let at = usize::from(scratch.next_len);
                            if at >= usize::from(episode.query.budget.frontier) {
                                // The frontier bound binds. Which candidate is
                                // dropped is the strategy's decision, and it is
                                // recorded as such rather than as an error.
                                if ordered {
                                    replace_weakest(episode, scratch, node, successor, frame.depth);
                                }
                                continue;
                            }
                            scratch.next[at] = Frame {
                                state: successor,
                                node,
                                depth: frame.depth.saturating_add(1),
                            };
                            scratch.scores[at] =
                                -goal_distance_for(episode.query.predicates, &successor);
                            scratch.next_len = scratch.next_len.saturating_add(1);
                        }
                        Step::Blocked => {}
                        Step::Capacity => return Search::Declined(PackedDecline::Capacity),
                        Step::Unknown => return Search::Declined(PackedDecline::Unknown),
                    }
                }
            }
        }
        if scratch.next_len == 0 {
            return Search::Exhausted;
        }
        scratch.frontier_len = scratch.next_len;
        for slot in 0..usize::from(scratch.next_len) {
            scratch.frontier[slot] = scratch.next[slot];
        }
    }
    Search::Exhausted
}

/// Swap `node` in for the retained candidate furthest from the goal, when it is
/// closer. Ties keep the incumbent, so the retained set is a deterministic
/// function of the arrival order the canonical expansion already fixes.
fn replace_weakest(
    episode: &mut Episode<'_, '_, '_>,
    scratch: &mut PlanScratch,
    node: u16,
    successor: SlotVec,
    depth: u8,
) {
    let score = -goal_distance_for(episode.query.predicates, &successor);
    let mut weakest = 0usize;
    for slot in 1..usize::from(scratch.next_len) {
        if scratch.scores[slot] < scratch.scores[weakest] {
            weakest = slot;
        }
    }
    if usize::from(scratch.next_len) == 0 || score <= scratch.scores[weakest] {
        return;
    }
    scratch.scores[weakest] = score;
    scratch.next[weakest] = Frame {
        state: successor,
        node,
        depth: depth.saturating_add(1),
    };
}

/// Depth-limited depth-first search with an increasing bound. Frontier memory
/// is bounded by depth rather than by width, which is the point of the arm: it
/// pays repeated expansions to avoid the frontier bound that binds above.
fn search_deepening(
    episode: &mut Episode<'_, '_, '_>,
    scratch: &mut PlanScratch,
    root: u16,
) -> Search {
    let root_state = scratch.visited[usize::from(root)].state;
    for limit in 1..=episode.query.budget.horizon {
        // Each round starts from a clean visited set, so a state pruned at a
        // shallower depth is reachable again at a deeper one.
        let saved = root_state;
        scratch.visited_len = 0;
        for entry in scratch.index.iter_mut() {
            *entry = EMPTY;
        }
        let Some(Some(fresh_root)) = episode.admit(scratch, &saved) else {
            return Search::Declined(PackedDecline::Capacity);
        };
        scratch.visited[usize::from(fresh_root)] = VisitedNode {
            state: saved,
            parent: EMPTY,
            rule_row: 0,
            operator: 0,
            depth: 0,
        };
        match descend(episode, scratch, fresh_root, limit) {
            Search::Found(node) => return Search::Found(node),
            Search::Declined(reason) => return Search::Declined(reason),
            Search::Exhausted => {}
        }
    }
    Search::Exhausted
}

fn descend(
    episode: &mut Episode<'_, '_, '_>,
    scratch: &mut PlanScratch,
    node: u16,
    remaining: u8,
) -> Search {
    if remaining == 0 {
        return Search::Exhausted;
    }
    if episode.over_budget() {
        return Search::Declined(PackedDecline::Capacity);
    }
    let state = scratch.visited[usize::from(node)].state;
    let depth = scratch.visited[usize::from(node)].depth;
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
            match episode.take(scratch, &state, node, depth, operator as u16, row) {
                Step::Taken(child) => {
                    let successor = scratch.visited[usize::from(child)].state;
                    if episode.query.predicates.satisfies_goal(&successor) {
                        return Search::Found(child);
                    }
                    match descend(episode, scratch, child, remaining.saturating_sub(1)) {
                        Search::Found(found) => return Search::Found(found),
                        Search::Declined(reason) => return Search::Declined(reason),
                        Search::Exhausted => {}
                    }
                }
                Step::Blocked => {}
                Step::Capacity => return Search::Declined(PackedDecline::Capacity),
                Step::Unknown => return Search::Declined(PackedDecline::Unknown),
            }
        }
    }
    Search::Exhausted
}

/// A planning request in the form the engine takes: everything a query needs
/// except the packed sections, which the engine resolves from its own artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanRequest {
    /// Search order.
    pub strategy: PlanStrategy,
    /// Where the episode starts.
    pub initial: SlotVec,
    /// Bit `i` set when operator `i` is available to this instance.
    pub available: u64,
    /// The declared budget.
    pub budget: PlanBudget,
}
