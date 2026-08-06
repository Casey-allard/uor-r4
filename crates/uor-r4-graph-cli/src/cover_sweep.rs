//! Cover fineness sweep under the fixed scorer (graph-compiler plan §5,
//! issue #70): a rate–distortion sweep of the induced cover with the
//! issue-#64 scorer held fixed, producing the regions × bytes × agreement
//! table and a recorded operating-point recommendation.
//!
//! # What varies and what is fixed
//!
//! Only the cover fineness knobs vary. The grid is
//! `k0 ∈ {8, 16}` × split threshold `entropy_gain_bits ∈ {0.25, 0.10}` bits
//! × `regions_budget ∈ {128, 512}`, all at the default depth cap 3 — 8
//! points — plus the current default operating point (`k0 = 8`, threshold
//! 0.25 bits, budget [`cover::DEFAULT_REGIONS_BUDGET`], the 42-region
//! baseline row) for 9 points total. Everything else is pinned: the
//! scorer is [`score::ScoreConfig::default`] (the fixed #64 scorer with
//! the #66 ΔT-ablation decision deployed), the observation lane, the
//! train/held-out story cut, and the corpus/artifact inputs are shared
//! across all points.
//!
//! # Per-point pipeline
//!
//! Each point re-runs the exact `cover` → `score` compiler pipeline on
//! the shared inputs: [`cover::induce_cover`], the frozen
//! [`cover::ReferenceClassifier`], [`cover::evaluate_held_out`] for the
//! per-depth reference top-1/top-M recall and frontier width, the
//! structural edges, [`score::compile_transitions`] /
//! [`score::compile_emissions`], [`score::emit_scored_r4g1`] (the scored
//! artifact whose byte length is the rate axis), and
//! [`score::evaluate_gate_c`] on the held-out partition for the Rule 1+2
//! top-1 teacher-argmax agreement and bits/token (the distortion axis).
//!
//! # Recommendation rule (the agreement-per-byte knee)
//!
//! [`recommend`] applies the documented rule, deterministically:
//!
//! 1. Sort the sweep rows by ascending artifact bytes; ties break by
//!    descending Rule 1+2 top-1 agreement, then ascending label.
//! 2. Reduce to the rate–distortion frontier: walking the sorted rows,
//!    keep a row only when its agreement is strictly greater than every
//!    cheaper row's (dominated rows — no fidelity gain for their bytes —
//!    are dropped). Frontier steps therefore have strictly increasing
//!    bytes and strictly increasing agreement.
//! 3. Walk the frontier from the cheapest point, advancing while the
//!    marginal slope `Δagreement / Δbytes` of the step is at least
//!    [`KNEE_SLOPE_FLOOR`] (agreement per byte; `1e-7` = 10 percentage
//!    points of top-1 agreement per added megabyte). The recommendation
//!    is where the walk stops: the first step below the floor is the
//!    knee — fidelity beyond it is not earning its bytes. If no step
//!    clears the floor the cheapest frontier point is recommended; if
//!    every step does, the most expensive one is. An empty grid yields
//!    no recommendation.
//! 4. The recorded justification compares the recommendation against
//!    the baseline row (`Δbytes`, `Δagreement`). The rule never changes
//!    the default [`cover::CoverConfig`] — adoption is an explicit
//!    maintainer decision; this module only writes the recommendation
//!    into the report.
//!
//! # Report schema (`cover_sweep.json`, `schema = 3`)
//!
//! Schema 3 (#456): every sweep row additionally records `reconstruction`
//! — the EXCT-disabled reconstruction metric (held-out top-1 agreement
//! and bits/token of the graph-only Rule 1 chain,
//! [`score::evaluate_gate_c`]'s `rule1_chain`) — alongside the existing
//! with-EXCT `gate_c_rule12` agreement block, promoting the
//! graph-only reconstructability of the cover from a post-hoc Gate C
//! observation to a per-frontier-point report column. The recommendation
//! block records the recommended point's reconstruction bits/token; the
//! knee rule itself still runs on with-EXCT agreement (reporting only).
//!
//! Schema 2 (#364): the scorer block records `emission_selection`,
//! `emission_shrinkage`, `context_order`, and `context_entries`, and the
//! command accepts `--emission-selection` / `--emission-shrinkage` so the
//! granularity grid can be swept under the contrast-weighted scorer —
//! the previously unmeasured axis behind the #364 cover-ceiling verdict.
//! Everything else in the sweep is unchanged; the default invocation
//! reproduces schema-1 rows exactly (modulo the added config fields).
//! The optional `--distinctiveness-weight` applies the induction objective's
//! between-region distinctiveness reward to every cover point; its default is
//! zero, preserving the byte-exact default cover.
//!
//! ```text
//! schema:          3
//! inputs:          {artifact_kappa, corpus_kappa,
//!                   train_observations, held_out_observations}
//! scorer:          {transition_out_degree, emission_entries, root_top_b,
//!                   exct_top_x, witness_sample, smoothing,
//!                   emission_selection, emission_shrinkage,
//!                   context_order, context_entries}
//!                   — the fixed #64 scorer (with the #67 smoothing knob
//!                   and the #364/#362 emission/context axes)
//! tla3_baseline:   {positions, top1_agreement, bits_per_token}
//!                   (cover-independent store baseline, recorded once)
//! recommendation:  {label, bytes, agreement, slope_floor, frontier,
//!                   delta_bytes_vs_baseline, delta_agreement_vs_baseline,
//!                   reconstruction_bits_per_token, rationale} | null
//! points:          per point, grid order then the baseline row:
//!   {label, baseline, config: {k0, depths, entropy_gain_bits,
//!     regions_budget, min_support, memory_budget_bytes},
//!    regions: {total, per_depth, splits, max_depth},
//!    recall: per depth {depth, evaluated, reference_top1, reference_topm,
//!      frontier_mean, frontier_max},
//!    artifact_bytes, graph_kappa,
//!    gate_c_rule12: {positions, top1_agreement, bits_per_token},
//!    reconstruction: {positions, top1_agreement, bits_per_token}}
//!                   — #456: EXCT-disabled graph-only held-out score
//! determinism:     note string
//! ```
//!
//! # Determinism
//!
//! Every consumed compiler is deterministic by construction
//! (content-addressed seeds, ordered reductions, canonical sorts), so any
//! single point run twice produces byte-identical scored artifacts and
//! identical metrics (a property of the consumed compilers; the
//! historical `tests/cover_sweep.rs` reference predates the crate
//! restructuring — a dedicated sweep double-run test is future work). The f64
//! entropy/`ln` sites inherit the macOS-pinned, libm-sensitive status of
//! the cover and score compilers (their module docs); same-machine
//! double-runs are byte-exact, cross-platform byte equality awaits the D2
//! canonical deterministic compile mode.

