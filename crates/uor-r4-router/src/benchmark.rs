use crate::geometry::{SemanticGeometry, SpectralGeometry, TypedObject, VsaGeometry};
use crate::UorR4Router;

/// Result of one geometry's soft-routing ablation.
///
/// Issue #434 item 2 trimmed this to the quantities that are actually
/// computed. It previously also carried `hits_at_3`, which was
/// `recall_at_3` under a second name (both incremented in the same branch
/// and divided by the same count); `unlearning_time_ns`, which timed
/// `ground()` under a comment claiming it measured route deletion; and
/// `migration_agreement`, a hard-coded `0.98` that the only test then
/// asserted equal to `0.98`. Reporting a literal as a measurement is worse
/// than reporting nothing, because it survives review.
///
/// This type answers one question: **for how many queries does the
/// geometry's `soft_route` place the ground-truth axis in its top-`k`?**
/// It is a wiring check on synthetic inputs, NOT a retrieval-quality
/// measurement — for that see `tests/geometry_ablation.rs`, which runs the
/// production retrieval surface over a real corpus with ground truth and a
/// deranged-key null.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BenchmarkResult {
    pub geometry_name: String,
    /// Fraction of queries whose ground-truth axis appears in the routes.
    pub recall_at_3: f32,
    /// Queries scored — so a zero recall over zero queries cannot be read
    /// as a measured zero.
    pub queries: usize,
}

pub fn run_ablation_benchmark(
    router: &UorR4Router,
    queries: &[(TypedObject, usize)], // (Object, Ground Truth Item ID)
) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();

    // 1. Benchmark Spectral Geometry
    let geom_spectral = SpectralGeometry {
        space_cid: "blake3:spectral_space".to_string(),
        active_state: None,
        identity: None,
    };
    results.push(evaluate_geometry(
        "Spectral Heuristic",
        &geom_spectral,
        router,
        queries,
    ));

    // 2. Benchmark VSA Geometry
    let geom_vsa = VsaGeometry {
        space_cid: "blake3:vsa_space".to_string(),
    };
    results.push(evaluate_geometry(
        "VSA Grounded",
        &geom_vsa,
        router,
        queries,
    ));

    results
}

fn evaluate_geometry<G: SemanticGeometry>(
    name: &str,
    geometry: &G,
    _router: &UorR4Router,
    queries: &[(TypedObject, usize)],
) -> BenchmarkResult {
    let mut recall_sum = 0.0;

    for (obj, gt_id) in queries {
        if let Ok(grounded) = geometry.ground(obj) {
            if let Ok(coords) = geometry.encode(&grounded) {
                if let Ok(routes) = geometry.soft_route(&coords, 3) {
                    // Check if ground truth maps to axis in routes
                    let mut matched = false;
                    for route in &routes {
                        if route.axis as usize == *gt_id {
                            matched = true;
                            break;
                        }
                    }
                    if matched {
                        recall_sum += 1.0;
                    }
                }
            }
        }
    }

    let q_len = queries.len() as f32;
    let recall = if q_len > 0.0 { recall_sum / q_len } else { 0.0 };

    BenchmarkResult {
        geometry_name: name.to_string(),
        recall_at_3: recall,
        queries: queries.len(),
    }
}
