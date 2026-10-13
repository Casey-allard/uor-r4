//! Native CUDA qualification support library.
//! Historical lib.rs and numerical modules are preserved but excluded from this build.
//! No feature can combine the historical executors with the qualified graph.
#[cfg(any(feature = "learned-reference", feature = "bench-internals"))]
compile_error!(
    "historical core numerical/benchmark features are excluded from native qualification"
);
pub mod answer_oracle;
#[path = "native_geometric/native_support.rs"]
pub mod native_geometric;
pub mod prime_route_attention;
pub mod report_output;
pub mod spiralcore_operator;
#[path = "transformerless/native_mod.rs"]
pub mod transformerless;
pub mod zeta_zeros;
pub fn wrap_to_pi(theta: f64) -> f64 {
    use std::f64::consts::PI;
    (theta + PI).rem_euclid(2.0 * PI) - PI
}
