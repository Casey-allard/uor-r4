//! Ahead-of-time CUDA kernels. The owner banned NVRTC (runtime CUDA
//! compilation) project-wide on 13 October 2026, and D29 makes the compiled
//! module the compliance boundary. Every kernel module is therefore compiled
//! by nvcc when the crate builds with the `cuda` feature, into PTX in
//! `OUT_DIR`, and loaded by the driver at run time. The sieve frame
//! compiler's module also builds alone, under `sieve-cuda`. Without either
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

/// One PTX module: its sources (concatenated in order), output name, extra
/// options and the Cargo feature that builds it.
struct Module {
    sources: &'static [&'static str],
    ptx: &'static str,
    options: &'static [&'static str],
    feature: &'static str,
}

const MODULES: [Module; 4] = [
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
        sources: &["src/starling_native.cu"],
        ptx: "starling_native.ptx",
        options: &["--fmad=false"],
        feature: CUDA,
    },
    Module {
        sources: &["src/sieve_frame.cu"],
        ptx: "sieve_frame.ptx",
        options: &[],
        feature: "CARGO_FEATURE_SIEVE_CUDA",
    },
];

/// The feature every Candle-era module builds under.
const CUDA: &str = "CARGO_FEATURE_CUDA";

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
