//! Learned occurrence admission followed by one bounded, causal byte cursor.
//! Dictionary primes are equality addresses. Exact payload bytes never become
//! selector labels, and output observations never select a source occurrence.
use super::completion_runtime::{candidate_rows, score_rows};
use super::response_entry_types::*;
use super::value_lexemes::{WordAtom, WORD_QUERY};
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::*;
use super::*;

pub(super) fn enabled(control: Control) -> bool {
    control != Control::WordCopyDisabled
}

pub(super) fn eligible(entry: &ResponseEntryState, values: &ValueState, control: Control) -> bool {
    enabled(control)
        && super::response_entry_runtime::eligible(values, control)
        && values.pending.is_none()
        && !entry.active
        && entry.steps == 0
        && entry.seen == values.seen
        && entry.boundary.is_some_and(|anchor| {
            anchor.at_seen == values.seen && anchor.at_seen == values.started_at
        })
        && values
            .lexemes
            .as_ref()
            .is_some_and(|words| words.query_len > 0)
}

fn address(head: &WordCopyModel, word: &WordAtom, work: &mut WordCopyWork) -> u32 {
    work.dictionary_lookups = work.dictionary_lookups.saturating_add(1);
    let result = head.dictionary.binary_search_by(|entry| {
        work.dictionary_comparisons = work.dictionary_comparisons.saturating_add(1);
        for offset in 0..usize::from(entry.len.min(word.len)) {
            work.dictionary_byte_comparisons = work.dictionary_byte_comparisons.saturating_add(1);
            let order = entry.bytes[offset].cmp(&word.bytes[offset]);
            if !order.is_eq() {
                return order;
            }
        }
        entry.len.cmp(&word.len)
    });
    result.map_or(0, |index| head.dictionary[index].prime)
}

