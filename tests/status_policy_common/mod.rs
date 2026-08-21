//! Shared fixture builders for the status-policy probe suite and its
//! allocation census (issue #78). The hand-built scored artifacts mirror
//! the helpers of `crates/uor-r4-core/tests/score.rs` (which are local to
//! that test binary and cannot be imported): a minimal deterministic
//! `Compiled` teacher plus a two-region scored graph emitted through the
//! real compiler entry points, persisted to a per-test temporary bundle
//! the deployed adapter can load.
//!
//! Shared by two test binaries (`status_policy`, `status_policy_census`)
//! that each use a subset of the helpers, so unused-item lints are
//! allowed module-wide.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use uor_r4_graph_certify::{
    self as score, EmissionTables, GraphScorer, RegionParams, ScoreStatus, Smoothing,
    StructuralEdge, EXCT_SUPPORT_MIN,
};
use uor_r4_graph_format::ScoreQ;
use uor_r4_wasm_router::r4g1::R4g1State;
use uor_r4_wasm_router::transformerless::compiler::{self, D, K, SIG_BYTES, STAGES};
use uor_r4_wasm_router::transformerless::runtime::{self, Store};

fn xorshift(s: &mut u64) -> u64 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *s = x;
    x
}

/// A minimal Compiled (random-but-deterministic tables; mirrors
/// `crates/uor-r4-core/tests/score.rs`).
pub fn synthetic_compiled() -> compiler::Compiled {
    let vocab = 64usize;
    let mut rng = 0xC0DE42u64;
    let mut rand_bytes =
        |n: usize| -> Vec<u8> { (0..n).map(|_| (xorshift(&mut rng) & 0xff) as u8).collect() };
    compiler::Compiled {
        token_codes: rand_bytes(vocab * STAGES),
        stage_books: (0..STAGES)
            .map(|_| rand_bytes(K * D).iter().map(|&b| b as i8).collect())
            .collect(),
        stage_shifts: vec![0; STAGES],
        thresholds: vec![0; D],
        class_sigs: (0..STAGES).map(|_| rand_bytes(K * SIG_BYTES)).collect(),
        ctx_cb: Vec::new(),
        token_stage_kappas: Vec::new(),
        dot_cb: Vec::new(),
        resid_cb: Vec::new(),
        resid_scale_shifts: Vec::new(),
        norm_fold_const: 0,
    }
}

/// A persisted scored-graph bundle plus the probe signatures/windows the
/// tests assert against. The temporary directory is removed on drop.
pub struct ProbeFixture {
    pub dir: PathBuf,
    pub bytes: Vec<u8>,
    pub teacher: Vec<u8>,
    /// Signature covered by region 0 and carrying exact-context evidence.
    pub covered_sig: [u8; SIG_BYTES],
    /// Signature covered by region 1 (graph rule only).
    pub graph_sig: [u8; SIG_BYTES],
    /// Out-of-distribution signature: outside every calibrated radius.
    pub ood_sig: [u8; SIG_BYTES],
    /// Window-fixture only: a token window whose signature is covered.
    pub covered_window: Vec<u32>,
}

impl ProbeFixture {
    /// Load the deployed adapter on this bundle.
    pub fn load(&self) -> R4g1State {
        R4g1State::load(
            &self.dir.join("score.r4g1"),
            &self.dir.join("tless_artifacts.bin"),
        )
        .expect("adapter loads the fixture bundle")
    }

    /// A reference scorer over the same bytes (witness-emitting path).
    pub fn reference_scorer(&self) -> GraphScorer {
        GraphScorer::from_artifact(&self.bytes, Some(&self.teacher), 64, 64)
            .expect("reference scorer builds")
    }
}

impl Drop for ProbeFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

static FIXTURE_SEQ: AtomicUsize = AtomicUsize::new(0);

