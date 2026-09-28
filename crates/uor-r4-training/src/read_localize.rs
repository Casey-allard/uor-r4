//! Read-only localization instrument for the mainline joint learner.
//!
//! This module supplies the token-id-only mutation helpers, the canonical
//! greedy generation loop that reproduces the frozen story-probe stop rule,
//! and the decision-0 read-mass / mixture capture. It changes no read, score,
//! weight or default behavior: every function is additive and is only called
//! by the `joint-read-localize` example and its focused tests.
//!
//! The canonical loop exists because the public `generate` path uses
//! `first_sentence = false` while `judge_story` (which produced the retained
//! verdicts) uses `first_sentence = true`; `generate` therefore does not
//! reproduce the retained story probes. `run_story_probes` is public and does,
//! but it cannot take caller-built token ids or expose read masses, so the
//! exact private loop is reimplemented here and validated by a parity check
//! against the retained `story-probes.json` and by the retained decision-0
//! audit values.

use serde::Serialize;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::joint_evaluation::{
    token_components, validate_generation_tokens, JointGenerationStop, TokenComponents,
};
use crate::joint_model::{JointModel, JointSession, JointStep, ReadMassIntervention, ReadMode};
use crate::reference_eval::short_cycle_period;
use crate::{invalid, Result};

/// Non-entity concrete noun used by condition C (matched final control) and
/// condition D' (matched near-query clause). A single token (`Ġcarrot`, id
/// 1956), concrete, and absent from the 32 story entities and from the
/// canonical prompt vocabulary.
pub const CONTROL_NOUN: &str = "carrot";

/// Frozen story-probe generation horizon (`STORY_PROBE_MAX_NEW_TOKENS`).
pub const CANONICAL_MAX_NEW_TOKENS: usize = 32;

/// Tolerance for `sum(read masses) + NoRead == 1`.
pub const MASS_SUM_TOLERANCE: f64 = 1e-5;

/// Longest token window the exact entity-span scan will consider.
pub const MAX_ENTITY_SPAN_TOKENS: usize = 6;

/// A minimal token window whose decoded bytes carry an entity mention.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct EntitySpan {
    pub start: usize,
    pub len: usize,
}

/// One causally available read occurrence observed at decision 0.
#[derive(Clone, Debug, Serialize)]
pub struct ReadEventMass {
    pub occurrence: usize,
    pub token: u32,
    pub text: String,
    pub mass: f64,
}

/// Decision-0 read/state capture taken from the step that consumed the final
/// prompt token. The final prompt token is written by that same step, so it is
/// never a read candidate (`occurrence == written_occurrence`).
#[derive(Clone, Debug, Serialize)]
pub struct DecisionZeroCapture {
    pub written_occurrence: usize,
    pub final_prompt_token_occurrence: usize,
    pub final_prompt_token_is_readable: bool,
    pub readable_max_occurrence: Option<usize>,
    pub read_mass_sum: f64,
    pub no_read_mass: f64,
    pub copy_gate: f64,
    pub effective_copy_mass: f64,
    pub normalization_ok: bool,
    pub read_row: Vec<ReadEventMass>,
    pub top_read_occurrence: Option<usize>,
    pub top_read_token: Option<u32>,
    pub top_read_token_text: Option<String>,
    pub top_read_mass: f64,
    pub emitted_token: u32,
    pub emitted_token_text: String,
    pub emitted_components: TokenComponents,
    pub probe_token: Option<u32>,
    pub probe_components: Option<TokenComponents>,
}

/// Complete canonical greedy continuation of exact caller-supplied ids.
#[derive(Clone, Debug, Serialize)]
pub struct CanonicalRun {
    pub prompt_token_ids: Vec<u32>,
    pub generated_token_ids: Vec<u32>,
    pub raw_decoded: String,
    pub response_text: String,
    pub utf8_decodable: bool,
    pub stop: JointGenerationStop,
    pub decision_zero: DecisionZeroCapture,
}

fn strip_leading_ascii_whitespace(bytes: &[u8]) -> &[u8] {
    match bytes.iter().position(|byte| !byte.is_ascii_whitespace()) {
        Some(index) => &bytes[index..],
        None => &[],
    }
}

fn starts_with_ascii_whitespace(bytes: &[u8]) -> bool {
    bytes.first().is_some_and(u8::is_ascii_whitespace)
}