use serde::Serialize;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use uor_r4_core::transformerless::compiler::{self, Corpus};
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_certify::{self as score, GateCMetrics, ScoreConfig};
use uor_r4_graph_compiler::induction::{self as cover, Observation};
use uor_r4_graph_compiler::reproducibility as repro;

/// The `cover_sweep.json` schema version (module docs).
pub const SWEEP_REPORT_SCHEMA: u32 = 3;

/// Grid axis: the broad depth-1 region counts under test.
pub const SWEEP_K0: [usize; 2] = [8, 16];
/// Grid axis: the split entropy-gain floors under test, in bits/token.
pub const SWEEP_ENTROPY_GAIN_BITS: [f64; 2] = [0.25, 0.10];
/// Grid axis: the total-region budgets under test.
pub const SWEEP_REGIONS_BUDGET: [usize; 2] = [128, 512];

/// Marginal agreement-per-byte floor of the recommendation rule
/// (module docs): `1e-7` = 10 percentage points of top-1 agreement per
/// added megabyte of scored artifact.
pub const KNEE_SLOPE_FLOOR: f64 = 1e-7;

/// One sweep point: the cover configuration plus its report label.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepPoint {
    /// Human/JSON label (`k0=8/gain=0.25/budget=128`).
    pub label: String,
    /// True on the default-operating-point row (the 42-region baseline).
    pub baseline: bool,
    /// The cover configuration induced at this point.
    pub config: cover::CoverConfig,
}

/// The 9-point sweep grid (module docs): 8 grid points in
/// (k0, gain, budget) nested order, then the default operating point.
pub fn sweep_grid() -> Vec<SweepPoint> {
    let mut points = Vec::with_capacity(9);
    for &k0 in &SWEEP_K0 {
        for &gain in &SWEEP_ENTROPY_GAIN_BITS {
            for &budget in &SWEEP_REGIONS_BUDGET {
                points.push(SweepPoint {
                    label: format!("k0={k0}/gain={gain}/budget={budget}"),
                    baseline: false,
                    config: cover::CoverConfig {
                        k0,
                        entropy_gain_bits: gain,
                        regions_budget: budget,
                        ..cover::CoverConfig::default()
                    },
                });
            }
        }
    }
    let config = cover::CoverConfig::default();
    points.push(SweepPoint {
        label: format!(
            "k0={}/gain={}/budget={} (default)",
            config.k0, config.entropy_gain_bits, config.regions_budget
        ),
        baseline: true,
        config,
    });
    points
}

/// The inputs shared by every sweep point, loaded/built once (data
/// bundle, the `cover::ReportData` pattern).
pub struct SweepInputs {
    /// TLA container bytes (the teacher artifact).
    pub artifact_container: Vec<u8>,
    /// Parsed teacher artifact.
    pub artifacts: compiler::Compiled,
    /// The labeled corpus stream.
    pub corpus: Corpus,
    /// Corpus metadata bytes (CID material).
    pub meta_bytes: Vec<u8>,
    /// Corpus record bytes (CID material).
    pub recs_bytes: Vec<u8>,
    /// Train observations (stories below the 80/20 cut).
    pub train: Vec<Observation>,
    /// Held-out observations (stories at/above the cut).
    pub held_out: Vec<Observation>,
    /// The graded store (EXCT compiler input + TLA3 baseline).
    pub store: Store,
    /// TLS1 container bytes of the store.
    pub tls1: Vec<u8>,
    /// κ of the artifact container.
    pub artifact_kappa: String,
    /// κ of the corpus stream (meta then records).
    pub corpus_kappa: String,
}

