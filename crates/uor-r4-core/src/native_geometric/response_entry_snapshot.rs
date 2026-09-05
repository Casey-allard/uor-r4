//! Host-only restoration of a distinct response-boundary origin and its actual
//! observed path. Retained evidence is validated; evicted source truth is not
//! authenticated by this checkpoint.
use super::completion_types::CompletionWork;
use super::response_entry_runtime::eligible;
use super::response_entry_types::*;
use super::value_types::ValueWork;
use super::*;

fn invalid(message: &str) -> Error {
    Error(format!("session response entry {message}"))
}

pub(super) fn validate_field_presence(model: &Model, wire: &serde_json::Value) -> Result<()> {
    let field = wire.get("response_entry");
    if model.response_entry.is_some() {
        if !field.is_some_and(serde_json::Value::is_object) {
            return Err(invalid("state is required by this artifact"));
        }
    } else if field.is_some() {
        return Err(invalid("state is foreign to this artifact"));
    }
    if field.is_some_and(|state| state.get("pending").is_some()) {
        return Err(invalid("checkpoint contains a transient prediction"));
    }
    Ok(())
}

impl Session {
    /// Restore only after typed state and optional numeric completion state
    /// have been validated. Candidate workspace is never checkpoint data.
    pub(super) fn restore_response_entry_state(
        &mut self,
        model: &Model,
        saved: ResponseEntryState,
        retained: &[u32],
        observed: u64,
    ) -> Result<()> {
        if model.response_entry.is_none() {
            return Err(invalid("requires a response-entry artifact"));
        }
        let values = self
            .values
            .as_ref()
            .ok_or_else(|| invalid("requires restored typed state"))?;
        if saved.seen != observed
            || values.seen != observed
            || saved.pending.is_some()
            || saved.steps >= RESPONSE_ENTRY_STEPS
            || (saved.active && (saved.boundary.is_none() || saved.steps == 0))
            || (saved.active && saved.steps == 1 && saved.last_action != ResponseEntryAction::Enter)
            || (!saved.active && saved.steps != 0)
            || (!saved.active
                && matches!(
                    saved.last_action,
                    ResponseEntryAction::Enter | ResponseEntryAction::Emit
                ))
            || (saved.last_action == ResponseEntryAction::Enter && saved.steps != 1)
            || (saved.last_action == ResponseEntryAction::Emit && saved.steps <= 1)
            || (saved.last_action == ResponseEntryAction::Stop && saved.last != EOS)
            || saved.last as usize >= model.vocabulary_size()
            || saved.previous as usize >= model.vocabulary_size()
            || observed < retained.len() as u64
            || values.recent_len != observed.min(32) as usize
        {
            return Err(invalid("shape, action or observation counters are invalid"));
        }
        let recent = |sequence: u64| {
            (sequence < observed && observed - sequence <= values.recent_len as u64)
                .then(|| values.recent[(sequence & 31) as usize])
        };
        let token_at = |sequence: u64| -> Result<u32> {
            let entry =
                recent(sequence).ok_or_else(|| invalid("actual token history is absent"))?;
            if entry.sequence != sequence || entry.token as usize >= model.vocabulary_size() {
                return Err(invalid("actual token history is invalid"));
            }
            let oldest = observed - retained.len() as u64;
            if sequence >= oldest && retained[(sequence - oldest) as usize] != entry.token {
                return Err(invalid(
                    "actual token history differs from the session window",
                ));
            }
            Ok(entry.token)
        };
        if observed == 0 {
            if saved.last != BOS
                || saved.previous != BOS
                || saved.last_action != ResponseEntryAction::Base
                || saved.boundary.is_some()
            {
                return Err(invalid("empty observation state is invalid"));
            }
        } else if saved.last != token_at(observed - 1)?
            || saved.previous
                != if observed > 1 {
                    token_at(observed - 2)?
                } else {
                    BOS
                }
        {
            return Err(invalid("last tokens differ from actual observations"));
        }
        if let Some(anchor) = saved.boundary {
            if !eligible(values, self.control)
                || values.pending.is_some()
                || anchor.at_seen == 0
                || anchor.at_seen != values.started_at
                || anchor.at_seen > observed
                || observed - anchor.at_seen != u64::from(saved.steps)
                || usize::from(anchor.pose) >= model.geometry.inverses.len()
                || anchor.query_prime != values.queries[0].cue
                || anchor.pose != values.queries[0].pose
                || anchor.phases != values.queries[0].phases
                || values.queries[0].sequence.checked_add(1) != Some(anchor.at_seen)
                || (saved.active && saved.last == EOS)
                || (!saved.active && saved.last_action != ResponseEntryAction::Base)
                || self.completion.as_ref().is_some_and(|state| state.active)
            {
                return Err(invalid("query origin or response boundary is invalid"));
            }
            let endpoint = recent(anchor.at_seen - 1)
                .ok_or_else(|| invalid("boundary endpoint evidence is absent"))?;
            if endpoint.pose != anchor.pose || endpoint.phases != anchor.phases {
                return Err(invalid(
                    "boundary frame differs from actual endpoint evidence",
                ));
            }
            let mut pose = anchor.pose;
            let mut phases = anchor.phases;
            for sequence in anchor.at_seen..observed {
                let token = token_at(sequence)?;
                if token == EOS {
                    return Err(invalid("active response crosses an observed end token"));
                }
                let geometry = &model.geometry.tokens[token as usize];
                pose = model.geometry.products
                    [model.geometry.row_bases[usize::from(pose)] + usize::from(geometry.leaf)];
                for (phase, delta) in phases.iter_mut().zip(geometry.phases) {
                    *phase = phase.wrapping_add(delta);
                }
                let actual = recent(sequence).ok_or_else(|| invalid("response frame is absent"))?;
                if pose != actual.pose || phases != actual.phases {
                    return Err(invalid(
                        "actual response frame differs from retained evidence",
                    ));
                }
            }
            if pose != values.pose || phases != values.phases {
                return Err(invalid(
                    "current frame differs from actual response history",
                ));
            }
            if saved.active {
                // Reconstruct only the first selector from its saved query and
                // source state. Host restoration may allocate this clone and
                // execute the bounded typed gate; none is steady-state work.
                // The future/actual response never supplies the entry score.
                let mut boundary_values = values.clone();
                boundary_values.seen = anchor.at_seen;
                boundary_values.pose = anchor.pose;
                boundary_values.phases = anchor.phases;
                boundary_values.pending = None;
                let baseline = Candidate {
                    token: BOS,
                    score: 0,
                };
                if boundary_values
                    .offer(model, baseline, self.control, &mut ValueWork::default())
                    .is_some()
                {
                    return Err(invalid("origin is superseded by a typed value selection"));
                }
                let mut origin = ResponseEntryState {
                    boundary: Some(anchor),
                    last: values.queries[0].token,
                    previous: if values.query_len > 1 {
                        values.queries[1].token
                    } else {
                        BOS
                    },
                    seen: anchor.at_seen,
                    ..ResponseEntryState::default()
                };
                let selection = origin.offer(
                    model,
                    &boundary_values,
                    baseline,
                    self.control,
                    &mut CompletionWork::default(),
                );
                if !selection.is_some_and(|candidate| {
                    token_at(anchor.at_seen).is_ok_and(|token| token == candidate.token)
                }) || !origin
                    .pending
                    .is_some_and(|decision| decision.action == ResponseEntryAction::Enter)
                {
                    return Err(invalid("origin is not an observed selected entry"));
                }
            }
        }
        self.response_entry = Some(saved);
        Ok(())
    }
}
