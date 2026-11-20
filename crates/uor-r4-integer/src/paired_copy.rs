//! Bind a requested whole word to earlier assertions, then agree on completed copies.
use super::{ExactCopyState, RolePolicy, ROLE_COPY_STOPS};
use crate::{
    assertion_value,
    content_selector::{Choice, WIDTH},
    invalid,
    ordered_role::{RoleBank, RoleFraming},
    paired_role::{PairedRoleDecision, PairedRoleState, RoleSpan},
    span_selector::{bounds, SpanChoice, START},
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub(super) struct Policy {
    pub(super) roles: Roles,
    pub(super) delimiters: Vec<u32>,
}

#[derive(Clone, Debug)]
pub(super) enum Roles {
    Frozen(PairedRoleState),
    Ordered {
        bank: RoleBank,
        framing: RoleFraming,
        classes: Vec<u8>,
        packed_roots: Vec<u32>,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OrderedHeader {
    pub(super) train_sha256: String,
    pub(super) framing: RoleFraming,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Header {
    pub(super) bank_bytes: usize,
    pub(super) bank_sha256: String,
    pub(super) delimiters: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) ordered: Option<OrderedHeader>,
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

#[derive(Clone, Copy, Debug, Serialize)]
pub struct PairedAssertionWitness {
    pub fact_span: [usize; 2],
    pub assertion: assertion_value::AssertionValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onset: Option<crate::all_starts::OnsetDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_source: Option<super::contextual_source::SourceSelection>,
    pub completed_source_range: Option<[u32; 2]>,
}

impl Policy {
    pub(super) fn validate_ids(ids: &[u32]) -> Result<()> {
        if ids.is_empty() || ids.len() > WIDTH || ids.iter().any(|&id| id >= 4096) {
            return Err(invalid("paired reader requires a bounded V4096 prefix"));
        }
        Ok(())
    }

    pub(super) fn assertion_sites<'a>(
        &'a self,
        old: &'a RolePolicy,
        ids: &'a [u32],
        query: RoleSpan,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        (1..query.start.saturating_sub(1)).filter_map(move |f| {
            if old.span.classes[ids[f] as usize] != START || !self.delimiters.contains(&ids[f - 1])
            {
                return None;
            }
            let (_, last) = bounds(ids, &old.span.classes, f);
            (last < query.start && ids[f..=last] == ids[query.start..query.end])
                .then_some((f, last))
        })
    }

    pub(super) fn choose_with_roles(
        &self,
        old: &RolePolicy,
        ids: &[u32],
        roots: &[u32],
        roles: PairedRoleDecision,
        selected: Option<&[RoleSpan]>,
        detached_heads: Option<&[[usize; 2]]>,
        mut visit: impl FnMut(PairedAssertionWitness),
        mut contextual_read: impl FnMut(
            RoleSpan,
            RoleSpan,
        ) -> Result<
            Option<(Option<u32>, super::contextual_source::SourceSelection)>,
        >,
    ) -> Result<PairedCopyDecision> {
        Self::validate_ids(ids)?;
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
        let inherited = detached_heads
            .is_none()
            .then(|| self.assertion_sites(old, ids, query))
            .into_iter()
            .flatten();
        let detached = detached_heads
            .into_iter()
            .flatten()
            .map(|&[start, end]| (start, end - 1));
        for (f, last) in inherited.chain(detached) {
            let fact = RoleSpan {
                start: f,
                end: last + 1,
            };
            if selected.is_some_and(|heads| !heads.contains(&fact)) {
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
            let (value, onset, contextual_source) =
                if let Some((fence, decision)) = contextual_read(fact, query)? {
                    (
                        assertion_value::AssertionValue {
                            fence,
                            source: decision.source,
                            ..Default::default()
                        },
                        None,
                        Some(decision),
                    )
                } else {
                    let mut value = assertion_value::decide(
                        ids,
                        roots,
                        &old.span,
                        &old.exact,
                        span,
                        ROLE_COPY_STOPS,
                    )?;
                    let onset = old
                        .onset
                        .as_ref()
                        .map(|policy| {
                            policy
                                .bank
                                .choose(ids, roots, &old.span.classes, last, q, value)
                        })
                        .transpose()?;
                    if let Some(decision) = onset {
                        value.source = decision.source;
                    }
                    (value, onset, None)
                };
            let completion = match value.source {
                Some(source) => completed_sequence(ids, source)?.map(|sequence| (source, sequence)),
                None => None,
            };
            visit(PairedAssertionWitness {
                fact_span: [f, last + 1],
                assertion: value,
                onset,
                contextual_source,
                completed_source_range: completion
                    .map(|(source, sequence)| [source, source + sequence.len() as u32]),
            });
            let Some((source, sequence)) = completion else {
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
pub(super) fn completed_sequence(ids: &[u32], source: u32) -> Result<Option<&[u32]>> {
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