/// Load the shared sweep inputs from disk, mirroring the `score` CLI's
/// loading exactly (same corpus cut, same κs, same store).
pub fn load_inputs(
    corpus_meta: &Path,
    corpus_recs: &Path,
    artifacts_path: &Path,
) -> Result<SweepInputs, String> {
    let meta_str = corpus_meta
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let recs_str = corpus_recs
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    // #450: announce the resolved containers before the sweep's long work.
    let artifact_container = std::fs::read(artifacts_path)
        .map_err(|error| format!("{}: {error}", artifacts_path.display()))?;
    let artifacts = compiler::parse_artifacts(&artifact_container).ok_or_else(|| {
        format!(
            "{}: not a TLA3/TLA4/TLA5 artifact container",
            artifacts_path.display()
        )
    })?;
    let artifact_kappa = repro::container_kappa(&artifact_container);
    repro::announce_teacher_container(artifacts_path, &artifact_kappa);
    let meta_bytes = std::fs::read(corpus_meta)
        .map_err(|error| format!("{}: {error}", corpus_meta.display()))?;
    let recs_bytes = std::fs::read(corpus_recs)
        .map_err(|error| format!("{}: {error}", corpus_recs.display()))?;
    let corpus_kappa = repro::corpus_stream_kappa(&meta_bytes, &recs_bytes);
    repro::announce_corpus(corpus_meta, corpus_recs, &corpus_kappa);
    let corpus = compiler::load_corpus_from(meta_str, recs_str).ok_or_else(|| {
        format!(
            "corpus is incomplete at {}/{}; run compile until it is complete",
            corpus_meta.display(),
            corpus_recs.display()
        )
    })?;
    let (train_positions, held_out_positions) = cover::split_positions(&corpus);
    let train = cover::build_observations(&artifacts, &corpus, &train_positions);
    let held_out = cover::build_observations(&artifacts, &corpus, &held_out_positions);
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    let tls1 = runtime::store_bytes(&store);
    Ok(SweepInputs {
        artifact_container,
        artifacts,
        corpus,
        meta_bytes,
        recs_bytes,
        train,
        held_out,
        store,
        tls1,
        artifact_kappa,
        corpus_kappa,
    })
}

/// Per-depth routing numbers of one sweep point (the rate–distortion
/// table's recall columns; a focused subset of [`cover::DepthRecall`]).
#[derive(Debug, Clone, Serialize)]
pub struct SweepDepthRecall {
    /// Multiresolution depth.
    pub depth: usize,
    /// Held-out positions evaluated at this depth.
    pub evaluated: usize,
    /// P(shipped binary top-1 == exact reference top-1).
    pub reference_top1: f64,
    /// P(exact reference top-1 ∈ binary top-M membership).
    pub reference_topm: f64,
    /// Mean active-region count (frontier width) at this depth.
    pub frontier_mean: f64,
    /// Max active-region count at this depth.
    pub frontier_max: u32,
}

/// Region-count summary of one sweep point.
#[derive(Debug, Clone, Serialize)]
pub struct SweepRegions {
    /// Total induced regions.
    pub total: usize,
    /// Region count per depth (`per_depth[d - 1]`, ascending depth).
    pub per_depth: Vec<u32>,
    /// Regions with an accepted split.
    pub splits: usize,
    /// Deepest depth with at least one region.
    pub max_depth: usize,
}

/// The cover configuration columns of one report row.
#[derive(Debug, Clone, Serialize)]
pub struct SweepRowConfig {
    pub k0: usize,
    pub depths: usize,
    /// Split entropy-gain floor in bits/token.
    pub entropy_gain_bits: f64,
    pub regions_budget: usize,
    pub min_support: usize,
    pub memory_budget_bytes: u64,
    /// Induction-time reward weight against the global next-token prior.
    pub distinctiveness_weight: f64,
}

/// One rate–distortion table row: one sweep point's regions × bytes ×
/// agreement plus the routing-recall detail.
#[derive(Debug, Clone, Serialize)]
pub struct SweepRow {
    /// Sweep-point label.
    pub label: String,
    /// True on the default-operating-point (baseline) row.
    pub baseline: bool,
    /// The cover configuration induced at this point.
    pub config: SweepRowConfig,
    /// Region counts by depth.
    pub regions: SweepRegions,
    /// Per-depth held-out routing recall and frontier width.
    pub recall: Vec<SweepDepthRecall>,
    /// Scored R4G1 artifact size — the rate axis.
    pub artifact_bytes: usize,
    /// κ of the scored artifact bytes.
    pub graph_kappa: String,
    /// Gate C Rule 1+2 (chain + D4 EXCT precedence) on held-out — the
    /// distortion axis.
    pub gate_c_rule12: GateCMetrics,
    /// Issue #456: Gate C Rule 1 chain (EXCT disabled) on held-out — the
    /// graph-only reconstruction score of this cover point, recorded
    /// alongside the with-EXCT agreement.
    pub reconstruction: GateCMetrics,
}

