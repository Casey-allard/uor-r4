//! v0.2.2 Phase Q.3 example: iterate a `StreamDriver` until productivity
//! countdown exhausts.
//!
//! `StreamDeclaration::new::<T>(productivity_bound)` seeds an unfold-style
//! stream with a fixed productivity budget. `run_stream` returns a `StreamDriver`
//! implementing `Iterator<Item = Result<Grounded<'static, T>, PipelineFailure>>`; each
//! `.next()` yields the next grounded step until the budget is exhausted.
//!
//! Run with: `cargo run --example stream_driver_iteration -p uor-foundation`

use uor_foundation::enforcement::{ConstrainedTypeInput, Grounded, Validated};
use uor_foundation::pipeline::{run_stream, StreamDeclaration, StreamDriver};
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

fn main() {
    // Productivity bound = 5 — the driver yields at most 5 grounded steps.
    let decl: Validated<StreamDeclaration<'_, N>> =
        validated_runtime(StreamDeclaration::new::<ConstrainedTypeInput>(5));

    let driver: StreamDriver<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32> = run_stream(decl);

    let mut step = 0u32;
    let mut last_address = None;
    for result in driver {
        let grounded: Grounded<'static, ConstrainedTypeInput, N> = result.expect("step succeeds");
        println!(
            "step {step}: unit_address={:?} witt_bits={}",
            grounded.unit_address(),
            grounded.witt_level_bits()
        );
        if let Some(prev) = last_address {
            assert_ne!(
                prev,
                grounded.unit_address(),
                "successive steps must have distinct unit_addresses"
            );
        }
        last_address = Some(grounded.unit_address());
        step += 1;
    }
    println!("stream terminated after {step} step(s)");
}
