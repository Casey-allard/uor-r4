//! Build for the FBS artifact evaluator (claude-128).
//!
//! 1. `src/fbs_core.h` is compiled as host C++ (`src/host_kernel.cpp`) for the
//!    finite parity tests against the Rust integer session only.
//! 2. With the `cuda` feature, `src/evaluator.cu` is compiled ahead of time by
//!    nvcc to PTX, admitted by the integer PTX rules of
//!    `docs/labs/m1-d11-artifact-scorer/instrument/build.rs` (no floating-point,
//!    MMA or tensor token; exactly the expected entries), and assembled to a
//!    cubin. No NVRTC.
use std::{env, fs, path::PathBuf, process::Command};

const ENTRIES: [&str; 1] = ["fbs_eval_sequences"];

/// Inspect the emitted module, not the CUDA spelling. Integer products are not
/// prohibited.
fn admit_ptx(ptx: &str, inventory: &[&str]) -> Result<(), String> {
    let mut entries = std::collections::BTreeSet::new();
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
                    line_number + 1
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
    let expected: std::collections::BTreeSet<_> =
        inventory.iter().map(|entry| entry.to_string()).collect();
    if entries != expected {
        return Err(format!(
            "integer PTX entry inventory differs: missing {:?}, unexpected {:?}",
            expected.difference(&entries).collect::<Vec<_>>(),
            entries.difference(&expected).collect::<Vec<_>>()
        ));
    }
    Ok(())
}

// Lexer controls only; these strings are never loaded or run.
fn ptx_admission_self_test() -> Result<(), String> {
    let inventory = ENTRIES
        .iter()
        .map(|entry| format!(".visible .entry {entry}() {{ ret; }}\n"))
        .collect::<String>();
    admit_ptx(
        &format!("// floats such as .f32 are comments\n{inventory}mul.lo.u64 %a,%b,8;"),
        &ENTRIES,
    )?;
    // One probe per forbidden component, so deleting any of them fails here.
    for forbidden in [
        "add.f32 %a,%b,%c;",
        "add.f64 %a,%b,%c;",
        "add.f16 %a,%b,%c;",
        ".reg .f16x2 %a;",
        ".reg .bf16 %a;",
        ".reg .bf16x2 %a;",
        "cvt.rna.tf32.u32 %a,%b;",
        ".reg .e4m3 %a;",
        ".reg .e5m2 %a;",
        "mma.sync.aligned.m16n8k8.row.col.s32.s8.s8.s32 %a,%b,%c,%d;",
        "wmma.load.a.sync.aligned.row %a,[%b];",
        "wgmma.fence.sync.aligned;",
        "tcgen05.alloc %a;",
        "cp.async.bulk.tensor %a;",
        ".param .tensormap %a;",
        "ex2.approx %a,%b;",
        ".visible .entry unexpected() { ret; }",
        ".visible .entry fbs_eval_sequences() { ret; }",
    ] {
        if admit_ptx(&format!("{inventory}{forbidden}"), &ENTRIES).is_ok() {
            return Err(format!(
                "PTX admission self-test failed to reject {forbidden}"
            ));
        }
    }
    if admit_ptx("", &ENTRIES).is_ok() {
        return Err("PTX admission self-test failed to reject missing entries".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/fbs_core.h");
    println!("cargo:rerun-if-changed=src/host_kernel.cpp");
    println!("cargo:rerun-if-changed=src/evaluator.cu");
    println!("cargo:rerun-if-env-changed=NVCC");
    println!("cargo:rerun-if-env-changed=UOR_NVCC_ARCH");
    ptx_admission_self_test()?;
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .include("src")
        .warnings(true)
        .file("src/host_kernel.cpp")
        .compile("fbs_host_kernel");

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
    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".into());
    let ptx = out.join("fbs_evaluator.ptx");
    let status = Command::new(&nvcc)
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .arg("-I")
        .arg("src")
        .arg("-o")
        .arg(&ptx)
        .arg("src/evaluator.cu")
        .status()?;
    if !status.success() {
        return Err(format!("ahead-of-time nvcc evaluator build failed: {status}").into());
    }
    admit_ptx(&fs::read_to_string(&ptx)?, &ENTRIES)?;
    // Assemble the admitted PTX, rather than compiling the source again.
    let image_target = arch.replacen("compute_", "sm_", 1);
    let report = Command::new(&nvcc)
        .args(["-cubin", "--ptxas-options=-v"])
        .arg(format!("-arch={arch}"))
        .arg(format!("--gpu-code={image_target}"))
        .arg("-o")
        .arg(out.join("fbs_evaluator.cubin"))
        .arg(&ptx)
        .output()?;
    let mut log = report.stdout;
    log.extend(report.stderr);
    fs::write(out.join("fbs_evaluator-resources.txt"), &log)?;
    if !report.status.success() {
        return Err(format!(
            "evaluator cubin assembly failed: {}",
            String::from_utf8_lossy(&log)
        )
        .into());
    }
    let image = fs::read(out.join("fbs_evaluator.cubin"))?;
    if image.len() < 64 || !image.starts_with(b"\x7fELF") {
        return Err("evaluator assembly did not emit a nonempty ELF cubin".into());
    }
    fs::write(out.join("cuda-image-target.txt"), image_target)?;
    Ok(())
}