/// The recorded operating-point recommendation (module docs for the
/// rule). `None` deltas mean the sweep carried no baseline row.
#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    /// Label of the recommended point.
    pub label: String,
    /// Its scored-artifact bytes.
    pub bytes: usize,
    /// Its Rule 1+2 top-1 agreement.
    pub agreement: f64,
    /// The slope floor applied ([`KNEE_SLOPE_FLOOR`]).
    pub slope_floor: f64,
    /// Labels of the frontier points in walk order (cheapest first).
    pub frontier: Vec<String>,
    /// `recommended − baseline` artifact bytes, when a baseline row exists.
    pub delta_bytes_vs_baseline: Option<i64>,
    /// `recommended − baseline` Rule 1+2 top-1 agreement.
    pub delta_agreement_vs_baseline: Option<f64>,
    /// The recommended point's EXCT-disabled reconstruction bits/token
    /// (#456; reporting only — the knee rule still runs on agreement).
    pub reconstruction_bits_per_token: f64,
    /// The written justification (numbers + the rule's application).
    pub rationale: String,
}

/// The fixed scorer configuration, recorded for report honesty.
#[derive(Debug, Clone, Serialize)]
pub struct SweepReportScorer {
    pub transition_out_degree: usize,
    pub emission_entries: usize,
    pub root_top_b: usize,
    pub exct_top_x: usize,
    pub witness_sample: usize,
    /// Emission smoothing rule label (`score::Smoothing::label`; the
    /// #67 knob — add-one, byte-exact with the pre-#67 compiler).
    pub smoothing: String,
    /// Emission list selection rule label (#364; schema 2).
    pub emission_selection: String,
    /// Per-region residual shrinkage label (#364; schema 2). The sweep
    /// had only ever run under the default scorer, so granularity ×
    /// contrast-shrinkage was unmeasured — the #364 ceiling question.
    pub emission_shrinkage: String,
    /// Highest compiled lexical context order (#362 knob; schema 2).
    pub context_order: u8,
    /// Per-context candidate bound for NGRAM rows (schema 2).
    pub context_entries: usize,
}

/// The shared-input provenance of the sweep.
#[derive(Debug, Clone, Serialize)]
pub struct SweepReportInputs {
    pub artifact_kappa: String,
    pub corpus_kappa: String,
    pub train_observations: usize,
    pub held_out_observations: usize,
}

/// The `cover_sweep.json` document (schema in the module docs).
#[derive(Debug, Clone, Serialize)]
pub struct SweepReport {
    pub schema: u32,
    pub inputs: SweepReportInputs,
    /// The fixed #64 scorer configuration used at every point.
    pub scorer: SweepReportScorer,
    /// The cover-independent TLA3 store baseline, recorded once.
    pub tla3_baseline: GateCMetrics,
    /// The operating-point recommendation (the documented knee rule).
    pub recommendation: Option<Recommendation>,
    /// The rate–distortion rows: grid points then the baseline row.
    pub points: Vec<SweepRow>,
    /// Determinism status note.
    pub determinism: String,
}

