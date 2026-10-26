//! Compare a converted SpiralCore/FBS trunk with the float GPU trunk it came
//! from (d11-exporter-float-trunk, claude-123).
//!
//! ```text
//! fbs-compare SNAPSHOT_JSON READOUT_JSON
//! ```
//!
//! `SNAPSHOT_JSON` is the converter's snapshot and `READOUT_JSON` the float
//! GPU trunk's own single-token outputs (`fbs_snapshot::FbsFloatReadout`).
//! It exports the snapshot, parses the artifact, and for every readout
//! position runs a fresh integer session and the snapshot's float reference,
//! plain and with its tables rounded as exported. It prints one JSON line of
//! counts: a reading, not a bar. It exits non-zero only when the snapshot does
//! not export, parse or serve, or the readout does not fit the snapshot.

use std::process::ExitCode;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::FbsFloatReadout;
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, FbsTrunkSnapshot, IntegerStackModel};

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [snapshot, readout] = args.as_slice() else {
        return Err("usage: fbs-compare SNAPSHOT_JSON READOUT_JSON".to_owned());
    };
    let snap: FbsTrunkSnapshot = read_json(snapshot)?;
    let readout: FbsFloatReadout = read_json(readout)?;
    if readout.vocab != snap.vocab || readout.width != snap.width {
        return Err("the readout's vocabulary or width differs from the snapshot's".to_owned());
    }
    let artifact = export_fbs_trunk_uorlut01(&snap).map_err(|e| format!("export: {e}"))?;
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("parse: {e}"))?;
    let (mut integer_equals_quantized, mut float_equals_reference) = (0usize, 0usize);
    let mut max_hidden_gap = 0f64;
    for p in &readout.positions {
        if p.hidden.len() != snap.width {
            return Err(format!(
                "token {}: hidden has {} values",
                p.token,
                p.hidden.len()
            ));
        }
        let step = model
            .session()
            .step_native(p.token)
            .map_err(|e| format!("token {}: integer step: {e}", p.token))?;
        let quantized = snap
            .forward_f32(&[p.token], true)
            .map_err(|e| format!("token {}: reference: {e}", p.token))?;
        let plain = snap
            .forward_f32(&[p.token], false)
            .map_err(|e| format!("token {}: reference: {e}", p.token))?;
        integer_equals_quantized += usize::from(step.token == quantized[0].token);
        float_equals_reference += usize::from(p.next == plain[0].token);
        for (h, q) in p.hidden.iter().zip(&plain[0].query) {
            max_hidden_gap = max_hidden_gap.max((f64::from(*h) - q).abs());
        }
    }
    println!(
        "{}",
        json!({
            "positions": readout.positions.len(),
            "integer_token_equals_quantized_reference": integer_equals_quantized,
            "float_gpu_token_equals_reference": float_equals_reference,
            "max_abs_float_hidden_minus_reference_query": max_hidden_gap,
            "artifact_sha256": hex::encode(Sha256::digest(&artifact)),
        })
    );
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
