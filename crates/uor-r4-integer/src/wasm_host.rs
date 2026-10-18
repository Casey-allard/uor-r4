//! Host interface for `wasm32-unknown-unknown` builds of the integer runtime.
//!
//! Two seams live here, and only here:
//!
//! 1. **Time.** `wasm32-unknown-unknown` has no `std::time::Instant`. A
//!    [`WasmInstant`] reads a monotonic nanosecond counter registered by the
//!    host ([`set_host_clock`]); with no clock registered, every duration is
//!    zero — telemetry fields stay exact integers and report "unmeasured"
//!    rather than failing or fabricating a value. The served computation
//!    itself never depends on time.
//!
//! 2. **Byte loading.** Serving material arrives from the host as bytes
//!    ([`crate::bundle::Bundle::from_parts`], [`crate::stack::IntegerStackModel::parse`]);
//!    the wasm module has no filesystem. This module owns no loading logic —
//!    it exists so the byte seam and the clock seam have one documented home.

#![cfg(target_arch = "wasm32")]

use core::sync::atomic::{AtomicU64, Ordering};

/// Host-registered monotonic clock in nanoseconds.
///
/// The host (a Web Worker or the test harness) calls [`set_host_clock`] once
/// before generation. `STATIC` is the value source: the host writes a fresh
/// reading with [`tick_host_clock`]; `WasmInstant::elapsed` diffs two readings.
static HOST_CLOCK_NS: AtomicU64 = AtomicU64::new(0);

/// Whether the host has ever provided a reading. Reads of the clock before
/// registration report zero durations, and the generation report's timing
/// fields are therefore exact zeroes — "unmeasured", not a fake number.
static HOST_CLOCK_SET: AtomicU64 = AtomicU64::new(0);

/// Register that the host clock source exists. Called by the host binding
/// layer once at module init (the JS glue calls this from `start()`).
pub fn set_host_clock() {
    HOST_CLOCK_SET.store(1, Ordering::Relaxed);
}

/// Push a fresh host clock reading (nanoseconds, monotonic). The JS glue
/// calls this at the start of every `step`/`generate` call from
/// `performance.now()`.
pub fn tick_host_clock(ns: u64) {
    HOST_CLOCK_NS.store(ns, Ordering::Relaxed);
}

/// A wall-clock instant for generation telemetry on wasm.
///
/// API-compatible with the slice of `std::time::Instant` the runtime uses:
/// `now()` and `elapsed()`. Reading it twice diffs the host readings that
/// bracket the call; if the host never ticks, the diff is zero.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WasmInstant {
    at_ns: u64,
}

impl WasmInstant {
    pub(crate) fn now() -> Self {
        // If the host has not registered a clock this is just 0, and every
        // elapsed() will be 0 — the "unmeasured" contract above.
        let at_ns = if HOST_CLOCK_SET.load(Ordering::Relaxed) == 1 {
            HOST_CLOCK_NS.load(Ordering::Relaxed)
        } else {
            0
        };
        Self { at_ns }
    }

    pub(crate) fn elapsed(&self) -> WasmDuration {
        let now = if HOST_CLOCK_SET.load(Ordering::Relaxed) == 1 {
            HOST_CLOCK_NS.load(Ordering::Relaxed)
        } else {
            0
        };
        WasmDuration {
            nanos: now.saturating_sub(self.at_ns),
        }
    }
}

/// The wasm duration type fed into `duration_nanos_exact`'s callers.
///
/// `duration_nanos_exact` takes a `std::time::Duration`; on wasm the nanosecond
/// total is carried directly and converted at the call site via
/// [`WasmDuration::as_nanos`], which keeps the exact-integer contract without
/// constructing a `Duration` on a target where it cannot be measured.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WasmDuration {
    nanos: u64,
}

impl WasmDuration {
    pub(crate) fn as_nanos(self) -> u128 {
        self.nanos as u128
    }
}
