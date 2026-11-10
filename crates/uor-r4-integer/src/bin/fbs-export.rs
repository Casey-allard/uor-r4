//! Export a SpiralCore/FBS float-trunk snapshot to a `uor-r4.lut-stack/4`
//! `UORLUT01` artifact (d11-exporter-float-trunk, claude-123).
//!
//! The snapshot is the JSON that `uor-r4-float-train`'s `fbs-snapshot` writes
//! from a saved model. The float converter builds it without the serving
//! build, and this serving-build binary turns it into the artifact.
//!
//! ```text
//! fbs-export [--raw-gates] [--raw-transition] [--run-config CONFIG_JSON]
//!     --stack-contract TOML --stack-contract-semantic-blake3 HEX
//!     --tokenizer TOKENIZER_JSON [--context C] SNAPSHOT_JSON OUT_PATH
//! ```
//!
//! `--context C` serves the model past its trained window under carry (integer-long-context,
//! claude-288): the artifact's context is C, its turn memory keys over the whole served window,
//! and `source.trained_context` records the snapshot's own context. It refuses a C below that
//! context or above 65,536. Without it the artifact is byte for byte the trained-window export.
//!
//! `--raw-gates` serves the two Hamilton gates, and `--raw-transition` the
//! recurrence transition, unsnapped: the unit quaternion of each 4-block by
//! integer square root and division, under `uor-r4.lut-stack/5` (claude-181).
//! With neither, the gates and the transition snap to the 120 icosians.
//!
//! A snapshot of a model with ask heads (claude-330) holds them under `hamming_ask`, one entry
//! per Hamming read, with the readout read's question carry (claude-335). They are served from
//! the artifact's `hamming.d{depth}.ask` and `.carry` sections (claude-336): the question table
//! at Q12, the gains at Q16. A snapshot without the key exports byte for byte as before.
//!
//! `--run-config` reads the run's `config.json` for its readout (integer-nearest-readout,
//! claude-231) and served contract. A `cell` or `nearest` objective serves the whole table's
//! nearest token (`head = "native_vector_nearest"`), for a snapshot taken before `fbs-snapshot`
//! recorded it. A run with `train_as_served: true` requires both raw flags. It also refuses a
//! config whose `retain_rates` disagrees with the snapshot's, or whose objective reads the writer
//! for a snapshot that reads nearest. A config with `hamming_copy_latest: true` (claude-345) sets
//! the Hamming record's `copy_latest`, so the copy heads serve their latest exact match; the
//! snapshot does not carry that rule, so such a run is exported with its config. A config with a
//! `turn_memory_tail` above 0 (claude-345) exports the memory chain with tail keys of that many
//! turn ids, and one without the key or with 0 exports none.
//!
//! It parses the artifact back before writing it, never replaces or leaves a
//! partial `OUT_PATH` (`fbs_snapshot::write_new_file`), and prints its sha256.
//! New exports bind the exact artifact identity under schema
//! `uor-r4.lut-stack/10`: `--stack-contract`,
//! `--stack-contract-semantic-blake3` and `--tokenizer` are required. The
//! product chat path compares the tokenizer bytes with this binding.
//! `--legacy-artifact` is the explicit migration path for reproducing an older
//! `/6`–`/9` artifact without provenance.
//!
//! ```text
//! fbs-export --point-key [--legacy-artifact] [--point-vote-k N]
//!     ARTIFACT TOKENS_U16 RESPONSE_MASK_U8 OUT_PATH
//! ```
//!
//! adds the served point key (claude-222, `point-key-integer`) to a turn-memory
//! artifact. A bound `/10` input remains `/10`. A historical `/7` input becomes
//! `/8` and requires `--legacy-artifact`. The table holds TRAIN's openings, from
//! a `UORT` token file and its response mask, in the float sheet's 192-token
//! windows. At each opening is the integer trunk's own Q12 query, read with the
//! memory off, and its target. `--point-vote-k N` records a vote over the N rows
//! with the least exact `(L1 distance, row)` order; omitting it preserves the
//! original one-nearest-row output byte for byte. It also prints the table's rows.

