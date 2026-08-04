//! #400 A/B harness: measure `syntactic_morphism_score`'s contribution on
//! the surface that actually calls it.
//!
//! Scope finding (recorded here because it shapes the whole issue): the
//! certify C-row serving surface (`R4Engine` in uor-r4-api) never calls the
//! Cayley–Dickson term — `syntactic_morphism_score` is reachable only from
//! the `R4G1Runtime` generation paths (`predict_candidates*` /
//! `predict_distribution*`, i.e. the chat/beam surface). The A/B therefore
//! grades that surface: teacher-forced next-token prediction through
//! `R4G1Runtime::predict_candidates` over the held-out fixture partition.
//!
//! The A and B arms come from two builds of this same test: stock (CD term
//! live) and a local, uncommitted patch that makes the term return zero.
//! Set `R4_CD_LABEL` to tag the run; `R4_CD_AB_BUNDLE` points at the
//! `score.r4g1` produced by `convert-r4g1` from the checked-in fixtures.
//!
//! Run:
//!   R4_CD_AB_BUNDLE=/tmp/score.r4g1 R4_CD_LABEL=cd-on \
//!   cargo test --release -p uor-r4-graph-certify --test r4g1_cd_ab -- --ignored --nocapture

use uor_r4_core::transformerless::compiler;

const SAMPLE_EVERY: usize = 5;
const WINDOW: usize = 8;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
#[ignore = "A/B measurement harness; run explicitly with --ignored"]
fn r4g1_cd_ab() {
    let label = std::env::var("R4_CD_LABEL").unwrap_or_else(|_| "unlabeled".to_owned());
    let bundle_path =
        std::env::var("R4_CD_AB_BUNDLE").unwrap_or_else(|_| "/tmp/score.r4g1".to_owned());
    let bytes = std::fs::read(&bundle_path).expect("score.r4g1 bundle (run convert-r4g1 first)");
    let runtime = uor_r4_graph_runtime::R4G1Runtime::parse(&bytes).expect("parse r4g1");
    let c = compiler::load_corpus_from(&fixture("c_meta.bin"), &fixture("c_recs.bin"))
        .expect("checked-in fixture corpus");
    let cut = (c.stories as f64 * 0.8) as u32;

    let num_nodes = runtime.node_count() as usize;
    let mut node_scores = vec![uor_r4_core::transformerless::score_q::ScoreQ::MIN; num_nodes];
    let mut cands = [(0u32, uor_r4_core::transformerless::score_q::ScoreQ::ZERO); 8];

    let (mut n, mut top1, mut agree, mut served) = (0u64, 0u64, 0u64, 0u64);
    for i in (0..c.n).step_by(SAMPLE_EVERY) {
        if c.story[i] < cut {
            continue;
        }
        // teacher-forced window ending at the record's input token (same
        // convention as certify's eval_context: j = WINDOW..1, j=1 is input)
        let mut window = Vec::with_capacity(WINDOW);
        for j in (1..=WINDOW).rev() {
            if let Some(t) = uor_r4_core::transformerless::runtime::history_token(&c, i, j) {
                window.push(t);
            }
        }
        if window.is_empty() {
            continue;
        }
        n += 1;
        let count = runtime.predict_candidates(&window, None, &mut node_scores, &mut cands);
        if count == 0 {
            continue;
        }
        served += 1;
        let pred = cands[0].0;
        if pred == c.next[i] {
            top1 += 1;
        }
        if pred == c.t_argmax[i] {
            agree += 1;
        }
    }
    println!(
        "cd-ab[{label}] positions {n} | served {served} ({:.1}%) | top1 {:.2}% | teacher-agree {:.2}%",
        100.0 * served as f64 / n as f64,
        100.0 * top1 as f64 / n as f64,
        100.0 * agree as f64 / n as f64
    );
}
