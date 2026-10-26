//! Compare a converted SpiralCore/FBS trunk with the float GPU trunk it came
//! from (d11-exporter-float-trunk, claude-123 and claude-124).
//!
//! ```text
//! fbs-compare SNAPSHOT_JSON READOUT_JSON [EXPORT_READOUT_JSON]
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
//! It prints one JSON line: readings, not bars. It exits non-zero only when
//! the snapshot does not export, parse or serve, or a readout does not fit.

use std::process::ExitCode;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::FbsFloatReadout;
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, FbsReferenceSnaps, FbsTrunkSnapshot, FbsWriterOutput,
    IntegerStackModel,
};

/// One position's writer token and query.
type Output = (u32, Vec<f64>);

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn check_readout(
    snap: &FbsTrunkSnapshot,
    readout: &FbsFloatReadout,
    export_mode: bool,
    path: &str,
) -> Result<(), String> {
    readout
        .check(snap.vocab, snap.width, snap.context, export_mode)
        .map_err(|e| format!("{path}: {e}"))
}

/// Token agreement and the largest query gap between two sets of outputs.
fn pair(a: &[Output], b: &[Output]) -> Value {
    let tokens = a.iter().zip(b).filter(|(x, y)| x.0 == y.0).count();
    let gap = a
        .iter()
        .zip(b)
        .flat_map(|(x, y)| x.1.iter().zip(&y.1).map(|(p, q)| (p - q).abs()))
        .fold(0f64, f64::max);
    json!({"tokens_agree": tokens, "max_abs_query_gap": gap})
}

/// The reference's token and query at every position of every input.
fn reference(
    snap: &FbsTrunkSnapshot,
    inputs: &[Vec<u32>],
    quantized: bool,
    snaps: FbsReferenceSnaps,
) -> Result<Vec<Output>, String> {
    let mut all = Vec::new();
    for tokens in inputs {
        let out: Vec<FbsWriterOutput> = snap
            .forward_f32_with(tokens, quantized, snaps)
            .map_err(|e| format!("{tokens:?}: reference: {e}"))?;
        all.extend(out.into_iter().map(|o| (o.token, o.query)));
    }
    Ok(all)
}

/// The integer session's token at every position of every input, each input
/// in a fresh session.
fn served(model: &IntegerStackModel, inputs: &[Vec<u32>]) -> Result<Vec<u32>, String> {
    let mut all = Vec::new();
    for tokens in inputs {
        let mut session = model.session();
        for &t in tokens {
            let step = session
                .step_native(t)
                .map_err(|e| format!("{tokens:?}: integer step: {e}"))?;
            all.push(step.token);
        }
    }
    Ok(all)
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
) -> Result<Value, String> {
    let float_ref = reference(snap, inputs, false, FbsReferenceSnaps::FLOAT)?;
    let export_ref = reference(snap, inputs, false, FbsReferenceSnaps::FLOAT_EXPORT)?;
    let integer_ref = reference(snap, inputs, false, FbsReferenceSnaps::INTEGER)?;
    let quantized_ref = reference(snap, inputs, true, FbsReferenceSnaps::INTEGER)?;
    let integer_vs_quantized = served(model, inputs)?
        .iter()
        .zip(&quantized_ref)
        .filter(|(s, r)| **s == r.0)
        .count();
    let mut out = json!({
        "positions": float_ref.len(),
        "parity_plain_gpu_vs_float_reference": pair(plain_gpu, &float_ref),
        "plain_gpu_vs_integer_reference": pair(plain_gpu, &integer_ref),
        "integer_token_equals_quantized_reference": integer_vs_quantized,
        "attribution": {
            "float_vs_float_export_reference": pair(&float_ref, &export_ref),
            "float_export_vs_integer_reference": pair(&export_ref, &integer_ref),
            "integer_plain_vs_quantized_reference": pair(&integer_ref, &quantized_ref),
        },
    });
    if let Some(export_gpu) = export_gpu {
        out["parity_export_gpu_vs_float_export_reference"] = pair(export_gpu, &export_ref);
    }
    Ok(out)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (snapshot, plain, export) = match args.as_slice() {
        [s, p] => (s, p, None),
        [s, p, e] => (s, p, Some(e)),
        _ => {
            return Err(
                "usage: fbs-compare SNAPSHOT_JSON READOUT_JSON [EXPORT_READOUT_JSON]".to_owned(),
            )
        }
    };
    let snap: FbsTrunkSnapshot = read_json(snapshot)?;
    let plain_readout: FbsFloatReadout = read_json(plain)?;
    check_readout(&snap, &plain_readout, false, plain)?;
    let export_readout: Option<FbsFloatReadout> = export.map(|e| read_json(e)).transpose()?;
    if let (Some(e), Some(readout)) = (export, &export_readout) {
        check_readout(&snap, readout, true, e)?;
    }

    let sequences: Vec<Vec<u32>> = plain_readout
        .sequences
        .iter()
        .map(|q| q.tokens.clone())
        .collect();
    if let Some(readout) = &export_readout {
        if readout
            .sequences
            .iter()
            .map(|q| &q.tokens)
            .ne(sequences.iter())
        {
            return Err("the two readouts hold different sequences".to_owned());
        }
    }

    let artifact = export_fbs_trunk_uorlut01(&snap).map_err(|e| format!("export: {e}"))?;
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("parse: {e}"))?;
    let singles: Vec<Vec<u32>> = (0..snap.vocab as u32).map(|t| vec![t]).collect();
    let (plain_singles, plain_sequences) = floats(&plain_readout);
    let export = export_readout.as_ref().map(floats);
    let report = json!({
        "artifact_sha256": hex::encode(Sha256::digest(&artifact)),
        "single_tokens": section(
            &snap,
            &model,
            &singles,
            &plain_singles,
            export.as_ref().map(|e| e.0.as_slice()),
        )?,
        "sequences": section(
            &snap,
            &model,
            &sequences,
            &plain_sequences,
            export.as_ref().map(|e| e.1.as_slice()),
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
