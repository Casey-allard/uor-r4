//! Serving-surface held-out evaluation — the certify C row (issue #280).
//!
//! Measures the configuration that actually serves: [`R4Engine`] over a
//! compiled bundle's `score.r4g1` with the D4 status policy (EXCT
//! disabled at load, widen-once, typed abstention), fed token windows —
//! the same entry the HTTP server drives — on the bundle's own held-out
//! partition.
//!
//! The previous C row measured the `convert_r4g1` certify scaffold,
//! which was never a functional prediction path (issue #280 diagnosis:
//! no per-node emissions, kind=2 transition walks structurally absent,
//! stride-8 EXCT decode over a variable-stride container). That row is
//! retired; the recorded 0.0% stands in issue #280 as the reason.
//!
//! Discipline carried over from the retired row (#279/#282/#232):
//! deterministic stride subsample, wall-clock budgets that turn silent
//! stalls into recorded skips, and a readiness probe extended with an
//! accuracy spot-check — scored + non-constant demonstrably does not
//! imply functional, so the probe now requires at least one served
//! prediction that matches the recorded corpus/teacher continuation
//! before the full sample is spent.

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction;

use crate::engine::{EngineParts, PolicyStatus, PredictDecision, R4Engine};

/// Default number of held-out positions in the stride subsample
/// (issue #244 amendment: a bounded, position-uniform measurement).
pub const SAMPLE_TARGET: usize = 1000;

/// Probe positions spent on the readiness/accuracy spot-check before
/// the full sample runs (#232 and its #280 extension).
pub const PROBE_POSITIONS: usize = 64;

/// What to evaluate: a compiled bundle directory holding the serving
/// artifacts. `graph/score.r4g1` is preferred, root `score.r4g1`
/// accepted; `score_report.json` is read from the graph's directory;
/// the teacher artifact and corpus files sit at the bundle root.
#[derive(Debug, Clone)]
pub struct ServingBundle {
    pub root: PathBuf,
    pub graph: PathBuf,
    pub teacher: PathBuf,
    pub corpus_meta: PathBuf,
    pub corpus_records: PathBuf,
}

/// Focused error enum for bundle discovery and loading.
#[derive(Debug)]
pub enum ServingEvalError {
    /// A required bundle file is missing or unreadable.
    Io { path: PathBuf, message: String },
    /// The corpus pair did not parse as a complete corpus.
    Corpus { message: String },
    /// The engine rejected the graph/teacher pair.
    Load { message: String },
    /// The bundle's held-out partition is empty.
    EmptyHeldOut,
}

impl fmt::Display for ServingEvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Corpus { message } => write!(f, "corpus: {message}"),
            Self::Load { message } => write!(f, "engine load: {message}"),
            Self::EmptyHeldOut => write!(f, "held-out partition is empty"),
        }
    }
}

impl std::error::Error for ServingEvalError {}

/// Wall-clock budgets. Defaults match the retired C row's env contract
/// (`R4_CERTIFY_R4G1_BUDGET_SECS` / `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS`
/// still override, so existing run scripts keep working).
#[derive(Debug, Clone, Copy)]
pub struct ServingEvalBudgets {
    pub probe: Duration,
    pub eval: Duration,
}

impl ServingEvalBudgets {
    /// Defaults (probe 120s, eval 600s) with the historical env overrides.
    pub fn from_env() -> Self {
        let secs = |name: &str, default: u64| {
            std::env::var(name)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        Self {
            probe: Duration::from_secs(secs("R4_CERTIFY_R4G1_BUDGET_SECS", 120)),
            eval: Duration::from_secs(secs("R4_CERTIFY_R4G1_EVAL_BUDGET_SECS", 600)),
        }
    }
}

/// Per-status decision counts (D4 policy vocabulary). Used for both
/// served and abstained positions — issue #234 item 3: every
/// evaluation run reports the count of held-out probes resolved at
/// each `ResolutionStatus` level, so blended headline numbers cannot
/// hide an exact-context-only distribution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatusBreakdown {
    pub exact_context: u64,
    pub graph: u64,
    pub novel: u64,
    pub contradictory: u64,
    /// Of `exact_context`, how many resolved via an explicit NGRAM
    /// context row rather than the EXCT probe (#362 attribution — the
    /// two mechanisms share the `ExactContext` status since e77b1d4,
    /// so era comparisons need the split).
    pub exact_context_ngram: u64,
}

