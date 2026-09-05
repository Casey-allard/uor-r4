//! Integer/table next-token transitions after a causally completed numeral.
//! No suffix string, answer buffer, task grammar or target lookup is stored.
use super::completion_types::*;
use super::value_types::{ValueAction, ValueState};
use super::*;

impl CompletionState {
    /// Caller response boundaries clear completion state, preserving the two
    /// actual observed tokens and their absolute observation count.
    pub(super) fn reset(&mut self) {
        self.anchor = None;
        self.steps = 0;
        self.active = false;
        self.last_action = CompletionAction::Base;
        self.pending = None;
    }

    /// Called after typed-value observation. The seed is taken from its prior
    /// pending decision, never reconstructed from the observed token's value.
    pub(super) fn observe(
        &mut self,
        _model: &Model,
        values: &ValueState,
        token: u32,
        seed: Option<CompletionSeed>,
        control: Control,
        work: &mut CompletionWork,
    ) {
        work.observations = work.observations.saturating_add(1);
        let pending = self.pending.take();
        let was_active = self.active;
        self.last_action = CompletionAction::Base;
        if was_active {
            self.steps = self.steps.saturating_add(1);
            let matched =
                pending.filter(|decision| decision.token == token && decision.at_seen == self.seen);
            if let Some(decision) = matched {
                self.last_action = decision.action;
                work.commits = work.commits.saturating_add(1);
            } else {
                self.last_action = CompletionAction::Base;
                work.base_steps = work.base_steps.saturating_add(1);
                if pending.is_some() {
                    work.mismatches = work.mismatches.saturating_add(1);
                }
            }
        }
        self.previous = self.last;
        self.last = token;
        self.seen = values.seen;
        work.state_copies = work.state_copies.saturating_add(3);
        if token == EOS {
            if was_active {
                work.stops = work.stops.saturating_add(1);
            }
            self.reset();
            self.last_action = CompletionAction::Stop;
            return;
        }
        if was_active {
            if self.steps >= COMPLETION_STEPS {
                let action = self.last_action;
                self.reset();
                self.last_action = action;
                work.step_limits = work.step_limits.saturating_add(1);
            }
            return;
        }
        if matches!(control, Control::ValuesDisabled | Control::MemoryDisabled)
            || !values.active
            || !values.consumed
            || values.emission.is_some()
            || values.query_len == 0
        {
            return;
        }
        let Some(seed) = seed
            .filter(|seed| seed.token == token && seed.at_seen.checked_add(1) == Some(values.seen))
        else {
            return;
        };
        work.metadata_reads = work.metadata_reads.saturating_add(2);
        let Some(record) = values.records.last().filter(|record| {
            record.id == seed.write_id
                && record.derived
                && record
                    .derivation
                    .is_some_and(|origin| origin.action == seed.action)
        }) else {
            return;
        };
        self.anchor = Some(CompletionAnchor {
            write_id: record.id,
            action: seed.action,
            at_seen: values.seen,
            pose: values.pose,
            phases: values.phases,
            query_prime: values.queries[0].cue,
        });
        self.steps = 0;
        self.active = true;
        self.last_action = CompletionAction::Base;
        work.anchors = work.anchors.saturating_add(1);
        // Five scalar fields and the eight-channel phase array are copied.
        work.state_copies = work.state_copies.saturating_add(13);
    }

    pub(super) fn features(
        &self,
        model: &Model,
        values: &ValueState,
        control: Control,
        work: &mut CompletionWork,
    ) -> ([Feature; COMPLETION_FEATURES], usize) {
        let mut features = [Feature { kind: 0, value: 0 }; COMPLETION_FEATURES];
        let Some(anchor) = self.anchor else {
            return (features, 0);
        };
        let last = u64::from(model.geometry.tokens[self.last as usize].prime);
        let previous = u64::from(model.geometry.tokens[self.previous as usize].prime);
        work.metadata_reads = work.metadata_reads.saturating_add(2);
        let mut len = 0;
        let mut add = |kind, value| {
            features[len] = Feature { kind, value };
            len += 1;
        };
        add(0, 0);
        add(1, last);
        add(2, (previous << 32) | last);
        add(3, u64::from(anchor.query_prime));
        add(4, (u64::from(anchor.query_prime) << 32) | last);
        let op = match anchor.action {
            ValueAction::Copy => 0,
            ValueAction::Add => 1,
        };
        add(5, (op << 8) | u64::from(self.steps));
        if !matches!(
            control,
            Control::GeometryDisabled
                | Control::H4Disabled
                | Control::ValueCompletionGeometryDisabled
        ) {
            let inverse = model.geometry.inverses[usize::from(anchor.pose)];
            let relative = model.geometry.products
                [model.geometry.row_bases[usize::from(inverse)] + usize::from(values.pose)];
            work.h4_reads = work.h4_reads.saturating_add(2);
            // The row-base addressing table is separate from the H4 entries.
            work.metadata_reads = work.metadata_reads.saturating_add(1);
            add(6, u64::from(relative));
            if !matches!(
                control,
                Control::OrientationDisabled | Control::HeatmapDisabled
            ) {
                add(
                    7,
                    u64::from(model.geometry.orientation[usize::from(relative)]),
                );
                work.orientation_reads = work.orientation_reads.saturating_add(1);
            }
        }
        if !matches!(
            control,
            Control::GeometryDisabled
                | Control::ZetaDisabled
                | Control::ValueCompletionGeometryDisabled
        ) {
            for channel in 0..PHASE_CHANNELS {
                let phase = values.phases[channel].wrapping_sub(anchor.phases[channel]);
                add(8 + channel as u8, u64::from(phase >> 12));
                work.phase_subtractions = work.phase_subtractions.saturating_add(1);
            }
        }
        (features, len)
    }