/// Every minimal token window of `content` whose decoded bytes, after
/// stripping leading ASCII whitespace, equal `noun`, with a word boundary
/// before the window. Windows are scanned shortest-first at each start so the
/// result is minimal on token length; the entity noun bytes are never
/// reconstructed by re-tokenizing a string.
pub fn entity_spans(
    content: &[u32],
    noun: &str,
    decode: impl Fn(&[u32]) -> Vec<u8>,
) -> Vec<EntitySpan> {
    let target = noun.as_bytes();
    let mut spans = Vec::new();
    for start in 0..content.len() {
        let limit = MAX_ENTITY_SPAN_TOKENS.min(content.len() - start);
        for len in 1..=limit {
            let raw = decode(&content[start..start + len]);
            if strip_leading_ascii_whitespace(&raw) != target {
                continue;
            }
            let prefix = decode(&content[..start]);
            let boundary = starts_with_ascii_whitespace(&raw)
                || prefix
                    .last()
                    .is_none_or(|byte| !byte.is_ascii_alphanumeric());
            if boundary {
                spans.push(EntitySpan { start, len });
                break;
            }
        }
    }
    spans
}

/// Exact occurrence index of a content span: BOS occupies occurrence 0, so a
/// content offset `start` is read/written at occurrence `1 + start`.
pub fn span_occurrence(span: EntitySpan) -> usize {
    1 + span.start
}

/// Largest token index `k` in `1..=content.len()` such that the decoded prefix
/// `content[..k]` ends with an ASCII period and either `k == content.len()` or
/// the next token's decoded bytes begin with ASCII whitespace. This is the
/// token boundary immediately after the final sentence break, used to insert
/// the D/D' clause without re-tokenizing the mutated prompt.
pub fn insert_index_before_final_query(
    content: &[u32],
    decode: impl Fn(&[u32]) -> Vec<u8>,
) -> Result<usize> {
    let mut candidate = None;
    for k in 1..=content.len() {
        if !decode(&content[..k]).ends_with(b".") {
            continue;
        }
        if k == content.len() || starts_with_ascii_whitespace(&decode(&content[k..k + 1])) {
            candidate = Some(k);
        }
    }
    candidate.ok_or_else(|| invalid("no final-query insertion boundary found"))
}

/// Replace the final content token with `replacement`.
pub fn splice_replace_final(content: &[u32], replacement: &[u32]) -> Result<Vec<u32>> {
    let keep = content
        .len()
        .checked_sub(1)
        .ok_or_else(|| invalid("cannot replace the final token of an empty prompt"))?;
    let mut out = Vec::with_capacity(keep + replacement.len());
    out.extend_from_slice(&content[..keep]);
    out.extend_from_slice(replacement);
    Ok(out)
}

/// Insert `clause` before content index `at`.
pub fn splice_insert(content: &[u32], at: usize, clause: &[u32]) -> Result<Vec<u32>> {
    if at > content.len() {
        return Err(invalid("insertion index is outside the prompt"));
    }
    let mut out = Vec::with_capacity(content.len() + clause.len());
    out.extend_from_slice(&content[..at]);
    out.extend_from_slice(clause);
    out.extend_from_slice(&content[at..]);
    Ok(out)
}

/// Signed change in prompt length produced by a mutation.
pub fn length_delta(original: usize, mutated: usize) -> i64 {
    mutated as i64 - original as i64
}

/// `sum(masses) + NoRead == 1` within `tolerance`. The instrument never
/// renormalizes; a mismatch is a measured failure of the loaded artifact.
pub fn masses_normalize(masses: &[f32], no_read: f64, tolerance: f64) -> bool {
    if !no_read.is_finite() || !(0.0..=1.0 + tolerance).contains(&no_read) {
        return false;
    }
    let mut sum = 0.0f64;
    for &mass in masses {
        let mass = f64::from(mass);
        if !mass.is_finite() || mass < 0.0 {
            return false;
        }
        sum += mass;
    }
    (sum + no_read - 1.0).abs() <= tolerance
}

/// Entity share of the non-NoRead read mass. `None` when NoRead is the whole
/// distribution (no read mass exists to divide).
pub fn entity_share(entity_mass: f64, no_read: f64) -> Option<f64> {
    let denominator = 1.0 - no_read;
    (denominator > 0.0).then(|| entity_mass / denominator)
}

