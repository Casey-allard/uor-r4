//! Bind a requested whole word to earlier assertions, then agree on completed copies.
use super::{ExactCopyState, RolePolicy, ROLE_COPY_STOPS};
use crate::{
    assertion_value,
    content_selector::{Choice, WIDTH},
    invalid,
    paired_role::PairedRoleState,
    span_selector::{bounds, SpanChoice, START},
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub(super) struct Policy {
    pub(super) roles: PairedRoleState,
    pub(super) delimiters: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Header {
    pub(super) bank_bytes: usize,
    pub(super) bank_sha256: String,
    pub(super) delimiters: Vec<u32>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PairedCopyDecision {
    pub choice: Choice,
    /// Complete role output, including both endpoints of requested and excluded spans.
    pub role_words: Option<[u32; 8]>,
    pub supporting_facts: u32,
    pub complete_values: u32,
    pub values_disagree: bool,
}

impl Policy {
    pub(super) fn choose(
        &self,
        old: &RolePolicy,
        ids: &[u32],
        roots: &[u32],
    ) -> Result<PairedCopyDecision> {
        if ids.is_empty() || ids.len() > WIDTH || ids.iter().any(|&id| id >= 4096) {
            return Err(invalid("paired reader requires a bounded V4096 prefix"));
        }
        let roles = self.roles.predict(ids)?;
        let mut result = PairedCopyDecision {
            role_words: Some(roles.words()),
            ..Default::default()
        };
        let Some(query) = roles.requested else {
            return Ok(result);
        };
        let q = query.start;
        result.choice.query = Some(q as u32);
        if q >= ids.len() || query.end <= q || query.end > ids.len() {
            return Err(invalid("invalid requested span from paired reader"));
        }
        // The training bank supplies the admissible preceding fact delimiters.
        // Original word IDs determine equality, including every continuation piece.
        let mut first = None;
        let mut agreed = None;
        for f in 1..q.saturating_sub(1) {
            if old.span.classes[ids[f] as usize] != START || !self.delimiters.contains(&ids[f - 1])
            {
                continue;
            }
            let (_, last) = bounds(ids, &old.span.classes, f);
            if last >= q || ids[f..=last] != ids[q..query.end] {
                continue;
            }
            result.supporting_facts += 1;
            let span = SpanChoice {
                query: Some(q as u32),
                fact: Some(f as u32),
                query_start: Some(q as u32),
                query_end: Some((query.end - 1) as u32),
                fact_start: Some(f as u32),
                fact_end: Some(last as u32),
                source: None,
            };
            let value =
                assertion_value::decide(ids, roots, &old.span, &old.exact, span, ROLE_COPY_STOPS)?;
            let Some(source) = value.source else { continue };
            let Some(sequence) = completed_sequence(ids, source)? else {
                continue;
            };
            result.complete_values += 1;
            match agreed {
                None => {
                    first = Some((f as u32, source));
                    agreed = Some(sequence);
                }
                Some(reference) if reference != sequence => result.values_disagree = true,
                _ => (),
            }
        }
        if result.complete_values == result.supporting_facts && !result.values_disagree {
            if let Some((fact, source)) = first {
                result.choice.fact = Some(fact);
                result.choice.source = Some(source);
            }
        }
        Ok(result)
    }
}

/// The exact cursor supplies all accepted IDs. A transition to ordinary prediction
/// before completion invalidates that candidate, independent of the fallback token.
fn completed_sequence(ids: &[u32], source: u32) -> Result<Option<&[u32]>> {
    let mut cursor = ExactCopyState::new(Some(source), ids.len() as u32);
    loop {
        let step = cursor
            .write(ids, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        if step.copied == 0 {
            return Ok(step
                .completed_span()
                .then(|| &ids[source as usize..cursor.cursor as usize]));
        }
        cursor = step.state;
    }
}