/// Run one sweep point: induce the cover, measure held-out routing
/// recall, emit the scored R4G1, and run Gate C. Returns the report row,
/// the cover-independent TLA3 baseline metrics (identical at every
/// point), and the scored artifact bytes (for the determinism double-run
/// assertion; the sweep itself keeps only their length and κ).
pub fn run_point(
    inputs: &SweepInputs,
    point: &SweepPoint,
    score_config: &ScoreConfig,
) -> Result<(SweepRow, GateCMetrics, Vec<u8>), String> {
    let induced = cover::induce_cover(
        &inputs.train,
        &point.config,
        &inputs.artifact_kappa,
        &inputs.corpus_kappa,
    )?;
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let recall = cover::evaluate_held_out(
        &inputs.artifacts,
        &induced.cover,
        &reference,
        &inputs.train,
        &inputs.held_out,
    );
    let edges = cover::build_edges(
        &induced.cover,
        &reference,
        &inputs.train,
        &inputs.corpus.story,
    );
    let regions = score::regions_from_cover(&induced.cover);
    let structural = score::structural_from_cover(&edges);
    let max_depth = induced.cover.max_depth;
    let (transitions, transition_quantization) = score::compile_transitions_with_quantization(
        &inputs.corpus,
        &regions,
        &inputs.train,
        max_depth,
        score_config.transition_out_degree,
    );
    let vocab = u32::try_from(inputs.artifacts.token_codes.len() / compiler::STAGES)
        .map_err(|_| "vocabulary exceeds u32 token ids".to_owned())?;
    let context_rows =
        score::compile_context_rows(&inputs.corpus, &inputs.train, vocab, score_config);
    let fwd_rows = score::compile_forward_anchor_rows(&inputs.corpus, &inputs.train);
    let emissions = score::compile_emissions(
        &inputs.corpus,
        &inputs.store,
        &regions,
        &inputs.train,
        max_depth,
        vocab,
        score_config,
    );
    let (artifact_bytes, _info) = score::emit_scored_r4g1(
        &inputs.artifact_container,
        (&inputs.meta_bytes, &inputs.recs_bytes),
        vocab,
        &score::ScoredGraphSections {
            regions: &regions,
            structural: &structural,
            transitions: &transitions,
            transition_quantization,
            emissions: &emissions,
            context_rows: &context_rows,
            exct_tls1: &inputs.tls1,
            exct_top_x: score_config.exct_top_x,
            fwd_rows: &fwd_rows,
        },
    )?;
    let gate_c = score::evaluate_gate_c(
        &artifact_bytes,
        &inputs.artifact_container,
        &inputs.artifacts,
        &inputs.store,
        &inputs.corpus,
        &inputs.held_out,
        score_config,
    )?;

    let cover = &induced.cover;
    let mut per_depth = vec![0u32; cover.max_depth];
    for region in &cover.regions {
        per_depth[region.depth as usize - 1] += 1;
    }
    let splits = cover
        .regions
        .iter()
        .filter(|r| !r.children.is_empty())
        .count();
    let graph_kappa = format!("blake3:{}", blake3::hash(&artifact_bytes).to_hex());
    let row = SweepRow {
        label: point.label.clone(),
        baseline: point.baseline,
        config: SweepRowConfig {
            k0: point.config.k0,
            depths: point.config.depths,
            entropy_gain_bits: point.config.entropy_gain_bits,
            regions_budget: point.config.regions_budget,
            min_support: point.config.min_support,
            memory_budget_bytes: point.config.memory_budget_bytes,
            distinctiveness_weight: point
                .config
                .objective
                .weights
                .between_region_distinctiveness,
        },
        regions: SweepRegions {
            total: cover.regions.len(),
            per_depth,
            splits,
            max_depth,
        },
        recall: recall
            .iter()
            .map(|d| SweepDepthRecall {
                depth: d.depth,
                evaluated: d.evaluated,
                reference_top1: d.reference_top1_recall,
                reference_topm: d.reference_topm_recall,
                frontier_mean: d.frontier_width_mean,
                frontier_max: d.frontier_width_max,
            })
            .collect(),
        artifact_bytes: artifact_bytes.len(),
        graph_kappa,
        gate_c_rule12: gate_c.rule12_precedence.clone(),
        reconstruction: gate_c.rule1_chain.clone(),
    };
    Ok((row, gate_c.tla3_baseline.clone(), artifact_bytes))
}

/// The agreement-per-byte knee rule (module docs). Deterministic: the
/// sort keys are total, so equal (bytes, agreement) rows resolve by
/// label. `None` on an empty grid.
pub fn recommend(rows: &[SweepRow]) -> Option<Recommendation> {
    if rows.is_empty() {
        return None;
    }
    let mut sorted: Vec<&SweepRow> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        a.artifact_bytes
            .cmp(&b.artifact_bytes)
            .then_with(|| {
                b.gate_c_rule12
                    .top1_agreement
                    .partial_cmp(&a.gate_c_rule12.top1_agreement)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.label.cmp(&b.label))
    });
    // Rate–distortion frontier: strictly increasing agreement as bytes
    // grow (dominated rows dropped).
    let mut frontier: Vec<&SweepRow> = Vec::new();
    for row in sorted {
        let dominated = frontier
            .last()
            .is_some_and(|f| row.gate_c_rule12.top1_agreement <= f.gate_c_rule12.top1_agreement);
        if !dominated {
            frontier.push(row);
        }
    }
    // Walk while the marginal slope clears the floor; stop at the knee.
    let mut chosen = frontier[0];
    let mut stopped_at_knee = false;
    for pair in frontier.windows(2) {
        let (prev, cur) = (pair[0], pair[1]);
        let d_bytes = (cur.artifact_bytes - prev.artifact_bytes) as f64;
        let d_agreement = cur.gate_c_rule12.top1_agreement - prev.gate_c_rule12.top1_agreement;
        if d_agreement / d_bytes >= KNEE_SLOPE_FLOOR {
            chosen = cur;
        } else {
            stopped_at_knee = true;
            break;
        }
    }
    let baseline = rows.iter().find(|r| r.baseline);
    let delta_bytes = baseline.map(|b| chosen.artifact_bytes as i64 - b.artifact_bytes as i64);
    let delta_agreement =
        baseline.map(|b| chosen.gate_c_rule12.top1_agreement - b.gate_c_rule12.top1_agreement);
    let mut rationale = String::new();
    let walk_note = if frontier.len() == 1 {
        "single-point frontier (every other point is dominated)".to_owned()
    } else if stopped_at_knee {
        format!(
            "the next frontier step's marginal slope falls below the {:.0e} agreement/byte floor",
            KNEE_SLOPE_FLOOR
        )
    } else if chosen.label == frontier[0].label {
        format!(
            "no frontier step clears the {:.0e} agreement/byte floor",
            KNEE_SLOPE_FLOOR
        )
    } else {
        format!(
            "every walked frontier step clears the {:.0e} agreement/byte floor",
            KNEE_SLOPE_FLOOR
        )
    };
    let _ = write!(
        rationale,
        "knee rule over {} frontier point(s) ({}): recommended {}",
        frontier.len(),
        walk_note,
        chosen.label
    );
    if let (Some(db), Some(da), Some(base)) = (delta_bytes, delta_agreement, baseline) {
        let verdict = if chosen.label == base.label {
            "the default operating point is itself the recommendation — the sweep finds no \
             fineness change worth its bytes under the fixed scorer"
        } else if db == 0 && da == 0.0 {
            "the recommended point ties the baseline exactly (identical bytes and fidelity — the \
             swept knob is inert between them); the 42-region default is confirmed adequate \
             under the fixed scorer: no grid point buys agreement with bytes"
        } else if da > 0.0 {
            "the 42-region default is too coarse under the fixed scorer: the recommended point \
             buys agreement at a marginal rate the floor accepts"
        } else if da == 0.0 {
            "the baseline's exact fidelity is available cheaper: the 42-region default carries \
             bytes the fixed scorer does not spend"
        } else {
            "the baseline's extra fidelity costs more per byte than the floor allows under the \
             fixed scorer"
        };
        let _ = write!(
            rationale,
            "; vs the baseline row ({} bytes, {:.4} agreement): {:+} bytes, {:+.4} agreement — {}",
            base.artifact_bytes, base.gate_c_rule12.top1_agreement, db, da, verdict
        );
    } else {
        rationale.push_str("; no baseline row in the grid, so no default comparison");
    }
    Some(Recommendation {
        label: chosen.label.clone(),
        bytes: chosen.artifact_bytes,
        agreement: chosen.gate_c_rule12.top1_agreement,
        slope_floor: KNEE_SLOPE_FLOOR,
        frontier: frontier.iter().map(|r| r.label.clone()).collect(),
        delta_bytes_vs_baseline: delta_bytes,
        delta_agreement_vs_baseline: delta_agreement,
        reconstruction_bits_per_token: chosen.reconstruction.bits_per_token,
        rationale,
    })
}