pub(super) fn context(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> WordCopyContext {
    let mut context = WordCopyContext {
        addresses: [0; WORD_QUERY],
        query_path: None,
        query_phases: None,
    };
    let Some(head) = model
        .response_entry
        .as_ref()
        .and_then(|head| head.copy.as_ref())
    else {
        return context;
    };
    let Some(words) = &values.lexemes else {
        return context;
    };
    for (index, word) in words.queries[..words.query_len].iter().enumerate() {
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        context.addresses[index] = address(head, word, work);
        work.selector.state_copies = work.selector.state_copies.saturating_add(1);
    }
    if words.query_len < 2 {
        return context;
    }
    let previous = &words.queries[1];
    let last = &words.queries[0];
    work.word_record_reads = work.word_record_reads.saturating_add(2);
    if !matches!(
        control,
        Control::GeometryDisabled | Control::H4Disabled | Control::WordCopyGeometryDisabled
    ) {
        let inverse = model.geometry.inverses[usize::from(previous.pose)];
        context.query_path = Some(
            model.geometry.products
                [model.geometry.row_bases[usize::from(inverse)] + usize::from(last.pose)],
        );
        work.selector.h4_reads = work.selector.h4_reads.saturating_add(2);
        work.selector.metadata_reads = work.selector.metadata_reads.saturating_add(1);
    }
    if !matches!(
        control,
        Control::GeometryDisabled | Control::ZetaDisabled | Control::WordCopyGeometryDisabled
    ) {
        let mut phases = [0; PHASE_CHANNELS];
        for (index, phase) in phases.iter_mut().enumerate() {
            *phase = last.phases[index].wrapping_sub(previous.phases[index]);
        }
        context.query_phases = Some(phases);
        work.selector.phase_subtractions = work
            .selector
            .phase_subtractions
            .saturating_add(PHASE_CHANNELS as u64);
    }
    context
}

pub(super) fn features(
    model: &Model,
    values: &ValueState,
    context: &WordCopyContext,
    index: usize,
    control: Control,
    work: &mut WordCopyWork,
) -> CopyFeatures {
    let mut features = [ValueFeature::default(); WORD_COPY_FEATURES];
    let Some(words) = &values.lexemes else {
        return (features, 0);
    };
    if index >= words.query_len || values.query_len == 0 {
        return (features, 0);
    }
    let candidate = &words.queries[index];
    let previous = (index + 1 < words.query_len).then(|| &words.queries[index + 1]);
    work.word_record_reads = work
        .word_record_reads
        .saturating_add(1 + u64::from(previous.is_some()));
    let preceding = context.addresses.get(index + 1).copied().unwrap_or(0);
    let following = index.checked_sub(1).map_or(0, |i| context.addresses[i]);
    let before_preceding = context.addresses.get(index + 2).copied().unwrap_or(0);
    let query = values.queries[0].cue;
    let query_previous = if values.query_len > 1 {
        values.queries[1].cue
    } else {
        0
    };
    work.selector.metadata_reads = work.selector.metadata_reads.saturating_add(8);
    let mut len = 0;
    let mut add = |kind, a, b| {
        features[len] = ValueFeature { kind, a, b };
        len += 1;
    };
    add(0, 0, 0);
    add(1, u64::from(query), 0);
    add(2, u64::from(query_previous), u64::from(query));
    add(3, index as u64, 0);
    add(4, u64::from(preceding), 0);
    add(5, u64::from(following), 0);
    add(6, u64::from(preceding), u64::from(following));
    add(7, u64::from(before_preceding), u64::from(preceding));
    add(8, u64::from(query), index as u64);
    let missing = u64::from(previous.is_none())
        | (u64::from(index == 0) << 1)
        | (u64::from(words.query_len < 2) << 2);
    add(9, u64::from(candidate.len), missing);
    if let (Some(previous), Some(query_path)) = (previous, context.query_path) {
        let inverse = model.geometry.inverses[usize::from(previous.pose)];
        let source_path = model.geometry.products
            [model.geometry.row_bases[usize::from(inverse)] + usize::from(candidate.pose)];
        let source_inverse = model.geometry.inverses[usize::from(source_path)];
        let relative = model.geometry.products
            [model.geometry.row_bases[usize::from(source_inverse)] + usize::from(query_path)];
        work.selector.h4_reads = work.selector.h4_reads.saturating_add(4);
        work.selector.metadata_reads = work.selector.metadata_reads.saturating_add(2);
        add(10, u64::from(relative), 0);
        if !matches!(
            control,
            Control::OrientationDisabled | Control::HeatmapDisabled
        ) {
            add(
                11,
                u64::from(model.geometry.orientation[usize::from(relative)]),
                0,
            );
            work.selector.orientation_reads = work.selector.orientation_reads.saturating_add(1);
        }
    }
    if let (Some(previous), Some(query_phases)) = (previous, context.query_phases) {
        for channel in 0..PHASE_CHANNELS {
            let source = candidate.phases[channel].wrapping_sub(previous.phases[channel]);
            let relative = query_phases[channel].wrapping_sub(source);
            add(12 + channel as u8, u64::from(relative >> 12), 0);
        }
        work.selector.phase_subtractions = work
            .selector
            .phase_subtractions
            .saturating_add((PHASE_CHANNELS + PHASE_CHANNELS) as u64);
    }
    (features, len)
}

pub(super) fn score(
    head: &WordCopyModel,
    features: &[ValueFeature],
    work: &mut WordCopyWork,
) -> i64 {
    let mut score = 0_i64;
    for feature in features {
        work.selector.feature_queries = work.selector.feature_queries.saturating_add(1);
        let found = head.rows.binary_search_by(|row| {
            work.selector.row_comparisons = work.selector.row_comparisons.saturating_add(1);
            row.feature.cmp(feature)
        });
        if let Ok(index) = found {
            score += i64::from(head.rows[index].weight);
            work.selector.matched_rows = work.selector.matched_rows.saturating_add(1);
            work.selector.score_lookups = work.selector.score_lookups.saturating_add(1);
        }
    }
    score
}

fn copy_history_entry<'a>(
    values: &'a ValueState,
    sequence: u64,
    work: &mut WordCopyWork,
) -> Option<&'a super::value_types::ValueEntry> {
    if sequence >= values.seen || values.seen - sequence > values.recent_len as u64 {
        return None;
    }
    work.selector.metadata_reads = work.selector.metadata_reads.saturating_add(1);
    let entry = values.recent.get((sequence & 31) as usize)?;
    (entry.sequence == sequence).then_some(entry)
}