/// Every recorded read occurrence must predate the current write.
pub fn positions_are_causal(read_occurrences: &[usize], written_occurrence: usize) -> bool {
    read_occurrences
        .iter()
        .all(|&position| position < written_occurrence)
}

fn argmax_strict(probabilities: &[f32]) -> Result<u32> {
    if probabilities.is_empty() {
        return Err(invalid("joint probability row is empty"));
    }
    let mut best = 0usize;
    for (token, &probability) in probabilities.iter().enumerate() {
        if !probability.is_finite() || probability < 0.0 {
            return Err(invalid("joint probability row contains an invalid value"));
        }
        if probability > probabilities[best] {
            best = token;
        }
    }
    u32::try_from(best).map_err(|_| invalid("joint vocabulary exceeds u32"))
}

fn decode_text(tokenizer: &HfBpeTokenizer, token: u32) -> String {
    String::from_utf8_lossy(&tokenizer.decode_bytes(&[token])).into_owned()
}

#[allow(clippy::too_many_arguments)]
fn decision_zero_capture(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    step: &JointStep,
    session: &JointSession,
    prompt_len: usize,
    row: &[f32],
    emitted_token: u32,
    probe_token: Option<u32>,
) -> Result<DecisionZeroCapture> {
    if step.written_occurrence != prompt_len - 1 {
        return Err(invalid(
            "decision-0 read step does not correspond to the final prompt token",
        ));
    }
    if !positions_are_causal(&step.read_occurrences, step.written_occurrence) {
        return Err(invalid(
            "decision-0 read row contains a non-causal occurrence",
        ));
    }
    let masses = step.read_masses.flatten_all()?.to_vec1::<f32>()?;
    if masses.len() != step.read_occurrences.len() {
        return Err(invalid("decision-0 read mass and occurrence counts differ"));
    }
    let no_read_values = step.no_read_mass.to_vec1::<f32>()?;
    let gate_values = step.copy_gate.to_vec1::<f32>()?;
    let no_read = f64::from(
        *no_read_values
            .first()
            .ok_or_else(|| invalid("decision-0 NoRead mass is missing"))?,
    );
    let copy_gate = f64::from(
        *gate_values
            .first()
            .ok_or_else(|| invalid("decision-0 copy gate is missing"))?,
    );
    let events = session.events();
    let mut read_row = Vec::with_capacity(masses.len());
    let mut top: Option<usize> = None;
    let mut top_mass = 0.0f64;
    let mut read_sum = 0.0f64;
    for (index, &mass) in masses.iter().enumerate() {
        let mass = f64::from(mass);
        if !mass.is_finite() || !(0.0..=1.0 + MASS_SUM_TOLERANCE).contains(&mass) {
            return Err(invalid("decision-0 read mass is invalid"));
        }
        let occurrence = step.read_occurrences[index];
        let token = events
            .get(occurrence)
            .and_then(|event| event.tokens.first())
            .copied()
            .ok_or_else(|| invalid("decision-0 read occurrence has no exact token identity"))?;
        read_sum += mass;
        if mass > top_mass {
            top = Some(index);
            top_mass = mass;
        }
        read_row.push(ReadEventMass {
            occurrence,
            token,
            text: decode_text(tokenizer, token),
            mass,
        });
    }
    let vocabulary_row = model
        .output_vocabulary(&step.state)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let copy_row = model
        .incremental_copy(&step.read_masses, &events[..step.written_occurrence], 1)?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let emitted_components = token_components(
        &vocabulary_row,
        &copy_row,
        row,
        emitted_token,
        no_read,
        copy_gate,
        model.config.vocab_size,
    )?;
    let probe_components = match probe_token {
        Some(token) => Some(token_components(
            &vocabulary_row,
            &copy_row,
            row,
            token,
            no_read,
            copy_gate,
            model.config.vocab_size,
        )?),
        None => None,
    };
    let top_read_occurrence = top.map(|index| step.read_occurrences[index]);
    let top_read_token = top.map(|index| read_row[index].token);
    let top_read_token_text = top.map(|index| read_row[index].text.clone());
    Ok(DecisionZeroCapture {
        written_occurrence: step.written_occurrence,
        final_prompt_token_occurrence: prompt_len - 1,
        final_prompt_token_is_readable: false,
        readable_max_occurrence: step.written_occurrence.checked_sub(1),
        read_mass_sum: read_sum,
        no_read_mass: no_read,
        copy_gate,
        effective_copy_mass: copy_gate * (1.0 - no_read),
        normalization_ok: masses_normalize(&masses, no_read, MASS_SUM_TOLERANCE),
        read_row,
        top_read_occurrence,
        top_read_token,
        top_read_token_text,
        top_read_mass: top_mass,
        emitted_token,
        emitted_token_text: decode_text(tokenizer, emitted_token),
        emitted_components,
        probe_token,
        probe_components,
    })
}

