//! CPU-only Hamming prototype stop candidate over complete writer queries.
//!
//! This does not add a token to writer admission. It separately classifies a
//! query as terminal or nonterminal and can emit BOS before the exact writer
//! fallback token.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

use uor_r4_integer::stack::{reply_stop, ReplyEnd, POINT_KEY_BOS};

use crate::reply_eval::{evaluate_generated, GeneratedReply, TimedReplyReading};
use crate::{invalid, CpuFbsForward, CpuFbsHead, CpuTrainError, MemoryWeightSample};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const STOP_SIGNATURE_BITS: usize = 256;
pub const STOP_SIGNATURE_WORDS: usize = STOP_SIGNATURE_BITS / 64;
pub type StopSignature = [u64; STOP_SIGNATURE_WORDS];

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopSignatureRow {
    pub signature: StopSignature,
    pub terminal: bool,
    pub writer_token: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopSignatureGate {
    pub signature_bits: usize,
    pub terminal_prototypes: Vec<StopSignature>,
    pub nonterminal_prototypes: Vec<StopSignature>,
    pub margin: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopSignatureTraining {
    pub gate: StopSignatureGate,
    pub terminal_rows: u64,
    pub nonterminal_rows: u64,
    pub terminal_stops: u64,
    pub nonterminal_stops: u64,
    pub nonterminal_stop_limit: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopSignatureReading {
    pub decisions: u64,
    pub terminal_decisions: u64,
    pub nonterminal_decisions: u64,
    pub terminal_stops: u64,
    pub nonterminal_stops: u64,
    pub writer_fallbacks: u64,
    pub gated_stops: u64,
    pub ungated_token_mismatches: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopSignatureDecision {
    pub terminal: bool,
    pub writer_token: u32,
    pub fired: bool,
    pub chosen_token: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopSignatureEvaluation {
    pub reading: StopSignatureReading,
    pub terminal_rows: Vec<bool>,
    pub decisions: Vec<StopSignatureDecision>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopSignatureReplyReading {
    pub reading: TimedReplyReading,
    pub gated_decisions: u64,
    pub fired: u64,
}

pub fn pack_stop_signature(query: &[i32]) -> Result<StopSignature> {
    if query.len() != STOP_SIGNATURE_BITS {
        return Err(invalid(format!(
            "stop signature needs {STOP_SIGNATURE_BITS} query coordinates, got {}",
            query.len()
        )));
    }
    let mut signature = [0u64; STOP_SIGNATURE_WORDS];
    for (index, &coordinate) in query.iter().enumerate() {
        if coordinate >= 0 {
            signature[index / 64] |= 1u64 << (index % 64);
        }
    }
    Ok(signature)
}

pub fn hamming_distance(left: &StopSignature, right: &StopSignature) -> u32 {
    left.iter()
        .zip(right)
        .map(|(&left, &right)| (left ^ right).count_ones())
        .sum()
}

pub fn derive_stop_signature_rows(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
) -> Result<Vec<StopSignatureRow>> {
    if samples.is_empty() {
        return Err(invalid("stop signature needs samples"));
    }
    let mut rows = Vec::new();
    for (sample_index, sample) in samples.iter().enumerate() {
        if sample.prompt.is_empty() || sample.reply.len() < 2 || sample.reply.last() != Some(&1) {
            return Err(invalid(format!(
                "stop signature sample {sample_index} needs a reply ending in corpus EOS"
            )));
        }
        let sequence = sample
            .prompt
            .iter()
            .chain(&sample.reply[..sample.reply.len() - 1])
            .copied()
            .collect::<Vec<_>>();
        let mut forward = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("stop signature forward: {error}")))?;
        if !memory_enabled {
            forward.disable_memory();
        }
        for (index, token) in sequence.into_iter().enumerate() {
            if index + 1 == sample.prompt.len() {
                forward.mark_reply_start();
            }
            let step = forward
                .step(token, CpuFbsHead::Writer)
                .map_err(|error| invalid(format!("stop signature step: {error}")))?;
            if index + 1 >= sample.prompt.len() {
                rows.push(StopSignatureRow {
                    signature: pack_stop_signature(&step.query_q12)?,
                    terminal: index + 1 == sample.prompt.len() + sample.reply.len() - 1,
                    writer_token: step.token,
                });
            }
        }
    }
    if !rows.iter().any(|row| row.terminal) || rows.iter().all(|row| row.terminal) {
        return Err(invalid(
            "stop signature rows need both terminal and nonterminal classes",
        ));
    }
    Ok(rows)
}

pub fn shuffle_stop_signature_rows(
    rows: &[StopSignatureRow],
    seed: u64,
) -> Result<Vec<StopSignatureRow>> {
    if rows.is_empty() {
        return Err(invalid("stop signature shuffle needs rows"));
    }
    let terminal = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.terminal.then_some(index))
        .collect::<Vec<_>>();
    if terminal.is_empty() || terminal.len() == rows.len() {
        return Err(invalid("stop signature shuffle needs both classes"));
    }
    let shift = 1 + seed as usize % (rows.len() - 1);
    let terminal = terminal.into_iter().collect::<BTreeSet<_>>();
    let mut shuffled = rows.to_vec();
    for row in &mut shuffled {
        row.terminal = false;
    }
    for index in terminal {
        shuffled[(index + shift) % rows.len()].terminal = true;
    }
    if shuffled
        .iter()
        .zip(rows)
        .all(|(left, right)| left.terminal == right.terminal)
    {
        return Err(invalid("stop signature shuffle did not change labels"));
    }
    Ok(shuffled)
}

pub fn fit_stop_signature_gate(
    rows: &[StopSignatureRow],
    prototypes_per_class: usize,
) -> Result<StopSignatureTraining> {
    if prototypes_per_class == 0 {
        return Err(invalid("stop prototypes per class must be positive"));
    }
    let terminal = rows.iter().filter(|row| row.terminal).collect::<Vec<_>>();
    let nonterminal = rows.iter().filter(|row| !row.terminal).collect::<Vec<_>>();
    if terminal.is_empty() || nonterminal.is_empty() {
        return Err(invalid("stop fitting needs both classes"));
    }
    let terminal_prototypes = choose_prototypes(
        terminal.iter().map(|row| row.signature),
        prototypes_per_class,
    )?;
    let nonterminal_prototypes = choose_prototypes(
        nonterminal.iter().map(|row| row.signature),
        prototypes_per_class,
    )?;
    let deltas = rows
        .iter()
        .map(|row| {
            score(
                &row.signature,
                &terminal_prototypes,
                &nonterminal_prototypes,
            )
        })
        .collect::<Vec<_>>();
    let nonterminal_stop_limit = nonterminal.len() as u64 / 100;
    let mut thresholds = deltas.clone();
    thresholds.sort_unstable();
    thresholds.dedup();
    let mut best = None;
    for margin in thresholds {
        let terminal_stops = rows
            .iter()
            .zip(&deltas)
            .filter(|(row, delta)| row.terminal && **delta >= margin)
            .count() as u64;
        let nonterminal_stops = rows
            .iter()
            .zip(&deltas)
            .filter(|(row, delta)| !row.terminal && **delta >= margin)
            .count() as u64;
        if terminal_stops == 0 || nonterminal_stops > nonterminal_stop_limit {
            continue;
        }
        if best.is_none_or(|(best_terminal, best_nonterminal, best_margin)| {
            terminal_stops > best_terminal
                || (terminal_stops == best_terminal
                    && (nonterminal_stops < best_nonterminal
                        || (nonterminal_stops == best_nonterminal && margin > best_margin)))
        }) {
            best = Some((terminal_stops, nonterminal_stops, margin));
        }
    }
    let (terminal_stops, nonterminal_stops, margin) = best.ok_or_else(|| {
        invalid("stop signature found no margin with nonzero terminal recall under 1% false stops")
    })?;
    Ok(StopSignatureTraining {
        gate: StopSignatureGate {
            signature_bits: STOP_SIGNATURE_BITS,
            terminal_prototypes,
            nonterminal_prototypes,
            margin,
        },
        terminal_rows: terminal.len() as u64,
        nonterminal_rows: nonterminal.len() as u64,
        terminal_stops,
        nonterminal_stops,
        nonterminal_stop_limit,
    })
}

fn choose_prototypes(
    signatures: impl Iterator<Item = StopSignature>,
    count: usize,
) -> Result<Vec<StopSignature>> {
    let mut signatures = signatures.collect::<Vec<_>>();
    signatures.sort_unstable();
    signatures.dedup();
    if signatures.is_empty() {
        return Err(invalid("stop prototype class is empty"));
    }
    let mut selected = vec![medoid(&signatures)?];
    while selected.len() < count.min(signatures.len()) {
        let mut best: Option<(u32, StopSignature)> = None;
        for candidate in &signatures {
            if selected.contains(candidate) {
                continue;
            }
            let nearest = selected
                .iter()
                .map(|selected| hamming_distance(candidate, selected))
                .min()
                .ok_or_else(|| invalid("stop prototype selection lost its class"))?;
            if best.is_none_or(|(distance, point)| {
                nearest > distance || (nearest == distance && *candidate < point)
            }) {
                best = Some((nearest, *candidate));
            }
        }
        let (_, point) = best.ok_or_else(|| invalid("stop prototype residual group is empty"))?;
        selected.push(point);
    }
    Ok(selected)
}

fn medoid(signatures: &[StopSignature]) -> Result<StopSignature> {
    let mut best: Option<(u64, StopSignature)> = None;
    for candidate in signatures {
        let total = signatures.iter().try_fold(0u64, |sum, signature| {
            sum.checked_add(u64::from(hamming_distance(candidate, signature)))
                .ok_or_else(|| invalid("stop medoid distance sum overflows"))
        })?;
        if best.is_none_or(|(distance, point)| {
            total < distance || (total == distance && *candidate < point)
        }) {
            best = Some((total, *candidate));
        }
    }
    Ok(best.ok_or_else(|| invalid("stop medoid class is empty"))?.1)
}

fn score(
    signature: &StopSignature,
    terminal: &[StopSignature],
    nonterminal: &[StopSignature],
) -> i64 {
    let nearest = |points: &[StopSignature]| {
        points
            .iter()
            .map(|point| hamming_distance(signature, point))
            .min()
            .unwrap_or(u32::MAX)
    };
    i64::from(nearest(nonterminal)) - i64::from(nearest(terminal))
}

pub fn evaluate_stop_signature_rows(
    rows: &[StopSignatureRow],
    gate: &StopSignatureGate,
) -> Result<StopSignatureEvaluation> {
    if gate.signature_bits != STOP_SIGNATURE_BITS
        || gate.terminal_prototypes.is_empty()
        || gate.nonterminal_prototypes.is_empty()
    {
        return Err(invalid("stop gate shape is empty or unsupported"));
    }
    let mut reading = StopSignatureReading::default();
    let mut terminal_rows = Vec::new();
    let mut decisions = Vec::with_capacity(rows.len());
    for row in rows {
        reading.decisions = checked_add(reading.decisions, 1, "stop decision count")?;
        let fires = gate_fires(gate, &row.signature);
        let chosen_token = if fires {
            POINT_KEY_BOS
        } else {
            row.writer_token
        };
        reading.ungated_token_mismatches = checked_add(
            reading.ungated_token_mismatches,
            u64::from(!fires && chosen_token != row.writer_token),
            "ungated token mismatch count",
        )?;
        if row.terminal {
            reading.terminal_decisions =
                checked_add(reading.terminal_decisions, 1, "terminal decision count")?;
            reading.terminal_stops += u64::from(fires);
            terminal_rows.push(fires);
        } else {
            reading.nonterminal_decisions = checked_add(
                reading.nonterminal_decisions,
                1,
                "nonterminal decision count",
            )?;
            reading.nonterminal_stops += u64::from(fires);
        }
        if fires {
            reading.gated_stops = checked_add(reading.gated_stops, 1, "gated stop count")?;
        } else {
            reading.writer_fallbacks =
                checked_add(reading.writer_fallbacks, 1, "writer fallback count")?;
        }
        decisions.push(StopSignatureDecision {
            terminal: row.terminal,
            writer_token: row.writer_token,
            fired: fires,
            chosen_token,
        });
    }
    if reading.terminal_decisions == 0 || reading.nonterminal_decisions == 0 {
        return Err(invalid("stop evaluation needs both classes"));
    }
    Ok(StopSignatureEvaluation {
        reading,
        terminal_rows,
        decisions,
    })
}

pub fn evaluate_stop_signature_no_gate(
    rows: &[StopSignatureRow],
) -> Result<StopSignatureEvaluation> {
    let mut reading = StopSignatureReading::default();
    let mut terminal_rows = Vec::new();
    let mut decisions = Vec::with_capacity(rows.len());
    for row in rows {
        reading.decisions = checked_add(reading.decisions, 1, "stop decision count")?;
        let bos = row.writer_token == POINT_KEY_BOS;
        if row.terminal {
            reading.terminal_decisions =
                checked_add(reading.terminal_decisions, 1, "terminal decision count")?;
            reading.terminal_stops += u64::from(bos);
            terminal_rows.push(bos);
        } else {
            reading.nonterminal_decisions = checked_add(
                reading.nonterminal_decisions,
                1,
                "nonterminal decision count",
            )?;
            reading.nonterminal_stops += u64::from(bos);
        }
        reading.writer_fallbacks =
            checked_add(reading.writer_fallbacks, 1, "writer fallback count")?;
        decisions.push(StopSignatureDecision {
            terminal: row.terminal,
            writer_token: row.writer_token,
            fired: false,
            chosen_token: row.writer_token,
        });
    }
    if reading.terminal_decisions == 0 || reading.nonterminal_decisions == 0 {
        return Err(invalid("no-gate stop evaluation needs both classes"));
    }
    Ok(StopSignatureEvaluation {
        reading,
        terminal_rows,
        decisions,
    })
}

pub fn evaluate_stop_signature_replies(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    cap: usize,
    memory_enabled: bool,
    gate: &StopSignatureGate,
) -> Result<StopSignatureReplyReading> {
    let gated = AtomicU64::new(0);
    let fired = AtomicU64::new(0);
    let reading = evaluate_generated(artifact, samples, cap, |prompt, cap| {
        generate_stop_signature_reply(artifact, prompt, cap, memory_enabled, gate, &gated, &fired)
    })?;
    Ok(StopSignatureReplyReading {
        reading,
        gated_decisions: gated.load(Ordering::Relaxed),
        fired: fired.load(Ordering::Relaxed),
    })
}

fn generate_stop_signature_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    memory_enabled: bool,
    gate: &StopSignatureGate,
    gated: &AtomicU64,
    fired: &AtomicU64,
) -> Result<GeneratedReply> {
    let mut forward = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("stop signature forward: {error}")))?;
    if !memory_enabled {
        forward.disable_memory();
    }
    let context = forward.context_length();
    if cap == 0 || cap > 64 || prompt.is_empty() || prompt.len() > context {
        return Err(invalid(
            "stop signature prompt, cap or artifact context is invalid",
        ));
    }
    let mut next = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("stop signature prompt step: {error}")))?;
        if index + 1 == prompt.len() {
            next = Some(gate_name_or_writer(gate, &step, gated, fired)?);
        }
    }
    let mut next = next.ok_or_else(|| invalid("stop signature prompt is empty"))?;
    let mut tokens = Vec::with_capacity(cap);
    loop {
        let stop = reply_stop(next, tokens.len(), prompt.len(), cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(next);
        }
        if let Some(stop) = stop {
            return Ok(GeneratedReply {
                tokens,
                stop: match stop {
                    ReplyEnd::Bos => "bos",
                    ReplyEnd::Length => "cap",
                    ReplyEnd::Context => "context",
                },
            });
        }
        let step = forward
            .step(next, CpuFbsHead::Writer)
            .map_err(|error| invalid(format!("stop signature generated step: {error}")))?;
        next = gate_name_or_writer(gate, &step, gated, fired)?;
    }
}

