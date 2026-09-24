//! Bounded causal exact-pair memory in the loaded native `TlModel` sequence path.
//!
//! The address is an ordinary exact BPE pair. The fitted four-bit table chooses NoRead,
//! most-recent or previous-different source. The model state and emitted-token feedback are
//! unchanged; a selected read bypasses the dense vocabulary head and emits its owned token.
//! This is a component experiment, not learned semantic addressing or a sparse whole model.
#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::state_lexical::SlFacts;
use super::transferable_lexical::{TlAction, TlModel, TL_EV_OBSERVE};

pub const MAX_PAIR_KEYS: usize = 4096;
const GATE_VERSION: u8 = 1;
type Table = [[[[[i8; 2]; 2]; 3]; 3]; 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Occurrence {
    value: u32,
    position: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    recent: Occurrence,
    previous_different: Option<Occurrence>,
    hits: u32,
    conflicts: u32,
}

/// One causally available source. `source=0` is most recent, `source=1` is the previous
/// *different* value; the latter's age is measured from its own occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub source: usize,
    pub value: u32,
    pub age: usize,
    pub hits: u32,
    pub conflicts: u32,
}

impl Candidate {
    fn indices(self, current: u32) -> [usize; 5] {
        [
            self.source,
            if self.age < 64 {
                0
            } else if self.age < 256 {
                1
            } else {
                2
            },
            if self.hits < 2 {
                0
            } else if self.hits < 4 {
                1
            } else {
                2
            },
            usize::from(self.conflicts != 0),
            usize::from(self.value == current),
        ]
    }
}

/// At most 4096 distinct keys; on capacity, existing keys continue to update and new keys
/// are ignored. Each key stores at most two source occurrences. No cross-document state.
#[derive(Clone, Debug, Default)]
pub struct PairMemory {
    entries: HashMap<(u32, u32), Entry>,
}

impl PairMemory {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Query before observing the target at `position`.
    pub fn candidates(
        &self,
        previous: u32,
        current: u32,
        position: usize,
    ) -> [Option<Candidate>; 2] {
        let Some(entry) = self.entries.get(&(previous, current)) else {
            return [None, None];
        };
        let candidate = |source, occurrence: Occurrence| Candidate {
            source,
            value: occurrence.value,
            age: position.saturating_sub(occurrence.position),
            hits: entry.hits,
            conflicts: entry.conflicts,
        };
        [
            Some(candidate(0, entry.recent)),
            entry.previous_different.map(|occ| candidate(1, occ)),
        ]
    }

    /// Write only after `next` has become an observed or emitted token.
    pub fn observe(&mut self, previous: u32, current: u32, next: u32, position: usize) {
        let key = (previous, current);
        if let Some(entry) = self.entries.get_mut(&key) {
            if entry.recent.value != next {
                entry.previous_different = Some(entry.recent);
                entry.conflicts = entry.conflicts.saturating_add(1);
            }
            entry.recent = Occurrence {
                value: next,
                position,
            };
            entry.hits = entry.hits.saturating_add(1);
        } else if self.entries.len() < MAX_PAIR_KEYS {
            self.entries.insert(
                key,
                Entry {
                    recent: Occurrence {
                        value: next,
                        position,
                    },
                    previous_different: None,
                    hits: 1,
                    conflicts: 0,
                },
            );
        }
    }

    /// Source-only intervention for a causal control. Metadata and model state are untouched.
    pub fn replace_value(&mut self, key: (u32, u32), source: usize, value: u32) -> bool {
        let Some(entry) = self.entries.get_mut(&key) else {
            return false;
        };
        let slot = match source {
            0 => Some(&mut entry.recent),
            1 => entry.previous_different.as_mut(),
            _ => None,
        };
        if let Some(occurrence) = slot {
            occurrence.value = value;
            true
        } else {
            false
        }
    }
}

/// A fitted example. Labels are used only by offline `fit`; serving sees `Candidate` fields.
#[derive(Clone, Copy, Debug)]
pub struct GateExample {
    pub candidate: Candidate,
    pub current: u32,
    pub target: u32,
    pub model_top: u32,
}

