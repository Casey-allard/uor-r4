//! Offline extraction of source-only development inputs. No model is loaded.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io, path::PathBuf};
use uor_r4_integer::report_output;
#[path = "support/source_probe.rs"]
mod support;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn run() -> Result<()> {
    let argv: Vec<_> = std::env::args().skip(1).collect();
    if argv.len() != 2 {
        return Err(invalid(
            "usage: prepare-geometric-dependent-read-probe SEALED_RADIAL_COMPILED_PANEL NEW_OUT",
        )
        .into());
    }
    let panel_path = PathBuf::from(&argv[0]);
    let out = PathBuf::from(&argv[1]);
    let output = support::prospective_output(&out)?;
    let source = fs::canonicalize(&panel_path)?;
    for ancestor in source.ancestors() {
        if ancestor.is_dir()
            && ancestor.join(report_output::MANIFEST_FILE).exists()
            && output.starts_with(ancestor)
        {
            return Err(invalid("derived output beneath sealed input ancestor").into());
        }
    }
    let parent = source
        .parent()
        .ok_or_else(|| invalid("panel parent absent"))?;
    if output.starts_with(parent) {
        return Err(invalid("derived output beneath input report").into());
    }
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        report_output::verify(parent)?;
        use std::io::Read;
        let mut bytes = Vec::new();
        fs::File::open(&source)?
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(invalid("compiled panel exceeds 4MiB").into());
        }
        let panel: Value = serde_json::from_slice(&bytes)?;
        let training = panel["training64"]
            .as_array()
            .ok_or_else(|| invalid("training64 absent"))?;
        if training.len() != 64 || panel["all64_each_update"] != true {
            return Err(invalid("radial development panel contract differs").into());
        }
        let mut ids = BTreeSet::new();
        let mut cases = Vec::with_capacity(64);
        for row in training {
            let id = row["id"]
                .as_str()
                .filter(|id| !id.is_empty())
                .ok_or_else(|| invalid("case ID absent"))?;
            if !ids.insert(id) {
                return Err(invalid("duplicate development case ID").into());
            }
            let mut packet = serde_json::Map::new();
            for key in [
                "id",
                "record",
                "commit",
                "entity",
                "relation",
                "original_source_ids",
                "query_ids",
            ] {
                let value = row
                    .get(key)
                    .ok_or_else(|| invalid("required source input absent"))?;
                packet.insert(key.into(), value.clone());
            }
            // Exact retained Episode::frame policy; no source view or label is copied.
            packet.insert("scope".into(), json!("m-world-v2"));
            packet.insert("view".into(), json!(0));
            packet.insert("status".into(), json!("Found"));
            packet.insert("actual_prefix_ids".into(), json!([]));
            cases.push(Value::Object(packet));
        }
        fs::write(
            out.join("development64-inputs.json"),
            serde_json::to_vec_pretty(&json!({
                "schema":"uor-r4.native-source-realizer-probe-input/1", "cases":cases
            }))?,
        )?;
        fs::write(
            out.join("preparation.json"),
            serde_json::to_vec_pretty(&json!({
                "schema":"uor-r4.geometric-dependent-read-preparation/1",
                "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
                "panel_sha256":hex::encode(Sha256::digest(&bytes)),
                "cases":64,"optimizer_updates":0,
                "frame_policy":"retained Episode::frame: m-world-v2, view0, Found, empty actual prefix",
                "scope":"offline field extraction; no targets, accepted answers, literals, oracle, desired length or source views copied to inference"
            }))?,
        )?;
        report_output::seal(&out)?;
        report_output::verify(&out)?;
        Ok(())
    })();
    if let Err(error) = &result {
        if !out.join(report_output::MANIFEST_FILE).exists() {
            fs::write(
                out.join("failure.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema":"uor-r4.geometric-dependent-read-preparation-failure/1",
                    "optimizer_updates":0,"error":error.to_string()
                }))?,
            )?;
            report_output::seal(&out)?;
            report_output::verify(&out)?;
        }
    }
    result
}

fn main() {
    if let Err(error) = run() {
        eprintln!("prepare geometric dependent read: {error}");
        std::process::exit(1);
    }
}
