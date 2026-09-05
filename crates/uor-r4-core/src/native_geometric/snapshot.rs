//! Host-only, artifact-bound persistence of a bounded conversation context.

use super::memory_types::{MemoryEntry, MemoryReference, ResponseAction, RESPONSE_MEMORY_SCHEMA};
use super::value_types::ValueState;
use super::*;
use serde::{Deserialize, Serialize};

const LEGACY_SESSION_SCHEMA: &str = "uor-r4.native-geometric-session/1";
const RESPONSE_SESSION_SCHEMA: &str = "uor-r4.native-geometric-session/2";
const VALUE_SESSION_SCHEMA: &str = "uor-r4.native-geometric-session/3";
const LEGACY_CHECKPOINT_LIMIT: usize = 1024 * 1024;
const RESPONSE_CHECKPOINT_LIMIT: usize = 8 * 1024 * 1024;
// Bound allocation before collecting the sparse index. A tuple has two
// unsigned integers; 40 bytes conservatively covers its JSON representation.
const SNAPSHOT_INDEX_LIMIT: usize = RESPONSE_CHECKPOINT_LIMIT / 40;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemoryOrigin {
    seen: u64,
    pose: u16,
    phases: [u16; PHASE_CHANNELS],
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseSnapshot {
    active: bool,
    started_at: u64,
    steps: u64,
    query_pose: u16,
    query_phases: [u16; PHASE_CHANNELS],
    queries: Vec<MemoryEntry>,
    // Slot numbers are an implementation detail of the circular buffer.
    // Absolute sequence identities survive replay into a different cursor.
    references: Vec<u64>,
    selected: Option<u64>,
    last_action: ResponseAction,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseMemorySnapshot {
    origin: MemoryOrigin,
    current: MemoryOrigin,
    // Include stale entries: occupied/stale and empty have different work
    // counters even when neither can offer a candidate.
    index: Vec<(usize, u64)>,
    response: ResponseSnapshot,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    schema: String,
    artifact_cid: String,
    retained_tokens: Vec<u32>,
    previous_evicted: Option<u32>,
    previous_h4: u16,
    h4: u16,
    phases: [u16; PHASE_CHANNELS],
    control: Control,
    work: Work,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response_memory: Option<ResponseMemorySnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    values: Option<ValueState>,
}

impl Session {
    /// Persist the ordered retained window; scored candidates are transient
    /// and are recomputed by the next predict call.
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        let retained_tokens = if self.length == self.ring.len() {
            self.ring[self.cursor..]
                .iter()
                .chain(&self.ring[..self.cursor])
                .copied()
                .collect()
        } else {
            self.ring[..self.length].to_vec()
        };
        let response_memory = self.response_memory_snapshot()?;
        let schema = if self.values.is_some() {
            VALUE_SESSION_SCHEMA
        } else if response_memory.is_some() {
            RESPONSE_SESSION_SCHEMA
        } else {
            LEGACY_SESSION_SCHEMA
        };
        let bytes = serde_json::to_vec(&Checkpoint {
            schema: schema.into(),
            artifact_cid: self.artifact_cid.clone(),
            retained_tokens,
            previous_evicted: self.previous_evicted,
            previous_h4: self.previous_h4,
            h4: self.h4,
            phases: self.phases,
            control: self.control,
            work: self.work,
            response_memory,
            values: self.values.clone(),
        })
        .map_err(|error| Error(error.to_string()))?;
        let limit = if schema != LEGACY_SESSION_SCHEMA {
            RESPONSE_CHECKPOINT_LIMIT
        } else {
            LEGACY_CHECKPOINT_LIMIT
        };
        if bytes.len() > limit {
            return Err(Error(
                "session checkpoint exceeds its format byte limit".into(),
            ));
        }
        Ok(bytes)
    }

    /// Rebuild the exact current geometric state from the ordered stored
    /// window. The separately retained evicted token verifies the preceding
    /// H4 trajectory state even after the ring filled.
    pub fn from_checkpoint(model: &Model, bytes: &[u8]) -> Result<Self> {
        if bytes.len() > RESPONSE_CHECKPOINT_LIMIT {
            return Err(Error("session checkpoint exceeds 8 MiB".into()));
        }
        let checkpoint: Checkpoint =
            serde_json::from_slice(bytes).map_err(|error| Error(error.to_string()))?;
        let response_model = model
            .memory_read
            .as_ref()
            .is_some_and(|memory| memory.schema == RESPONSE_MEMORY_SCHEMA);
        let expected_schema = if model.values.is_some() {
            VALUE_SESSION_SCHEMA
        } else if response_model {
            RESPONSE_SESSION_SCHEMA
        } else {
            LEGACY_SESSION_SCHEMA
        };
        let object: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|error| Error(error.to_string()))?;
        value_snapshot::validate_lexeme_field_presence(model, &object)?;
        // The additive /2 field must remain an unknown field for historical
        // /1 inputs, including an explicit JSON null. Keep their old loader
        // law as well as their byte serialization unchanged.
        if checkpoint.schema == LEGACY_SESSION_SCHEMA {
            if object.get("response_memory").is_some() {
                return Err(Error("historical session contains response state".into()));
            }
        }
        if checkpoint.schema != VALUE_SESSION_SCHEMA && object.get("values").is_some() {
            return Err(Error(
                "historical session contains typed value state".into(),
            ));
        }
        if object
            .get("values")
            .and_then(|value| value.get("pending"))
            .is_some()
        {
            return Err(Error(
                "session checkpoint contains transient value selection".into(),
            ));
        }
        if checkpoint.schema != expected_schema
            || checkpoint.response_memory.is_some() != response_model
            || checkpoint.values.is_some() != model.values.is_some()
            || (checkpoint.schema == LEGACY_SESSION_SCHEMA && bytes.len() > LEGACY_CHECKPOINT_LIMIT)
            || checkpoint.artifact_cid != model.artifact_cid()
            || checkpoint.retained_tokens.len() > model.config().context_tokens
            || checkpoint.work.observed_tokens < checkpoint.retained_tokens.len() as u64
            || checkpoint.retained_tokens.len()
                != (checkpoint
                    .work
                    .observed_tokens
                    .min(model.config().context_tokens as u64) as usize)
            || checkpoint.work.evictions
                != checkpoint
                    .work
                    .observed_tokens
                    .saturating_sub(model.config().context_tokens as u64)
            || checkpoint.previous_evicted.is_some() != (checkpoint.work.evictions > 0)
        {
            return Err(Error(
                "session checkpoint identity/window counters are invalid".into(),
            ));
        }
        let mut restored = model.session(checkpoint.control)?;
        if let Some(memory_snapshot) = &checkpoint.response_memory {
            let origin = &memory_snapshot.origin;
            let current = &memory_snapshot.current;
            if usize::from(origin.pose) >= model.geometry.inverses.len()
                || usize::from(current.pose) >= model.geometry.inverses.len()
                || current.seen != checkpoint.work.observed_tokens
                || origin.seen != current.seen - checkpoint.retained_tokens.len() as u64
                || (origin.seen == 0
                    && (origin.pose != model.geometry.identity
                        || origin.phases != [0; PHASE_CHANNELS]))
            {
                return Err(Error("session absolute memory origin is invalid".into()));
            }
            let memory = restored
                .memory
                .as_mut()
                .ok_or_else(|| Error("response checkpoint requires a memory model".into()))?;
            memory.seen = origin.seen;
            memory.pose = origin.pose;
            memory.phases = origin.phases;
            memory.origin_pose = origin.pose;
            memory.origin_phases = origin.phases;
        }
        for &token in &checkpoint.retained_tokens {
            restored.observe(model, token)?;
        }
        let product = |left: u16, right: u16| {
            model.geometry.products
                [model.geometry.row_bases[usize::from(left)] + usize::from(right)]
        };
        let expected_previous = if let Some(&last) = checkpoint.retained_tokens.last() {
            let mut previous = product(
                restored.h4,
                model.geometry.inverses[usize::from(model.geometry.tokens[last as usize].leaf)],
            );
            if let Some(evicted) = checkpoint.previous_evicted {
                let old = model
                    .geometry
                    .tokens
                    .get(evicted as usize)
                    .ok_or_else(|| Error("session evicted token is invalid".into()))?;
                previous = product(old.leaf, previous);
            }
            previous
        } else {
            model.geometry.identity
        };
        if checkpoint.h4 != restored.h4
            || checkpoint.phases != restored.phases
            || checkpoint.previous_h4 != expected_previous
        {
            return Err(Error(
                "session geometric state does not match its ordered token window".into(),
            ));
        }
        restored.previous_h4 = expected_previous;
        restored.previous_evicted = checkpoint.previous_evicted;
        if let Some(memory_snapshot) = checkpoint.response_memory {
            restored.restore_response_memory(model, memory_snapshot)?;
        }
        if let Some(values) = checkpoint.values {
            restored.restore_value_state(
                model,
                values,
                &checkpoint.retained_tokens,
                checkpoint.work.observed_tokens,
                checkpoint.work.values.input_bytes,
            )?;
        }
        restored.work = checkpoint.work;
        Ok(restored)
    }

    fn response_memory_snapshot(&self) -> Result<Option<ResponseMemorySnapshot>> {
        let Some(memory) = &self.memory else {
            return Ok(None);
        };
        let Some(response) = &memory.response else {
            return Ok(None);
        };
        let occupied = memory
            .index
            .iter()
            .filter(|entry| entry.sequence != u64::MAX)
            .count();
        if occupied > SNAPSHOT_INDEX_LIMIT {
            return Err(Error(
                "session memory index exceeds checkpoint capacity".into(),
            ));
        }
        // Derive the prefix origin without needing any evicted token bytes.
        // The final cumulative pose is origin * ordered retained window.
        // Its inverse cannot be computed here without the model, so retain
        // the oldest entry's pre-token origin during observation instead.
        let origin = MemoryOrigin {
            seen: memory.seen - memory.length as u64,
            pose: memory.origin_pose,
            phases: memory.origin_phases,
        };
        Ok(Some(ResponseMemorySnapshot {
            origin,
            current: MemoryOrigin {
                seen: memory.seen,
                pose: memory.pose,
                phases: memory.phases,
            },
            index: memory
                .index
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.sequence != u64::MAX)
                .map(|(address, entry)| (address, entry.sequence))
                .collect(),
            response: ResponseSnapshot {
                active: response.active,
                started_at: response.started_at,
                steps: response.steps,
                query_pose: response.query_pose,
                query_phases: response.query_phases,
                queries: response.queries.clone(),
                references: response
                    .references
                    .iter()
                    .map(|entry| entry.sequence)
                    .collect(),
                selected: response.selected.map(|entry| entry.sequence),
                last_action: response.last_action,
            },
        }))
    }

    fn restore_response_memory(
        &mut self,
        model: &Model,
        saved: ResponseMemorySnapshot,
    ) -> Result<()> {
        let Some(memory_model) = &model.memory_read else {
            return Err(Error("response checkpoint requires a memory model".into()));
        };
        let Some(memory) = &mut self.memory else {
            return Err(Error("response checkpoint requires memory state".into()));
        };
        if memory.seen != saved.current.seen
            || memory.pose != saved.current.pose
            || memory.phases != saved.current.phases
            || saved.index.len() > SNAPSHOT_INDEX_LIMIT
        {
            return Err(Error(
                "session memory state differs from its retained replay".into(),
            ));
        }
        memory.origin_pose = saved.origin.pose;
        memory.origin_phases = saved.origin.phases;
        let oldest = saved.origin.seen;
        let reference = |sequence: u64| -> Result<MemoryReference> {
            if sequence != u64::MAX && sequence >= memory.seen {
                return Err(Error("session memory reference is in the future".into()));
            }
            let slot = if sequence == u64::MAX || sequence < oldest {
                0
            } else {
                usize::try_from(sequence - oldest)
                    .map_err(|_| Error("session memory reference is outside the window".into()))?
            };
            Ok(MemoryReference { sequence, slot })
        };
        let mut previous_address = None;
        let mut previous_sequence = 0;
        for &(address, sequence) in &saved.index {
            if address >= memory.index.len()
                || sequence == u64::MAX
                || previous_address.is_some_and(|previous| previous >= address)
            {
                return Err(Error("session sparse memory index is invalid".into()));
            }
            let source_distance = ((address >> memory_model.posting_shift)
                & ((1usize << memory_model.source_shift) - 1))
                + 1;
            let posting_rank = address & ((1usize << memory_model.posting_shift) - 1);
            let cue = address >> memory_model.posting_shift >> memory_model.source_shift;
            let same_posting_row = previous_address.is_some_and(|previous| {
                previous >> memory_model.posting_shift == address >> memory_model.posting_shift
            });
            if source_distance > memory_model.config.source_offsets
                || posting_rank >= memory_model.config.postings_per_address
                || (same_posting_row
                    && (previous_address != address.checked_sub(1)
                        || previous_sequence <= sequence))
                || (!same_posting_row && posting_rank != 0)
                || memory_model
                    .cue_aliases
                    .as_ref()
                    .is_some_and(|aliases| aliases.representatives[cue] as usize != cue)
            {
                return Err(Error("session memory index address is invalid".into()));
            }
            previous_address = Some(address);
            previous_sequence = sequence;
            if memory.index[address].sequence != u64::MAX
                && memory.index[address].sequence != sequence
            {
                return Err(Error(
                    "session memory index differs from retained evidence".into(),
                ));
            }
            let entry = reference(sequence)?;
            if let Some(source_sequence) = sequence.checked_sub(source_distance as u64) {
                if source_sequence >= oldest {
                    let source = memory.ring[(source_sequence - oldest) as usize];
                    let actual_cue = memory_model
                        .cue_aliases
                        .as_ref()
                        .map_or(source.token, |aliases| {
                            aliases.representatives[source.token as usize]
                        });
                    if actual_cue as usize != cue {
                        return Err(Error(
                            "session memory index cue differs from retained tokens".into(),
                        ));
                    }
                }
            } else {
                return Err(Error("session memory index precedes its source cue".into()));
            }
            memory.index[address] = entry;
        }
        // Replay may have populated valid entries whose recorded presence was
        // maliciously omitted. Require the sparse wire to describe every
        // nonempty replay entry before replacing the full index below.
        for (address, entry) in memory.index.iter().enumerate() {
            if entry.sequence != u64::MAX
                && saved
                    .index
                    .binary_search_by_key(&address, |row| row.0)
                    .is_err()
            {
                return Err(Error(
                    "session sparse memory index omits retained evidence".into(),
                ));
            }
        }
        let response = saved.response;
        if usize::from(response.query_pose) >= model.geometry.inverses.len()
            || response.started_at > memory.seen
            || response.steps > memory.seen - response.started_at
            || response.queries.len() > memory_model.config.query_tokens
            || response.references.len() > memory_model.config.candidate_limit
            || (response.active
                && (response.steps != memory.seen - response.started_at
                    || response.last_action == ResponseAction::Stop
                    || matches!(
                        self.control,
                        Control::MemoryDisabled | Control::ResponseStateDisabled
                    )))
            || (matches!(
                self.control,
                Control::MemoryDisabled | Control::ResponseStateDisabled
            ) && (response.started_at != 0
                || response.steps != 0
                || !response.queries.is_empty()
                || !response.references.is_empty()
                || response.selected.is_some()
                || response.last_action != ResponseAction::Base))
            || (!response.active && response.selected.is_some())
            || response.queries.len()
                != response
                    .started_at
                    .min(memory_model.config.query_tokens.min(memory.ring.len()) as u64)
                    as usize
            || response.references.len()
                != memory_model.config.candidate_limit.min(
                    memory_model.config.postings_per_address
                        * memory_model.config.source_offsets
                        * response.queries.len(),
                )
            || response
                .selected
                .is_some_and(|sequence| sequence >= response.started_at || sequence < oldest)
            || (response.selected.is_some()
                && (response.steps == 0
                    || !matches!(
                        response.last_action,
                        ResponseAction::Continue | ResponseAction::Requery
                    )))
        {
            return Err(Error(
                "session response state shape or counters are invalid".into(),
            ));
        }
        for (offset, query) in response.queries.iter().enumerate() {
            if query.token as usize >= model.vocabulary_size()
                || usize::from(query.pose) >= model.geometry.inverses.len()
                || query.sequence >= response.started_at
                || query.sequence.checked_add(offset as u64 + 1) != Some(response.started_at)
            {
                return Err(Error("session response query entry is invalid".into()));
            }
            if query.sequence >= oldest && memory.ring[(query.sequence - oldest) as usize] != *query
            {
                return Err(Error(
                    "session response query differs from retained memory".into(),
                ));
            }
            if offset == 0 {
                if query.pose != response.query_pose || query.phases != response.query_phases {
                    return Err(Error("session response query endpoint differs".into()));
                }
            } else {
                let newer = response.queries[offset - 1];
                let geometry = &model.geometry.tokens[newer.token as usize];
                let expected = model.geometry.products[model.geometry.row_bases
                    [usize::from(query.pose)]
                    + usize::from(geometry.leaf)];
                if expected != newer.pose
                    || newer
                        .phases
                        .iter()
                        .zip(query.phases)
                        .zip(geometry.phases)
                        .any(|((&newer, older), delta)| older.wrapping_add(delta) != newer)
                {
                    return Err(Error("session response query path is inconsistent".into()));
                }
            }
        }
        for (visit, &sequence) in response.references.iter().enumerate() {
            if sequence != u64::MAX && sequence >= response.started_at {
                return Err(Error(
                    "session frozen response reference is not prior evidence".into(),
                ));
            }
            reference(sequence)?;
            if sequence == u64::MAX {
                continue;
            }
            // Capture order is posting rank, source distance, then query.
            // Bind every still-usable value to that query's actual cue;
            // otherwise an edited in-range sequence could widen admission.
            let query_count = response.queries.len();
            let source_distance = (visit / query_count) % memory_model.config.source_offsets + 1;
            let source_sequence =
                sequence
                    .checked_sub(source_distance as u64)
                    .ok_or_else(|| {
                        Error("session frozen response reference precedes its source cue".into())
                    })?;
            if source_sequence >= oldest {
                let source = memory.ring[(source_sequence - oldest) as usize];
                let query = response.queries[visit % query_count];
                let cue = |token: u32| {
                    memory_model
                        .cue_aliases
                        .as_ref()
                        .map_or(token, |aliases| aliases.representatives[token as usize])
                };
                if cue(source.token) != cue(query.token) {
                    return Err(Error(
                        "session frozen response cue differs from retained tokens".into(),
                    ));
                }
            }
        }
        let selected = response.selected.map(reference).transpose()?;
        if selected.is_some_and(|entry| {
            memory.ring[entry.slot].token != memory.ring[memory.length - 1].token
        }) {
            return Err(Error(
                "session selected response token differs from last observation".into(),
            ));
        }
        let Some(state) = &mut memory.response else {
            return Err(Error(
                "response checkpoint requires response model state".into(),
            ));
        };
        state.active = response.active;
        state.started_at = response.started_at;
        state.steps = response.steps;
        state.query_pose = response.query_pose;
        state.query_phases = response.query_phases;
        state.queries.clear();
        state.queries.extend(response.queries);
        state.references.clear();
        for sequence in response.references {
            state.references.push(reference(sequence)?);
        }
        state.selected = selected;
        state.last_action = response.last_action;
        memory.pending = None;
        Ok(())
    }
}

impl Model {
    pub fn restore_session(&self, bytes: &[u8]) -> Result<Session> {
        Session::from_checkpoint(self, bytes)
    }
}

#[cfg(test)]
#[path = "response_snapshot_tests.rs"]
mod response_snapshot_tests;

#[path = "value_snapshot.rs"]
mod value_snapshot;

#[cfg(test)]
#[path = "value_snapshot_tests.rs"]
mod value_snapshot_tests;
