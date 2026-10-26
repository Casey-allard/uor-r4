//! Compare a converted SpiralCore/FBS trunk with the float GPU trunk it came
//! from (d11-exporter-float-trunk, claude-123 and claude-124).
//!
//! ```text
//! fbs-compare [--parity-only] SNAPSHOT_JSON READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]
//! ```
//!
//! `SNAPSHOT_JSON` is the converter's snapshot. `READOUT_JSON` is the float
//! GPU trunk's own plain outputs, and `EXPORT_READOUT_JSON` the same in the
//! float sheets' D11 export mode (`fbs_snapshot::FbsFloatReadout`). Each
//! readout holds every token once, in order, and the same fixed sequences;
//! readings are reported for single tokens and for sequence positions.
//!
//! It exports and parses the snapshot, then for every token compares:
//! - the plain readout with the reference that snaps nothing
//!   (`FbsReferenceSnaps::FLOAT`), and the export readout with the reference
//!   in export form (`FLOAT_EXPORT`): agreement here says the converter mapped
//!   every table right (the parity proof);
//! - the integer session with the fully snapped, quantized reference;
//! - the reference variants with each other, which needs no GPU: how much of
//!   the float-to-integer gap each snap and the table rounding explain.
//!
//! With `--parity-only` it reports only the D11 readout against the integer
//! session, and runs no float reference: a GPU job's parity proof, since a
//! host reference run is a host fallback (CHARTER §3.5, claude-126).
//!
//! It prints one JSON line: readings, not bars. It exits non-zero only when
//! the snapshot does not export, parse or serve, or a readout does not fit.

use std::process::ExitCode;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::{FbsFloatReadout, FbsReadoutMode};
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, FbsReferenceSnaps, FbsTieMargins, FbsTrunkSnapshot, FbsWriterOutput,
    IntegerStackModel,
};

/// One position's writer token and query.
type Output = (u32, Vec<f64>);

/// One kind of [`FbsTieMargins`] margin.
type Margin = fn(&FbsTieMargins) -> f64;

const USAGE: &str =
    "usage: fbs-compare [--parity-only] SNAPSHOT_JSON READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]";

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn check_readout(
    snap: &FbsTrunkSnapshot,
    readout: &FbsFloatReadout,
    mode: FbsReadoutMode,
    path: &str,
) -> Result<(), String> {
    readout
        .check(snap.vocab, snap.width, snap.context, mode)
        .map_err(|e| format!("{path}: {e}"))
}

/// Token agreement and the largest query gap between two sets of outputs.
/// The parity bar's gap (claude-124, claude-125).
const GAP_BAR: f64 = 1e-3;

/// The largest query gap at each position between two sets of outputs.
fn gaps(a: &[Output], b: &[Output]) -> Vec<f64> {
    a.iter()
        .zip(b)
        .map(|(x, y)| {
            x.1.iter()
                .zip(&y.1)
                .map(|(p, q)| (p - q).abs())
                .fold(0f64, f64::max)
        })
        .collect()
}

/// Token agreement and the largest query gap between two sets of outputs,
/// with how many positions exceed [`GAP_BAR`] and the worst of them
/// (`[position, gap, tokens agree]`), so a miss can be traced. Positions
/// count across all inputs in order: for sequences, sequence `s` position
/// `t` is `s * context + t`.
fn pair(a: &[Output], b: &[Output]) -> Value {
    let tokens = a.iter().zip(b).filter(|(x, y)| x.0 == y.0).count();
    let gaps = gaps(a, b);
    let gap = gaps.iter().copied().fold(0f64, f64::max);
    let mut over: Vec<usize> = (0..gaps.len()).filter(|&i| gaps[i] > GAP_BAR).collect();
    over.sort_by(|&i, &j| gaps[j].total_cmp(&gaps[i]).then(i.cmp(&j)));
    let worst: Vec<Value> = over
        .iter()
        .take(6)
        .map(|&i| json!([i, gaps[i], a[i].0 == b[i].0]))
        .collect();
    json!({
        "tokens_agree": tokens,
        "max_abs_query_gap": gap,
        "positions_over_gap_bar": over.len(),
        "worst": worst,
    })
}

