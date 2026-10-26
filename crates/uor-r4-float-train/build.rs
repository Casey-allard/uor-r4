//! Ahead-of-time CUDA kernels for the D37 offline float training module (CHARTER §3.4 / D29).
//!
//! Runtime CUDA compilation (NVRTC) is banned project-wide. When compiled with `--features cuda`,
//! `nvcc` compiles the 24-kernel extract (`src/cuda_stack.cu` + `src/spiralcore_fbs.cu`) ahead of
//! time into `cuda_stack_f32.ptx` and `cuda_stack_bf16.ptx` inside `OUT_DIR`. The SpiralCore/FBS
//! sheet arms (`src/spiralcore_fbs_arms.cu`) build as their own module, `spiralcore_fbs_arms_f32.ptx`,
//! from a prelude, the unchanged `src/spiralcore_fbs.cu` (for its route helpers) and the arms. On
//! CPU-only laptop builds, `build.rs` exits immediately without invoking `nvcc`.

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
    let built: Vec<&Module> = MODULES
        .iter()
        .filter(|module| env::var_os(module.feature).is_some())
        .collect();
    if built.is_empty() {
        return;
    }
    let out = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| {
        panic!("cargo did not set OUT_DIR for the ahead-of-time CUDA kernels")
    }));
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
