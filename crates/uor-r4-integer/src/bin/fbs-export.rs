//! Export a SpiralCore/FBS float-trunk snapshot to a `uor-r4.lut-stack/4`
//! `UORLUT01` artifact (d11-exporter-float-trunk, claude-123).
//!
//! The snapshot is the JSON that `uor-r4-float-train`'s `fbs-snapshot` writes
//! from a saved model. The float converter builds it without the serving
//! build, and this serving-build binary turns it into the artifact.
//!
//! ```text
//! fbs-export [--raw-gates] [--raw-transition] [--run-config CONFIG_JSON] SNAPSHOT_JSON OUT_PATH
//! ```
//!
//! `--raw-gates` serves the two Hamilton gates, and `--raw-transition` the
//! recurrence transition, unsnapped: the unit quaternion of each 4-block by
//! integer square root and division, under `uor-r4.lut-stack/5` (claude-181).
//! With neither, the gates and the transition snap to the 120 icosians.
//!
//! `--run-config` reads the run's `config.json` for its readout (integer-nearest-readout,
//! claude-231) and served contract. A `cell` or `nearest` objective serves the whole table's
//! nearest token (`head = "native_vector_nearest"`), for a snapshot taken before `fbs-snapshot`
//! recorded it. A run with `train_as_served: true` requires both raw flags. It also refuses a
//! config whose `retain_rates` disagrees with the snapshot's, or whose objective reads the writer
//! for a snapshot that reads nearest.
//!
//! It parses the artifact back before writing it, never replaces or leaves a
//! partial `OUT_PATH` (`fbs_snapshot::write_new_file`), and prints its sha256.
//!
//! ```text
//! fbs-export --point-key [--point-vote-k N] ARTIFACT TOKENS_U16 RESPONSE_MASK_U8 OUT_PATH
//! ```
//!
//! adds the served point key (claude-222, `point-key-integer`) to a turn-memory
//! `uor-r4.lut-stack/7` artifact, as `/8`. The table holds TRAIN's openings, from
//! a `UORT` token file and its response mask, in the float sheet's 192-token
//! windows. At each opening is the integer trunk's own Q12 query, read with the
//! memory off, and its target. `--point-vote-k N` records a vote over the N rows
//! with the least exact `(L1 distance, row)` order; omitting it preserves the
//! original one-nearest-row output byte for byte. It also prints the table's rows.

use std::path::Path;
use std::process::ExitCode;

use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::{run_readout_nearest, write_new_file};
use uor_r4_integer::stack::{
    add_point_key_uorlut01, build_point_key, check_point_key_read_back,
    export_fbs_trunk_uorlut01_with, point_key_windows, FbsExportSnaps, FbsTrunkSnapshot,
    IntegerStackModel, POINT_KEY_BOS, POINT_KEY_WINDOW,
};

const USAGE: &str = "usage: fbs-export [--raw-gates] [--raw-transition] [--run-config CONFIG_JSON] SNAPSHOT_JSON OUT_PATH\n       fbs-export --point-key [--point-vote-k N] ARTIFACT TOKENS_U16 RESPONSE_MASK_U8 OUT_PATH";

fn check_run_export_contract(config: &str, snaps: FbsExportSnaps) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(config).map_err(|error| format!("the run config: {error}"))?;
    let config = value
        .as_object()
        .ok_or_else(|| "the run config root is not an object".to_owned())?;
    let trained_as_served = match config.get("train_as_served") {
        None => false,
        Some(serde_json::Value::Bool(value)) => *value,
        Some(_) => return Err("the run config's train_as_served is not a bool".to_owned()),
    };
    if trained_as_served && (snaps.gates || snaps.transition) {
        return Err(
            "the run trained with --train-as-served 1 requires both --raw-gates and \
             --raw-transition"
                .to_owned(),
        );
    }
    Ok(())
}

/// A `UORT` token file's ids: its 64-byte header, whose count must match, then little-endian u16s.
fn uort_tokens(path: &str) -> Result<Vec<u32>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err(format!("{path}: not a UORT token file"));
    }
    let count = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| format!("{path}: truncated"))?,
    ) as usize;
    let payload = &bytes[64..];
    if count.checked_mul(2) != Some(payload.len()) {
        return Err(format!(
            "{path}: the header counts {count} tokens, the payload holds {} bytes",
            payload.len()
        ));
    }
    Ok(payload
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect())
}

