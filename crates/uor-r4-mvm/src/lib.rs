#![forbid(unsafe_code)]
//! Deterministic RV64IMAC Linux-userland MicroVM (`uor-r4-mvm`) for native CLI and
//! browser WebAssembly studio execution (`#3939`).
//!
//! Raw emulator and C compiler internals remain crate-private:
//!
//! ```compile_fail
//! use uor_r4_mvm::compile_c_to_rv64_elf;
//! ```
//!
//! ```compile_fail
//! use uor_r4_mvm::mvm;
//! ```
//!
//! Public callers interact through [`MvmSandbox`], [`guest_identity`], and [`run_collatz`]:
//!
//! ```
//! let mut vm = uor_r4_mvm::MvmSandbox::new().expect("MVM boots");
//! let res = vm.exec("uname -m", "/workspace");
//! assert_eq!(res.exit_code, 0);
//! assert!(res.stdout.contains("riscv64"));
//! ```

mod elf;
#[allow(dead_code)]
#[rustfmt::skip]
mod lexlean_admission;
#[allow(dead_code, unused_imports)]
#[rustfmt::skip]
mod mvm;
#[allow(dead_code)]
#[rustfmt::skip]
mod mvm_cc;
#[rustfmt::skip]
mod mvm_initramfs;
#[allow(dead_code)]
mod spiralcore_vsock;
#[allow(dead_code, unused_imports)]
#[rustfmt::skip]
mod vsock;

pub mod sandbox;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

pub use sandbox::{ExecResult, MvmSandbox};

#[cfg(test)]
#[path = "../tests/internal/mvm.rs"]
mod mvm_tests;
#[cfg(test)]
#[path = "../tests/internal/provenance.rs"]
mod provenance_tests;

/// Boot the pinned guest and return its architecture name and vsock CID.
pub fn guest_identity() -> Result<(&'static str, u16), String> {
    let vm = mvm::MvmSandbox::boot_default().map_err(|error| error.to_string())?;
    Ok((vm.architecture(), vm.guest_cid()))
}

/// Return the fixed guest program's stdout for a Collatz search over 1 through `limit`.
/// The inclusive supported range is 1 through 512.
pub fn run_collatz(limit: u16) -> Result<String, String> {
    if !(1..=512).contains(&limit) {
        return Err("Collatz limit must be an integer from 1 to 512".into());
    }
    let mut vm = mvm::MvmSandbox::boot_default().map_err(|error| error.to_string())?;
    let workload = include_str!("workloads/collatz.c");
    let source = format!("long limit = {limit};\n{workload}");
    vm.write_guest_file("/workspace/collatz.c", source.as_bytes())
        .map_err(|error| error.to_string())?;
    let outcome = vm
        .exec_command("cc collatz.c -o collatz && ./collatz", "/workspace")
        .map_err(|error| error.to_string())?;
    if outcome.exit_code != 0 {
        return Err(format!(
            "guest exit {}: {}",
            outcome.exit_code, outcome.stderr
        ));
    }
    if outcome.compiled_elf_machine != Some(243) || outcome.instructions_retired == 0 {
        return Err("expected the guest to compile and execute a RISC-V ELF".into());
    }
    Ok(outcome.stdout)
}