/// The reference's token and query at every position of every input.
fn reference(
    snap: &FbsTrunkSnapshot,
    inputs: &[Vec<u32>],
    quantized: bool,
    snaps: FbsReferenceSnaps,
) -> Result<Vec<Output>, String> {
    let inputs: Vec<&[u32]> = inputs.iter().map(Vec::as_slice).collect();
    let each: Vec<Vec<FbsWriterOutput>> = snap
        .forward_f32_each(&inputs, quantized, snaps)
        .map_err(|e| format!("reference: {e}"))?;
    Ok(each
        .into_iter()
        .flatten()
        .map(|o| (o.token, o.query))
        .collect())
}

/// The integer session's token at every position of every input, each input
/// in a fresh session.
fn served(model: &IntegerStackModel, inputs: &[Vec<u32>]) -> Result<Vec<Output>, String> {
    let mut all = Vec::new();
    for tokens in inputs {
        let mut session = model.session();
        for &t in tokens {
            let step = session
                .step_native(t)
                .map_err(|e| format!("{tokens:?}: integer step: {e}"))?;
            let query = step
                .query_q12
                .iter()
                .map(|&q| f64::from(q) / 4096.0)
                .collect();
            all.push((step.token, query));
        }
    }
    Ok(all)
}

/// The quantized, fully snapped reference's [`FbsTieMargins`] at every
/// position of every input (those met on the way to it, in one pass per
/// input), and how many snaps of a zero vector each whole input met.
fn tie_margins(
    snap: &FbsTrunkSnapshot,
    inputs: &[Vec<u32>],
) -> Result<(Vec<FbsTieMargins>, u64), String> {
    let inputs: Vec<&[u32]> = inputs.iter().map(Vec::as_slice).collect();
    let each = snap
        .tie_margins_each(&inputs, true, FbsReferenceSnaps::INTEGER)
        .map_err(|e| format!("tie margins: {e}"))?;
    let zero = each
        .iter()
        .filter_map(|m| m.last())
        .map(|m| u64::from(m.zero_snaps))
        .sum();
    Ok((each.into_iter().flatten().collect(), zero))
}

/// Each position over [`GAP_BAR`] in any of `pairings`, traced to its
/// near-tie: the reference's margins there, with how many positions came
/// nearer on each (0 is the nearest), and the median of each margin.
fn trace_ties(margins: &[FbsTieMargins], zero_snaps: u64, pairings: &[Vec<f64>]) -> Value {
    let kinds: [(&str, Margin); 3] = [
        ("transition", |m| m.transition),
        ("gate", |m| m.gate),
        ("rate", |m| m.rate),
    ];
    let over: std::collections::BTreeSet<usize> = pairings
        .iter()
        .flat_map(|g| (0..g.len()).filter(move |&i| g[i] > GAP_BAR))
        .collect();
    let traced: Vec<Value> = over
        .iter()
        .map(|&i| {
            let mut entry = json!({ "position": i, "zero_snaps": margins[i].zero_snaps });
            for (name, pick) in kinds {
                let nearer = margins
                    .iter()
                    .filter(|m| pick(m) < pick(&margins[i]))
                    .count();
                entry[name] = json!({ "margin": pick(&margins[i]), "nearer_positions": nearer });
            }
            entry
        })
        .collect();
    let mut medians = json!({});
    for (name, pick) in kinds {
        let mut values: Vec<f64> = margins.iter().map(pick).collect();
        values.sort_by(f64::total_cmp);
        medians[name] = json!(values.get(values.len() / 2));
    }
    let zero = margins.iter().filter(|m| m.zero_snaps > 0).count();
    json!({
        "median": medians,
        "positions_with_zero_snaps": zero,
        "zero_snaps": zero_snaps,
        "over_gap_bar": traced,
    })
}

