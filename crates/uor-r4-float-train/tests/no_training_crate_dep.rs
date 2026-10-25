//! Guard tests from the D37 reactivation architect review (`#2823`):
//! 1. `cargo tree -p uor-r4-float-train` must never list `uor-r4-training` or `candle-nn`.
//! 2. `uor-r4-float-train` carries all 24 kernel entry points (`22` in `src/cuda_stack.cu` +
//!    `2` in `src/spiralcore_fbs.cu`) and zero banned PTX/source tokens.

use std::path::PathBuf;
use std::process::Command;

const EXPECTED_24_KERNELS: [&str; 24] = [
    "straight_through_fwd",
    "hamilton_gate_fwd",
    "hamilton_gate_bwd",
    "rate_select_fwd",
    "rate_select_bwd",
    "rms_norm_fwd",
    "rms_norm_bwd_dx",
    "rms_norm_dw_partial",
    "rms_norm_dw_reduce",
    "quaternion_scan_fwd",
    "quaternion_scan_bwd",
    "margin_rank_rows",
    "margin_rank_grad",
    "recurrence_scan_fwd",
    "recurrence_tile_fold",
    "recurrence_tile_carry",
    "recurrence_tile_expand",
    "recurrence_bwd_params",
    "poison_words",
    "adam_update",
    "sq_lanes",
    "sq_tree",
    "spiralcore_fbs_fwd",
    "spiralcore_fbs_bwd",
];

fn cargo_bin() -> PathBuf {
    if let Some(c) = std::env::var_os("CARGO") {
        return PathBuf::from(c);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let p = PathBuf::from(home).join(".cargo/bin/cargo");
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from("cargo")
}

#[test]
fn cargo_tree_excludes_uor_r4_training_and_candle_nn() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(cargo_bin())
        .arg("tree")
        .arg("--offline")
        .arg("-p")
        .arg("uor-r4-float-train")
        .current_dir(&manifest_dir)
        .output()
        .expect("run cargo tree -p uor-r4-float-train");
    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8_lossy(&output.stdout);
    assert!(
        !tree.contains("uor-r4-training"),
        "D37 violation: uor-r4-float-train dependency graph must not contain uor-r4-training:\n{tree}"
    );
    assert!(
        !tree.contains("candle-nn"),
        "D29 violation: uor-r4-float-train dependency graph must not contain candle-nn:\n{tree}"
    );
}

#[test]
fn twenty_four_kernel_entry_points_match_cuda_sources_and_pass_d29_scan() {
    assert_eq!(
        uor_r4_float_train::KERNEL_ENTRY_POINTS,
        EXPECTED_24_KERNELS,
        "KERNEL_ENTRY_POINTS must list all 24 kernels in exact order"
    );

    let cuda_stack = include_str!("../src/cuda_stack.cu");
    let spiralcore_fbs = include_str!("../src/spiralcore_fbs.cu");
    let combined = format!("{cuda_stack}\n{spiralcore_fbs}");

    let mut found = Vec::new();
    for line in combined.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("extern \"C\" __global__ void ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            found.push(name);
        }
    }
    assert_eq!(
        found.len(),
        24,
        "expected 24 __global__ kernels across cuda_stack.cu + spiralcore_fbs.cu, found {}: {found:?}",
        found.len()
    );
    let found_refs: Vec<&str> = found.iter().map(String::as_str).collect();
    assert_eq!(found_refs, EXPECTED_24_KERNELS);

    let lower = combined.to_ascii_lowercase();
    for banned in [
        concat!("soft", "max"),
        concat!("soft", "plus"),
        concat!("sig", "moid"),
        concat!("ta", "nh"),
        concat!("ge", "lu"),
        concat!("si", "lu"),
        "cublas",
        "gemm",
        concat!("w", "mma"),
        concat!("mma", "_sync"),
    ] {
        assert!(
            !lower.contains(banned),
            "CUDA kernel source contains banned token {banned}"
        );
    }
}
