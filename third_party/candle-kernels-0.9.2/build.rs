//! UOR D29: build only the explicit lawful tensor modules. Historical kernels
//! are retained under historical/ and never passed to the CUDA compiler.
use std::{env, fs, path::PathBuf, process::Command};

mod entry_allowlist;

const MODULES: &[&str] = &[
    "affine", "binary", "cast", "fill", "indexing", "reduce", "sort", "ternary", "unary",
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=entry_allowlist.rs");
    for header in [
        "compatibility.cuh",
        "cuda_utils.cuh",
        "binary_op_macros.cuh",
    ] {
        println!("cargo:rerun-if-changed=src/{header}");
    }
    for name in ["NVCC", "UOR_NVCC_ARCH", "CUDA_COMPUTE_CAP"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".into());
    let arch = env::var("UOR_NVCC_ARCH").unwrap_or_else(|_| {
        format!(
            "compute_{}",
            env::var("CUDA_COMPUTE_CAP").unwrap_or_else(|_| "80".into())
        )
    });
    let mut bindings = String::new();
    for name in MODULES {
        let source = format!("src/{name}.cu");
        println!("cargo:rerun-if-changed={source}");
        let ptx = out.join(format!("{name}.ptx"));
        let status = Command::new(&nvcc)
            .args([
                "--ptx",
                "--expt-relaxed-constexpr",
                "-std=c++17",
                "-O3",
                "--default-stream",
                "per-thread",
                "-ftz=false",
                "-prec-div=true",
                "-prec-sqrt=true",
            ])
            .arg(format!("--gpu-architecture={arch}"))
            .args(["-I", "src", "-o"])
            .arg(&ptx)
            .arg(&source)
            .status()
            .expect("D29 CUDA build requires nvcc; no runtime compiler fallback");
        assert!(status.success(), "nvcc failed for {source}");
        let text = fs::read_to_string(&ptx).expect("read generated PTX");
        entry_allowlist::check_ptx_exports(name, &text)
            .unwrap_or_else(|error| panic!("D29 export admission: {error}"));
        for forbidden in [
            "softmax",
            "sigmoid",
            "tanh",
            "gelu",
            "silu",
            "uexp_",
            "mma.sync",
            "wmma.",
            "ex2.approx",
        ] {
            assert!(
                !text.contains(forbidden),
                "D29 prohibited {forbidden} in {}",
                ptx.display()
            );
        }
        bindings.push_str(&format!(
            "pub const {}: &str = include_str!({:?});\n",
            name.to_uppercase(),
            ptx
        ));
    }
    // Compatibility descriptors exist so historical calls return a typed
    // backend refusal. They do not embed any executable kernel or linked FFI.
    bindings.push_str("pub const CONV: &str = \"\";\npub const QUANTIZED: &str = \"\";\n");
    fs::write(out.join("ptx.rs"), bindings).expect("write explicit PTX bindings");
}