fn emit_and_persist(
    regions: &[RegionParams],
    emissions: &EmissionTables,
    store: &Store,
    status_policy_override: Option<serde_json::Value>,
) -> (PathBuf, Vec<u8>, Vec<u8>) {
    let artifacts = synthetic_compiled();
    let teacher = compiler::artifact_bytes(&artifacts);
    let tls1 = runtime::store_bytes(store);
    let (bytes, _) = score::emit_scored_r4g1(
        &teacher,
        (b"status-meta", b"status-recs"),
        64,
        &score::ScoredGraphSections {
            regions,
            structural: &[
                StructuralEdge {
                    src: 0,
                    kind: 0,
                    dst: 1,
                    score_q: ScoreQ::ZERO,
                },
                StructuralEdge {
                    src: 0,
                    kind: 0,
                    dst: 2,
                    score_q: ScoreQ::ZERO,
                },
            ],
            transitions: &[],
            transition_quantization: score::QuantizationErrorStats::default(),
            emissions,
            context_rows: &[],
            fwd_rows: &[],
            skipmix_rows: &[],
            psi_bag_rows: &[],
            exct_tls1: &tls1,
            exct_top_x: score::ScoreConfig::default().exct_top_x,
        },
    );

    let seq = FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("r4g1-status-policy-{}-{seq}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create fixture dir");
    std::fs::write(dir.join("score.r4g1"), &bytes).expect("write graph");
    std::fs::write(dir.join("tless_artifacts.bin"), &teacher).expect("write teacher");
    if let Some(policy) = status_policy_override {
        let report = serde_json::json!({ "config": { "status_policy": policy } });
        std::fs::write(dir.join("score_report.json"), report.to_string())
            .expect("write score report");
    }
    (dir, bytes, teacher)
}

/// The hand-computed root prior and region emission lists of the worked
/// example (identical values to the `score.rs` hand artifact, so the
/// expected selections below are integer arithmetic by hand).
fn hand_emissions() -> EmissionTables {
    EmissionTables {
        root_prior: [(10u32, 100i32), (20, 200), (30, 300), (40, 50)]
            .into_iter()
            .map(|(t, s)| (t, ScoreQ::from_raw(s)))
            .collect(),
        root_floor: ScoreQ::from_raw(-7000),
        root_total: 1000,
        region_lists: vec![
            vec![(10, ScoreQ::from_raw(1000)), (20, ScoreQ::from_raw(-500))],
            vec![(20, ScoreQ::from_raw(2000)), (30, ScoreQ::from_raw(100))],
        ],
        smoothing: Smoothing::AddOne,
        root_prior_quantization: score::QuantizationErrorStats::default(),
        emission_quantization: score::QuantizationErrorStats::default(),
        selection_stats: score::EmissionSelectionStats::default(),
    }
}

/// The signature-level fixture: two depth-1 regions at the all-zeros and
/// all-ones extremes (radius 4), exact-context evidence attached to the
/// covered signature's FULL graded code only (#234: Rule 2 fires at full
/// code only), so one fixture exercises all three resolution statuses
/// deterministically:
/// - `covered_sig` ([0x00; _]) → ExactContext (support ≥ EXCT_SUPPORT_MIN),
/// - `graph_sig` ([0xFF; _]) → Graph,
/// - `ood_sig` (144 of 288 bits set) → Novel (distance 144 from both
///   prototypes, outside every radius).
pub fn signature_fixture(status_policy_override: Option<serde_json::Value>) -> ProbeFixture {
    let artifacts = synthetic_compiled();
    let covered_sig = [0x00; SIG_BYTES];
    let graph_sig = [0xFF; SIG_BYTES];
    let mut ood_sig = [0x00; SIG_BYTES];
    for byte in ood_sig.iter_mut().take(18) {
        *byte = 0xFF;
    }

    // Exact-context evidence: the covered signature's FULL graded code
    // only, with total ≥ EXCT_SUPPORT_MIN. Rule 2 fires at full code
    // only (#234, maintainer decision 2026-07-29) — a shallower prefix
    // would be recorded as backoff and resolve Graph, not ExactContext.
    let codes =
        [covered_sig, graph_sig, ood_sig].map(|sig| runtime::assign_plain(&artifacts, &sig));
    assert!(
        codes[0] != codes[1] && codes[0] != codes[2],
        "the covered probe's full code must distinguish it from the graph/ood probes"
    );
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    store[STAGES].insert(
        codes[0].to_vec(),
        [(10u32, EXCT_SUPPORT_MIN + 1)].into_iter().collect(),
    );

    let regions = [
        RegionParams {
            node: 1,
            depth: 1,
            radius: 4,
            sig: covered_sig,
            parent: None,
        },
        RegionParams {
            node: 2,
            depth: 1,
            radius: 4,
            sig: graph_sig,
            parent: None,
        },
    ];
    let (dir, bytes, teacher) =
        emit_and_persist(&regions, &hand_emissions(), &store, status_policy_override);
    ProbeFixture {
        dir,
        bytes,
        teacher,
        covered_sig,
        graph_sig,
        ood_sig,
        covered_window: Vec::new(),
    }
}