/// Deterministic greedy continuation under the frozen story-probe stop rule:
/// stop at EOS, at the first generated ASCII period after retaining the
/// complete final token, at a terminal short cycle, or at `max_new_tokens`.
/// Prompt ids must already start with BOS 0. The read path is enabled and the
/// returned `decision_zero` records the step that consumed the final prompt
/// token.
pub fn canonical_greedy(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    prompt_token_ids: &[u32],
    max_new_tokens: usize,
    probe_token: Option<u32>,
) -> Result<CanonicalRun> {
    validate_generation_tokens(
        &model.config,
        tokenizer.vocab_size(),
        prompt_token_ids,
        max_new_tokens,
    )?;
    let mut session = model.new_session(1)?;
    let mut current: Option<JointStep> = None;
    for &token in prompt_token_ids {
        current = Some(model.step(&mut session, &[token], ReadMode::Enabled)?);
    }
    let mut current = Some(current.ok_or_else(|| invalid("canonical prompt is empty"))?);
    let mut generated: Vec<u32> = Vec::with_capacity(max_new_tokens);
    let mut decision_zero = None;
    let mut stop = JointGenerationStop::MaximumNewTokens;
    for decision in 0..max_new_tokens {
        let step = current
            .take()
            .ok_or_else(|| invalid("canonical generation has no current prediction"))?;
        let row = step.probabilities.flatten_all()?.to_vec1::<f32>()?;
        let selected = argmax_strict(&row)?;
        if decision == 0 {
            decision_zero = Some(decision_zero_capture(
                model,
                tokenizer,
                &step,
                &session,
                prompt_token_ids.len(),
                &row,
                selected,
                probe_token,
            )?);
        }
        generated.push(selected);
        if selected == 1 {
            stop = JointGenerationStop::Eos;
            break;
        }
        if tokenizer.decode_bytes(&generated).contains(&b'.') {
            stop = JointGenerationStop::FirstSentenceBoundary;
            break;
        }
        if let Some(period) = short_cycle_period(&generated) {
            stop = JointGenerationStop::ShortCycle { period };
            break;
        }
        if decision + 1 < max_new_tokens {
            current = Some(model.step(&mut session, &[selected], ReadMode::Enabled)?);
        }
    }
    let end = generated
        .iter()
        .position(|&token| token == 1)
        .unwrap_or(generated.len());
    let response_bytes = tokenizer.decode_bytes(&generated[..end]);
    let raw_bytes = tokenizer.decode_bytes(&generated);
    assemble_run(
        prompt_token_ids,
        generated,
        &raw_bytes,
        &response_bytes,
        stop,
        decision_zero,
    )
}

/// Assemble the canonical report from a completed greedy pass. Shared by the
/// baseline and intervened loops so their decoding, UTF-8 and verdict fields
/// are identical.
fn assemble_run(
    prompt_token_ids: &[u32],
    generated: Vec<u32>,
    raw_bytes: &[u8],
    response_bytes: &[u8],
    stop: JointGenerationStop,
    decision_zero: Option<DecisionZeroCapture>,
) -> Result<CanonicalRun> {
    Ok(CanonicalRun {
        prompt_token_ids: prompt_token_ids.to_vec(),
        generated_token_ids: generated,
        raw_decoded: String::from_utf8_lossy(raw_bytes).into_owned(),
        response_text: String::from_utf8_lossy(response_bytes).trim().to_owned(),
        utf8_decodable: std::str::from_utf8(raw_bytes).is_ok()
            && std::str::from_utf8(response_bytes).is_ok(),
        stop,
        decision_zero: decision_zero
            .ok_or_else(|| invalid("canonical generation produced no decision-0 capture"))?,
    })
}

