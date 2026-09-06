//! Bounded learned associations. Exact members outlive the raw token window;
//! the directory names current versions and never averages away their values.
use super::value_lexemes::{LexemeState, WordAtom};
use super::value_types::{ValueFeature, ValueRow, ValueState, ValueWork};
use super::*;

pub(super) const RELATIONS: usize = 16;
pub(super) const RELATION_SOURCE: u8 = 32;
pub(super) const RELATION_FEATURES: usize = 64;
pub(super) const RELATION_ROWS: usize = 16384;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RelationModel {
    pub schema: String,
    /// Version 2 word-local transport, compiled from the parent tokenizer and
    /// fixed geometry. Dictionary primes are exact keys, not semantic distances.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_context: Vec<TokenGeometry>,
    pub parent: String,
    pub writer: Vec<ValueRow>,
    pub reader: Vec<ValueRow>,
    pub training: Vec<DocumentReceipt>,
    pub epochs: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RelationRecord {
    pub id: u64,
    pub owner: WordAtom,
    pub value: WordAtom,
    pub previous: u64,
    /// 1=assert, 2=explicit revision, 3=contradiction. Learned choice.
    pub action: u8,
    pub conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RelationState {
    pub records: [RelationRecord; RELATIONS],
    /// Zero is empty; each nonzero entry is a current exact record id.
    pub directory: [u64; RELATIONS],
    pub next_id: u64,
    pub last_word_end: Option<u64>,
}
impl Default for RelationState {
    fn default() -> Self {
        Self {
            records: [RelationRecord::default(); RELATIONS],
            directory: [0; RELATIONS],
            next_id: 1,
            last_word_end: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationWork {
    pub word_boundaries: u64,
    pub feature_queries: u64,
    pub row_comparisons: u64,
    pub score_lookups: u64,
    pub dictionary_comparisons: u64,
    pub dictionary_byte_comparisons: u64,
    pub candidates: u64,
    pub directory_reads: u64,
    pub directory_writes: u64,
    pub record_reads: u64,
    pub record_writes: u64,
    pub record_evictions: u64,
    pub no_writes: u64,
    pub revisions: u64,
    pub conflicts: u64,
    pub reads: u64,
    pub abstentions: u64,
    pub feature_writes: u64,
    pub source_presence_checks: u64,
    pub phase_subtractions: u64,
    #[serde(default, skip_serializing_if = "relation_count_zero")]
    pub role_path_probes: u64,
    #[serde(default, skip_serializing_if = "relation_count_zero")]
    pub role_path_comparisons: u64,
    #[serde(default, skip_serializing_if = "relation_count_zero")]
    pub role_path_steps: u64,
    #[serde(default, skip_serializing_if = "relation_count_zero")]
    pub role_phase_additions: u64,
}
fn relation_count_zero(value: &u64) -> bool {
    *value == 0
}
impl RelationWork {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

pub(super) fn head(model: &Model) -> Option<&RelationModel> {
    super::role_read::head(model)?.relations.as_ref()
}

pub(super) fn source(values: &ValueState, index: u8) -> Option<&WordAtom> {
    if index >= RELATION_SOURCE {
        let r = values
            .relations
            .as_ref()?
            .records
            .get(usize::from(index - RELATION_SOURCE))?;
        (r.id != 0).then_some(&r.value)
    } else {
        let words = values.lexemes.as_ref()?;
        (usize::from(index) < words.query_len).then(|| &words.queries[usize::from(index)])
    }
}

pub(super) fn source_version(values: &ValueState, index: u8) -> Option<u64> {
    if index < RELATION_SOURCE {
        return None;
    }
    values
        .relations
        .as_ref()?
        .records
        .get(usize::from(index - RELATION_SOURCE))
        .filter(|r| r.id != 0)
        .map(|r| r.id)
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
pub(super) fn addresses(model: &Model, words: &[WordAtom], work: &mut ValueWork) -> [u32; 16] {
    let mut out = [0; 16];
    let Some(read) = super::role_read::head(model) else {
        return out;
    };
    for (i, w) in words.iter().take(16).enumerate() {
        work.relations.record_reads = work.relations.record_reads.saturating_add(1);
        let found = read.dictionary.binary_search_by(|d| {
            work.relations.dictionary_comparisons =
                work.relations.dictionary_comparisons.saturating_add(1);
            for j in 0..usize::from(w.len.min(d.len)) {
                work.relations.dictionary_byte_comparisons =
                    work.relations.dictionary_byte_comparisons.saturating_add(1);
                let cmp = d.bytes[j].cmp(&w.bytes[j]);
                if !cmp.is_eq() {
                    return cmp;
                }
            }
            d.len.cmp(&w.len)
        });
        out[i] = found.map_or(0, |j| read.dictionary[j].prime);
    }
    out
}

pub(super) fn write_features(
    model: &Model,
    words: &[WordAtom],
    addr: &[u32; 16],
    owner: usize,
    value: usize,
    work: &mut ValueWork,
) -> ([ValueFeature; RELATION_FEATURES], usize) {
    let context = head(model)
        .filter(|h| h.schema == "uor-r4.exact-relation/2")
        .map(|h| h.role_context.as_slice());
    write_features_with_context(model, words, addr, owner, value, context, work)
}

pub(super) fn write_features_with_context(
    model: &Model,
    words: &[WordAtom],
    addr: &[u32; 16],
    owner: usize,
    value: usize,
    context: Option<&[TokenGeometry]>,
    work: &mut ValueWork,
) -> ([ValueFeature; RELATION_FEATURES], usize) {
    let mut out = [ValueFeature::default(); RELATION_FEATURES];
    let mut n = 0;
    let mut add = |kind, a, b| {
        out[n] = ValueFeature { kind, a, b };
        n += 1;
    };
    let at = |i: usize| {
        if context.is_some() && i == owner {
            u64::MAX
        } else if context.is_some() && i == value {
            u64::MAX - 1
        } else if i < words.len() {
            u64::from(addr[i])
        } else {
            0
        }
    };
    add(0, 1, 0);
    add(1, at(owner + 1), 0);
    add(2, owner.checked_sub(1).map_or(0, at), 0);
    add(3, at(value + 1), 0);
    add(4, value.checked_sub(1).map_or(0, at), 0);
    add(5, at(owner + 1), at(value + 1));
    add(6, (owner + 8 - value) as u64, 0);
    add(7, at(value + 2), at(value + 1));
    add(8, at(owner + 2), at(owner + 1));
    let (rel, phases) = if let Some(context) = context {
        // Recent words are newest first. Compose only the ordered interior,
        // excluding both payloads and all global prefix state. Unknown interior
        // words contribute identity/zero; their distinctions are not retained.
        let mut pose = model.geometry.identity;
        let mut phases = [0_u16; PHASE_CHANNELS];
        for i in (owner.min(value) + 1..owner.max(value)).rev() {
            work.relations.role_path_probes = work.relations.role_path_probes.saturating_add(1);
            if let Ok(j) = context.binary_search_by(|g| {
                work.relations.role_path_comparisons =
                    work.relations.role_path_comparisons.saturating_add(1);
                g.prime.cmp(&addr[i])
            }) {
                let g = &context[j];
                pose = model.geometry.products
                    [model.geometry.row_bases[usize::from(pose)] + usize::from(g.leaf)];
                work.h4_reads = work.h4_reads.saturating_add(2);
                for (p, d) in phases.iter_mut().zip(g.phases) {
                    *p = p.wrapping_add(d);
                }
                work.relations.role_path_steps = work.relations.role_path_steps.saturating_add(1);
                work.relations.role_phase_additions = work
                    .relations
                    .role_phase_additions
                    .saturating_add(PHASE_CHANNELS as u64);
            }
        }
        if owner < value {
            pose = model.geometry.inverses[usize::from(pose)];
            work.h4_reads = work.h4_reads.saturating_add(1);
            for p in &mut phases {
                *p = 0_u16.wrapping_sub(*p);
            }
            work.relations.phase_subtractions = work
                .relations
                .phase_subtractions
                .saturating_add(PHASE_CHANNELS as u64);
        }
        (pose, phases)
    } else {
        let a = &words[owner];
        let b = &words[value];
        let inv = model.geometry.inverses[usize::from(a.pose)];
        let rel = model.geometry.products
            [model.geometry.row_bases[usize::from(inv)] + usize::from(b.pose)];
        work.h4_reads = work.h4_reads.saturating_add(3);
        work.relations.phase_subtractions = work
            .relations
            .phase_subtractions
            .saturating_add(PHASE_CHANNELS as u64);
        (
            rel,
            std::array::from_fn(|c| b.phases[c].wrapping_sub(a.phases[c])),
        )
    };
    add(10, u64::from(rel), 0);
    add(
        11,
        u64::from(model.geometry.orientation[usize::from(rel)]),
        0,
    );
    if context.is_some() {
        work.h4_reads = work.h4_reads.saturating_add(1);
    }
    for (channel, phase) in phases.into_iter().enumerate() {
        add(12, channel as u64, u64::from(phase >> 12));
    }
    work.relations.feature_writes = work.relations.feature_writes.saturating_add(n as u64);
    (out, n)
}

pub(super) fn key(mut f: ValueFeature, action: u8) -> ValueFeature {
    f.kind |= action << 5;
    f
}
pub(super) fn score(
    rows: &[ValueRow],
    features: &[ValueFeature],
    action: u8,
    work: &mut ValueWork,
) -> i64 {
    let mut sum = 0;
    for f in features {
        work.relations.feature_queries = work.relations.feature_queries.saturating_add(1);
        let k = key(*f, action);
        if let Ok(i) = rows.binary_search_by(|r| {
            work.relations.row_comparisons = work.relations.row_comparisons.saturating_add(1);
            r.feature.cmp(&k)
        }) {
            sum += i64::from(rows[i].weight);
            work.relations.score_lookups = work.relations.score_lookups.saturating_add(1);
        }
    }
    sum
}

pub(super) fn write_choice(
    model: &Model,
    words: &[WordAtom],
    work: &mut ValueWork,
) -> Option<(usize, usize, u8)> {
    let h = head(model)?;
    let words = &words[..words.len().min(8)];
    let addr = addresses(model, words, work);
    let mut best = None;
    let mut best_score = 0;
    for owner in 0..words.len() {
        for value in 0..words.len() {
            if owner == value || (owner != 0 && value != 0) {
                continue;
            }
            let (f, n) = write_features(model, words, &addr, owner, value, work);
            for action in 1..=3 {
                let s = score(&h.writer, &f[..n], action, work);
                work.relations.candidates = work.relations.candidates.saturating_add(1);
                if s > best_score {
                    best_score = s;
                    best = Some((owner, value, action));
                }
            }
        }
    }
    best
}

impl RelationState {
    pub(super) fn record(&self, id: u64) -> Option<&RelationRecord> {
        if id == 0 {
            return None;
        }
        let r = &self.records[((id - 1) & 15) as usize];
        (r.id == id).then_some(r)
    }
    pub(super) fn commit(
        &mut self,
        owner: WordAtom,
        value: WordAtom,
        action: u8,
        work: &mut ValueWork,
    ) {
        let Some(next) = self.next_id.checked_add(1) else {
            return;
        };
        let mut previous = None;
        for (i, id) in self.directory.iter().copied().enumerate() {
            work.relations.directory_reads = work.relations.directory_reads.saturating_add(1);
            if let Some(r) = self.record(id) {
                work.relations.record_reads = work.relations.record_reads.saturating_add(1);
                if owner.matches(&r.owner, work) {
                    previous = Some((i, *r));
                    break;
                }
            }
        }
        let conflict = action == 3
            || (action == 1
                && previous.is_some_and(|(_, r)| r.conflict || !r.value.matches(&value, work)));
        let slot = ((self.next_id - 1) & 15) as usize;
        let evicted = self.records[slot].id;
        if evicted != 0 {
            work.relations.record_evictions = work.relations.record_evictions.saturating_add(1);
        }
        for id in &mut self.directory {
            work.relations.directory_reads = work.relations.directory_reads.saturating_add(1);
            if *id != 0 && *id == evicted {
                *id = 0;
                work.relations.directory_writes = work.relations.directory_writes.saturating_add(1);
            }
        }
        let target = previous.map(|(i, _)| i).or_else(|| {
            self.directory.iter().position(|&id| {
                work.relations.directory_reads = work.relations.directory_reads.saturating_add(1);
                id == 0
            })
        });
        let Some(target) = target else {
            return;
        };
        self.records[slot] = RelationRecord {
            id: self.next_id,
            owner,
            value,
            previous: previous.map_or(0, |(_, r)| r.id),
            action,
            conflict,
        };
        self.directory[target] = self.next_id;
        self.next_id = next;
        work.relations.record_writes = work.relations.record_writes.saturating_add(1);
        work.relations.directory_writes = work.relations.directory_writes.saturating_add(1);
        work.relations.revisions = work
            .relations
            .revisions
            .saturating_add(u64::from(action == 2));
        work.relations.conflicts = work.relations.conflicts.saturating_add(u64::from(conflict));
    }
    pub(super) fn observe(&mut self, model: &Model, words: &LexemeState, work: &mut ValueWork) {
        if words.recent_len == 0 || self.last_word_end == Some(words.recent[0].byte_end) {
            return;
        }
        self.last_word_end = Some(words.recent[0].byte_end);
        work.relations.word_boundaries = work.relations.word_boundaries.saturating_add(1);
        if let Some((o, v, a)) = write_choice(model, &words.recent[..words.recent_len], work) {
            self.commit(words.recent[o], words.recent[v], a, work);
        } else {
            work.relations.no_writes = work.relations.no_writes.saturating_add(1);
        }
    }
}

pub(super) fn read_features(
    model: &Model,
    record: &RelationRecord,
    words: &LexemeState,
    addr: &[u32; 16],
    work: &mut ValueWork,
) -> ([ValueFeature; RELATION_FEATURES], usize) {
    let mut out = [ValueFeature::default(); RELATION_FEATURES];
    let mut n = 0;
    let mut add = |kind, a, b| {
        out[n] = ValueFeature { kind, a, b };
        n += 1;
    };
    add(0, 1, 0);
    add(1, u64::from(record.conflict), 0);
    for q in 0..words.query_len.min(8) {
        if record.owner.matches(&words.queries[q], work) {
            let before = if q + 1 < words.query_len {
                u64::from(addr[q + 1])
            } else {
                0
            };
            let after = q.checked_sub(1).map_or(0, |j| u64::from(addr[j]));
            add(2, 1, 0);
            add(3, before, after);
            add(4, u64::from(record.conflict), before);
            add(5, u64::from(record.conflict), after);
        }
    }
    let _ = model;
    work.relations.feature_writes = work.relations.feature_writes.saturating_add(n as u64);
    (out, n)
}

/// Return a persistent source/action, or defer to the unchanged recent reader.
/// Values still present in the recent capture retain the #1137 selection law.
pub(super) fn read_choice(
    model: &Model,
    values: &ValueState,
    work: &mut ValueWork,
) -> Option<(u8, usize)> {
    let h = head(model)?;
    let state = values.relations.as_ref()?;
    let words = values.lexemes.as_ref()?;
    let addr = addresses(model, &words.queries[..words.query_len], work);
    let mut best = None;
    let mut best_score = 0;
    let role = super::role_read::head(model)?;
    for id in state.directory {
        work.relations.directory_reads = work.relations.directory_reads.saturating_add(1);
        let Some(record) = state.record(id) else {
            continue;
        };
        work.relations.record_reads = work.relations.record_reads.saturating_add(1);
        if words.queries[..words.query_len].iter().any(|w| {
            work.relations.source_presence_checks =
                work.relations.source_presence_checks.saturating_add(1);
            *w == record.value
        }) {
            continue;
        }
        let (f, n) = read_features(model, record, words, &addr, work);
        for (ai, action) in role.actions.iter().enumerate() {
            if action.copy
                && (usize::from(record.value.len) + 1 + usize::from(action.prefix.is_some())
                    > usize::from(super::response_entry_types::RESPONSE_ENTRY_STEPS))
            {
                continue;
            }
            let s = score(&h.reader, &f[..n], (ai + 1) as u8, work);
            work.relations.candidates = work.relations.candidates.saturating_add(1);
            if s > best_score {
                best_score = s;
                best = Some((
                    if action.copy {
                        RELATION_SOURCE + ((id - 1) & 15) as u8
                    } else {
                        super::role_read::NO_SOURCE
                    },
                    ai,
                ));
            }
        }
    }
    if let Some((source, _)) = best {
        if source == super::role_read::NO_SOURCE {
            work.relations.abstentions = work.relations.abstentions.saturating_add(1);
        } else {
            work.relations.reads = work.relations.reads.saturating_add(1);
        }
    }
    best
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

impl RelationState {
    pub(super) fn validate(&self, values: &ValueState, model: &Model) -> Result<()> {
        let fail = || Error("invalid relation snapshot identity, directory or version".into());
        let words = values.lexemes.as_ref().ok_or_else(fail)?;
        if self.next_id == 0
            || self
                .last_word_end
                .is_some_and(|b| b >= words.source_bytes_seen)
        {
            return Err(fail());
        }
        let count = self.next_id.saturating_sub(1).min(16) as usize;
        if self.records.iter().filter(|r| r.id != 0).count() != count {
            return Err(fail());
        }
        for (slot, r) in self.records.iter().enumerate() {
            if r.id == 0 {
                if *r != RelationRecord::default() {
                    return Err(fail());
                }
                continue;
            }
            if r.id >= self.next_id
                || self.next_id - r.id > 16
                || ((r.id - 1) & 15) as usize != slot
                || !(1..=3).contains(&r.action)
                || r.previous >= r.id
                || r.owner.len == 0
                || r.value.len == 0
                || !r.owner.snapshot_valid(
                    values.seen,
                    words.source_bytes_seen,
                    model.geometry.inverses.len(),
                )
                || !r.value.snapshot_valid(
                    values.seen,
                    words.source_bytes_seen,
                    model.geometry.inverses.len(),
                )
            {
                return Err(fail());
            }
            if let Some(prior) = self.record(r.previous) {
                if !r.owner.matches(&prior.owner, &mut ValueWork::default()) {
                    return Err(fail());
                }
                let conflict = r.action == 3
                    || (r.action == 1
                        && (prior.conflict
                            || !r.value.matches(&prior.value, &mut ValueWork::default())));
                if conflict != r.conflict {
                    return Err(fail());
                }
            } else if r.action == 2 && r.conflict {
                return Err(fail());
            }
        }
        for (i, id) in self.directory.iter().copied().enumerate() {
            if id == 0 {
                continue;
            }
            let record = self.record(id).ok_or_else(fail)?;
            if self
                .records
                .iter()
                .any(|r| r.id > id && r.owner.matches(&record.owner, &mut ValueWork::default()))
                || self.directory[..i].iter().any(|&other| {
                    self.record(other)
                        .is_some_and(|r| r.owner.matches(&record.owner, &mut ValueWork::default()))
                })
            {
                return Err(fail());
            }
        }
        // Evicting a current version forgets that association; remaining old
        // versions are historical evidence and must not silently reactivate.
        if self.next_id > 1 && !self.directory.contains(&(self.next_id - 1)) {
            return Err(fail());
        }
        Ok(())
    }
}
