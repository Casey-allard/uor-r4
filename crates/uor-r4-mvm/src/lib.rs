#![forbid(unsafe_code)]
//! Run the two checked native workbench operations on a fresh, in-memory guest.
//!
//! The import's general-purpose execution interface is private.
//!
//! ```compile_fail
//! use uor_r4_mvm::MvmSandbox;
//! ```
//!
//! ```compile_fail
//! use uor_r4_mvm::compile_c_to_rv64_elf;
//! ```

mod elf;
// Preserve the upstream implementation and its private parity fixtures. Product callers
// can only use the two functions below.
#[allow(dead_code)]
#[rustfmt::skip]
mod mvm;
#[rustfmt::skip]
mod mvm_cc;
#[rustfmt::skip]
mod mvm_initramfs;
#[allow(dead_code)]
#[rustfmt::skip]
mod spiralcore_vsock;

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