use std::path::Path;
use std::process::ExitCode;

use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::{
    quantize_hamming_ask, run_readout_nearest, write_new_file, FbsHammingAskRead,
    FBS_HAMMING_ASK_KEY,
};
use uor_r4_integer::stack::{
    add_hamming_ask_uorlut01, add_point_key_uorlut01, build_point_key, check_point_key_read_back,
    export_fbs_trunk_uorlut01_with_provenance, point_key_windows, set_hamming_copy_latest_uorlut01,
    AskImage, FbsExportProvenance, FbsExportSnaps, FbsTrunkSnapshot, IntegerStackModel,
    StackContractProvenance, POINT_KEY_BOS, POINT_KEY_WINDOW,
};

const USAGE: &str = "usage: fbs-export [--raw-gates] [--raw-transition] [--run-config CONFIG_JSON] --stack-contract TOML --stack-contract-semantic-blake3 HEX --tokenizer TOKENIZER_JSON [--context C] SNAPSHOT_JSON OUT_PATH\n       fbs-export --legacy-artifact [--raw-gates] [--raw-transition] [--run-config CONFIG_JSON] [--context C] SNAPSHOT_JSON OUT_PATH\n       fbs-export --point-key [--legacy-artifact] [--point-vote-k N] ARTIFACT TOKENS_U16 RESPONSE_MASK_U8 OUT_PATH";

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

