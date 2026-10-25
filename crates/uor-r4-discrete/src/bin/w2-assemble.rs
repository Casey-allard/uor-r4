//! Assemble the W2 `uor-r4.read-code-capacity/1` record from a finished GPU capture.
//! Host-only counting over captured codes; no model execution.
//! Usage: w2-assemble CAPTURE_DIR NEW_RECORD_DIR --ratchets-pass true|false
//! (run from the root that holds `local/driver/data`; NEW_RECORD_DIR must not exist)

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use uor_r4_discrete::w2_capture::{
    sha256_hex, CAPTURE_FILE, QUERY_SIDECAR_FILE, TRAIN_STREAM_FILE,
};
use uor_r4_discrete::w2_record::{assemble, runtime_manifest_bytes, FilePins};

const DATA: &str = "local/driver/data";

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn hash_of(path: &str) -> Result<String, String> {
    Ok(sha256_hex(&read(&Path::new(DATA).join(path))?))
}

fn write_new(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let path = dir.join(name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn run(args: &[String]) -> Result<(), String> {
    let [capture_dir, out_dir, flag, value] = args else {
        return Err(
            "usage: w2-assemble CAPTURE_DIR NEW_RECORD_DIR --ratchets-pass true|false".into(),
        );
    };
    if flag != "--ratchets-pass" {
        return Err(format!("unknown flag {flag}"));
    }
    let ratchets_pass = match value.as_str() {
        "true" => true,
        "false" => false,
        other => return Err(format!("--ratchets-pass takes true or false, not {other}")),
    };
    let capture_dir = Path::new(capture_dir);
    let capture: serde_json::Value =
        serde_json::from_slice(&read(&capture_dir.join(CAPTURE_FILE))?)
            .map_err(|e| format!("{CAPTURE_FILE}: {e}"))?;
    let train_stream = read(&capture_dir.join(TRAIN_STREAM_FILE))?;
    let query_sidecar = read(&capture_dir.join(QUERY_SIDECAR_FILE))?;
    let dev_tokens = read(&Path::new(DATA).join("ft/dev/tokens.u16"))?;
    let dev_mask = read(&Path::new(DATA).join("ft/dev/response_mask.u8"))?;
    let pins = FilePins {
        dev_tokens_sha256: sha256_hex(&dev_tokens),
        dev_mask_sha256: sha256_hex(&dev_mask),
        train_tokens_sha256: hash_of("mw-balp/train/tokens.u16")?,
        train_mask_sha256: hash_of("mw-balp/train/response_mask.u8")?,
        tokenizer_sha256: hash_of("tokenizer.json")?,
    };
    let (record, manifest) = assemble(
        &capture,
        &train_stream,
        &query_sidecar,
        &dev_tokens,
        &dev_mask,
        &pins,
        ratchets_pass,
    )?;
    let out_dir = Path::new(out_dir);
    if let Some(parent) = out_dir.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::create_dir(out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;
    write_new(
        out_dir,
        "runtime-manifest.json",
        &runtime_manifest_bytes(&manifest)?,
    )?;
    let record_bytes = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
    write_new(out_dir, "record.json", &record_bytes)?;
    eprintln!(
        "w2-assemble: wrote {} (verdict {})",
        out_dir.display(),
        record["verdict"]
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
