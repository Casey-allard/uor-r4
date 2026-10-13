//! Ahead-of-time CUDA kernels. The owner banned NVRTC (runtime CUDA
//! compilation) project-wide on 13 October 2026, and D29 makes the compiled
//! module the compliance boundary. Every kernel module is therefore compiled
//! by nvcc when the crate builds with the `cuda` feature, into PTX in
//! `OUT_DIR`, and loaded by the driver at run time. Without the `cuda`
//! feature nothing runs.
//!
//! `NVCC` names the compiler (default `nvcc`); `UOR_NVCC_ARCH` the virtual
//! architecture (default `compute_80`, which the driver loads on the 4090,
//! 5090 and PRO 6000 of the pod ladder).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The precision options the NVRTC builds used, kept so the compiled
/// arithmetic is unchanged: no fast math, IEEE division and square root, no
/// flush-to-zero (fused multiply-add stays at the compiler default).
const PRECISE: [&str; 3] = ["-ftz=false", "-prec-div=true", "-prec-sqrt=true"];

/// One PTX module: its sources (concatenated in order), output name and extra
/// options.
struct Module {
    sources: &'static [&'static str],
    ptx: &'static str,
    options: &'static [&'static str],
}

const MODULES: [Module; 3] = [
    Module {
        sources: &["src/cuda_stack.cu", "src/spiralcore_fbs.cu"],
        ptx: "cuda_stack_f32.ptx",
        options: &PRECISE,
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
    },
    Module {
        sources: &["src/starling_native.cu"],
        ptx: "starling_native.ptx",
        options: &["--fmad=false"],
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
    if env::var_os("CARGO_FEATURE_CUDA").is_none() {
        return;
    }
    let out = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| {
        panic!("cargo did not set OUT_DIR for the ahead-of-time CUDA kernels")
    }));
    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".to_string());
    let arch = env::var("UOR_NVCC_ARCH").unwrap_or_else(|_| "compute_80".to_string());
    for module in &MODULES {
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

/// The module's single source, or its sources concatenated into `OUT_DIR`.
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
