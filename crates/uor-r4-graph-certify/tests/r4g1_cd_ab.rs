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
    let sample_every: usize = std::env::var("R4_CD_SAMPLE_EVERY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let bundle_path =
        std::env::var("R4_CD_AB_BUNDLE").unwrap_or_else(|_| "/tmp/score.r4g1".to_owned());
    let bytes = std::fs::read(&bundle_path).expect("score.r4g1 bundle (run convert-r4g1 first)");
    let runtime = uor_r4_graph_runtime::R4G1Runtime::parse(&bytes).expect("parse r4g1");
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let art = compiler::load_artifacts_from(&art_path).expect("artifacts");
    println!("stack: {meta_path} | {art_path}");
    let rot = compiler::derive_rotations();
    let cut = (c.stories as f64 * 0.8) as u32;

    let num_nodes = runtime.node_count() as usize;
    let mut node_scores = vec![uor_r4_core::transformerless::score_q::ScoreQ::MIN; num_nodes];
    let mut cands = [(0u32, uor_r4_core::transformerless::score_q::ScoreQ::ZERO); 8];

    let (mut n, mut top1, mut agree, mut served) = (0u64, 0u64, 0u64, 0u64);
    // #400 root-cause slicing: count>1 means the graph/node path ran (the
    // context_backoff short-circuit yields exactly one candidate). The CD
    // term can only influence the multi-candidate slice.
    let (mut n_multi, mut top1_multi, mut agree_multi) = (0u64, 0u64, 0u64);
    for i in (0..c.n).step_by(sample_every) {
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
        // ROUT addressing input: the context signature, exactly as the chat
        // surface computes it (bundle_window_plain + sig_plain).
        let bundle =
            uor_r4_core::transformerless::runtime::bundle_window_plain(&art, &rot, &window);
        let sig = uor_r4_core::transformerless::runtime::sig_plain(&art, &bundle);
        let count =
            runtime.predict_candidates(&window, Some(&sig[..]), &mut node_scores, &mut cands);
        if count == 0 {
            continue;
        }
        served += 1;
        let pred = cands[0].0;
        if served <= 5 {
            println!(
                "diag: window {:?} -> cands {:?} truth {} t_argmax {}",
                window,
                &cands[..count.min(3)],
                c.next[i],
                c.t_argmax[i]
            );
        }
        if pred == c.next[i] {
            top1 += 1;
        }
        if pred == c.t_argmax[i] {
            agree += 1;
        }
        if count > 1 {
            n_multi += 1;
            if pred == c.next[i] {
                top1_multi += 1;
            }
            if pred == c.t_argmax[i] {
                agree_multi += 1;
            }
        }
    }
    println!(
        "cd-ab[{label}] sample-every {sample_every} | positions {n} | served {served} ({:.1}%) | top1 {:.2}% | teacher-agree {:.2}%",
        100.0 * served as f64 / n as f64,
        100.0 * top1 as f64 / n as f64,
        100.0 * agree as f64 / n as f64
    );
    println!(
        "cd-ab[{label}] MULTI-candidate slice (graph path ran): {n_multi} ({:.1}% of served) | top1 {:.2}% | agree {:.2}%",
        100.0 * n_multi as f64 / served.max(1) as f64,
        100.0 * top1_multi as f64 / n_multi.max(1) as f64,
        100.0 * agree_multi as f64 / n_multi.max(1) as f64
    );
}