impl WordCopyState {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    /// The optional completed-word frame removes source spelling/length from
    /// suffix progress while preserving the actual observed geometric path.
    /// Its anchor is derived from retained metadata, never from a target or
    /// source WordAtom pose. No additional persistent anchor is introduced.
    pub(super) fn continuation_features(
        &self,
        model: &Model,
        entry: &ResponseEntryState,
        values: &ValueState,
        control: Control,
        work: &mut WordCopyWork,
    ) -> ([Feature; RESPONSE_ENTRY_FEATURES], usize) {
        let suffix_control = if control == Control::WordCopyGeometryDisabled {
            Control::ResponseEntryGeometryDisabled
        } else {
            control
        };
        if !model
            .response_entry
            .as_ref()
            .and_then(|head| head.copy.as_ref())
            .is_some_and(|copy| copy.completed_word_suffix)
        {
            return entry.features(model, values, suffix_control, &mut work.selector);
        }
        let absent = ([Feature { kind: 0, value: 0 }; RESPONSE_ENTRY_FEATURES], 0);
        if self.progress != WordCopyProgress::Complete
            || !entry.active
            || entry.seen != values.seen
            || entry.steps >= RESPONSE_ENTRY_STEPS
        {
            return absent;
        }
        let Some(origin) = self.origin else {
            return absent;
        };
        let Some(words) = &values.lexemes else {
            return absent;
        };
        if usize::from(origin) >= words.query_len {
            return absent;
        }
        let Some(word) = words.queries.get(usize::from(origin)) else {
            return absent;
        };
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        if word.len == 0 || word.len >= RESPONSE_ENTRY_STEPS {
            return absent;
        }
        let Some(anchor) = entry.boundary else {
            return absent;
        };
        if anchor.at_seen != values.started_at {
            return absent;
        }
        let Some(steps) = entry.steps.checked_sub(word.len) else {
            return absent;
        };
        let Some(final_seen) = anchor.at_seen.checked_add(u64::from(word.len)) else {
            return absent;
        };
        let Some(sequence) = final_seen.checked_sub(1) else {
            return absent;
        };
        if final_seen.checked_add(u64::from(steps)) != Some(values.seen) {
            return absent;
        }
        let Some(endpoint) = copy_history_entry(values, sequence, work) else {
            return absent;
        };
        work.byte_reads = work.byte_reads.saturating_add(1);
        if usize::from(endpoint.pose) >= model.geometry.inverses.len()
            || endpoint.token != u32::from(word.bytes[usize::from(word.len) - 1]) + 2
        {
            return absent;
        }
        let last = if steps > 0 {
            let Some(sequence) = values.seen.checked_sub(1) else {
                return absent;
            };
            let Some(actual) = copy_history_entry(values, sequence, work) else {
                return absent;
            };
            actual.token
        } else {
            BOS
        };
        let previous = if steps > 1 {
            let Some(sequence) = values.seen.checked_sub(2) else {
                return absent;
            };
            let Some(actual) = copy_history_entry(values, sequence, work) else {
                return absent;
            };
            actual.token
        } else {
            BOS
        };
        if last as usize >= model.geometry.tokens.len()
            || previous as usize >= model.geometry.tokens.len()
        {
            return absent;
        }
        let suffix = ResponseEntryState {
            boundary: Some(ResponseEntryAnchor {
                at_seen: final_seen,
                pose: endpoint.pose,
                phases: endpoint.phases,
                query_prime: anchor.query_prime,
            }),
            last,
            previous,
            seen: values.seen,
            steps,
            active: true,
            ..ResponseEntryState::default()
        };
        // Eleven anchor scalars/channels plus five actual-history fields.
        work.selector.state_copies = work.selector.state_copies.saturating_add(16);
        suffix.features(model, values, suffix_control, &mut work.selector)
    }