fn validate_binding_flags(
    legacy: bool,
    contract: Option<&str>,
    semantic: Option<&str>,
    tokenizer: Option<&str>,
) -> Result<bool, String> {
    if legacy {
        if contract.is_some() || semantic.is_some() || tokenizer.is_some() {
            return Err(
                "--legacy-artifact cannot be mixed with contract or tokenizer binding flags"
                    .to_owned(),
            );
        }
        return Ok(false);
    }
    let contract = contract.ok_or_else(|| format!("--stack-contract is required: {USAGE}"))?;
    if contract.trim().is_empty() {
        return Err("--stack-contract must not be empty".to_owned());
    }
    let semantic = semantic.ok_or_else(|| {
        "--stack-contract-semantic-blake3 is required with --stack-contract".to_owned()
    })?;
    if semantic.len() != 64 || !semantic.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(
            "--stack-contract-semantic-blake3 must be 64 hexadecimal characters".to_owned(),
        );
    }
    if tokenizer.is_none() {
        return Err(format!("--tokenizer is required: {USAGE}"));
    }
    Ok(true)
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
fn point_key(paths: &[String], vote_k: usize, legacy: bool) -> Result<(), String> {
    let [artifact, tokens, mask, out] = paths else {
        return Err(USAGE.to_owned());
    };
    let bytes = std::fs::read(artifact).map_err(|e| format!("{artifact}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("{artifact}: {e}"))?;
    if !legacy && model.tokenizer_sha256().is_none() {
        return Err(
            "--point-key on a legacy artifact needs --legacy-artifact; new inputs must be /10"
                .to_owned(),
        );
    }
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
        let legacy = point_args.first().map(String::as_str) == Some("--legacy-artifact");
        if legacy {
            point_args = &point_args[1..];
        }
        if point_args.first().map(String::as_str) == Some("--point-vote-k") {
            let value = point_args
                .get(1)
                .ok_or_else(|| format!("--point-vote-k needs a value: {USAGE}"))?;
            vote_k = value
                .parse()
                .map_err(|_| format!("--point-vote-k needs a positive integer: {value}"))?;
            point_args = &point_args[2..];
        }
        return point_key(point_args, vote_k, legacy);
    }
    let mut snaps = FbsExportSnaps::SNAPPED;
    let (mut paths, mut run_config) = (Vec::new(), None::<String>);
    // The copy rule the run trained (claude-345), read from its config below.
    let mut copy_latest = false;
    // The memory chain's tail the run trained (claude-345), 0 for none, read from its config below.
    let mut chain_tail = 0usize;
    let mut carry = None::<usize>;
    let mut stack_contract = None::<String>;
    let mut stack_contract_semantic = None::<String>;
    let mut provenance_tokenizer = None::<String>;
    let mut legacy_artifact = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--raw-gates" if !snaps.gates => return Err("--raw-gates given twice".to_owned()),
            "--raw-transition" if !snaps.transition => {
                return Err("--raw-transition given twice".to_owned())
            }
            "--raw-gates" => snaps.gates = false,
            "--raw-transition" => snaps.transition = false,
            "--legacy-artifact" if legacy_artifact => {
                return Err("--legacy-artifact given twice".to_owned())
            }
            "--legacy-artifact" => legacy_artifact = true,
            "--run-config" if run_config.is_some() => {
                return Err("--run-config given twice".to_owned())
            }
            "--run-config" => {
                run_config = Some(
                    args.next()
                        .ok_or_else(|| format!("--run-config needs a path: {USAGE}"))?,
                )
            }
            "--context" if carry.is_some() => return Err("--context given twice".to_owned()),
            "--context" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("--context needs a value: {USAGE}"))?;
                carry = Some(
                    value
                        .parse()
                        .ok()
                        .filter(|&c: &usize| c > 0)
                        .ok_or_else(|| format!("--context needs a positive integer: {value}"))?,
                );
            }
            "--stack-contract" if stack_contract.is_some() => {
                return Err("--stack-contract given twice".to_owned())
            }
            "--stack-contract" => {
                stack_contract = Some(
                    args.next()
                        .ok_or_else(|| format!("--stack-contract needs a path: {USAGE}"))?,
                )
            }
            "--stack-contract-semantic-blake3" if stack_contract_semantic.is_some() => {
                return Err("--stack-contract-semantic-blake3 given twice".to_owned())
            }
            "--stack-contract-semantic-blake3" => {
                stack_contract_semantic = Some(args.next().ok_or_else(|| {
                    format!("--stack-contract-semantic-blake3 needs a value: {USAGE}")
                })?)
            }
            "--tokenizer" if provenance_tokenizer.is_some() => {
                return Err("--tokenizer given twice".to_owned())
            }
            "--tokenizer" => {
                provenance_tokenizer = Some(
                    args.next()
                        .ok_or_else(|| format!("--tokenizer needs a path: {USAGE}"))?,
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
    // The ask heads (claude-336) ride beside the trunk snapshot under their own key; the rest
    // must be a trunk snapshot, which refuses any other field.
    let mut value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{snapshot}: {e}"))?;
    let asks: Option<Vec<Option<FbsHammingAskRead>>> = match value
        .as_object_mut()
        .and_then(|o| o.remove(FBS_HAMMING_ASK_KEY))
    {
        None => None,
        Some(asks) => Some(
            serde_json::from_value(asks)
                .map_err(|e| format!("{snapshot}: {FBS_HAMMING_ASK_KEY}: {e}"))?,
        ),
    };
    let mut snap: FbsTrunkSnapshot =
        serde_json::from_value(value).map_err(|e| format!("{snapshot}: {e}"))?;
    // A block in which no read asks is the plain snapshot (claude-336 review).
    let asks = match asks {
        None => None,
        Some(asks) if asks.iter().all(Option::is_none) => None,
        Some(asks) => {
            let hamming = snap.hamming.as_ref().ok_or_else(|| {
                format!("{snapshot}: {FBS_HAMMING_ASK_KEY} without Hamming reads")
            })?;
            if asks.len() != hamming.depths.len() {
                return Err(format!(
                    "{snapshot}: {FBS_HAMMING_ASK_KEY} has {} entries for {} Hamming reads",
                    asks.len(),
                    hamming.depths.len()
                ));
            }
            let images = asks
                .iter()
                .zip(&hamming.depths)
                .map(|(ask, &depth)| {
                    ask.as_ref()
                        .map(|ask| {
                            quantize_hamming_ask(depth, ask).and_then(|(table, carry)| {
                                AskImage::checked(Some(table), carry, snap.vocab, snap.width)
                                    .map_err(|e| e.to_string())
                            })
                        })
                        .transpose()
                        .map(Option::flatten)
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("{snapshot}: {e}"))?;
            Some(images)
        }
    };
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
        copy_latest = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| format!("{config}: {e}"))?
            .get("hamming_copy_latest")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let tail = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| format!("{config}: {e}"))?
            .get("turn_memory_tail")
            .map_or(Ok(0), |value| {
                // A tail that is not a whole number is refused: read as 0 it would export no chain.
                value.as_u64().ok_or_else(|| {
                    format!("{config}: turn_memory_tail {value} is not a whole number")
                })
            })?;
        chain_tail = usize::try_from(tail)
            .map_err(|_| format!("{config}: turn_memory_tail {tail} is too large"))?;
    }
    // A run trained with a memory chain (claude-345) writes its record with the export, so the
    // export reads its reply keys back at that exact tail.
    let bound = validate_binding_flags(
        legacy_artifact,
        stack_contract.as_deref(),
        stack_contract_semantic.as_deref(),
        provenance_tokenizer.as_deref(),
    )?;
    let stack_contract = match stack_contract {
        None if !bound => None,
        None => unreachable!("binding flags were validated"),
        Some(path) => {
            let bytes = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
            if bytes.len() > 1 << 20 {
                return Err(format!("{path}: stack contract is larger than 1 MiB"));
            }
            Some(StackContractProvenance {
                source: path,
                sha256: hex::encode(Sha256::digest(&bytes)),
                semantic_blake3: stack_contract_semantic
                    .as_deref()
                    .ok_or_else(|| {
                        "--stack-contract-semantic-blake3 is required with --stack-contract"
                            .to_owned()
                    })?
                    .to_ascii_lowercase(),
            })
        }
    };
    let tokenizer_sha256 = match provenance_tokenizer {
        None if !bound => None,
        None => unreachable!("binding flags were validated"),
        Some(path) => {
            let bytes = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
            Some(hex::encode(Sha256::digest(&bytes)))
        }
    };
    let provenance = match (stack_contract, tokenizer_sha256) {
        (None, None) if !bound => None,
        (Some(stack_contract), Some(tokenizer_sha256)) => Some(FbsExportProvenance {
            stack_contract,
            tokenizer_sha256,
        }),
        _ => return Err(format!("a bound export needs both identities: {USAGE}")),
    };
    let artifact = export_fbs_trunk_uorlut01_with_provenance(
        &snap,
        snaps,
        carry,
        chain_tail,
        provenance.as_ref(),
    )
    .map_err(|e| format!("export: {e}"))?;
    let artifact = match asks {
        None => artifact,
        Some(asks) => {
            add_hamming_ask_uorlut01(&artifact, &asks).map_err(|e| format!("the ask heads: {e}"))?
        }
    };
    // A run trained with `--hamming-copy-latest` serves the latest exact match: the rule is in
    // its config, not in the snapshot, and goes into the Hamming record here.
    let artifact = if copy_latest {
        set_hamming_copy_latest_uorlut01(&artifact).map_err(|e| format!("the copy rule: {e}"))?
    } else {
        artifact
    };
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

    #[test]
    fn binding_is_mandatory_and_legacy_is_explicit() {
        let semantic = "a".repeat(64);
        assert!(validate_binding_flags(false, None, None, None).is_err());
        assert!(
            validate_binding_flags(false, Some("contract.toml"), Some(&semantic), None).is_err()
        );
        assert!(validate_binding_flags(
            false,
            Some("contract.toml"),
            Some("abc"),
            Some("tokenizer.json")
        )
        .is_err());
        assert_eq!(
            validate_binding_flags(
                false,
                Some("contract.toml"),
                Some(&semantic),
                Some("tokenizer.json")
            ),
            Ok(true)
        );
        assert_eq!(validate_binding_flags(true, None, None, None), Ok(false));
        assert!(validate_binding_flags(true, Some("contract.toml"), None, None).is_err());
        assert!(validate_binding_flags(true, None, None, Some("tokenizer.json")).is_err());
    }
}