/// A readout's single-token outputs, then its sequence outputs, as the
/// reference lays them out for `singles` and the readout's own sequences.
fn floats(readout: &FbsFloatReadout) -> (Vec<Output>, Vec<Output>) {
    let widen = |h: &[f32]| h.iter().map(|&v| f64::from(v)).collect::<Vec<f64>>();
    let singles = readout
        .positions
        .iter()
        .map(|p| (p.next, widen(&p.hidden)))
        .collect();
    let sequences = readout
        .sequences
        .iter()
        .flat_map(|q| q.next.iter().zip(&q.hidden).map(|(&n, h)| (n, widen(h))))
        .collect();
    (singles, sequences)
}

/// Every reading for one set of inputs: the parity of each float readout with
/// its matching reference, the integer control and the gap attribution.
fn section(
    snap: &FbsTrunkSnapshot,
    model: &IntegerStackModel,
    inputs: &[Vec<u32>],
    plain_gpu: &[Output],
    export_gpu: Option<&[Output]>,
    d11_gpu: Option<&[Output]>,
) -> Result<Value, String> {
    let float_ref = reference(snap, inputs, false, FbsReferenceSnaps::FLOAT)?;
    let export_ref = reference(snap, inputs, false, FbsReferenceSnaps::FLOAT_EXPORT)?;
    let integer_ref = reference(snap, inputs, false, FbsReferenceSnaps::INTEGER)?;
    let quantized_ref = reference(snap, inputs, true, FbsReferenceSnaps::INTEGER)?;
    let served = served(model, inputs)?;
    let integer_vs_quantized = served
        .iter()
        .zip(&quantized_ref)
        .filter(|(s, r)| s.0 == r.0)
        .count();
    let mut out = json!({
        "positions": float_ref.len(),
        "parity_plain_gpu_vs_float_reference": pair(plain_gpu, &float_ref),
        "plain_gpu_vs_integer_reference": pair(plain_gpu, &integer_ref),
        "integer_token_equals_quantized_reference": integer_vs_quantized,
        "integer_session_vs_quantized_reference": pair(&served, &quantized_ref),
        "attribution": {
            "float_vs_float_export_reference": pair(&float_ref, &export_ref),
            "float_export_vs_integer_reference": pair(&export_ref, &integer_ref),
            "integer_plain_vs_quantized_reference": pair(&integer_ref, &quantized_ref),
        },
    });
    if let Some(export_gpu) = export_gpu {
        out["parity_export_gpu_vs_float_export_reference"] = pair(export_gpu, &export_ref);
    }
    if let Some(d11_gpu) = d11_gpu {
        // The D11 mode against what the artifact computes: the quantized,
        // fully snapped reference, and the integer session itself.
        out["parity_d11_gpu_vs_quantized_reference"] = pair(d11_gpu, &quantized_ref);
        out["d11_gpu_token_equals_integer_token"] = json!(d11_gpu
            .iter()
            .zip(&served)
            .filter(|(g, s)| g.0 == s.0)
            .count());
        // The artifact's own Q12 query, not a reference of it.
        out["d11_gpu_vs_integer_session"] = pair(d11_gpu, &served);
        // Where the three disagree, how near the reference came to a switch.
        let (margins, zero_snaps) = tie_margins(snap, inputs)?;
        out["tie_margins"] = trace_ties(
            &margins,
            zero_snaps,
            &[
                gaps(d11_gpu, &quantized_ref),
                gaps(d11_gpu, &served),
                gaps(&served, &quantized_ref),
            ],
        );
    }
    Ok(out)
}

/// The D11 readout against the integer session alone, with no float
/// reference (`--parity-only`, claude-126): the artifact's own tokens and Q12
/// query, which is all a GPU job's parity proof needs, since a host reference
/// run is a host fallback (CHARTER §3.5).
fn parity_section(
    model: &IntegerStackModel,
    inputs: &[Vec<u32>],
    d11_gpu: &[Output],
) -> Result<Value, String> {
    let served = served(model, inputs)?;
    if d11_gpu.len() != served.len() {
        return Err(format!(
            "the D11 readout has {} positions, the integer session {}",
            d11_gpu.len(),
            served.len()
        ));
    }
    let agree = d11_gpu
        .iter()
        .zip(&served)
        .filter(|(g, s)| g.0 == s.0)
        .count();
    Ok(json!({
        "positions": served.len(),
        "d11_gpu_token_equals_integer_token": agree,
        "d11_gpu_vs_integer_session": pair(d11_gpu, &served),
    }))
}