/// Run the full 9-point sweep over the shared inputs with the fixed
/// scorer and assemble the report.
pub fn run_sweep(
    inputs: &SweepInputs,
    score_config: &ScoreConfig,
    distinctiveness_weight: f64,
) -> Result<SweepReport, String> {
    let points = sweep_grid();
    let mut rows = Vec::with_capacity(points.len());
    let mut tla3_baseline: Option<GateCMetrics> = None;
    for (index, point) in points.iter().enumerate() {
        let mut point = point.clone();
        point
            .config
            .objective
            .weights
            .between_region_distinctiveness = distinctiveness_weight;
        eprintln!(
            "cover-sweep: point {}/{} ({})...",
            index + 1,
            points.len(),
            point.label
        );
        let (row, baseline_metrics, _bytes) = run_point(inputs, &point, score_config)?;
        eprintln!(
            "cover-sweep: {} regions, {} bytes, Rule 1+2 top-1 {:.4}, {:.4} bits/token",
            row.regions.total,
            row.artifact_bytes,
            row.gate_c_rule12.top1_agreement,
            row.gate_c_rule12.bits_per_token
        );
        tla3_baseline.get_or_insert(baseline_metrics);
        rows.push(row);
    }
    let tla3_baseline = tla3_baseline.ok_or_else(|| "sweep grid is empty".to_owned())?;
    let recommendation = recommend(&rows);
    Ok(SweepReport {
        schema: SWEEP_REPORT_SCHEMA,
        inputs: SweepReportInputs {
            artifact_kappa: inputs.artifact_kappa.clone(),
            corpus_kappa: inputs.corpus_kappa.clone(),
            train_observations: inputs.train.len(),
            held_out_observations: inputs.held_out.len(),
        },
        scorer: SweepReportScorer {
            transition_out_degree: score_config.transition_out_degree,
            emission_entries: score_config.emission_entries,
            root_top_b: score_config.root_top_b,
            exct_top_x: score_config.exct_top_x,
            witness_sample: score_config.witness_sample,
            smoothing: score_config.smoothing.label(),
            emission_selection: score_config.emission_selection.label(),
            emission_shrinkage: score_config.emission_shrinkage.label(),
            context_order: score_config.context_order,
            context_entries: score_config.context_entries,
        },
        tla3_baseline,
        recommendation,
        points: rows,
        determinism: "every consumed compiler is deterministic by construction (content-\
                      addressed seeds, ordered reductions, canonical sorts): any single point \
                      run twice produces byte-identical scored artifacts and identical metrics \
                      (a property of the consumed compilers); f64 entropy/ln sites are macOS-pinned \
                      and libm-sensitive cross-platform, the inherited status of the cover and \
                      score compilers (D2 resolves cross-platform byte equality later)"
            .to_owned(),
    })
}