/// `--point-key`: the `/8` stack of a `/7` one, with TRAIN's point table.
fn point_key(paths: &[String], vote_k: usize) -> Result<(), String> {
    let [artifact, tokens, mask, out] = paths else {
        return Err(USAGE.to_owned());
    };
    let bytes = std::fs::read(artifact).map_err(|e| format!("{artifact}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("{artifact}: {e}"))?;
    let tokens = uort_tokens(tokens)?;
    let mask = std::fs::read(mask).map_err(|e| format!("{mask}: {e}"))?;
    let windows = point_key_windows(&tokens, &mask, POINT_KEY_WINDOW, POINT_KEY_BOS)
        .map_err(|e| format!("windows: {e}"))?;
    let mut image = build_point_key(&model, &windows).map_err(|e| format!("the table: {e}"))?;
    image.vote_k = vote_k;
    let rows = image.rows();
    let with = add_point_key_uorlut01(&bytes, &image).map_err(|e| format!("the /8 stack: {e}"))?;
    IntegerStackModel::parse(&with).map_err(|e| format!("the /8 stack does not parse: {e}"))?;
    // The run's instrument (claude-228): the table reads back from these bytes as built.
    check_point_key_read_back(&with, &image).map_err(|e| format!("the /8 stack: {e}"))?;
    write_new_file(Path::new(out), &with).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}  {out}  {rows} openings in {} windows, vote_k={vote_k}",
        hex::encode(Sha256::digest(&with)),
        windows.len()
    );
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--point-key") {
        let mut point_args = &args[1..];
        let mut vote_k = 1usize;
        if point_args.first().map(String::as_str) == Some("--point-vote-k") {
            let value = point_args
                .get(1)
                .ok_or_else(|| format!("--point-vote-k needs a value: {USAGE}"))?;
            vote_k = value
                .parse()
                .map_err(|_| format!("--point-vote-k needs a positive integer: {value}"))?;
            point_args = &point_args[2..];
        }
        return point_key(point_args, vote_k);
    }
    let mut snaps = FbsExportSnaps::SNAPPED;
    let (mut paths, mut run_config) = (Vec::new(), None::<String>);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--raw-gates" if !snaps.gates => return Err("--raw-gates given twice".to_owned()),
            "--raw-transition" if !snaps.transition => {
                return Err("--raw-transition given twice".to_owned())
            }
            "--raw-gates" => snaps.gates = false,
            "--raw-transition" => snaps.transition = false,
            "--run-config" if run_config.is_some() => {
                return Err("--run-config given twice".to_owned())
            }
            "--run-config" => {
                run_config = Some(
                    args.next()
                        .ok_or_else(|| format!("--run-config needs a path: {USAGE}"))?,
                )
            }
            flag if flag.starts_with("--") => return Err(format!("{flag}: {USAGE}")),
            _ => paths.push(arg),
        }
    }
    let [snapshot, out] = paths.as_slice() else {
        return Err(USAGE.to_owned());
    };
    let bytes = std::fs::read(snapshot).map_err(|e| format!("{snapshot}: {e}"))?;
    let mut snap: FbsTrunkSnapshot =
        serde_json::from_slice(&bytes).map_err(|e| format!("{snapshot}: {e}"))?;
    // The run's readout (integer-nearest-readout, claude-231), for a snapshot taken before
    // `fbs-snapshot` recorded it. The config's rate reading must agree with the snapshot's.
    if let Some(config) = &run_config {
        let text = std::fs::read_to_string(config).map_err(|e| format!("{config}: {e}"))?;
        check_run_export_contract(&text, snaps).map_err(|error| format!("{config}: {error}"))?;
        let nearest = run_readout_nearest(&text).map_err(|e| format!("{config}: {e}"))?;
        if snap.readout_nearest && !nearest {
            return Err(format!(
                "{snapshot} decodes by nearest, but {config}'s objective decodes by the writer"
            ));
        }
        let retain = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| format!("{config}: {e}"))?
            .get("retain_rates")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if retain != snap.retain_rates {
            return Err(format!(
                "{config}'s retain_rates is {retain}, {snapshot}'s is {}",
                snap.retain_rates
            ));
        }
        snap.readout_nearest = nearest;
    }
    let artifact =
        export_fbs_trunk_uorlut01_with(&snap, snaps).map_err(|e| format!("export: {e}"))?;
    IntegerStackModel::parse(&artifact)
        .map_err(|e| format!("the exported artifact does not parse: {e}"))?;
    write_new_file(Path::new(out), &artifact).map_err(|e| format!("{out}: {e}"))?;
    println!("{}  {out}", hex::encode(Sha256::digest(&artifact)));
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fbs-export: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn served_training_requires_both_raw_flags() {
        let served = r#"{"steps": 8192, "train_as_served": true}"#;
        for snaps in [
            FbsExportSnaps::SNAPPED,
            FbsExportSnaps {
                gates: false,
                transition: true,
            },
            FbsExportSnaps {
                gates: true,
                transition: false,
            },
        ] {
            let error = check_run_export_contract(served, snaps).unwrap_err();
            assert!(
                error.contains("requires both --raw-gates and --raw-transition"),
                "{error}"
            );
        }
        assert!(check_run_export_contract(
            served,
            FbsExportSnaps {
                gates: false,
                transition: false,
            }
        )
        .is_ok());
    }

    #[test]
    fn legacy_and_false_contracts_keep_the_caller_choice() {
        for config in [
            r#"{"steps": 8192}"#,
            r#"{"steps": 8192, "train_as_served": false}"#,
        ] {
            assert!(check_run_export_contract(config, FbsExportSnaps::SNAPPED).is_ok());
            assert!(check_run_export_contract(
                config,
                FbsExportSnaps {
                    gates: false,
                    transition: false,
                }
            )
            .is_ok());
        }
    }

    #[test]
    fn malformed_served_flags_fail_closed() {
        for config in [
            r#"[]"#,
            r#""served""#,
            r#"{"train_as_served": null}"#,
            r#"{"train_as_served": 1}"#,
            r#"{"train_as_served": "true"}"#,
            r#"{"train_as_served": true"#,
        ] {
            assert!(
                check_run_export_contract(
                    config,
                    FbsExportSnaps {
                        gates: false,
                        transition: false,
                    }
                )
                .is_err(),
                "{config}"
            );
        }
    }
}