impl StatusBreakdown {
    fn record(&mut self, status: PolicyStatus, ngram_hit: bool) {
        match status {
            PolicyStatus::ExactContext => {
                self.exact_context += 1;
                if ngram_hit {
                    self.exact_context_ngram += 1;
                }
            }
            PolicyStatus::Graph => self.graph += 1,
            PolicyStatus::Novel => self.novel += 1,
            PolicyStatus::Contradictory => self.contradictory += 1,
        }
    }
    pub fn total(&self) -> u64 {
        self.exact_context + self.graph + self.novel + self.contradictory
    }
}

/// The measured serving-surface row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServingEvalRow {
    /// Bundle root the row measured.
    pub bundle: PathBuf,
    /// Stride-subsample size actually evaluated.
    pub sample_n: usize,
    /// Positions where the policy served a token.
    pub served: u64,
    /// Served predictions that ran the widened re-probe first.
    pub served_widened: u64,
    /// Served predictions by resolving status (#234 item 3).
    pub served_by: StatusBreakdown,
    /// Abstentions by resolving status.
    pub abstained: StatusBreakdown,
    /// Served predictions matching the recorded corpus continuation.
    pub top1_served: u64,
    /// Served predictions matching the recorded teacher argmax.
    pub agree_served: u64,
    /// Probe evidence: positions probed / served / hits.
    pub probe_positions: usize,
    pub probe_served: u64,
    pub probe_hits: u64,
}

/// A recorded skip: the row was not measured, and the reason is data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingEvalSkip {
    /// The probe could not finish inside its budget.
    ProbeBudgetExceeded { probed: usize, elapsed: Duration },
    /// The probe served predictions but none matched the corpus
    /// continuation or teacher argmax (#280 functional spot-check).
    ProbeFunctionalCheckFailed { served: u64, probed: usize },
    /// The subsampled evaluation exceeded its budget; partial counts
    /// are discarded (a truncated prefix is biased by construction).
    EvalBudgetExceeded {
        done: usize,
        sample_n: usize,
        elapsed: Duration,
    },
}

/// Outcome of one serving-surface evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingEvalOutcome {
    Row(ServingEvalRow),
    Skipped(ServingEvalSkip),
}

impl ServingBundle {
    /// Resolve a bundle directory into its serving artifact paths.
    /// Returns `None` when any required file is absent — callers treat
    /// that as "this directory is not a compiled serving bundle".
    pub fn discover(root: &Path) -> Option<Self> {
        let graph_nested = root.join("graph").join("score.r4g1");
        let graph_flat = root.join("score.r4g1");
        let graph = if graph_nested.is_file() {
            graph_nested
        } else if graph_flat.is_file() {
            graph_flat
        } else {
            return None;
        };
        let teacher = root.join("tless_artifacts.bin");
        let corpus_meta = root.join("corpus.meta");
        let corpus_records = root.join("corpus.records");
        if !teacher.is_file() || !corpus_meta.is_file() || !corpus_records.is_file() {
            return None;
        }
        Some(Self {
            root: root.to_path_buf(),
            graph,
            teacher,
            corpus_meta,
            corpus_records,
        })
    }

    /// Scan `.uor-models/compiled/*` under `base` for serving bundles,
    /// in deterministic (sorted) directory order.
    pub fn scan(base: &Path) -> Vec<Self> {
        let compiled = base.join(".uor-models").join("compiled");
        let Ok(entries) = std::fs::read_dir(&compiled) else {
            return Vec::new();
        };
        let mut roots: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        roots.sort();
        roots.iter().filter_map(|r| Self::discover(r)).collect()
    }
}