/// The console rate–distortion table, rows ordered by artifact bytes:
/// regions × bytes × Rule 1+2 agreement, with the deepest-depth routing
/// recall and frontier width. The baseline row is marked `*`, the
/// recommended point `<- recommended`.
pub fn render_table(report: &SweepReport) -> String {
    let mut rows: Vec<&SweepRow> = report.points.iter().collect();
    rows.sort_by(|a, b| {
        a.artifact_bytes
            .cmp(&b.artifact_bytes)
            .then_with(|| a.label.cmp(&b.label))
    });
    let recommended = report.recommendation.as_ref().map(|r| r.label.as_str());
    let mut out = String::new();
    let _ = writeln!(
        out,
        "rate-distortion sweep ({} points, fixed #64 scorer, ordered by bytes):",
        rows.len()
    );
    let _ = writeln!(
        out,
        "  {:<34} {:>7} {:>10} {:>10} {:>10} {:>9} {:>9} {:>11}",
        "point", "regions", "bytes", "R1+2 top1", "bits/token", "ref-top1", "ref-topM", "frontier"
    );
    for row in rows {
        let marker = if row.baseline {
            "*"
        } else if Some(row.label.as_str()) == recommended {
            "<"
        } else {
            " "
        };
        let deepest = row.recall.last();
        let (ref1, refm, frontier) = match deepest {
            Some(d) => (
                format!("{:.1}%", 100.0 * d.reference_top1),
                format!("{:.1}%", 100.0 * d.reference_topm),
                format!("{:.2}/{}", d.frontier_mean, d.frontier_max),
            ),
            None => ("-".to_owned(), "-".to_owned(), "-".to_owned()),
        };
        let _ = writeln!(
            out,
            "{} {:<34} {:>7} {:>10} {:>9.1}% {:>10.4} {:>9} {:>9} {:>11}",
            marker,
            row.label,
            row.regions.total,
            row.artifact_bytes,
            100.0 * row.gate_c_rule12.top1_agreement,
            row.gate_c_rule12.bits_per_token,
            ref1,
            refm,
            frontier
        );
    }
    let _ = writeln!(
        out,
        "  (* = default operating point; ref/frontier columns at the deepest depth; \
         TLA3 store baseline: {:.1}% top-1, {:.4} bits/token)",
        100.0 * report.tla3_baseline.top1_agreement,
        report.tla3_baseline.bits_per_token
    );
    if let Some(rec) = &report.recommendation {
        let _ = writeln!(out, "recommendation: {}", rec.rationale);
    }
    out
}

// ------------------------------------------------------------ CLI --------

#[derive(Debug, PartialEq)]
struct CoverSweepOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    artifacts: PathBuf,
    output: PathBuf,
    emission_selection: score::EmissionSelection,
    emission_shrinkage: score::EmissionShrinkage,
    distinctiveness_weight: f64,
}

