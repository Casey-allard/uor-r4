//! Ahead-of-time integer CUDA PTX build for C-R5 kernel-level status-12 checks.
//! Host builds (no `cuda` feature) return immediately without invoking nvcc.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::{env, fs, path::PathBuf, process::Command};

const ENTRIES: [&str; 11] = [
    "discrete_init",
    "discrete_encode",
    "discrete_forward",
    "discrete_counterfactual",
    "discrete_choose",
    "discrete_apply",
    "discrete_validate",
    "discrete_evaluate",
    "discrete_evaluation_aggregate",
    "discrete_count",
    "discrete_count_score",
];

fn admit_ptx(ptx: &str) -> Result<(), String> {
    let mut entries = BTreeSet::new();
    let mut expect_entry = false;
    for (line_number, line) in ptx.lines().enumerate() {
        let instruction = line.split("//").next().unwrap_or_default();
        for token in instruction
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
            .filter(|token| !token.is_empty())
        {
            let components: Vec<_> = token.split('.').collect();
            if components.iter().any(|component| {
                matches!(
                    *component,
                    "f16"
                        | "f16x2"
                        | "f32"
                        | "f64"
                        | "bf16"
                        | "bf16x2"
                        | "tf32"
                        | "e4m3"
                        | "e5m2"
                        | "mma"
                        | "wmma"
                        | "wgmma"
                        | "tcgen05"
                        | "tensor"
                        | "tensormap"
                )
            }) || token.starts_with("ex2.approx")
            {
                return Err(format!(
                    "integer PTX admission rejected {token} at line {}",
                    line_number + 1,
                ));
            }
            if expect_entry {
                if token.contains('.') || !entries.insert(token.to_owned()) {
                    return Err(format!("invalid or duplicate PTX entry {token}"));
                }
                expect_entry = false;
            } else if token == ".entry" {
                expect_entry = true;
            }
        }
    }
    if expect_entry {
        return Err("unterminated PTX entry declaration".into());
    }
    let expected: BTreeSet<_> = ENTRIES.iter().map(|entry| entry.to_string()).collect();
    if entries != expected {
        return Err(format!(
            "integer PTX entry inventory differs: missing {:?}, unexpected {:?}",
            expected.difference(&entries).collect::<Vec<_>>(),
            entries.difference(&expected).collect::<Vec<_>>(),
        ));
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../../../crates/uor-r4-discrete/src/discrete.cu");
    println!("cargo:rerun-if-env-changed=NVCC");
    println!("cargo:rerun-if-env-changed=UOR_NVCC_ARCH");
    if env::var_os("CARGO_FEATURE_CUDA").is_none() {
        return Ok(());
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("Cargo did not supply OUT_DIR")?);
    let arch = env::var("UOR_NVCC_ARCH")
        .map_err(|_| "CUDA build requires the explicit admitted UOR_NVCC_ARCH")?;
    if !arch
        .strip_prefix("compute_")
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err("UOR_NVCC_ARCH must be compute_<decimal capability>".into());
    }
    let status = Command::new(env::var("NVCC").unwrap_or_else(|_| "nvcc".into()))
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .arg("-o")
        .arg(out.join("discrete.ptx"))
        .arg("../../../../crates/uor-r4-discrete/src/discrete.cu")
        .status()?;
    if !status.success() {
        return Err(format!("ahead-of-time nvcc discrete build failed: {status}").into());
    }
    admit_ptx(&fs::read_to_string(out.join("discrete.ptx"))?)?;
    Ok(())
}
