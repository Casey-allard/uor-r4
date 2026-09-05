//! Host validation of observed post-numeral state. The anchor is derived from
//! a checked typed write, then compared with the retained actual token path.
//! This authenticates consistency with retained evidence, not evicted history.
use super::*;
use crate::native_geometric::completion_types::{CompletionAction, CompletionState};
use crate::native_geometric::numeral::Numeral;
use crate::prime_route_attention::ZPhi;

fn invalid(message: &str) -> Error {
    Error(format!("session value completion {message}"))
}

/// Presence is versioned separately from serde's optional-field default.
/// Historical models must reject even an explicit null completion field.
pub(super) fn validate_field_presence(model: &Model, wire: &serde_json::Value) -> Result<()> {
    let field = wire.get("completion");
    if model.completion.is_some() {
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
    /// Call only after restoring and validating ValueState. The completion
    /// candidate workspace is a fixed local array, so no serialized capacity
    /// or allocation is accepted from a checkpoint.
    pub(super) fn restore_completion_state(
        &mut self,
        model: &Model,
        saved: CompletionState,
        retained: &[u32],
        observed: u64,
    ) -> Result<()> {
        if model.completion.is_none() {
            return Err(invalid("requires a completion artifact"));
        }
        let values = self
            .values
            .as_ref()
            .ok_or_else(|| invalid("requires restored typed state"))?;
        if saved.seen != observed
            || values.seen != observed
            || saved.pending.is_some()
            || saved.active != saved.anchor.is_some()
            || saved.steps >= 32
            || (!saved.active && saved.steps != 0)
            || (saved.active && saved.steps == 0 && saved.last_action != CompletionAction::Base)
            || saved.last as usize >= model.vocabulary_size()
            || saved.previous as usize >= model.vocabulary_size()
            || (saved.last_action == CompletionAction::Stop && saved.last != EOS)
            || (saved.last_action == CompletionAction::Emit && saved.last == EOS)
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
                || saved.last_action != CompletionAction::Base
                || saved.active
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
        // Emit survives automatic deactivation only on the exact cap step.
        // A later source observation or explicit response reset clears it.
        if !saved.active && saved.last_action == CompletionAction::Emit {
            let record = values
                .records
                .last()
                .filter(|record| {
                    values.active
                        && values.consumed
                        && values.emission.is_none()
                        && record.derived
                        && record.start >= values.started_at
                })
                .ok_or_else(|| invalid("inactive emit lacks a capped typed response"))?;
            let numeral = Numeral::from_zphi(ZPhi::new(record.value, 0))
                .ok_or_else(|| invalid("capped write has no exact numeral"))?;
            if record
                .start
                .checked_add(u64::from(numeral.len))
                .and_then(|seen| seen.checked_add(32))
                != Some(observed)
            {
                return Err(invalid("inactive emit does not end at the completion cap"));
            }
        }
        if let Some(anchor) = saved.anchor {
            if !values.active
                || !values.consumed
                || values.emission.is_some()
                || values.query_len == 0
                || anchor.at_seen == 0
                || anchor.at_seen > observed
                || observed - anchor.at_seen != u64::from(saved.steps)
                || usize::from(anchor.pose) >= model.geometry.inverses.len()
                || anchor.query_prime != values.queries[0].cue
                || saved.last == EOS
                || matches!(
                    self.control,
                    Control::ValuesDisabled | Control::MemoryDisabled
                )
            {
                return Err(invalid("anchor or response boundary is invalid"));
            }
            let record = values
                .records
                .last()
                .filter(|record| {
                    record.id == anchor.write_id
                        && record.derived
                        && record.id.checked_add(1) == Some(values.next_id)
                        && record.start >= values.started_at
                        && record.start == record.end
                })
                .ok_or_else(|| invalid("anchor does not identify the current derived write"))?;
            let derivation = record
                .derivation
                .as_ref()
                .ok_or_else(|| invalid("anchor write lacks checked provenance"))?;
            if derivation.action != anchor.action
                || usize::from(record.pose) >= model.geometry.inverses.len()
            {
                return Err(invalid("anchor action or write frame is invalid"));
            }
            let numeral = Numeral::from_zphi(ZPhi::new(record.value, 0))
                .ok_or_else(|| invalid("anchor write has no exact numeral"))?;
            if record.start.checked_add(u64::from(numeral.len)) != Some(anchor.at_seen) {
                return Err(invalid(
                    "anchor precedes or exceeds completed numeral emission",
                ));
            }
            let advance = |pose: &mut u16, phases: &mut [u16; PHASE_CHANNELS], token: u32| {
                let geometry = &model.geometry.tokens[token as usize];
                *pose = model.geometry.products
                    [model.geometry.row_bases[usize::from(*pose)] + usize::from(geometry.leaf)];
                for (phase, delta) in phases.iter_mut().zip(geometry.phases) {
                    *phase = phase.wrapping_add(delta);
                }
            };
            // Derived record geometry is the state before its first observed
            // numeral byte. Reconstruct the complete, fixed spelling and check
            // every part of that spelling which remains in token evidence.
            let mut pose = record.pose;
            let mut phases = record.phases;
            for (offset, &token) in numeral.tokens[..usize::from(numeral.len)]
                .iter()
                .enumerate()
            {
                let sequence = record.start + offset as u64;
                if recent(sequence).is_some() && token_at(sequence)? != token {
                    return Err(invalid("anchor numeral differs from observed bytes"));
                }
                advance(&mut pose, &mut phases, token);
            }
            if pose != anchor.pose || phases != anchor.phases {
                return Err(invalid(
                    "anchor frame differs from the completed typed write",
                ));
            }
            // Active completion has fewer than 32 following observations. Its
            // endpoint and full actual suffix therefore fit in ValueState's
            // independently validated 32-entry token/geometry ring, even when
            // the ordinary context window is smaller.
            let endpoint = recent(anchor.at_seen - 1)
                .ok_or_else(|| invalid("anchor endpoint evidence is absent"))?;
            if endpoint.pose != anchor.pose || endpoint.phases != anchor.phases {
                return Err(invalid(
                    "anchor frame differs from retained endpoint evidence",
                ));
            }
            for sequence in anchor.at_seen..observed {
                let token = token_at(sequence)?;
                if token == EOS {
                    return Err(invalid("active suffix crosses an observed end token"));
                }
                advance(&mut pose, &mut phases, token);
                let entry =
                    recent(sequence).ok_or_else(|| invalid("suffix frame evidence is absent"))?;
                if pose != entry.pose || phases != entry.phases {
                    return Err(invalid(
                        "actual suffix frame differs from retained evidence",
                    ));
                }
            }
            if pose != values.pose || phases != values.phases {
                return Err(invalid("current frame differs from the actual suffix"));
            }
        }
        self.completion = Some(saved);
        Ok(())
    }
}