    pub(super) fn offer(
        &mut self,
        model: &Model,
        values: &ValueState,
        baseline: Candidate,
        control: Control,
        work: &mut CompletionWork,
    ) -> Option<Candidate> {
        self.pending = None;
        if !self.active
            || !values.active
            || self.steps >= COMPLETION_STEPS
            || matches!(
                control,
                Control::ValueCompletionDisabled
                    | Control::ValuesDisabled
                    | Control::MemoryDisabled
            )
        {
            return None;
        }
        let head = model.completion.as_ref()?;
        let anchor = self.anchor?;
        let (features, len) = self.features(model, values, control, work);
        let (tokens, count, rows, row_count) = candidates(head, &features[..len], work);
        let mut best = None;
        let mut best_score = 0_i64;
        for token in tokens[..count].iter().copied() {
            let score = score_candidate(head, token, &rows[..row_count], work);
            if score > best_score
                || (score == best_score && score > 0 && best.is_some_and(|known| token < known))
            {
                best = Some(token);
                best_score = score;
            }
        }
        let token = best?;
        let score = baseline.score + best_score;
        self.pending = Some(CompletionDecision {
            token,
            score,
            write_id: anchor.write_id,
            step: self.steps,
            at_seen: self.seen,
            action: if token == EOS {
                CompletionAction::Stop
            } else {
                CompletionAction::Emit
            },
        });
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
}

pub(super) fn candidates(
    head: &CompletionModel,
    features: &[Feature],
    work: &mut CompletionWork,
) -> (
    [u32; COMPLETION_CANDIDATES],
    usize,
    [usize; COMPLETION_FEATURES],
    usize,
) {
    candidate_rows(&head.rows, &head.global_postings, features, work)
}

pub(super) fn candidate_rows(
    score_rows: &[ScoreRow],
    global_postings: &[u32],
    features: &[Feature],
    work: &mut CompletionWork,
) -> (
    [u32; COMPLETION_CANDIDATES],
    usize,
    [usize; COMPLETION_FEATURES],
    usize,
) {
    candidate_rows_bounded::<COMPLETION_FEATURES>(score_rows, global_postings, features, work)
}

/// Fixed scratch selected by the caller's feature bound. Existing completion
/// callers retain their sixteen-row scratch; composed binding uses thirty-two.
pub(super) fn candidate_rows_bounded<const N: usize>(
    score_rows: &[ScoreRow],
    global_postings: &[u32],
    features: &[Feature],
    work: &mut CompletionWork,
) -> ([u32; COMPLETION_CANDIDATES], usize, [usize; N], usize) {
    let mut tokens = [0; COMPLETION_CANDIDATES];
    let mut count = 0;
    let mut rows = [0; N];
    let mut row_count = 0;
    for feature in features {
        work.feature_queries = work.feature_queries.saturating_add(1);
        if let Ok(index) = score_rows.binary_search_by(|row| {
            work.row_comparisons = work.row_comparisons.saturating_add(1);
            row.feature.cmp(feature)
        }) {
            rows[row_count] = index;
            row_count += 1;
            work.matched_rows = work.matched_rows.saturating_add(1);
            for &token in &score_rows[index].postings {
                offer_token(&mut tokens, &mut count, token, work);
            }
        }
    }
    for &token in global_postings {
        offer_token(&mut tokens, &mut count, token, work);
    }
    (tokens, count, rows, row_count)
}

fn offer_token(
    tokens: &mut [u32; COMPLETION_CANDIDATES],
    count: &mut usize,
    token: u32,
    work: &mut CompletionWork,
) {
    work.posting_offers = work.posting_offers.saturating_add(1);
    for &known in &tokens[..*count] {
        work.candidate_comparisons = work.candidate_comparisons.saturating_add(1);
        if known == token {
            return;
        }
    }
    if *count == COMPLETION_CANDIDATES {
        work.candidate_drops = work.candidate_drops.saturating_add(1);
    } else {
        tokens[*count] = token;
        *count += 1;
        work.candidate_writes = work.candidate_writes.saturating_add(1);
    }
}

pub(super) fn score_candidate(
    head: &CompletionModel,
    token: u32,
    rows: &[usize],
    work: &mut CompletionWork,
) -> i64 {
    score_rows(&head.rows, token, rows, work)
}

pub(super) fn score_rows(
    score_rows: &[ScoreRow],
    token: u32,
    rows: &[usize],
    work: &mut CompletionWork,
) -> i64 {
    let mut score = 0;
    for &index in rows {
        let row = &score_rows[index];
        work.score_lookups = work.score_lookups.saturating_add(1);
        score += i64::from(
            row.scores
                .binary_search_by(|entry| {
                    work.score_comparisons = work.score_comparisons.saturating_add(1);
                    entry.token.cmp(&token)
                })
                .map_or(row.default_score, |index| row.scores[index].score),
        );
    }
    work.candidate_evaluations = work.candidate_evaluations.saturating_add(1);
    score
}