    /// Existing lexical selection and copy selection share the same Base.
    /// Complete copies use a distinct continuation namespace; aborted copies
    /// resume the inherited actual-history entry law.
    pub(super) fn offer(
        &mut self,
        model: &Model,
        entry: &mut ResponseEntryState,
        values: &ValueState,
        baseline: Candidate,
        lexical: Option<Candidate>,
        control: Control,
        work: &mut WordCopyWork,
    ) -> Option<Candidate> {
        self.pending = None;
        if !enabled(control)
            || !super::response_entry_runtime::eligible(values, control)
            || values.pending.is_some()
            || entry.steps >= RESPONSE_ENTRY_STEPS
            || entry.seen != values.seen
        {
            return lexical;
        }
        let Some(head) = model
            .response_entry
            .as_ref()
            .and_then(|head| head.copy.as_ref())
        else {
            return lexical;
        };
        let Some(anchor) = entry.boundary else {
            return lexical;
        };
        let Some(words) = &values.lexemes else {
            return lexical;
        };
        let mut chosen = None;
        if eligible(entry, values, control) {
            let context = context(model, values, control, work);
            let mut threshold = lexical
                .map_or(0, |candidate| candidate.score - baseline.score)
                .max(0);
            for index in 0..words.query_len {
                let word = &words.queries[index];
                work.word_record_reads = work.word_record_reads.saturating_add(1);
                work.word_candidates = work.word_candidates.saturating_add(1);
                if word.len == 0
                    || usize::from(word.len) + 1 > usize::from(RESPONSE_ENTRY_STEPS - entry.steps)
                {
                    work.bound_rejections = work.bound_rejections.saturating_add(1);
                    continue;
                }
                let (features, len) = features(model, values, &context, index, control, work);
                let increment = score(head, &features[..len], work);
                work.selector.candidate_evaluations =
                    work.selector.candidate_evaluations.saturating_add(1);
                work.selector.candidate_comparisons =
                    work.selector.candidate_comparisons.saturating_add(1);
                if increment > threshold {
                    threshold = increment;
                    chosen = Some((
                        index as u8,
                        0,
                        u32::from(word.bytes[0]) + 2,
                        baseline.score + increment,
                        WordCopyAction::Start,
                    ));
                    work.byte_reads = work.byte_reads.saturating_add(1);
                    work.selector.candidate_writes =
                        work.selector.candidate_writes.saturating_add(1);
                }
            }
        } else if entry.active {
            if let Some(index) = self
                .origin
                .filter(|index| usize::from(*index) < words.query_len)
            {
                let word = &words.queries[usize::from(index)];
                work.word_record_reads = work.word_record_reads.saturating_add(1);
                match self.progress {
                    WordCopyProgress::Emitting { cursor } if cursor < word.len => {
                        work.byte_reads = work.byte_reads.saturating_add(1);
                        chosen = Some((
                            index,
                            cursor,
                            u32::from(word.bytes[usize::from(cursor)]) + 2,
                            baseline.score + 1,
                            WordCopyAction::Byte,
                        ));
                    }
                    WordCopyProgress::Complete => {
                        let (features, len) =
                            self.continuation_features(model, entry, values, control, work);
                        if len == 0 {
                            return lexical;
                        }
                        let (tokens, count, rows, row_count) = candidate_rows(
                            &head.continuation_rows,
                            &head.continuation_postings,
                            &features[..len],
                            &mut work.selector,
                        );
                        let mut best = None;
                        let mut best_score = 0;
                        for token in tokens[..count].iter().copied() {
                            let increment = score_rows(
                                &head.continuation_rows,
                                token,
                                &rows[..row_count],
                                &mut work.selector,
                            );
                            if increment > best_score
                                || (increment == best_score
                                    && increment > 0
                                    && best.is_some_and(|known| token < known))
                            {
                                best = Some(token);
                                best_score = increment;
                            }
                        }
                        if let Some(token) = best {
                            chosen = Some((
                                index,
                                word.len,
                                token,
                                baseline.score + best_score,
                                if token == EOS {
                                    WordCopyAction::Stop
                                } else {
                                    WordCopyAction::Emit
                                },
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
        let Some((index, cursor, token, score, action)) = chosen else {
            return lexical;
        };
        let word = &words.queries[usize::from(index)];
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        self.pending = Some(WordCopyDecision {
            token,
            score,
            word_index: index,
            cursor,
            source_end: word.end,
            source_byte_end: word.byte_end,
            at_seen: values.seen,
            step: entry.steps,
            action,
        });
        entry.pending = Some(ResponseEntryDecision {
            token,
            score,
            boundary_seen: anchor.at_seen,
            step: entry.steps,
            at_seen: values.seen,
            action: if token == EOS {
                ResponseEntryAction::Stop
            } else if entry.active {
                ResponseEntryAction::Emit
            } else {
                ResponseEntryAction::Enter
            },
        });
        work.selector.state_copies = work.selector.state_copies.saturating_add(16);
        Some(Candidate { token, score })
    }

    pub(super) fn selected(&mut self, best: Candidate) {
        if self
            .pending
            .is_some_and(|decision| decision.token != best.token || decision.score != best.score)
        {
            self.pending = None;
        }
    }

    /// Called after ordinary entry observation so its gate/cap/EOS law wins.
    pub(super) fn observe(
        &mut self,
        entry: &ResponseEntryState,
        values: &ValueState,
        token: u32,
        work: &mut WordCopyWork,
    ) {
        work.selector.observations = work.selector.observations.saturating_add(1);
        let pending = self.pending.take();
        if token == EOS {
            if pending.is_some_and(|decision| {
                decision.token == token && decision.at_seen.checked_add(1) == Some(values.seen)
            }) {
                work.selector.commits = work.selector.commits.saturating_add(1);
            } else if pending.is_some() {
                work.selector.mismatches = work.selector.mismatches.saturating_add(1);
            }
            if self.origin.is_some() {
                work.selector.stops = work.selector.stops.saturating_add(1);
            }
            self.reset();
            return;
        }
        if !entry.active || entry.boundary.is_none() {
            if pending.is_some_and(|decision| {
                decision.token == token && decision.at_seen.checked_add(1) == Some(values.seen)
            }) {
                work.selector.commits = work.selector.commits.saturating_add(1);
            } else if pending.is_some() {
                work.selector.mismatches = work.selector.mismatches.saturating_add(1);
            }
            self.reset();
            return;
        }
        let matched = pending.filter(|decision| {
            decision.token == token
                && decision.at_seen.checked_add(1) == Some(values.seen)
                && decision.step.checked_add(1) == Some(entry.steps)
        });
        if let Some(decision) = matched {
            work.selector.commits = work.selector.commits.saturating_add(1);
            if decision.action == WordCopyAction::Start {
                self.origin = Some(decision.word_index);
                work.word_record_reads = work.word_record_reads.saturating_add(1);
                let len = values.lexemes.as_ref().map_or(0, |words| {
                    words.queries[usize::from(decision.word_index)].len
                });
                self.progress = if len == 1 {
                    WordCopyProgress::Complete
                } else {
                    WordCopyProgress::Emitting { cursor: 1 }
                };
            } else if decision.action == WordCopyAction::Byte {
                let cursor = decision.cursor.saturating_add(1);
                work.word_record_reads = work.word_record_reads.saturating_add(1);
                let len = values.lexemes.as_ref().map_or(0, |words| {
                    words.queries[usize::from(decision.word_index)].len
                });
                self.progress = if cursor == len {
                    WordCopyProgress::Complete
                } else {
                    WordCopyProgress::Emitting { cursor }
                };
            }
        } else if matches!(self.progress, WordCopyProgress::Emitting { .. }) {
            self.progress = WordCopyProgress::Aborted;
            work.selector.mismatches = work.selector.mismatches.saturating_add(1);
        }
        work.selector.state_copies = work.selector.state_copies.saturating_add(3);
    }
}