/// Evaluate one bundle's serving surface on its held-out partition.
///
/// The split is the canonical 80/20 story cut
/// ([`induction::split_positions`]) — the `score` stage's default. A
/// bundle scored against a D3 stories-index partition is measured on a
/// superset split here; the row records rates, not a κ-pinned claim, so
/// the caveat is carried in the row text by the caller.
///
/// `progress` is called every 256 evaluated positions with
/// `(done, sample_n, elapsed_secs)` — the #279 visibility contract: a
/// stall is distinguishable from work within minutes, not hours.
pub fn evaluate_serving_bundle(
    bundle: &ServingBundle,
    budgets: ServingEvalBudgets,
    progress: &mut dyn FnMut(usize, usize, u64),
) -> Result<ServingEvalOutcome, ServingEvalError> {
    let read = |path: &Path| {
        std::fs::read(path).map_err(|error| ServingEvalError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
    };
    let graph_bytes = read(&bundle.graph)?;
    let teacher_bytes = read(&bundle.teacher)?;
    // Historical serving behavior (src/r4g1.rs): a score report that
    // does not parse is ignored (D4 defaults), not an error.
    let score_report = bundle
        .graph
        .parent()
        .and_then(|parent| std::fs::read(parent.join("score_report.json")).ok())
        .filter(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).is_ok());

    let (meta, recs) = (
        bundle
            .corpus_meta
            .to_str()
            .ok_or_else(|| ServingEvalError::Corpus {
                message: "corpus metadata path is not UTF-8".to_owned(),
            })?,
        bundle
            .corpus_records
            .to_str()
            .ok_or_else(|| ServingEvalError::Corpus {
                message: "corpus records path is not UTF-8".to_owned(),
            })?,
    );
    let corpus =
        compiler::load_corpus_from(meta, recs).ok_or_else(|| ServingEvalError::Corpus {
            message: format!(
                "incomplete corpus at {} / {}",
                bundle.corpus_meta.display(),
                bundle.corpus_records.display()
            ),
        })?;
    let (_, held_out) = induction::split_positions(&corpus);
    if held_out.is_empty() {
        return Err(ServingEvalError::EmptyHeldOut);
    }

    let mut engine = R4Engine::load(EngineParts {
        graph: &graph_bytes,
        signature_artifact: &teacher_bytes,
        tokenizer: None,
        score_report: score_report.as_deref(),
    })
    .map_err(|error| ServingEvalError::Load {
        message: error.to_string(),
    })?;

    // Deterministic stride subsample (#244 amendment; #282 mechanics).
    let sample_stride = (held_out.len() / SAMPLE_TARGET).max(1);
    let sample: Vec<usize> = held_out.iter().copied().step_by(sample_stride).collect();

    // One decision on one held-out position through the serving entry:
    // the token window the server would feed, oldest first.
    let decide = |engine: &mut R4Engine, i: usize| {
        let window = induction::context_window(&corpus, i);
        engine.predict_decision(&window)
    };

    // Readiness probe with the #280 accuracy spot-check: the first
    // PROBE_POSITIONS sample positions, budgeted. "Hit" is liberal —
    // corpus continuation or teacher argmax — because the check exists
    // to catch a structurally non-functional path, not to grade one.
    let probe_n = sample.len().min(PROBE_POSITIONS);
    let probe_start = Instant::now();
    let (mut probe_served, mut probe_hits) = (0u64, 0u64);
    for (done, &i) in sample[..probe_n].iter().enumerate() {
        if probe_start.elapsed() >= budgets.probe {
            return Ok(ServingEvalOutcome::Skipped(
                ServingEvalSkip::ProbeBudgetExceeded {
                    probed: done,
                    elapsed: probe_start.elapsed(),
                },
            ));
        }
        if let PredictDecision::Serve(outcome) =
            decide(&mut engine, i).map_err(|error| ServingEvalError::Load {
                message: error.to_string(),
            })?
        {
            probe_served += 1;
            if outcome.token == corpus.next[i] || outcome.token == corpus.t_argmax[i] {
                probe_hits += 1;
            }
        }
    }
    if probe_served > 0 && probe_hits == 0 {
        return Ok(ServingEvalOutcome::Skipped(
            ServingEvalSkip::ProbeFunctionalCheckFailed {
                served: probe_served,
                probed: probe_n,
            },
        ));
    }

    // Full subsample, from a reset engine so the row is a pure function
    // of the sample (the probe warmed the widen-once memory).
    engine.reset();
    let eval_start = Instant::now();
    let mut row = ServingEvalRow {
        bundle: bundle.root.clone(),
        sample_n: sample.len(),
        served: 0,
        served_widened: 0,
        served_by: StatusBreakdown::default(),
        abstained: StatusBreakdown::default(),
        top1_served: 0,
        agree_served: 0,
        probe_positions: probe_n,
        probe_served,
        probe_hits,
    };
    for (done, &i) in sample.iter().enumerate() {
        if eval_start.elapsed() >= budgets.eval {
            return Ok(ServingEvalOutcome::Skipped(
                ServingEvalSkip::EvalBudgetExceeded {
                    done,
                    sample_n: sample.len(),
                    elapsed: eval_start.elapsed(),
                },
            ));
        }
        match decide(&mut engine, i).map_err(|error| ServingEvalError::Load {
            message: error.to_string(),
        })? {
            PredictDecision::Serve(outcome) => {
                row.served += 1;
                row.served_by
                    .record(outcome.status.into(), outcome.ngram_hit);
                if outcome.widened {
                    row.served_widened += 1;
                }
                if outcome.token == corpus.next[i] {
                    row.top1_served += 1;
                }
                if outcome.token == corpus.t_argmax[i] {
                    row.agree_served += 1;
                }
            }
            PredictDecision::Abstain(outcome) => {
                row.abstained
                    .record(outcome.status.into(), outcome.ngram_hit);
            }
        }
        if (done + 1).is_multiple_of(256) {
            progress(done + 1, sample.len(), eval_start.elapsed().as_secs());
        }
    }
    Ok(ServingEvalOutcome::Row(row))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_requires_all_bundle_files() {
        let root = std::env::temp_dir().join(format!("r4-serving-eval-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("graph")).unwrap();
        assert!(ServingBundle::discover(&root).is_none());
        for name in ["tless_artifacts.bin", "corpus.meta", "corpus.records"] {
            std::fs::write(root.join(name), b"x").unwrap();
        }
        assert!(
            ServingBundle::discover(&root).is_none(),
            "no graph artifact yet"
        );
        std::fs::write(root.join("graph").join("score.r4g1"), b"x").unwrap();
        let bundle = ServingBundle::discover(&root).expect("bundle");
        assert_eq!(bundle.graph, root.join("graph").join("score.r4g1"));
        assert_eq!(bundle.teacher, root.join("tless_artifacts.bin"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn abstain_breakdown_records_by_status() {
        let mut b = StatusBreakdown::default();
        b.record(PolicyStatus::Novel, false);
        b.record(PolicyStatus::Novel, false);
        b.record(PolicyStatus::Graph, false);
        assert_eq!(b.novel, 2);
        assert_eq!(b.graph, 1);
        assert_eq!(b.total(), 3);
    }

    /// #362 attribution: the NGRAM subcount tracks only exact-context
    /// records, and the flag is ignored on other statuses.
    #[test]
    fn breakdown_splits_exact_context_by_ngram() {
        let mut b = StatusBreakdown::default();
        b.record(PolicyStatus::ExactContext, true);
        b.record(PolicyStatus::ExactContext, false);
        b.record(PolicyStatus::Graph, true);
        assert_eq!(b.exact_context, 2);
        assert_eq!(b.exact_context_ngram, 1);
        assert_eq!(b.graph, 1);
        assert_eq!(b.total(), 3, "the ngram split is not a fourth bucket");
    }
}
