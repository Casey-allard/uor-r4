//! Causal response query and selected-read state. This complete file is part
//! of the bounded integer/table kernel, like memory_runtime.rs.
use super::memory_types::*;
use super::{Candidate, Model, Work, BOS, EOS, PHASE_CHANNELS};

impl MemoryState {
    pub(super) fn response_view(&self) -> Option<ResponseStateView> {
        self.response.as_ref().map(|response| ResponseStateView {
            active: response.active,
            started_at: response.started_at,
            steps: response.steps,
            query_tokens: response.queries.len(),
            query_pose: response.query_pose,
            query_phases: response.query_phases,
            selected_sequence: response.selected.map(|reference| reference.sequence),
            last_action: response.last_action,
        })
    }

    /// Capture query and posting visits before any response token is observed.
    /// This is a caller-supplied boundary, never a grammar/answer detector.
    pub(super) fn begin_response(&mut self, model: &Model, memory: &MemoryModel, work: &mut Work) {
        if self.response.is_none() {
            return;
        }
        self.end_response();
        for distance in 1..=memory.config.query_tokens {
            let Some(entry) = self.recent(distance) else {
                break;
            };
            if let Some(response) = &mut self.response {
                response.queries.push(entry);
            }
        }
        if let Some(response) = &mut self.response {
            response.active = true;
            response.started_at = self.seen;
            response.query_pose = self.pose;
            response.query_phases = self.phases;
            'visits: for offset in 0..memory.config.postings_per_address {
                for source_distance in 1..=memory.config.source_offsets {
                    for query in &response.queries {
                        if response.references.len() == memory.config.candidate_limit {
                            break 'visits;
                        }
                        let cue = super::memory_runtime::cue_identity(memory, query.token, work);
                        let base = (((cue as usize) << memory.source_shift)
                            | (source_distance - 1))
                            << memory.posting_shift;
                        response.references.push(self.index[base + offset]);
                        work.memory_index_reads = work.memory_index_reads.saturating_add(1);
                    }
                }
            }
            work.response_query_captures = work.response_query_captures.saturating_add(1);
        }
        // Token and geometry validity is established by the artifact/session.
        let _ = model;
    }

    pub(super) fn end_response(&mut self) {
        self.pending = None;
        if let Some(response) = &mut self.response {
            response.active = false;
            response.started_at = 0;
            response.steps = 0;
            response.queries.clear();
            response.references.clear();
            response.selected = None;
            response.last_action = ResponseAction::Base;
        }
    }

    pub(super) fn reference_for_sequence(&self, sequence: u64) -> Option<MemoryReference> {
        if sequence >= self.seen || sequence < self.seen.saturating_sub(self.length as u64) {
            return None;
        }
        let distance = (self.seen - sequence) as usize;
        let slot = if self.cursor >= distance {
            self.cursor - distance
        } else {
            self.ring.len() - (distance - self.cursor)
        };
        (self.ring[slot].sequence == sequence).then_some(MemoryReference { sequence, slot })
    }

    /// Choose provenance from scored model alternatives alone. No observation
    /// token/target participates. Prediction may replace this transient value;
    /// repeated predictions never advance the committed read cursor.
    pub(super) fn select_response(&mut self, _model: &Model, best: Candidate, work: &mut Work) {
        self.pending = None;
        if !self
            .response
            .as_ref()
            .is_some_and(|response| response.active)
        {
            return;
        }
        let mut decision = ResponseDecision {
            token: best.token,
            score: best.score,
            sequence: None,
            slot: None,
            action: if best.token == EOS {
                ResponseAction::Stop
            } else {
                ResponseAction::Base
            },
            at_seen: self.seen,
        };
        if best.token != EOS {
            for candidate in &self.composed {
                work.response_reference_reads = work.response_reference_reads.saturating_add(1);
                if candidate.token == best.token && candidate.score == best.score {
                    if let Some(reference) = self.reference_for_sequence(candidate.sequence) {
                        decision.sequence = Some(reference.sequence);
                        decision.slot = Some(reference.slot);
                        decision.action = candidate.action;
                        break;
                    }
                }
            }
        }
        self.pending = Some(decision);
    }

    /// Commit only a decision computed before this observation. Teacher-forced
    /// disagreement clears the cursor; it cannot select a matching source.
    pub(super) fn commit_response(&mut self, token: u32, work: &mut Work) {
        let pending = self.pending.take();
        let Some(response) = &mut self.response else {
            return;
        };
        if !response.active {
            return;
        }
        response.steps = response.steps.saturating_add(1);
        response.selected = None;
        if token == EOS {
            response.active = false;
            response.last_action = ResponseAction::Stop;
            work.response_stops = work.response_stops.saturating_add(1);
            return;
        }
        let Some(decision) =
            pending.filter(|value| value.at_seen == self.seen && value.token == token)
        else {
            response.last_action = ResponseAction::Base;
            work.response_mismatches = work.response_mismatches.saturating_add(1);
            return;
        };
        response.last_action = decision.action;
        work.response_commits = work.response_commits.saturating_add(1);
        match decision.action {
            ResponseAction::Continue | ResponseAction::Requery => {
                if let (Some(sequence), Some(slot)) = (decision.sequence, decision.slot) {
                    if sequence < response.started_at && self.ring[slot].sequence == sequence {
                        response.selected = Some(MemoryReference { sequence, slot });
                    }
                }
                if decision.action == ResponseAction::Continue {
                    work.response_continuations = work.response_continuations.saturating_add(1);
                } else {
                    work.response_requeries = work.response_requeries.saturating_add(1);
                }
            }
            ResponseAction::Base => {
                work.response_base_steps = work.response_base_steps.saturating_add(1)
            }
            ResponseAction::Stop => {}
        }
    }

    /// One optional direct successor of the previously selected occurrence.
    /// Source/query local steps retain H4 order and full modular phases. The
    /// reserved high-bit action addresses are disjoint from ordinary routes.
    pub(super) fn collect_continuation(&mut self, model: &Model, work: &mut Work) {
        let Some(response) = self.response.as_ref().filter(|response| response.active) else {
            return;
        };
        let Some(selected) = response.selected else {
            return;
        };
        let Some(next_sequence) = selected.sequence.checked_add(1) else {
            return;
        };
        if next_sequence >= response.started_at {
            return;
        }
        let Some(previous) = self.reference_for_sequence(selected.sequence) else {
            return;
        };
        let Some(next) = self.reference_for_sequence(next_sequence) else {
            return;
        };
        let previous = self.ring[previous.slot];
        let value = self.ring[next.slot];
        if value.token == BOS || value.token == EOS {
            return;
        }
        let Some(current) = self.recent(1) else {
            return;
        };
        let last = model.geometry.tokens[current.token as usize].prime;
        let before_last = self
            .recent(2)
            .map(|entry| model.geometry.tokens[entry.token as usize].prime)
            .unwrap_or(2);
        let source_inverse = model.geometry.inverses[usize::from(previous.pose)];
        let source_path = model.geometry.products
            [model.geometry.row_bases[usize::from(source_inverse)] + usize::from(value.pose)];
        let inverse = model.geometry.inverses[usize::from(source_path)];
        let query_step = &model.geometry.tokens[current.token as usize];
        let relative = model.geometry.products
            [model.geometry.row_bases[usize::from(inverse)] + usize::from(query_step.leaf)];
        work.memory_h4_reads = work.memory_h4_reads.saturating_add(5);
        let mut age = self.seen - 1 - value.sequence;
        let mut age_bin = 0;
        while age > 0 {
            age >>= 1;
            age_bin += 1;
        }
        let mut features = [MemoryFeature { kind: 0, value: 0 }; MEMORY_FEATURE_COUNT];
        features[1] = MemoryFeature {
            kind: 1,
            value: 1 << 63,
        };
        features[2] = MemoryFeature {
            kind: 2,
            value: (u64::from(last) << 16) | (1 << 15),
        };
        features[3] = MemoryFeature {
            kind: 3,
            value: age_bin,
        };
        features[4] = MemoryFeature {
            kind: 4,
            value: (u64::from(last) << 8) | age_bin,
        };
        features[5] = MemoryFeature {
            kind: 5,
            value: u64::from(relative),
        };
        features[6] = MemoryFeature {
            kind: 6,
            value: (u64::from(relative) << 16) | (1 << 15),
        };
        features[7] = MemoryFeature {
            kind: 7,
            value: u64::from(model.geometry.orientation[usize::from(relative)]),
        };
        for channel in 0..PHASE_CHANNELS {
            let phase = query_step.phases[channel]
                .wrapping_sub(value.phases[channel].wrapping_sub(previous.phases[channel]));
            features[8 + channel] = MemoryFeature {
                kind: 8 + channel as u8,
                value: u64::from(phase >> 12),
            };
        }
        work.memory_phase_updates = work
            .memory_phase_updates
            .saturating_add((PHASE_CHANNELS + PHASE_CHANNELS) as u64);
        features[16] = MemoryFeature {
            kind: 16,
            value: (u64::from(before_last) << 32) | u64::from(last),
        };
        features[17] = MemoryFeature {
            kind: 17,
            value: (1 << 63) | (u64::from(before_last) << 24) | u64::from(last),
        };
        self.candidates.push(MemoryCandidate {
            action: ResponseAction::Continue,
            sequence: value.sequence,
            token: value.token,
            score: i64::from(model.prior_scores[value.token as usize]),
            features,
        });
        work.memory_candidates = work.memory_candidates.saturating_add(1);
        work.response_reference_reads = work.response_reference_reads.saturating_add(2);
    }
}
