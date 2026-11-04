//! Ahead-of-time CUDA kernels for the D37 offline float training module (CHARTER §3.4 / D29).
//!
//! Runtime CUDA compilation (NVRTC) is banned project-wide. When compiled with `--features cuda`,
//! `nvcc` compiles the kernel extract (`src/cuda_stack.cu` + `src/spiralcore_fbs.cu`) ahead of
//! time into `cuda_stack_f32.ptx` and `cuda_stack_bf16.ptx` inside `OUT_DIR`. The SpiralCore/FBS
//! sheet arms (`src/spiralcore_fbs_arms.cu`) build as their own module, `spiralcore_fbs_arms_f32.ptx`,
//! from a prelude, the unchanged `src/spiralcore_fbs.cu` (for its route helpers) and the arms.
//!
//! On every build, CPU-only laptop builds included, `build.rs` first writes `kernel_lists.rs`
//! into `OUT_DIR`: each module's `extern "C" __global__ void <name>` entry points, scanned from
//! the sources it compiles. `lib.rs` includes it, and the dispatch refuses a name its module does
//! not list. A CPU-only build then exits without invoking `nvcc`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PRECISE: [&str; 3] = ["-ftz=false", "-prec-div=true", "-prec-sqrt=true"];

struct Module {
    sources: &'static [&'static str],
    ptx: &'static str,
    options: &'static [&'static str],
    feature: &'static str,
}

const CUDA: &str = "CARGO_FEATURE_CUDA";

const MODULES: [Module; 3] = [
    Module {
        sources: &["src/cuda_stack.cu", "src/spiralcore_fbs.cu"],
        ptx: "cuda_stack_f32.ptx",
        options: &PRECISE,
        feature: CUDA,
    },
    Module {
        sources: &["src/cuda_stack.cu", "src/spiralcore_fbs.cu"],
        ptx: "cuda_stack_bf16.ptx",
        options: &[
            "-ftz=false",
            "-prec-div=true",
            "-prec-sqrt=true",
            "-DUOR_STORAGE_BF16",
        ],
        feature: CUDA,
    },
    Module {
        sources: &[
            "src/spiralcore_fbs_arms_prelude.cu",
            "src/spiralcore_fbs.cu",
            "src/spiralcore_fbs_arms.cu",
        ],
        ptx: "spiralcore_fbs_arms_f32.ptx",
        options: &PRECISE,
        feature: CUDA,
    },
];

fn main() {
    for module in &MODULES {
        for source in module.sources {
            println!("cargo:rerun-if-changed={source}");
        }
    }
    println!("cargo:rerun-if-env-changed=NVCC");
    println!("cargo:rerun-if-env-changed=UOR_NVCC_ARCH");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| {
        panic!("cargo did not set OUT_DIR for the ahead-of-time CUDA kernels")
    }));
    write_kernel_lists(&out);
    let built: Vec<&Module> = MODULES
        .iter()
        .filter(|module| env::var_os(module.feature).is_some())
        .collect();
    if built.is_empty() {
        return;
    }
    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".to_string());
    let arch = env::var("UOR_NVCC_ARCH").unwrap_or_else(|_| "compute_80".to_string());
    for module in built {
        let input = combined_source(&out, module);
        let status = Command::new(&nvcc)
            .arg("-ptx")
            .arg(format!("-arch={arch}"))
            .args(module.options)
            .arg("-o")
            .arg(out.join(module.ptx))
            .arg(&input)
            .status();
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => panic!("nvcc failed ({status}) compiling {}", module.ptx),
            Err(error) => panic!(
                "the cuda feature needs nvcc to build {} ahead of time \
                 (runtime compilation is banned): {error}",
                module.ptx
            ),
        }
    }
}

const DECLARATION: &str = "extern \"C\" __global__ void ";

/// The entry points of one module's sources, in declaration order. Every line that names
/// `__global__` must be a declaration this scan reads, so no kernel nvcc compiles is missed,
/// and none may sit inside an `#if`, `#ifdef` or `#ifndef` block: one build may compile such a
/// kernel and another not, which a text scan cannot tell apart.
fn kernels_of(module: &Module) -> Vec<String> {
    let mut kernels = Vec::new();
    for source in module.sources {
        let text = fs::read_to_string(source)
            .unwrap_or_else(|error| panic!("cannot read kernel source {source}: {error}"));
        let mut depth = 0usize;
        for (index, line) in text.lines().enumerate() {
            if let Some(directive) = line.trim_start().strip_prefix('#').map(str::trim_start) {
                if directive.starts_with("endif") {
                    depth = depth.saturating_sub(1);
                } else if directive.starts_with("if") {
                    depth += 1;
                }
            }
            if !line.contains("__global__") {
                continue;
            }
            if depth > 0 {
                panic!(
                    "{source}:{}: a kernel inside an `#if` block; one build may compile it and \
                     another not, which the kernel list's text scan cannot tell, so it refuses it",
                    index + 1
                );
            }
            let name: String = line
                .trim_start()
                .strip_prefix(DECLARATION)
                .unwrap_or_default()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                panic!(
                    "{source}:{}: a `__global__` line that is not an \
                     `extern \"C\" __global__ void <name>` declaration; the kernel list \
                     cannot read it",
                    index + 1
                );
            }
            if kernels.contains(&name) {
                panic!("{}: the kernel `{name}` is declared twice", module.ptx);
            }
            kernels.push(name);
        }
    }
    if kernels.is_empty() {
        panic!("{}: its sources declare no kernel", module.ptx);
    }
    kernels
}

/// Writes `kernel_lists.rs`: one `&[&str]` per module, named after its PTX file, and `MODULES`,
/// each module's PTX file, sources and kernels.
fn write_kernel_lists(out: &Path) {
    let mut text = String::from(
        "// Generated by build.rs from each PTX module's kernel sources, on every build.\n",
    );
    let mut entries = String::new();
    for module in &MODULES {
        let constant = module.ptx.trim_end_matches(".ptx").to_ascii_uppercase();
        let names: Vec<String> = kernels_of(module)
            .iter()
            .map(|name| format!("{name:?}"))
            .collect();
        let sources: Vec<String> = module
            .sources
            .iter()
            .map(|source| format!("{source:?}"))
            .collect();
        text.push_str(&format!(
            "/// The entry points of `{}`, from {}.\npub const {constant}: &[&str] = &[{}];\n",
            module.ptx,
            module.sources.join(" + "),
            names.join(", ")
        ));
        entries.push_str(&format!(
            "    KernelModule {{ ptx: {:?}, sources: &[{}], kernels: {constant} }},\n",
            module.ptx,
            sources.join(", ")
        ));
    }
    text.push_str(&format!(
        "/// Every PTX module, in the order build.rs builds them.\n\
         pub const MODULES: &[KernelModule] = &[\n{entries}];\n"
    ));
    let path = out.join("kernel_lists.rs");
    fs::write(&path, text)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
}

fn combined_source(out: &Path, module: &Module) -> PathBuf {
    if let [single] = module.sources {
        return PathBuf::from(single);
    }
    let mut text = String::new();
    for source in module.sources {
        text.push_str(
            &fs::read_to_string(source)
                .unwrap_or_else(|error| panic!("cannot read kernel source {source}: {error}")),
        );
        text.push('\n');
    }
    let path = out.join(module.ptx.replace(".ptx", ".cu"));
    fs::write(&path, text)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
    path
}
