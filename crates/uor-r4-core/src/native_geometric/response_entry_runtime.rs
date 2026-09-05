//! Integer/table response transitions anchored to an actual query boundary.
//! Entry commits only when its selected token is observed; no answer buffer,
//! numeric write or target-derived state is synthesized.
use super::completion_runtime::{candidate_rows, score_rows};
use super::completion_types::CompletionWork;
use super::response_entry_types::*;
use super::value_types::ValueState;
use super::*;

pub(super) fn eligible(values: &ValueState, control: Control) -> bool {
    !matches!(
        control,
        Control::ResponseEntryDisabled | Control::ValuesDisabled | Control::MemoryDisabled
    ) && values.active
        && !values.consumed
        && values.emission.is_none()
        && values.query_len > 0
        && !values.sources.is_empty()
        && values.next_id != u64::MAX
}

impl ResponseEntryState {
    /// Response boundaries clear selection while preserving actual history.
    pub(super) fn reset(&mut self) {
        self.boundary = None;
        self.steps = 0;
        self.active = false;
        self.last_action = ResponseEntryAction::Base;
        self.pending = None;
    }

    pub(super) fn begin(
        &mut self,
        values: &ValueState,
        control: Control,
        work: &mut CompletionWork,
    ) {
        self.reset();
        if !eligible(values, control)
            || values.pending.is_some()
            || values.started_at != values.seen
            || self.seen != values.seen
        {
            return;
        }
        self.boundary = Some(ResponseEntryAnchor {
            at_seen: values.seen,
            pose: values.pose,
            phases: values.phases,
            query_prime: values.queries[0].cue,
        });
        work.anchors = work.anchors.saturating_add(1);
        work.metadata_reads = work.metadata_reads.saturating_add(1);
        work.state_copies = work.state_copies.saturating_add(11);
    }

    pub(super) fn observe(
        &mut self,
        _model: &Model,
        values: &ValueState,
        token: u32,
        control: Control,
        work: &mut CompletionWork,
    ) {
        work.observations = work.observations.saturating_add(1);
        let pending = self.pending.take();
        let was_active = self.active;
        let boundary = self.boundary;
        let matched = pending.filter(|decision| {
            decision.token == token
                && decision.at_seen == self.seen
                && decision.step == self.steps
                && boundary.is_some_and(|anchor| anchor.at_seen == decision.boundary_seen)
                && decision.at_seen.checked_add(1) == Some(values.seen)
        });
        self.previous = self.last;
        self.last = token;
        self.seen = values.seen;
        self.last_action = ResponseEntryAction::Base;
        work.state_copies = work.state_copies.saturating_add(3);

        // EOS updates typed state before this call, so its active bit is
        // already false. Commit the actual selected stop before gate checks.
        if token == EOS {
            if matched.is_some() {
                work.commits = work.commits.saturating_add(1);
            } else if boundary.is_some() {
                work.base_steps = work.base_steps.saturating_add(1);
                if pending.is_some() {
                    work.mismatches = work.mismatches.saturating_add(1);
                }
            }
            if was_active || matched.is_some() {
                work.stops = work.stops.saturating_add(1);
            }
            self.reset();
            self.last_action = ResponseEntryAction::Stop;
            return;
        }
        if !eligible(values, control) {
            self.reset();
            return;
        }
        if was_active {
            self.steps = self.steps.saturating_add(1);
            if let Some(decision) = matched {
                self.last_action = decision.action;
                work.commits = work.commits.saturating_add(1);
            } else {
                work.base_steps = work.base_steps.saturating_add(1);
                if pending.is_some() {
                    work.mismatches = work.mismatches.saturating_add(1);
                }
            }
            if self.steps >= RESPONSE_ENTRY_STEPS {
                // A cap ends the component without fabricating EOS. Clearing
                // the action makes inactive state independent of stale origin.
                self.reset();
                work.step_limits = work.step_limits.saturating_add(1);
            }
        } else if let Some(decision) = matched.filter(|decision| {
            decision.action == ResponseEntryAction::Enter
                && decision.step == 0
                && boundary.is_some_and(|anchor| anchor.at_seen == decision.at_seen)
        }) {
            self.active = true;
            self.steps = 1;
            self.last_action = decision.action;
            work.commits = work.commits.saturating_add(1);
        } else {
            if boundary.is_some() {
                work.base_steps = work.base_steps.saturating_add(1);
                if pending.is_some() {
                    work.mismatches = work.mismatches.saturating_add(1);
                }
            }
            // An unselected or mismatched first observation closes eligibility
            // until the caller begins another response.
            self.reset();
        }
    }

    pub(super) fn features(
        &self,
        model: &Model,
        values: &ValueState,
        control: Control,
        work: &mut CompletionWork,
    ) -> ([Feature; RESPONSE_ENTRY_FEATURES], usize) {
        let mut features = [Feature { kind: 0, value: 0 }; RESPONSE_ENTRY_FEATURES];
        let Some(anchor) = self.boundary else {
            return (features, 0);
        };
        let last = u64::from(model.geometry.tokens[self.last as usize].prime);
        let previous = u64::from(model.geometry.tokens[self.previous as usize].prime);
        work.metadata_reads = work.metadata_reads.saturating_add(2);
        let offset = if self.active { 16 } else { 0 };
        let mut len = 0;
        let mut add = |kind, value| {
            features[len] = Feature {
                kind: kind + offset,
                value,
            };
            len += 1;
        };
        add(0, 0);
        add(1, last);
        add(2, (previous << 32) | last);
        add(3, u64::from(anchor.query_prime));
        add(4, (u64::from(anchor.query_prime) << 32) | last);
        add(5, u64::from(self.steps));
        if !matches!(
            control,
            Control::GeometryDisabled
                | Control::H4Disabled
                | Control::ResponseEntryGeometryDisabled
        ) {
            let inverse = model.geometry.inverses[usize::from(anchor.pose)];
            let relative = model.geometry.products
                [model.geometry.row_bases[usize::from(inverse)] + usize::from(values.pose)];
            work.h4_reads = work.h4_reads.saturating_add(2);
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
                | Control::ResponseEntryGeometryDisabled
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
        if !eligible(values, control)
            || values.pending.is_some()
            || self.steps >= RESPONSE_ENTRY_STEPS
            || self.seen != values.seen
        {
            return None;
        }
        let head = model.response_entry.as_ref()?;
        let anchor = self.boundary?;
        if anchor.at_seen != values.started_at
            || (!self.active && (self.steps != 0 || self.seen != anchor.at_seen))
        {
            return None;
        }
        let (features, len) = self.features(model, values, control, work);
        let (tokens, count, rows, row_count) =
            candidate_rows(&head.rows, &head.global_postings, &features[..len], work);
        let mut best = None;
        let mut best_score = 0_i64;
        for token in tokens[..count].iter().copied() {
            let score = score_rows(&head.rows, token, &rows[..row_count], work);
            if score > best_score
                || (score == best_score && score > 0 && best.is_some_and(|known| token < known))
            {
                best = Some(token);
                best_score = score;
            }
        }
        let token = best?;
        let score = baseline.score + best_score;
        self.pending = Some(ResponseEntryDecision {
            token,
            score,
            boundary_seen: anchor.at_seen,
            step: self.steps,
            at_seen: self.seen,
            action: if token == EOS {
                ResponseEntryAction::Stop
            } else if self.active {
                ResponseEntryAction::Emit
            } else {
                ResponseEntryAction::Enter
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
