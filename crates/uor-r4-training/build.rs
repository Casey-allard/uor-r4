//! Ahead-of-time CUDA kernels. The owner banned NVRTC (runtime CUDA
//! compilation) project-wide on 13 October 2026, so new kernels are compiled
//! by nvcc when the crate builds with the `cuda` feature, each into its own
//! PTX module in `OUT_DIR`, and loaded by the driver at run time. Without the
//! `cuda` feature nothing runs.
//!
//! `NVCC` names the compiler (default `nvcc`); `UOR_NVCC_ARCH` the virtual
//! architecture (default `compute_80`, which the driver loads on the 4090,
//! 5090 and PRO 6000 of the pod ladder).

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Kernel sources compiled ahead of time: (source, PTX file name).
const KERNELS: [(&str, &str); 1] = [("src/starling_native.cu", "starling_native.ptx")];

fn main() {
    for (source, _) in KERNELS {
        println!("cargo:rerun-if-changed={source}");
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
    for (source, ptx) in KERNELS {
        let status = Command::new(&nvcc)
            .arg("-ptx")
            .arg(format!("-arch={arch}"))
            .arg("--fmad=false")
            .arg("-o")
            .arg(out.join(ptx))
            .arg(source)
            .status();
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => panic!("nvcc failed ({status}) compiling {source}"),
            Err(error) => panic!(
                "the cuda feature needs nvcc to build {source} ahead of time \
                 (runtime compilation is banned): {error}"
            ),
        }
    }
}
