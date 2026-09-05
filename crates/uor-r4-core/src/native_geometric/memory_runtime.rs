//! Bounded prime-address memory reads. All arithmetic in this file belongs to
//! the integer/table kernel; construction and fitting live in other modules.
use super::memory_types::*;
use super::{Control, Model, Work, BOS, PHASE_CHANNELS};

fn cue_identity(memory: &MemoryModel, token: u32, work: &mut Work) -> u32 {
    if let Some(aliases) = &memory.cue_aliases {
        work.memory_cue_reads = work.memory_cue_reads.saturating_add(1);
        aliases.representatives[token as usize]
    } else {
        token
    }
}

/// Exact schema-/3 address. Host validation bounds both complete primes to
/// 24 bits; configured loops bound query/source distances to 1..32/1..16 and
/// posting rank to 0..7. All 60 occupied bits are disjoint; none are hashed.
pub(super) fn pack_query_occurrence(
    previous_prime: u32,
    last_prime: u32,
    query_distance: usize,
    source_distance: usize,
    posting_rank: usize,
) -> u64 {
    (u64::from(previous_prime) << 36)
        | (u64::from(last_prime) << 12)
        | (((query_distance - 1) as u64) << 7)
        | (((source_distance - 1) as u64) << 3)
        | posting_rank as u64
}

impl MemoryFeature {
    pub(super) fn admitted(self, control: Control) -> bool {
        match control {
            Control::GeometryDisabled => self.kind < 5 || self.kind > 15,
            Control::H4Disabled => !(5..=7).contains(&self.kind),
            Control::ZetaDisabled => !(8..=15).contains(&self.kind),
            Control::OrientationDisabled | Control::HeatmapDisabled => self.kind != 7,
            _ => true,
        }
    }
}

impl MemoryState {
    pub(super) fn state(&self) -> MemoryStateView {
        let mut view = self.view;
        view.retained_tokens = self.length;
        view
    }

    fn recent(&self, distance: usize) -> Option<MemoryEntry> {
        if distance == 0 || distance > self.length {
            return None;
        }
        let slot = if self.cursor >= distance {
            self.cursor - distance
        } else {
            self.ring.len() - (distance - self.cursor)
        };
        Some(self.ring[slot])
    }

    pub(super) fn observe(
        &mut self,
        model: &Model,
        memory: &MemoryModel,
        token: u32,
        work: &mut Work,
    ) {
        // Index values following each recent cue. No token classes, parser or
        // answer labels participate in write admission.
        for distance in 1..=memory.config.source_offsets {
            let Some(cue) = self.recent(distance) else {
                break;
            };
            let cue = cue_identity(memory, cue.token, work);
            let base =
                (((cue as usize) << memory.source_shift) | (distance - 1)) << memory.posting_shift;
            for offset in (1..memory.config.postings_per_address).rev() {
                self.index[base + offset] = self.index[base + offset - 1];
                work.memory_index_reads = work.memory_index_reads.saturating_add(1);
                work.memory_index_writes = work.memory_index_writes.saturating_add(1);
            }
            self.index[base] = MemoryReference {
                sequence: self.seen,
                slot: self.cursor,
            };
            work.memory_index_writes = work.memory_index_writes.saturating_add(1);
        }
        let geometry = &model.geometry.tokens[token as usize];
        self.pose = model.geometry.products
            [model.geometry.row_bases[usize::from(self.pose)] + usize::from(geometry.leaf)];
        work.memory_h4_reads = work.memory_h4_reads.saturating_add(1);
        for (phase, delta) in self.phases.iter_mut().zip(geometry.phases) {
            *phase = phase.wrapping_add(delta);
        }
        work.memory_phase_updates = work
            .memory_phase_updates
            .saturating_add(PHASE_CHANNELS as u64);
        self.ring[self.cursor] = MemoryEntry {
            sequence: self.seen,
            token,
            pose: self.pose,
            phases: self.phases,
        };
        self.cursor += 1;
        if self.cursor == self.ring.len() {
            self.cursor = 0;
        }
        if self.length < self.ring.len() {
            self.length += 1;
        }
        self.seen = self.seen.saturating_add(1);
    }

