//! Host validation of copy provenance against the captured source and the
//! actual retained output prefix. This is consistency, not authentication of
//! source material that predates the retained ordinary token window.
use super::response_entry_types::*;
use super::word_copy_types::*;
use super::*;

fn invalid(message: &str) -> Error {
    Error(format!("session word copy {message}"))
}

pub(super) fn validate_field_presence(model: &Model, wire: &serde_json::Value) -> Result<()> {
    let field = wire.get("word_copy");
    let present = model
        .response_entry
        .as_ref()
        .is_some_and(|entry| entry.copy.is_some());
    if present {
        if !field.is_some_and(serde_json::Value::is_object) {
            return Err(invalid("state is required by this artifact"));
        }
    } else if field.is_some() {
        return Err(invalid("state is absent from this artifact"));
    }
    if field.is_some_and(|state| state.get("pending").is_some()) {
        return Err(invalid("contains a transient prediction"));
    }
    Ok(())
}

impl Session {
    pub(super) fn restore_word_copy_state(
        &mut self,
        model: &Model,
        saved: WordCopyState,
    ) -> Result<()> {
        let entry = self
            .response_entry
            .as_ref()
            .ok_or_else(|| invalid("requires restored entry"))?;
        let values = self
            .values
            .as_ref()
            .ok_or_else(|| invalid("requires restored typed state"))?;
        let words = values
            .lexemes
            .as_ref()
            .ok_or_else(|| invalid("requires captured words"))?;
        if saved.pending.is_some()
            || saved.origin.is_none() != (saved.progress == WordCopyProgress::Idle)
        {
            return Err(invalid("origin and progress shape differ"));
        }
        if !entry.active {
            if saved != WordCopyState::default() {
                return Err(invalid("inactive entry retains an origin"));
            }
            self.word_copy = Some(saved);
            return Ok(());
        }
        let anchor = entry
            .boundary
            .ok_or_else(|| invalid("active entry has no boundary"))?;
        let token_at = |sequence: u64| -> Result<u32> {
            if sequence >= values.seen || values.seen - sequence > values.recent_len as u64 {
                return Err(invalid("actual output evidence is absent"));
            }
            let item = values.recent[(sequence & 31) as usize];
            if item.sequence != sequence {
                return Err(invalid("actual output sequence differs"));
            }
            Ok(item.token)
        };
        // Recreate the actual first combined choice independently of the
        // ordinary total score: every head increment shares that same Base.
        let mut boundary_values = values.clone();
        boundary_values.seen = anchor.at_seen;
        boundary_values.pose = anchor.pose;
        boundary_values.phases = anchor.phases;
        boundary_values.pending = None;
        let mut initial = ResponseEntryState {
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
        let baseline = Candidate {
            token: BOS,
            score: 0,
        };
        if (!super::word_copy_runtime::composed(model) && saved.start_step != 0)
            || saved.start_step >= entry.steps
            || (saved.origin.is_none() && saved.start_step != 0)
        {
            return Err(invalid("copy start is outside the observed entry"));
        }
        let mut choice = WordCopyState::default();
        for step in 0..=saved.start_step {
            let inherited = initial.offer(
                model,
                &boundary_values,
                baseline,
                self.control,
                &mut CompletionWork::default(),
            );
            let selected = choice.offer(
                model,
                &mut initial,
                &boundary_values,
                baseline,
                inherited,
                self.control,
                &mut WordCopyWork::default(),
            );
            let actual = token_at(anchor.at_seen + u64::from(step))?;
            if selected.is_none_or(|candidate| candidate.token != actual) {
                return Err(invalid(
                    "start differs from actual selected lexical/copy prefix",
                ));
            }
            if step == saved.start_step {
                if choice.pending.map(|decision| decision.word_index) != saved.origin {
                    return Err(invalid("origin differs from observed copy selection"));
                }
            } else {
                if choice.pending.is_some() {
                    return Err(invalid("copy began before saved start"));
                }
                boundary_values.seen += 1;
                let observed = values.recent[((boundary_values.seen - 1) & 31) as usize];
                boundary_values.pose = observed.pose;
                boundary_values.phases = observed.phases;
                initial.observe(
                    model,
                    &boundary_values,
                    actual,
                    self.control,
                    &mut CompletionWork::default(),
                );
                choice.observe(
                    &initial,
                    &boundary_values,
                    actual,
                    &mut WordCopyWork::default(),
                );
            }
        }
        if let Some(index) = saved.origin {
            if !super::word_copy_runtime::enabled(self.control)
                || usize::from(index) >= words.query_len
            {
                return Err(invalid("occurrence is outside the captured source"));
            }
            let word = words.queries[usize::from(index)];
            if word.len == 0 || word.len >= RESPONSE_ENTRY_STEPS {
                return Err(invalid("word exceeds complete-response admission"));
            }
            let prefix = match saved.progress {
                WordCopyProgress::Emitting { cursor }
                    if cursor > 0
                        && cursor < word.len
                        && Some(cursor) == entry.steps.checked_sub(saved.start_step) =>
                {
                    usize::from(cursor)
                }
                WordCopyProgress::Complete
                    if entry.steps.saturating_sub(saved.start_step) >= word.len =>
                {
                    usize::from(word.len)
                }
                // Omission of predict before a byte can abort even if its
                // value agrees; checkpoint consistency cannot prove a call.
                WordCopyProgress::Aborted
                    if word.len > 1 && entry.steps.saturating_sub(saved.start_step) >= 2 =>
                {
                    1
                }
                _ => return Err(invalid("progress is inconsistent with actual observations")),
            };
            for offset in 0..prefix {
                if token_at(anchor.at_seen + u64::from(saved.start_step) + offset as u64)?
                    != u32::from(word.bytes[offset]) + 2
                {
                    return Err(invalid(
                        "observed byte prefix differs from selected occurrence",
                    ));
                }
            }
        }
        self.word_copy = Some(saved);
        Ok(())
    }
}
