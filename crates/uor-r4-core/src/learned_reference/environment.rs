use super::{NativeError, NativeErrorTag};
use serde_json::{json, Value};

/// Read the current thread's floating-point controls without changing them.
/// This observation is part of the admitted CPU profile, not a parity claim.
pub fn floating_point_environment() -> Result<Value, NativeError> {
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    {
        let fpcr: u64;
        // SAFETY: MRS FPCR reads a userspace architectural register; it does
        // not access memory, change modes, or dereference a pointer. Deliberately
        // not `pure`: each call must observe the invoking thread's current mode.
        unsafe {
            core::arch::asm!("mrs {value}, fpcr", value = out(reg) fpcr,
                options(nomem, nostack, preserves_flags));
        }
        let rounding = (fpcr >> 22) & 3;
        let flush_to_zero = (fpcr >> 24) & 1;
        let flush_half = (fpcr >> 19) & 1;
        let alternative = (fpcr >> 1) & 1;
        let flush_inputs = fpcr & 1;
        if rounding | flush_to_zero | flush_half | alternative | flush_inputs != 0 {
            return Err(NativeError::new(NativeErrorTag::UnsupportedProfile));
        }
        Ok(
            json!({"fpcr":fpcr,"rounding_mode":rounding,"fz":flush_to_zero,
            "fz16":flush_half,"ah":alternative,"fiz":flush_inputs}),
        )
    }
    #[cfg(not(all(target_arch = "aarch64", target_os = "macos")))]
    Err(NativeError::new(NativeErrorTag::UnsupportedProfile))
}
