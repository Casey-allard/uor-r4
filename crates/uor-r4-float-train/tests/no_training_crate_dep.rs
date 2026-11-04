//! Guard tests from the D37 reactivation architect review (`#2823`):
//! 1. `cargo tree -p uor-r4-float-train` must never list `uor-r4-training` or `candle-nn`.
//! 2. Each PTX module's generated kernel list (`uor_r4_float_train::kernel_lists`, written by
//!    `build.rs`) is exactly the entry points this test finds in that module's sources itself,
//!    no kernel sits inside a conditional block, every `src/*.cu` file is in some module, and no
//!    source carries a banned token.

use std::path::PathBuf;
use std::process::Command;

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

/// The `__global__` entry points this test finds in `text`, in declaration order, scanned here
/// rather than by `build.rs`, so a scan fault on either side shows as a disagreement.
fn declared_kernels(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("extern \"C\" __global__ void ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            found.push(name);
        }
    }
    found
}

/// The lines of one source that name `__global__` inside an `#if`, `#ifdef` or `#ifndef`
/// block, counted here rather than by `build.rs`. A kernel there may be compiled by one build
/// and not another, so neither scan may list it.
fn globals_inside_conditionals(text: &str) -> Vec<usize> {
    let mut depth = 0i32;
    let mut inside = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if let Some(directive) = line.trim_start().strip_prefix('#') {
            let directive = directive.trim_start();
            if directive.starts_with("endif") {
                depth -= 1;
            } else if directive.starts_with("if") {
                depth += 1;
            }
        }
        if depth > 0 && line.contains("__global__") {
            inside.push(index + 1);
        }
    }
    inside
}

#[test]
fn kernel_entry_points_match_cuda_sources_and_pass_d29_scan() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let modules = uor_r4_float_train::kernel_lists::MODULES;
    assert_eq!(
        uor_r4_float_train::KERNEL_ENTRY_POINTS,
        uor_r4_float_train::kernel_lists::CUDA_STACK_F32,
        "KERNEL_ENTRY_POINTS is the stack module's generated list"
    );

    let mut combined = String::new();
    for module in modules {
        let mut text = String::new();
        for source in module.sources {
            let path = manifest_dir.join(source);
            let body = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            let inside = globals_inside_conditionals(&body);
            assert!(
                inside.is_empty(),
                "{source}: `__global__` inside a conditional block at lines {inside:?}"
            );
            text.push_str(&body);
            text.push('\n');
        }
        let found = declared_kernels(&text);
        let found_refs: Vec<&str> = found.iter().map(String::as_str).collect();
        assert_eq!(
            found_refs, module.kernels,
            "{}: the generated list must be the kernels its sources {:?} declare",
            module.ptx, module.sources
        );
        assert!(
            text.matches("__global__").count() == found.len(),
            "{}: every `__global__` in its sources is a declaration the list holds",
            module.ptx
        );
        combined.push_str(&text);
    }

    let src = manifest_dir.join("src");
    for entry in std::fs::read_dir(&src).expect("read src/") {
        let name = entry.expect("a src/ entry").file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".cu") {
            let source = format!("src/{name}");
            assert!(
                modules
                    .iter()
                    .any(|module| module.sources.contains(&source.as_str())),
                "{source} is compiled by no module, so no kernel list covers it"
            );
        }
    }

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
