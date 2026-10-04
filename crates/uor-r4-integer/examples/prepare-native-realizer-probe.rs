//! Offline extraction of immutable source-only inputs; no model is loaded.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use uor_r4_integer::report_output;
#[path = "support/source_probe.rs"]
mod support;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn bytes(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(invalid("input cap exceeded").into());
    }
    Ok(bytes)
}
fn packet(label: &Value, frame: Option<&Value>) -> Result<Value> {
    let mut result = serde_json::Map::new();
    for key in [
        "id",
        "record",
        "commit",
        "entity",
        "relation",
        "original_source_ids",
        "query_ids",
    ] {
        let value = label
            .get(key)
            .ok_or_else(|| invalid("required source field absent"))?;
        if let Some(frame) = frame {
            if key != "id" && frame.get(key) != Some(value) {
                return Err(invalid("saved source packet and source label differ").into());
            }
        }
        result.insert(key.into(), value.clone());
    }
    // These are the retained Episode::frame inputs, not answer-derived values.
    for (key, value) in [
        ("scope", json!("m-world-v2")),
        ("view", json!(0)),
        ("status", json!("Found")),
        ("actual_prefix_ids", json!([])),
    ] {
        if let Some(frame) = frame {
            if frame.get(key) != Some(&value) {
                return Err(invalid("saved frame contract differs").into());
            }
        }
        result.insert(key.into(), value);
    }
    Ok(Value::Object(result))
}
fn run() -> Result<()> {
    let argv: Vec<_> = std::env::args().skip(1).collect();
    if argv.len() != 3 {
        return Err(invalid(
            "usage: prepare-native-realizer-probe FOUR_PROBES FROZEN_PANEL NEW_OUT",
        )
        .into());
    }
    let four_path = PathBuf::from(&argv[0]);
    let panel_path = PathBuf::from(&argv[1]);
    let out = PathBuf::from(&argv[2]);
    let output = support::prospective_output(&out)?;
    for source in [&four_path, &panel_path] {
        for ancestor in fs::canonicalize(source)?.ancestors() {
            if ancestor.is_dir()
                && ancestor.join(report_output::MANIFEST_FILE).exists()
                && output.starts_with(ancestor)
            {
                return Err(
                    invalid("derived output beneath a sealed input ancestor is forbidden").into(),
                );
            }
        }
        if output.starts_with(fs::canonicalize(
            source
                .parent()
                .ok_or_else(|| invalid("input parent absent"))?,
        )?) {
            return Err(invalid("derived output beneath input report is forbidden").into());
        }
    }
    report_output::claim(&out)?;
    let four_bytes = bytes(&four_path)?;
    let panel_bytes = bytes(&panel_path)?;
    let four: Value = serde_json::from_slice(&four_bytes)?;
    let panel: Value = serde_json::from_slice(&panel_bytes)?;
    let labels = four["probes"]
        .as_array()
        .ok_or_else(|| invalid("four labels absent"))?;
    let frames = four["source_packets"]
        .as_array()
        .ok_or_else(|| invalid("four source packets absent"))?;
    let original = panel["original"]
        .as_array()
        .ok_or_else(|| invalid("original cases absent"))?;
    let construction = panel["construction"]
        .as_array()
        .ok_or_else(|| invalid("construction cases absent"))?;
    if labels.len() != 4 || frames.len() != 4 || original.len() != 20 || construction.len() != 8 {
        return Err(invalid("retained case counts differ").into());
    }
    let probes = labels
        .iter()
        .zip(frames)
        .map(|(label, frame)| packet(label, Some(frame)))
        .collect::<Result<Vec<_>>>()?;
    let development = original
        .iter()
        .chain(construction)
        .map(|label| packet(label, None))
        .collect::<Result<Vec<_>>>()?;
    for (name, cases) in [
        ("four-inputs.json", probes),
        ("development28-inputs.json", development),
    ] {
        fs::write(
            out.join(name),
            serde_json::to_vec_pretty(
                &json!({"schema":"uor-r4.native-source-realizer-probe-input/1","cases":cases}),
            )?,
        )?;
    }
    fs::write(
        out.join("preparation.json"),
        serde_json::to_vec_pretty(&json!({
        "schema":"uor-r4.native-realizer-probe-preparation/1",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "four_input_sha256":hex::encode(Sha256::digest(&four_bytes)),
        "panel_input_sha256":hex::encode(Sha256::digest(&panel_bytes)),
        "four_cases":4,"development_cases":28,"optimizer_updates":0,
        "frame_contract":"retained Episode::frame: m-world-v2, view0, Found, empty own-prefix; four source packets cross-checked",
        "scope":"offline field extraction only; outputs exclude targets, accepted answers, literals, desired length and unchecked source views"}))?,
    )?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("prepare native realizer probe: {error}");
        std::process::exit(1);
    }
}