fn gate_name_or_writer(
    gate: &StopSignatureGate,
    step: &crate::CpuFbsStep,
    gated: &AtomicU64,
    fired: &AtomicU64,
) -> Result<u32> {
    gated.fetch_add(1, Ordering::Relaxed);
    if gate_fires(gate, &pack_stop_signature(&step.query_q12)?) {
        fired.fetch_add(1, Ordering::Relaxed);
        Ok(POINT_KEY_BOS)
    } else {
        Ok(step.token)
    }
}

fn gate_fires(gate: &StopSignatureGate, signature: &StopSignature) -> bool {
    score(
        signature,
        &gate.terminal_prototypes,
        &gate.nonterminal_prototypes,
    ) >= gate.margin
}

fn checked_add(value: u64, amount: u64, name: &str) -> Result<u64> {
    value
        .checked_add(amount)
        .ok_or_else(|| invalid(format!("{name} overflows")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluate_replies;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn signature(bit: usize) -> StopSignature {
        let mut signature = [0u64; STOP_SIGNATURE_WORDS];
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn rows() -> Vec<StopSignatureRow> {
        vec![
            StopSignatureRow {
                signature: [u64::MAX; STOP_SIGNATURE_WORDS],
                terminal: true,
                writer_token: 7,
            },
            StopSignatureRow {
                signature: [u64::MAX - 1; STOP_SIGNATURE_WORDS],
                terminal: true,
                writer_token: 8,
            },
            StopSignatureRow {
                signature: [0; STOP_SIGNATURE_WORDS],
                terminal: false,
                writer_token: 9,
            },
            StopSignatureRow {
                signature: signature(17),
                terminal: false,
                writer_token: 10,
            },
            StopSignatureRow {
                signature: signature(199),
                terminal: false,
                writer_token: 11,
            },
        ]
    }

    fn artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            1321,
            FbsSynth {
                vocab: crate::VOCABULARY,
                d: STOP_SIGNATURE_BITS,
                mlp: 64,
                layers: 1,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        snapshot.memory = None;
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn samples() -> Vec<MemoryWeightSample> {
        vec![MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![42, 77, 1],
        }]
    }

    #[test]
    fn packing_and_hamming_use_coordinate_order() {
        let query = (0..STOP_SIGNATURE_BITS)
            .map(|index| if index % 3 == 0 { -1 } else { 0 })
            .collect::<Vec<_>>();
        let signature = pack_stop_signature(&query).unwrap();
        let zeros = [0u64; STOP_SIGNATURE_WORDS];
        assert_eq!(
            hamming_distance(&signature, &zeros),
            (0..STOP_SIGNATURE_BITS)
                .filter(|index| index % 3 != 0)
                .count() as u32
        );
        assert!(pack_stop_signature(&query[..255]).is_err());
    }

    #[test]
    fn prototype_selection_is_deterministic_and_prefers_far_points() {
        let prototypes = choose_prototypes(rows().into_iter().map(|row| row.signature), 2).unwrap();
        assert_eq!(prototypes.len(), 2);
        assert_eq!(
            prototypes,
            choose_prototypes(rows().into_iter().map(|row| row.signature), 2).unwrap()
        );
    }

    #[test]
    fn margin_fit_respects_the_nonterminal_limit_and_refuses_bad_classes() {
        let training = fit_stop_signature_gate(&rows(), 2).unwrap();
        assert!(training.terminal_stops > 0);
        assert!(training.nonterminal_stops <= training.nonterminal_stop_limit);
        assert!(fit_stop_signature_gate(&rows()[..1], 2).is_err());
    }

    #[test]
    fn shuffled_labels_preserve_counts_and_change_the_subset() {
        let shuffled = shuffle_stop_signature_rows(&rows(), 3).unwrap();
        assert_eq!(
            shuffled.iter().filter(|row| row.terminal).count(),
            rows().iter().filter(|row| row.terminal).count()
        );
        assert!(shuffled
            .iter()
            .zip(rows())
            .any(|(left, right)| left.terminal != right.terminal));
    }

    #[test]
    fn evaluation_counts_terminal_and_nonterminal_stops_separately() {
        let training = fit_stop_signature_gate(&rows(), 2).unwrap();
        let evaluation = evaluate_stop_signature_rows(&rows(), &training.gate).unwrap();
        assert_eq!(evaluation.terminal_rows.len(), 2);
        assert_eq!(evaluation.decisions.len(), rows().len());
        assert_eq!(
            evaluation.reading.terminal_stops,
            evaluation
                .terminal_rows
                .iter()
                .filter(|fire| **fire)
                .count() as u64
        );
        assert_eq!(
            evaluation.reading.gated_stops,
            evaluation.reading.terminal_stops + evaluation.reading.nonterminal_stops
        );
        assert_eq!(evaluation.reading.ungated_token_mismatches, 0);
    }

    #[test]
    fn full_query_rows_mark_only_the_final_reply_content_row_terminal() {
        let rows = derive_stop_signature_rows(&artifact(), &samples(), false).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows.iter().map(|row| row.terminal).collect::<Vec<_>>(),
            vec![false, false, true]
        );
    }

    #[test]
    fn no_gate_preserves_writer_tokens_and_gate_fit_is_deterministic() {
        let rows = derive_stop_signature_rows(&artifact(), &samples(), false).unwrap();
        let no_gate = evaluate_stop_signature_no_gate(&rows).unwrap();
        assert!(no_gate
            .decisions
            .iter()
            .all(|decision| { !decision.fired && decision.chosen_token == decision.writer_token }));
        let first = fit_stop_signature_gate(&rows, 2).unwrap();
        let second = fit_stop_signature_gate(&rows, 2).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn gated_free_replies_match_no_gate_or_fire_exactly_at_bos() {
        let artifact = artifact();
        let samples = samples();
        let rows = derive_stop_signature_rows(&artifact, &samples, false).unwrap();
        let base = fit_stop_signature_gate(&rows, 2).unwrap().gate;
        let no_gate = evaluate_replies(&artifact, &samples, 4).unwrap();
        let never = StopSignatureGate {
            margin: i64::MAX,
            ..base.clone()
        };
        let never_reading =
            evaluate_stop_signature_replies(&artifact, &samples, 4, false, &never).unwrap();
        assert_eq!(never_reading.reading.reading, no_gate.reading);
        assert_eq!(never_reading.reading.reply_sha256, no_gate.reply_sha256);
        assert_eq!(never_reading.reading.rows, no_gate.rows);
        assert_eq!(never_reading.fired, 0);

        let always = StopSignatureGate {
            margin: i64::MIN,
            ..base
        };
        let always_reading =
            evaluate_stop_signature_replies(&artifact, &samples, 4, false, &always).unwrap();
        assert!(always_reading.fired > 0);
        assert_eq!(always_reading.reading.reading.bos_complete, 1);
    }
}