/// The window-level fixture: region 0 is anchored at the derived signature
/// of the single-token window `[5]` (radius 4, emission token 10) so the
/// token-level path has a deterministic served probe; region 1 sits at the
/// all-ones extreme. No exact-context evidence (every store level empty),
/// so served probes resolve Graph.
pub fn window_fixture() -> ProbeFixture {
    let artifacts = synthetic_compiled();
    let rotations = compiler::derive_rotations();
    let covered_window = vec![5u32];
    let bundle = runtime::bundle_window_plain(&artifacts, &rotations, &covered_window);
    let covered_sig = runtime::sig_plain(&artifacts, &bundle);
    let graph_sig = [0xFF; SIG_BYTES];
    let mut distance_to_extreme = 0u32;
    for (a, b) in covered_sig.iter().zip(graph_sig.iter()) {
        distance_to_extreme += (a ^ b).count_ones();
    }
    assert!(
        distance_to_extreme > 4,
        "the anchored window signature must sit outside region 1"
    );
    let ood_sig = [0x55; SIG_BYTES]; // informational only for this fixture

    let store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let regions = [
        RegionParams {
            node: 1,
            depth: 1,
            radius: 4,
            sig: covered_sig,
            parent: None,
        },
        RegionParams {
            node: 2,
            depth: 1,
            radius: 4,
            sig: graph_sig,
            parent: None,
        },
    ];
    let (dir, bytes, teacher) = emit_and_persist(&regions, &hand_emissions(), &store, None);
    ProbeFixture {
        dir,
        bytes,
        teacher,
        covered_sig,
        graph_sig,
        ood_sig,
        covered_window,
    }
}

/// The derived signature of a token window under this fixture's teacher.
pub fn window_sig(fixture: &ProbeFixture, window: &[u32]) -> [u8; SIG_BYTES] {
    let artifacts = compiler::parse_artifacts(&fixture.teacher).expect("teacher parses");
    let rotations = compiler::derive_rotations();
    let bundle = runtime::bundle_window_plain(&artifacts, &rotations, window);
    runtime::sig_plain(&artifacts, &bundle)
}

/// Brute-force a deterministic token window whose prediction resolves with
/// the wanted status under the reference scorer (single tokens, then
/// pairs). Used to find the out-of-distribution window probes.
pub fn find_window_by_status(fixture: &ProbeFixture, want: ScoreStatus) -> Vec<u32> {
    let scorer = fixture.reference_scorer();
    for t in 0..64u32 {
        let window = vec![t];
        let sig = window_sig(fixture, &window);
        let outcome = scorer.score_candidates(&sig, &[]).expect("scores");
        if outcome.witness.status == want {
            return window;
        }
    }
    for t1 in 0..64u32 {
        for t2 in 0..64u32 {
            let window = vec![t1, t2];
            let sig = window_sig(fixture, &window);
            let outcome = scorer.score_candidates(&sig, &[]).expect("scores");
            if outcome.witness.status == want {
                return window;
            }
        }
    }
    panic!("no token window with status {want:?} found");
}
