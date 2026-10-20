//! Offline native-context CUDA kernels. No served-runtime code uses this module.
//! Strict q4 forward scores classes and time in parallel; backward stages reverse
//! time with parallel class credit and double scratch. Speed remains unmeasured.
#![allow(unsafe_code)]
use candle_core::{CudaDevice, Error, Result};
use cudarc::driver::{CudaView, LaunchConfig, PushKernelArg};
pub(crate) enum Arg<'a> {
    F(CudaView<'a, f32>),
    I(CudaView<'a, i64>),
    D(CudaView<'a, f64>),
    U(CudaView<'a, u32>),
    N(u32),
}
pub(crate) fn launch(
    device: &CudaDevice,
    name: &str,
    count: usize,
    args: &[Arg<'_>],
) -> Result<()> {
    let count =
        u32::try_from(count).map_err(|_| Error::Msg("native CUDA launch overflow".into()))?;
    if count == 0 {
        candle_core::bail!("empty native CUDA launch");
    }
    if HISTORICAL_KERNELS.contains(&name) {
        candle_core::bail!(
            "D29: `{name}` is a historical exp/log-normalising kernel excluded from every compiled module"
        );
    }
    let fun = device.get_or_load_custom_func(name, "uor_native_context", ptx()?)?;
    let mut builder = fun.builder();
    for arg in args {
        match arg {
            Arg::F(v) => {
                builder.arg(v);
            }
            Arg::I(v) => {
                builder.arg(v);
            }
            Arg::D(v) => {
                builder.arg(v);
            }
            Arg::U(v) => {
                builder.arg(v);
            }
            Arg::N(v) => {
                builder.arg(v);
            }
        }
    }
    // SAFETY: callers validate dimensions and allocate every indexed buffer;
    // arguments exactly match the corresponding signatures below.
    unsafe {
        builder.launch(LaunchConfig {
            grid_dim: (count.div_ceil(64), 1, 1),
            block_dim: (64, 1, 1),
            shared_mem_bytes: 0,
        })
    }
    .map_err(|e| Error::Cuda(Box::new(e)))?;
    Ok(())
}
/// Kernels D29 removed from the compiled module: they normalise with exp/log.
/// Their source is evidence only, in `native_context_historical.cu`.
pub(crate) const HISTORICAL_KERNELS: [&str; 3] = ["action_credit", "emit", "whole_path_alias_log"];

/// D29 quarantine (Codex review, #2028 comment 6149949044, accepted 13 October
/// 2026): the native-context module's class-bank readout dots the query with
/// freely learned per-class coefficients over the 120/120/33 banks, a learned
/// dense 4 -> 120/33 projection. The module is no longer compiled; its source is
/// evidence in `archive/history/d29-excluded-kernels-2026-10-13/native_context_quarantined.cu`
/// until a typed geometric replacement is designed. Line B stays unfinished, not
/// negative (D22).
fn ptx() -> Result<&'static str> {
    candle_core::bail!(
        "D29: the native-context CUDA module is quarantined (its class-bank readout is a learned \
         dense 4->120/33 projection); see archive/history/d29-excluded-kernels-2026-10-13/"
    )
}