    pub(super) fn collect(
        &mut self,
        model: &Model,
        memory: &MemoryModel,
        control: Control,
        work: &mut Work,
    ) {
        self.candidates.clear();
        let last = self
            .recent(1)
            .map(|entry| model.geometry.tokens[entry.token as usize].prime)
            .unwrap_or(2);
        let query_context = memory.schema == QUERY_CONTEXT_MEMORY_SCHEMA;
        let previous_query = if query_context {
            self.recent(2)
                .map(|entry| model.geometry.tokens[entry.token as usize].prime)
                .unwrap_or(2)
        } else {
            2
        };
        let oldest = self.seen.saturating_sub(self.ring.len() as u64);
        let mut visits = 0;
        // Visit each query position before spending the budget on older
        // postings, so a larger query window is not a silently dead setting.
        'queries: for offset in 0..memory.config.postings_per_address {
            for source_distance in 1..=memory.config.source_offsets {
                for query_distance in 1..=memory.config.query_tokens {
                    let Some(query) = self.recent(query_distance) else {
                        break;
                    };
                    if visits >= memory.config.candidate_limit {
                        break 'queries;
                    }
                    let query = cue_identity(memory, query.token, work);
                    let base = (((query as usize) << memory.source_shift) | (source_distance - 1))
                        << memory.posting_shift;
                    visits += 1;
                    work.memory_index_reads = work.memory_index_reads.saturating_add(1);
                    let reference = self.index[base + offset];
                    if reference.sequence == u64::MAX {
                        continue;
                    }
                    let value = self.ring[reference.slot];
                    if value.sequence != reference.sequence
                        || reference.sequence < oldest
                        || reference.sequence.saturating_sub(source_distance as u64) < oldest
                        || reference.sequence < source_distance as u64
                    {
                        work.memory_stale_rejections =
                            work.memory_stale_rejections.saturating_add(1);
                        continue;
                    }
                    if value.token == BOS {
                        continue;
                    }
                    let age = self.seen - 1 - reference.sequence;
                    let mut remaining = age;
                    let mut age_bin = 0_u64;
                    while remaining > 0 {
                        remaining >>= 1;
                        age_bin += 1;
                    }
                    let inverse = model.geometry.inverses[usize::from(value.pose)];
                    let relative = model.geometry.products
                        [model.geometry.row_bases[usize::from(inverse)] + usize::from(self.pose)];
                    work.memory_h4_reads = work.memory_h4_reads.saturating_add(2);
                    let offsets = ((query_distance as u64) << 8) | source_distance as u64;
                    let mut features = [MemoryFeature { kind: 0, value: 0 }; MEMORY_FEATURE_COUNT];
                    features[0] = MemoryFeature { kind: 0, value: 0 };
                    features[1] = MemoryFeature {
                        kind: 1,
                        value: offsets,
                    };
                    features[2] = MemoryFeature {
                        kind: 2,
                        value: (u64::from(last) << 16) | offsets,
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
                        value: (u64::from(relative) << 16) | offsets,
                    };
                    features[7] = MemoryFeature {
                        kind: 7,
                        value: u64::from(model.geometry.orientation[usize::from(relative)]),
                    };
                    work.memory_h4_reads = work.memory_h4_reads.saturating_add(1);
                    for channel in 0..PHASE_CHANNELS {
                        // Subtract full phases before binning, preserving common
                        // frame-offset invariance across checkpoint replay.
                        features[8 + channel] = MemoryFeature {
                            kind: 8 + channel as u8,
                            value: u64::from(
                                self.phases[channel].wrapping_sub(value.phases[channel]) >> 12,
                            ),
                        };
                    }
                    work.memory_phase_updates = work
                        .memory_phase_updates
                        .saturating_add(PHASE_CHANNELS as u64);
                    if query_context {
                        features[16] = MemoryFeature {
                            kind: 16,
                            value: (u64::from(previous_query) << 32) | u64::from(last),
                        };
                        features[17] = MemoryFeature {
                            kind: 17,
                            value: pack_query_occurrence(
                                previous_query,
                                last,
                                query_distance,
                                source_distance,
                                offset,
                            ),
                        };
                    } else {
                        let previous_slot = if reference.slot == 0 {
                            self.ring.len() - 1
                        } else {
                            reference.slot - 1
                        };
                        let previous_prime =
                            model.geometry.tokens[self.ring[previous_slot].token as usize].prime;
                        features[16] = MemoryFeature {
                            kind: 16,
                            value: (u64::from(last) << 32) | u64::from(previous_prime),
                        };
                        features[17] = MemoryFeature {
                            kind: 17,
                            value: u64::from(previous_prime),
                        };
                    }
                    let mut score = i64::from(model.prior_scores[value.token as usize]);
                    for feature in features {
                        if !feature.admitted(control) {
                            continue;
                        }
                        work.memory_score_lookups = work.memory_score_lookups.saturating_add(1);
                        if let Ok(index) = memory
                            .rows
                            .binary_search_by_key(&feature, |row| row.feature)
                        {
                            score += i64::from(memory.rows[index].score);
                        }
                    }
                    self.candidates.push(MemoryCandidate {
                        token: value.token,
                        score,
                        features,
                    });
                    work.memory_candidates = work.memory_candidates.saturating_add(1);
                }
            }
        }
    }
}