fn run() -> Result<(), String> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let parity_only = args.first().is_some_and(|a| a == "--parity-only");
    if parity_only {
        args.remove(0);
    }
    let [snapshot, plain, rest @ ..] = args.as_slice() else {
        return Err(USAGE.to_owned());
    };
    if rest.len() > 2 {
        return Err(USAGE.to_owned());
    }
    let snap: FbsTrunkSnapshot = read_json(snapshot)?;
    let plain_readout: FbsFloatReadout = read_json(plain)?;
    check_readout(&snap, &plain_readout, FbsReadoutMode::Plain, plain)?;
    // The optional readouts, in order: export mode, then the D11 mode.
    let mut extra: Vec<Option<FbsFloatReadout>> = vec![None, None];
    for ((path, mode), slot) in rest
        .iter()
        .zip([FbsReadoutMode::Export, FbsReadoutMode::D11])
        .zip(&mut extra)
    {
        let readout: FbsFloatReadout = read_json(path)?;
        check_readout(&snap, &readout, mode, path)?;
        *slot = Some(readout);
    }
    let [export_readout, d11_readout] = [extra[0].take(), extra[1].take()];

    let sequences: Vec<Vec<u32>> = plain_readout
        .sequences
        .iter()
        .map(|q| q.tokens.clone())
        .collect();
    for readout in export_readout.iter().chain(&d11_readout) {
        if readout
            .sequences
            .iter()
            .map(|q| &q.tokens)
            .ne(sequences.iter())
        {
            return Err("the readouts hold different sequences".to_owned());
        }
    }

    let artifact = export_fbs_trunk_uorlut01(&snap).map_err(|e| format!("export: {e}"))?;
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("parse: {e}"))?;
    let singles: Vec<Vec<u32>> = (0..snap.vocab as u32).map(|t| vec![t]).collect();
    let (plain_singles, plain_sequences) = floats(&plain_readout);
    let export = export_readout.as_ref().map(floats);
    let d11 = d11_readout.as_ref().map(floats);
    if parity_only {
        let (d11_singles, d11_sequences) =
            d11.as_ref().ok_or("--parity-only needs the D11 readout")?;
        let report = json!({
            "artifact_sha256": hex::encode(Sha256::digest(&artifact)),
            "parity_only": true,
            "single_tokens": parity_section(&model, &singles, d11_singles)?,
            "sequences": parity_section(&model, &sequences, d11_sequences)?,
        });
        println!("{report}");
        return Ok(());
    }
    let report = json!({
        "artifact_sha256": hex::encode(Sha256::digest(&artifact)),
        "single_tokens": section(
            &snap,
            &model,
            &singles,
            &plain_singles,
            export.as_ref().map(|e| e.0.as_slice()),
            d11.as_ref().map(|e| e.0.as_slice()),
        )?,
        "sequences": section(
            &snap,
            &model,
            &sequences,
            &plain_sequences,
            export.as_ref().map(|e| e.1.as_slice()),
            d11.as_ref().map(|e| e.1.as_slice()),
        )?,
    });
    println!("{report}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fbs-compare: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_counts_and_ranks_the_positions_over_the_gap_bar() {
        let a: Vec<Output> = vec![
            (1, vec![0.0, 0.5]),
            (2, vec![0.0, 0.0]),
            (3, vec![1.0, 0.0]),
        ];
        let b: Vec<Output> = vec![
            (1, vec![0.0, 0.5005]),
            (9, vec![0.0, 0.0]),
            (3, vec![0.0, 0.0]),
        ];
        let p = pair(&a, &b);
        assert_eq!(p["tokens_agree"], 2);
        assert_eq!(p["positions_over_gap_bar"], 1);
        assert_eq!(p["max_abs_query_gap"], 1.0);
        assert_eq!(p["worst"], json!([[2, 1.0, true]]));
    }
}
