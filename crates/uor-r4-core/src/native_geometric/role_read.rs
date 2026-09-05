//! Candidate-relative, position-shared source/entry selection. No grammar
//! parser or response text enters this bounded integer/table operation.
use super::response_entry_types::*;
use super::value_types::{ValueFeature, ValueRow, ValueState};
use super::word_copy_types::*;
use super::*;

pub(super) const READ_FEATURES: usize = 512;
pub(super) const READ_ROWS: usize = 16384;
pub(super) const READ_ACTIONS: usize = 8;
pub(super) const NO_SOURCE: u8 = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadAction {
    pub copy: bool,
    /// None means start the selected word directly. Otherwise emit this learned
    /// lexical token and commit the same source for the following byte.
    pub prefix: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoleReadModel {
    pub schema: String,
    pub baseline_artifact: String,
    pub dictionary: Vec<WordCopyAddress>,
    pub actions: Vec<ReadAction>,
    pub rows: Vec<ValueRow>,
    pub training: Vec<DocumentReceipt>,
    pub epochs: usize,
    pub learning_rate_bits: u64,
    /// Remove pooled query identities; keep candidate-relative role context.
    #[serde(default, skip_serializing_if = "roles_include_query_identity")]
    pub local_roles_only: bool,
}

fn roles_include_query_identity(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadCommit {
    pub source: Option<u8>,
    pub source_end: u64,
    pub source_byte_end: u64,
    pub at_seen: u64,
    pub token: u32,
    pub prepare: bool,
}

pub(super) fn head(model: &Model) -> Option<&RoleReadModel> {
    model
        .response_entry
        .as_ref()?
        .copy
        .as_ref()?
        .role_read
        .as_ref()
}

pub(super) fn context(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> WordCopyContext {
    let mut ctx = super::word_copy_runtime::context(model, values, control, work);
    if let (Some(read), Some(words)) = (head(model), &values.lexemes) {
        for (i, word) in words.queries[..words.query_len].iter().enumerate() {
            work.dictionary_lookups = work.dictionary_lookups.saturating_add(1);
            work.word_record_reads = work.word_record_reads.saturating_add(1);
            let found = read.dictionary.binary_search_by(|item| {
                work.dictionary_comparisons = work.dictionary_comparisons.saturating_add(1);
                for j in 0..usize::from(item.len.min(word.len)) {
                    work.dictionary_byte_comparisons =
                        work.dictionary_byte_comparisons.saturating_add(1);
                    let order = item.bytes[j].cmp(&word.bytes[j]);
                    if !order.is_eq() {
                        return order;
                    }
                }
                item.len.cmp(&word.len)
            });
            ctx.addresses[i] = found.map_or(0, |j| read.dictionary[j].prime);
            work.selector.state_copies = work.selector.state_copies.saturating_add(1);
        }
    }
    ctx
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
pub(super) fn features(
    model: &Model,
    values: &ValueState,
    ctx: &WordCopyContext,
    index: usize,
    control: Control,
    work: &mut WordCopyWork,
) -> ([ValueFeature; READ_FEATURES], usize) {
    let mut out = [ValueFeature::default(); READ_FEATURES];
    let Some(words) = &values.lexemes else {
        return (out, 0);
    };
    let mut len = 0;
    let mut add = |kind, a, b| {
        // Fewer than 384 features under the 16-word cap, including all matches.
        out[len] = ValueFeature { kind, a, b };
        len += 1;
    };
    add(0, u64::from(index < words.query_len), 0);
    // Query context has shared positions: identities and ordered adjacent pairs.
    for q in 0..if head(model).is_some_and(|h| h.local_roles_only) {
        0
    } else {
        words.query_len.min(8)
    } {
        add(7, u64::from(ctx.addresses[q]), 0);
        if q + 1 < words.query_len.min(8) {
            add(
                8,
                u64::from(ctx.addresses[q + 1]),
                u64::from(ctx.addresses[q]),
            );
        }
    }
    if index < words.query_len {
        let at = |i: usize| {
            if i < words.query_len {
                u64::from(ctx.addresses[i])
            } else {
                0
            }
        };
        let before = at(index + 1);
        let after = index.checked_sub(1).map_or(0, at);
        add(1, before, 0);
        add(2, after, 0);
        add(3, before, after);
        add(4, at(index + 2), before);
        // Relative recency remains explicit for revisions; no absolute query position.
        add(6, index as u64, 0);
        for q in 0..index {
            for offset in [-3_i32, -2, -1, 1, 2, 3, 4] {
                let source = index as i32 + offset;
                if source <= q as i32 || source < 0 || source as usize >= words.query_len {
                    continue;
                }
                let a = &words.queries[q];
                let b = &words.queries[source as usize];
                work.word_record_reads = work.word_record_reads.saturating_add(2);
                if a.len == 0 || a.len != b.len {
                    continue;
                }
                let mut equal = true;
                for j in 0..usize::from(a.len) {
                    work.equality_byte_comparisons =
                        work.equality_byte_comparisons.saturating_add(1);
                    if a.bytes[j] != b.bytes[j] {
                        equal = false;
                        break;
                    }
                }
                if equal {
                    let role = (offset + 4) as u64;
                    add(9, role, at(q + 1));
                    add(10, role, q.checked_sub(1).map_or(0, at));
                    add(11, (role << 32) | before, at(q + 1));
                }
            }
        }
        // Retain the existing signed relative H4 and wrapped zeta channels;
        // discard old absolute-query masks and pooled source votes.
        let (geometric, n) =
            super::word_copy_runtime::features(model, values, ctx, index, control, work);
        for feature in &geometric[..n] {
            if (10..20).contains(&feature.kind) {
                add(feature.kind + 10, feature.a, feature.b);
            }
        }
    }
    work.selector.state_copies = work.selector.state_copies.saturating_add(len as u64);
    work.selector.metadata_reads = work.selector.metadata_reads.saturating_add(len as u64);
    (out, len)
}

pub(super) fn key(mut feature: ValueFeature, action: usize) -> ValueFeature {
    feature.kind |= (action as u8) << 5;
    feature
}

pub(super) fn choose(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<(u8, usize, i64)> {
    let read = head(model)?;
    let words = values.lexemes.as_ref()?;
    let ctx = context(model, values, control, work);
    let mut best = None;
    for index in 0..=words.query_len {
        let source = if index == words.query_len {
            NO_SOURCE
        } else {
            index as u8
        };
        let (features, len) = features(model, values, &ctx, index, control, work);
        work.word_candidates = work
            .word_candidates
            .saturating_add(u64::from(source != NO_SOURCE));
        for (action_index, action) in read.actions.iter().enumerate() {
            if action.copy != (source != NO_SOURCE) {
                continue;
            }
            if action.copy
                && usize::from(words.queries[index].len) + 1 + usize::from(action.prefix.is_some())
                    > usize::from(RESPONSE_ENTRY_STEPS)
            {
                work.bound_rejections = work.bound_rejections.saturating_add(1);
                continue;
            }
            let mut score = 0_i64;
            for feature in &features[..len] {
                let k = key(*feature, action_index);
                work.selector.feature_queries = work.selector.feature_queries.saturating_add(1);
                let found = read.rows.binary_search_by(|row| {
                    work.selector.row_comparisons = work.selector.row_comparisons.saturating_add(1);
                    row.feature.cmp(&k)
                });
                if let Ok(i) = found {
                    score += i64::from(read.rows[i].weight);
                    work.selector.matched_rows = work.selector.matched_rows.saturating_add(1);
                    work.selector.score_lookups = work.selector.score_lookups.saturating_add(1);
                }
            }
            work.selector.candidate_evaluations =
                work.selector.candidate_evaluations.saturating_add(1);
            work.selector.candidate_comparisons =
                work.selector.candidate_comparisons.saturating_add(1);
            // Strict comparison retains the newest equal-scoring occurrence.
            if best.is_none_or(|(_, _, prior)| score > prior) {
                best = Some((source, action_index, score));
                work.selector.candidate_writes = work.selector.candidate_writes.saturating_add(1);
            }
        }
    }
    best
}

pub(super) fn offer(
    copy: &mut WordCopyState,
    model: &Model,
    entry: &mut ResponseEntryState,
    values: &ValueState,
    baseline: Candidate,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<Candidate> {
    let (source, action_index, _) = choose(model, values, control, work)?;
    let action = &head(model)?.actions[action_index];
    let word = values.lexemes.as_ref()?.queries.get(usize::from(source));
    let token = if let Some(prefix) = action.prefix {
        prefix
    } else {
        work.byte_reads = work.byte_reads.saturating_add(1);
        u32::from(word?.bytes[0]) + 2
    };
    // Internal read scores compare joint actions. The winning operator uses a
    // dispatch marker relative to Base, not a fabricated calibrated LM score.
    let score = baseline.score + 1;
    let action = if !action.copy {
        WordCopyAction::NoRead
    } else if action.prefix.is_some() {
        WordCopyAction::Prepare
    } else {
        WordCopyAction::Read
    };
    copy.pending = Some(WordCopyDecision {
        token,
        score,
        word_index: source,
        cursor: 0,
        source_end: word.map_or(0, |w| w.end),
        source_byte_end: word.map_or(0, |w| w.byte_end),
        at_seen: values.seen,
        step: entry.steps,
        action,
    });
    entry.pending = Some(ResponseEntryDecision {
        token,
        score,
        boundary_seen: entry.boundary?.at_seen,
        step: entry.steps,
        at_seen: values.seen,
        action: if token == EOS {
            ResponseEntryAction::Stop
        } else {
            ResponseEntryAction::Enter
        },
    });
    work.selector.state_copies = work.selector.state_copies.saturating_add(16);
    Some(Candidate { token, score })
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END