/// Deterministic greedy continuation with an optional read-mass intervention
/// applied to exactly the decision-0 step (the step consuming the final prompt
/// token) and cleared immediately after it. `None` reproduces
/// [`canonical_greedy`]; `Some` requires the dense Full-admission batch-1 model
/// that [`ReadMassIntervention`] declares, and any other step errors.
pub fn canonical_greedy_intervened(
    model: &mut JointModel,
    tokenizer: &HfBpeTokenizer,
    prompt_token_ids: &[u32],
    max_new_tokens: usize,
    probe_token: Option<u32>,
    intervention: Option<ReadMassIntervention>,
) -> Result<CanonicalRun> {
    validate_generation_tokens(
        &model.config,
        tokenizer.vocab_size(),
        prompt_token_ids,
        max_new_tokens,
    )?;
    if prompt_token_ids.is_empty() {
        return Err(invalid("canonical prompt is empty"));
    }
    let mut session = model.new_session(1)?;
    let last = prompt_token_ids.len() - 1;
    let mut current: Option<JointStep> = None;
    for (index, &token) in prompt_token_ids.iter().enumerate() {
        if index == last {
            model.set_read_intervention(intervention);
        }
        current = Some(model.step(&mut session, &[token], ReadMode::Enabled)?);
        if index == last {
            model.set_read_intervention(None);
        }
    }
    let mut current = Some(current.ok_or_else(|| invalid("canonical prompt is empty"))?);
    let mut generated: Vec<u32> = Vec::with_capacity(max_new_tokens);
    let mut decision_zero = None;
    let mut stop = JointGenerationStop::MaximumNewTokens;
    for decision in 0..max_new_tokens {
        let step = current
            .take()
            .ok_or_else(|| invalid("canonical generation has no current prediction"))?;
        let row = step.probabilities.flatten_all()?.to_vec1::<f32>()?;
        let selected = argmax_strict(&row)?;
        if decision == 0 {
            decision_zero = Some(decision_zero_capture(
                model,
                tokenizer,
                &step,
                &session,
                prompt_token_ids.len(),
                &row,
                selected,
                probe_token,
            )?);
        }
        generated.push(selected);
        if selected == 1 {
            stop = JointGenerationStop::Eos;
            break;
        }
        if tokenizer.decode_bytes(&generated).contains(&b'.') {
            stop = JointGenerationStop::FirstSentenceBoundary;
            break;
        }
        if let Some(period) = short_cycle_period(&generated) {
            stop = JointGenerationStop::ShortCycle { period };
            break;
        }
        if decision + 1 < max_new_tokens {
            current = Some(model.step(&mut session, &[selected], ReadMode::Enabled)?);
        }
    }
    let end = generated
        .iter()
        .position(|&token| token == 1)
        .unwrap_or(generated.len());
    let raw_bytes = tokenizer.decode_bytes(&generated);
    let response_bytes = tokenizer.decode_bytes(&generated[..end]);
    assemble_run(
        prompt_token_ids,
        generated,
        &raw_bytes,
        &response_bytes,
        stop,
        decision_zero,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_decode(table: &'static [&'static [u8]]) -> impl Fn(&[u32]) -> Vec<u8> + 'static {
        move |ids: &[u32]| {
            let mut out = Vec::new();
            for &id in ids {
                out.extend_from_slice(table[id as usize]);
            }
            out
        }
    }

    #[test]
    fn read_localize_mutation_preserves_length_or_declares() {
        let content = [10u32, 11, 12, 13];
        // Single-token replacement: declared delta is replacement_len - 1.
        let replaced = splice_replace_final(&content, &[20]).expect("replace");
        assert_eq!(replaced, vec![10, 11, 12, 20]);
        assert_eq!(length_delta(content.len(), replaced.len()), (1i64) - 1);
        // Two-token replacement: declared delta is replacement_len - 1.
        let replaced_pair = splice_replace_final(&content, &[20, 21]).expect("replace");
        assert_eq!(replaced_pair, vec![10, 11, 12, 20, 21]);
        assert_eq!(length_delta(content.len(), replaced_pair.len()), 2i64 - 1);
        // Insertion: declared delta is exactly the clause length.
        let inserted = splice_insert(&content, 2, &[30, 31, 32]).expect("insert");
        assert_eq!(inserted, vec![10, 11, 30, 31, 32, 12, 13]);
        assert_eq!(length_delta(content.len(), inserted.len()), 3i64);
        // Out-of-range and empty cases are declared errors, not silent repairs.
        assert!(splice_replace_final(&[], &[1]).is_err());
        assert!(splice_insert(&content, 5, &[1]).is_err());
    }

    /// 0 "", 1 " her", 2 " apple", 3 "pineapple", 4 " app",
    /// 5 "le", 6 " rib", 7 "bon", 8 "green", 9 "apple" (no leading space)
    const TABLE: &[&[u8]] = &[
        b"",
        b" her",
        b" apple",
        b"pineapple",
        b" app",
        b"le",
        b" rib",
        b"bon",
        b"green",
        b"apple",
    ];

    #[test]
    fn read_localize_entity_span_is_exact() {
        let decode = table_decode(TABLE);
        // Exact single-token span and its BOS-corrected occurrence.
        let single = entity_spans(&[1, 2], "apple", decode);
        assert_eq!(single, vec![EntitySpan { start: 1, len: 1 }]);
        assert_eq!(span_occurrence(single[0]), 2);
        // Minimal window: a two-token split resolves to the two-token span.
        assert_eq!(
            entity_spans(&[1, 4, 5], "apple", table_decode(TABLE)),
            vec![EntitySpan { start: 1, len: 2 }]
        );
        // A longer word containing the noun is rejected by the equality test.
        assert!(entity_spans(&[3], "apple", table_decode(TABLE)).is_empty());
        // No word boundary before a bare-token suffix inside a larger word.
        assert!(entity_spans(&[8, 9], "apple", table_decode(TABLE)).is_empty());
        // A leading-space token supplies its own boundary.
        assert_eq!(
            entity_spans(&[8, 2], "apple", table_decode(TABLE)),
            vec![EntitySpan { start: 1, len: 1 }]
        );
        // Multi-token entity resolves minimally.
        assert_eq!(
            entity_spans(&[1, 6, 7], "ribbon", table_decode(TABLE)),
            vec![EntitySpan { start: 1, len: 2 }]
        );
    }

    #[test]
    fn read_localize_masses_normalize_with_no_read() {
        assert!(masses_normalize(&[0.2, 0.3], 0.5, MASS_SUM_TOLERANCE));
        assert!(!masses_normalize(&[0.2, 0.3], 0.6, MASS_SUM_TOLERANCE));
        assert!(!masses_normalize(&[f32::NAN], 0.5, MASS_SUM_TOLERANCE));
        assert!(!masses_normalize(&[-0.1, 0.6], 0.5, MASS_SUM_TOLERANCE));
        // Entity share divides only by the available read mass.
        assert_eq!(entity_share(0.1, 0.5), Some(0.2));
        assert_eq!(entity_share(0.1, 1.0), None);
    }

    #[test]
    fn read_localize_positions_are_causal() {
        assert!(positions_are_causal(&[0, 1, 2], 3));
        assert!(!positions_are_causal(&[0, 3], 3));
        assert!(positions_are_causal(&[], 0));
    }

    #[test]
    fn read_localize_deterministic_replay() {
        let content = [1u32, 6, 7, 9];
        let first = entity_spans(&content, "ribbon", table_decode(TABLE));
        let second = entity_spans(&content, "ribbon", table_decode(TABLE));
        assert_eq!(first, second);
        let boundary_table: &[&[u8]] = &[b"", b" her", b" apple", b".", b" next"];
        let boundary_a =
            insert_index_before_final_query(&[1, 2, 3, 4], table_decode(boundary_table))
                .expect("boundary");
        let boundary_b =
            insert_index_before_final_query(&[1, 2, 3, 4], table_decode(boundary_table))
                .expect("boundary");
        assert_eq!(boundary_a, boundary_b);
        assert_eq!(boundary_a, 3);
        let mutated = splice_insert(&content, 1, &[20]).expect("insert");
        let mutated_again = splice_insert(&content, 1, &[20]).expect("insert");
        assert_eq!(mutated, mutated_again);
    }
}
