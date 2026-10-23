//! `case::load_verified`: a synthetic full-shape case with its two receipts loads;
//! a one-byte change, a wrong receipt entry, a missing entry, a duplicate entry and a
//! missing receipt are each refused. Binary32 values are written here, outside `src/`.

use std::path::{Path, PathBuf};

use legal_exact_solver::case::{self, FAMILIES, GUARDS};
use legal_exact_solver::receipts::{sha256_hex, GRADIENT_RECEIPT, MARGIN_RECEIPT};
use serde_json::{json, Value};

fn write_le(dir: &Path, name: &str, values: &[f32]) -> String {
    let bytes: Vec<u8> = values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect();
    std::fs::write(dir.join(name), &bytes).unwrap();
    sha256_hex(&bytes)
}

fn margin(i: usize) -> String {
    format!("protected-margin-{i:03}.f32le")
}

/// Write a full-shape case (1920 coefficients, 380 guards) and its receipts.
fn write_case(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("case-verified-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut files = vec![
        json!({"file": "coupled-gradient-prefix.coefficients-term-00.f32le", "sha256": "ignored"}),
    ];
    let mut n = 0;
    for (i, (stem, count)) in FAMILIES.iter().enumerate() {
        let base = if i == 0 { 0.25 } else { -0.5 };
        let m: Vec<f32> = (0..*count).map(|j| base + 0.25 * (j % 3) as f32).collect();
        let g: Vec<f32> = (0..*count)
            .map(|j| 0.125 * ((j % 5) as f32 - 2.0))
            .collect();
        for (kind, vals) in [("initial-master", &m), ("gradient", &g)] {
            let file = format!("coupled-{stem}-{kind}.f32le");
            let sha = write_le(&dir, &file, vals);
            files.push(json!({"file": file, "kind": kind, "sha256": sha}));
        }
        n += count;
    }
    std::fs::write(
        dir.join(GRADIENT_RECEIPT),
        json!({"files": files}).to_string(),
    )
    .unwrap();
    let mut terms = Vec::new();
    for i in 0..GUARDS {
        let row: Vec<f32> = (0..n).map(|j| ((i + j) % 7) as f32 * 0.25).collect();
        let sha = write_le(&dir, &margin(i), &row);
        terms.push(json!({"file": margin(i), "guard_index": i, "sha256": sha}));
    }
    std::fs::write(
        dir.join(MARGIN_RECEIPT),
        json!({"terms": terms}).to_string(),
    )
    .unwrap();
    dir
}

fn edit_receipt(dir: &Path, receipt: &str, f: impl FnOnce(&mut Value)) {
    let path = dir.join(receipt);
    let mut v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    f(&mut v);
    std::fs::write(&path, v.to_string()).unwrap();
}

#[test]
fn synthetic_case_with_receipts_loads() {
    let dir = write_case("ok");
    let (case, verified) = case::load_verified(&dir).unwrap();
    assert_eq!(case.problem.n(), 1920);
    assert_eq!(case.guard_rows, GUARDS);
    assert_eq!(verified.len(), 384);
    assert_eq!(
        verified[0].file,
        "coupled-prefix.coefficients-initial-master.f32le"
    );
    assert_eq!(verified[4].file, margin(0));
    assert_eq!(verified[383].file, margin(379));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn one_changed_byte_is_refused_naming_the_file() {
    let dir = write_case("tampered");
    let file = margin(17);
    let path = dir.join(&file);
    let mut bytes = std::fs::read(&path).unwrap();
    let before = sha256_hex(&bytes);
    bytes[5] ^= 0x01;
    let after = sha256_hex(&bytes);
    std::fs::write(&path, bytes).unwrap();
    let err = case::load_verified(&dir).unwrap_err();
    assert!(
        err.contains(&file) && err.contains(&before) && err.contains(&after),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_wrong_receipt_entry_is_refused() {
    let dir = write_case("wrong-entry");
    let file = "coupled-generate.unary-gradient.f32le";
    let wrong = "0".repeat(64);
    edit_receipt(&dir, GRADIENT_RECEIPT, |v| {
        for e in v["files"].as_array_mut().unwrap() {
            if e["file"] == file {
                e["sha256"] = json!(wrong);
            }
        }
    });
    let err = case::load_verified(&dir).unwrap_err();
    assert!(err.contains(file) && err.contains(&wrong), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_and_duplicate_entries_and_a_missing_receipt_are_refused() {
    let dir = write_case("entries");
    let file = "coupled-prefix.coefficients-gradient.f32le";
    edit_receipt(&dir, GRADIENT_RECEIPT, |v| {
        let a = v["files"].as_array_mut().unwrap();
        let e = a.iter().find(|e| e["file"] == file).unwrap().clone();
        a.push(e);
    });
    let err = case::load_verified(&dir).unwrap_err();
    assert!(
        err.contains(file) && err.contains("more than once"),
        "{err}"
    );

    edit_receipt(&dir, GRADIENT_RECEIPT, |v| {
        v["files"]
            .as_array_mut()
            .unwrap()
            .retain(|e| e["file"] != file);
    });
    let err = case::load_verified(&dir).unwrap_err();
    assert!(err.contains(file) && err.contains("not listed"), "{err}");

    let dir2 = write_case("short-margin");
    edit_receipt(&dir2, MARGIN_RECEIPT, |v| {
        v["terms"].as_array_mut().unwrap().pop();
    });
    let err = case::load_verified(&dir2).unwrap_err();
    assert!(err.contains("379 terms"), "{err}");

    std::fs::remove_file(dir2.join(MARGIN_RECEIPT)).unwrap();
    let err = case::load_verified(&dir2).unwrap_err();
    assert!(err.contains(MARGIN_RECEIPT), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}