#[derive(Clone, Copy, Default)]
struct Cell {
    n: i32,
    advantage: i32,
}

/// A hard, four-bit learned source selector. The table contains only signed values in -7..=7.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseReadGate {
    pub version: u8,
    pub model_sha256: String,
    pub tokenizer_sha256: String,
    pub fit_source_sha256: String,
    table: Table,
    pub threshold: i8,
}

impl SparseReadGate {
    pub fn fit(
        rows: &[GateExample],
        model_sha256: String,
        tokenizer_sha256: String,
        fit_source_sha256: String,
    ) -> Self {
        let mut cells = [[[[[Cell::default(); 2]; 2]; 3]; 3]; 2];
        for row in rows {
            let [s, a, h, c, same] = row.candidate.indices(row.current);
            let cell = &mut cells[s][a][h][c][same];
            cell.n += 1;
            cell.advantage += i32::from(row.candidate.value == row.target)
                - i32::from(row.model_top == row.target);
        }
        let mut table = [[[[[0i8; 2]; 2]; 3]; 3]; 2];
        for s in 0..2 {
            for a in 0..3 {
                for h in 0..3 {
                    for c in 0..2 {
                        for same in 0..2 {
                            let cell = cells[s][a][h][c][same];
                            // Offline integer fit with a 16-observation zero-advantage prior.
                            table[s][a][h][c][same] =
                                ((cell.advantage * 32) / (cell.n + 16)).clamp(-7, 7) as i8;
                        }
                    }
                }
            }
        }
        Self {
            version: GATE_VERSION,
            model_sha256,
            tokenizer_sha256,
            fit_source_sha256,
            table,
            threshold: 0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != GATE_VERSION || !(0..=4).contains(&self.threshold) {
            return Err("invalid sparse-read gate version or threshold".into());
        }
        if self
            .table
            .iter()
            .flatten()
            .flatten()
            .flatten()
            .flatten()
            .any(|w| !(-7..=7).contains(w))
        {
            return Err("sparse-read gate weight exceeds four-bit bound".into());
        }
        for digest in [
            &self.model_sha256,
            &self.tokenizer_sha256,
            &self.fit_source_sha256,
        ] {
            if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("invalid sparse-read gate binding".into());
            }
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let gate: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        gate.validate()?;
        Ok(gate)
    }

    /// One or two four-bit table reads; strict positive threshold admits a source.
    pub fn choose(&self, current: u32, candidates: [Option<Candidate>; 2]) -> Option<Candidate> {
        let mut best = None;
        let mut score = self.threshold;
        for candidate in candidates.into_iter().flatten() {
            let [s, a, h, c, same] = candidate.indices(current);
            let value = self.table[s][a][h][c][same];
            if value > score {
                score = value;
                best = Some(candidate);
            }
        }
        best
    }

    pub fn positive_cells(&self) -> usize {
        self.table
            .iter()
            .flatten()
            .flatten()
            .flatten()
            .flatten()
            .filter(|w| **w > self.threshold)
            .count()
    }
}

#[derive(Clone, Copy)]
pub enum SparsePolicy<'a> {
    NoRead,
    Latest,
    Learned(&'a SparseReadGate),
}

impl SparsePolicy<'_> {
    fn select(self, current: u32, candidates: [Option<Candidate>; 2]) -> Option<Candidate> {
        match self {
            Self::NoRead => None,
            Self::Latest => candidates[0],
            Self::Learned(gate) => gate.choose(current, candidates),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SparseStep {
    pub token: Option<u32>,
    pub action: String,
    pub selected_source: Option<usize>,
    pub candidate_age: Option<usize>,
    pub memory_keys: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SparseRollout {
    pub tokens: Vec<u32>,
    pub steps: Vec<SparseStep>,
    pub stopped: bool,
    pub prompt_parameter_reads: usize,
    pub output_parameter_reads: usize,
    pub candidate_slot_reads: usize,
}

/// One loaded-model session. The prompt is consumed once, so cloned sessions permit exact
/// NoRead/Latest/Learned comparisons and source-only edits from identical recurrent states.
#[derive(Clone)]
pub struct SparseSession {
    memory: PairMemory,
    history: Vec<u32>,
    h: Vec<i32>,
    m: Vec<i32>,
    f: Vec<i32>,
    event: usize,
    prompt_parameter_reads: usize,
}

impl SparseSession {
    pub fn new(model: &TlModel, observed: &[u32]) -> Result<Self, String> {
        model.validate()?;
        if observed.iter().any(|t| *t as usize >= model.vocab) {
            return Err("observed token exceeds loaded vocabulary".into());
        }
        let m = vec![0; model.h_dim];
        let f = model.typed_block(&[], &[], SlFacts::default());
        let mut h = model.init_state(&m, &f);
        let mut memory = PairMemory::default();
        for (i, token) in observed.iter().enumerate() {
            if i >= 2 {
                memory.observe(observed[i - 2], observed[i - 1], *token, i);
            }
            h = model.transition(&h, TL_EV_OBSERVE, Some(*token), &m, &f);
        }
        Ok(Self {
            memory,
            history: observed.to_vec(),
            h,
            m,
            f,
            event: TL_EV_OBSERVE,
            prompt_parameter_reads: model.wi.nonzero()
                + observed.len() * (model.wh.nonzero() + model.wf.nonzero()),
        })
    }

    pub fn current_candidates(&self) -> [Option<Candidate>; 2] {
        if self.history.len() < 2 {
            return [None, None];
        }
        let n = self.history.len();
        self.memory
            .candidates(self.history[n - 2], self.history[n - 1], n)
    }

    pub fn current_token(&self) -> Option<u32> {
        self.history.last().copied()
    }

    pub fn current_key(&self) -> Option<(u32, u32)> {
        let n = self.history.len();
        (n >= 2).then(|| (self.history[n - 2], self.history[n - 1]))
    }

    pub fn replace_current_source(&mut self, source: usize, value: u32) -> bool {
        self.current_key()
            .is_some_and(|key| self.memory.replace_value(key, source, value))
    }

    pub fn generate(
        &mut self,
        model: &TlModel,
        policy: SparsePolicy<'_>,
        max_new: usize,
    ) -> Result<SparseRollout, String> {
        if !(1..=64).contains(&max_new) {
            return Err("max_new must be 1..=64".into());
        }
        let mut output = SparseRollout {
            tokens: Vec::new(),
            steps: Vec::new(),
            stopped: false,
            prompt_parameter_reads: self.prompt_parameter_reads,
            output_parameter_reads: 0,
            candidate_slot_reads: 0,
        };
        for _ in 0..max_new {
            let candidates = if matches!(policy, SparsePolicy::NoRead) {
                [None, None]
            } else {
                self.current_candidates()
            };
            output.candidate_slot_reads += candidates.iter().flatten().count();
            let selected = self
                .current_token()
                .and_then(|cur| policy.select(cur, candidates));
            let action = match selected {
                Some(_) => TlAction::Copy,
                None => {
                    output.output_parameter_reads += model.wo.nonzero();
                    model.decide(&self.h, &self.m, &self.f, self.event, false)
                }
            };
            let token = match action {
                TlAction::Copy => selected.map(|candidate| candidate.value),
                TlAction::Generate(v) => Some(v),
                TlAction::Stop => None,
            };
            output.steps.push(SparseStep {
                token,
                action: format!("{action:?}"),
                selected_source: selected.map(|c| c.source),
                candidate_age: selected.map(|c| c.age),
                memory_keys: self.memory.len(),
            });
            let Some(token) = token else {
                output.stopped = true;
                break;
            };
            let n = self.history.len();
            if n >= 2 {
                self.memory
                    .observe(self.history[n - 2], self.history[n - 1], token, n);
            }
            self.history.push(token);
            self.h = model.transition(&self.h, action.event(), Some(token), &self.m, &self.f);
            self.event = action.event();
            output.output_parameter_reads += model.wh.nonzero() + model.wf.nonzero();
            output.tokens.push(token);
        }
        Ok(output)
    }
}