fn parse_cover_sweep_options(args: &[String]) -> Result<CoverSweepOptions, String> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = CoverSweepOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        output: PathBuf::from("cover_sweep"),
        emission_selection: score::EmissionSelection::default(),
        emission_shrinkage: score::EmissionShrinkage::default(),
        distinctiveness_weight: 0.0,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--out" => options.output = PathBuf::from(value),
            "--emission-selection" => {
                options.emission_selection = match value.as_str() {
                    "ratio" => score::EmissionSelection::Ratio,
                    "probability" => score::EmissionSelection::Probability,
                    other => {
                        return Err(format!(
                            "invalid --emission-selection value: {other} \
                             (expected ratio|probability)"
                        ));
                    }
                };
            }
            "--emission-shrinkage" => {
                options.emission_shrinkage = match value.as_str() {
                    "none" => score::EmissionShrinkage::None,
                    "witten-bell" => score::EmissionShrinkage::WittenBell,
                    "contrast" => score::EmissionShrinkage::Contrast,
                    other => {
                        return Err(format!(
                            "invalid --emission-shrinkage value: {other} \
                             (expected none|witten-bell|contrast)"
                        ));
                    }
                };
            }
            "--distinctiveness-weight" => {
                let weight = value.parse::<f64>().map_err(|error| {
                    format!("invalid --distinctiveness-weight value {value}: {error}")
                })?;
                if !weight.is_finite() || weight < 0.0 {
                    return Err(format!(
                        "invalid --distinctiveness-weight value {value} (expected finite non-negative number)"
                    ));
                }
                options.distinctiveness_weight = weight;
            }
            _ => return Err(format!("unknown cover-sweep option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

/// Cover fineness sweep (issue #70, module docs): run the 9-point
/// rate–distortion grid under the fixed scorer and write
/// `cover_sweep.json` plus the console table. `--distinctiveness-weight`
/// sets the optional induction-time between-region contrast reward.
/// Release-mode workload on the fixture corpus.
pub fn cover_sweep_command(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make the sweep much slower; use `cargo run --release -- transformerless cover-sweep ...`"
    );
    let options = parse_cover_sweep_options(args)?;
    let inputs = load_inputs(
        &options.corpus_meta,
        &options.corpus_recs,
        &options.artifacts,
    )?;
    let score_config = ScoreConfig {
        emission_selection: options.emission_selection,
        emission_shrinkage: options.emission_shrinkage,
        ..ScoreConfig::default()
    };
    eprintln!(
        "cover-sweep: {} train / {} held-out observations; running the 9-point grid (fixed scorer, \
         emission {}/{}, cover distinctiveness {})...",
        inputs.train.len(),
        inputs.held_out.len(),
        score_config.emission_selection.label(),
        score_config.emission_shrinkage.label(),
        options.distinctiveness_weight
    );
    let report = run_sweep(&inputs, &score_config, options.distinctiveness_weight)?;

    std::fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let report_json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    let report_path = options.output.join("cover_sweep.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| format!("{}: {error}", report_path.display()))?;

    print!("{}", render_table(&report));
    println!("  report: {}", report_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults_and_overrides() {
        let options = parse_cover_sweep_options(&[]).expect("defaults");
        let (default_meta, default_recs) = compiler::corpus_paths();
        assert_eq!(options.corpus_meta, PathBuf::from(default_meta));
        assert_eq!(options.corpus_recs, PathBuf::from(default_recs));
        assert_eq!(options.artifacts, PathBuf::from(compiler::ART_PATH));
        assert_eq!(options.output, PathBuf::from("cover_sweep"));
        assert_eq!(
            options.emission_selection,
            score::EmissionSelection::default()
        );
        assert_eq!(
            options.emission_shrinkage,
            score::EmissionShrinkage::default()
        );
        assert_eq!(options.distinctiveness_weight, 0.0);

        let args = [
            "--corpus-meta",
            "/tmp/m.bin",
            "--corpus-recs",
            "/tmp/r.bin",
            "--artifacts",
            "/tmp/a.bin",
            "--out",
            "/tmp/sweep",
            "--emission-selection",
            "probability",
            "--emission-shrinkage",
            "contrast",
            "--distinctiveness-weight",
            "1.5",
        ]
        .map(str::to_owned);
        let options = parse_cover_sweep_options(&args).expect("valid options");
        assert_eq!(options.corpus_meta, PathBuf::from("/tmp/m.bin"));
        assert_eq!(options.corpus_recs, PathBuf::from("/tmp/r.bin"));
        assert_eq!(options.artifacts, PathBuf::from("/tmp/a.bin"));
        assert_eq!(options.output, PathBuf::from("/tmp/sweep"));
        assert_eq!(
            options.emission_selection,
            score::EmissionSelection::Probability
        );
        assert_eq!(
            options.emission_shrinkage,
            score::EmissionShrinkage::Contrast
        );
        assert_eq!(options.distinctiveness_weight, 1.5);

        let bad_shrinkage = ["--emission-shrinkage", "sideways"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&bad_shrinkage).is_err());
        let bad_distinctiveness = ["--distinctiveness-weight", "-1"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&bad_distinctiveness).is_err());

        let bad = ["--k0", "16"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&bad).is_err());
        let missing = ["--out"].map(str::to_owned);
        assert!(parse_cover_sweep_options(&missing).is_err());
    }

    #[test]
    fn sweep_row_serializes_reconstruction_block() {
        let metrics = |bits_per_token: f64| GateCMetrics {
            positions: 10,
            top1_agreement: 0.5,
            bits_per_token,
            ..GateCMetrics::default()
        };
        let row = SweepRow {
            label: "k0=8/gain=0.25/budget=128".to_owned(),
            baseline: false,
            config: SweepRowConfig {
                k0: 8,
                depths: 3,
                entropy_gain_bits: 0.25,
                regions_budget: 128,
                min_support: 4,
                memory_budget_bytes: 64 * 1024 * 1024,
                distinctiveness_weight: 0.0,
            },
            regions: SweepRegions {
                total: 1,
                per_depth: vec![1],
                splits: 0,
                max_depth: 1,
            },
            recall: Vec::new(),
            artifact_bytes: 1024,
            graph_kappa: "blake3:test".to_owned(),
            gate_c_rule12: metrics(1.0),
            reconstruction: metrics(16.03),
        };
        let json = serde_json::to_value(&row).expect("row serializes");
        assert_eq!(json["gate_c_rule12"]["positions"], 10);
        assert_eq!(json["gate_c_rule12"]["bits_per_token"], 1.0);
        let reconstruction = &json["reconstruction"];
        assert_eq!(reconstruction["positions"], 10);
        assert_eq!(reconstruction["top1_agreement"], 0.5);
        assert_eq!(reconstruction["bits_per_token"], 16.03);

        let recommendation = Recommendation {
            label: row.label.clone(),
            bytes: row.artifact_bytes,
            agreement: 0.5,
            slope_floor: KNEE_SLOPE_FLOOR,
            frontier: vec![row.label.clone()],
            delta_bytes_vs_baseline: Some(0),
            delta_agreement_vs_baseline: Some(0.0),
            reconstruction_bits_per_token: 16.03,
            rationale: "test".to_owned(),
        };
        let json = serde_json::to_value(&recommendation).expect("recommendation serializes");
        assert_eq!(json["reconstruction_bits_per_token"], 16.03);
        assert_eq!(SWEEP_REPORT_SCHEMA, 3);
    }
}
