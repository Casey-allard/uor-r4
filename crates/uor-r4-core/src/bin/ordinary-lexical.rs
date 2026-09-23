//! One shared native served path for ordinary-text continuation and grounded exact copying.
//!
//! This runner trains and executes the single shared interface in
//! `learner::transferable_lexical`:
//!
//! ```text
//! c        = pinned request meaning + selected evidence/result + exact provenance
//! h_0      = learned_init(c)
//! a_t      = learned Generate(token) / Copy(owned occurrence) / Stop
//! x_t      = token actually emitted by a_t
//! h_(t+1)  = learned_update(h_t, representation(x_t), event, c)
//! ```
//!
//! Ordinary prose and the authored grounded world train **one** artifact. Two causal preflights run
//! on the loaded artifact before the large prose fit: an independently authored temporal meaning
//! computed from typed source state, and a post-copy vocabulary decision whose accepted word depends
//! on the copied token identity with the request, typed facts, copy length and prefixes fixed.
//!
//! The runner claims an exclusive report root before loading anything, binds the tokenizer, derived
//! tokenizer, corpus documents, configuration and source identities, writes its evidence, seals the
//! attempt and verifies the sealed file set.
#![forbid(unsafe_code)]
// The state-probe receipt is one large `serde_json::json!` object; the macro expands recursively, so
// the bin's default macro recursion limit is too small for it.
#![recursion_limit = "512"]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use sha2::{Digest, Sha256};

use uor_r4_core::native_geometric::learner::prior_learning::PriorCore;
use uor_r4_core::native_geometric::learner::realtext_support::{
    ctx2, family_p, reconstruct_corpus, tune_lambdas, Cond, Split, Uni, LAMBDA_GRID, VOCAB,
};
use uor_r4_core::native_geometric::learner::state_lexical::SlFacts;
use uor_r4_core::native_geometric::learner::transferable_lexical::{
    state_digest, TlAction, TlConfig, TlEmbed, TlExample, TlLinear, TlModel, TlTrainConfig,
    TlTrainer, TL_EMB_BOUND, TL_EMB_MAX_SHIFT, TL_EVENTS, TL_EV_COPY, TL_EV_GENERATE,
    TL_EV_OBSERVE, TL_F_DIM, TL_MAX_SHIFT,
};
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::derive_tokenizer;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

/// Context window of one ordinary-text sequence, in tokens.
const WINDOW: usize = 64;
/// Prompt tokens observed without supervision at the head of every window.
const PREFIX: usize = 8;
/// Declared exposure caps.
const FIT_MAX_WINDOWS: usize = 6000;
const DEV_MAX_DOCS: usize = 24;
const DEV_WINDOWS_PER_DOC: usize = 4;
const GEN_TOKENS: usize = 64;
/// Frozen fit-only marginal initialisation: `bo[t] = round(2^score_shift · log2 p_fit(t))`, the
/// declared cold start of the output bias, equal to `2^score_shift`. The residual maps start at
/// exactly zero codes.
const BIAS_SCALE: f64 = 16.0;
/// Declared supervision weight of one grounded example.
const GROUND_WEIGHT: f32 = 4.0;
const BOOTSTRAP_DRAWS: usize = 2000;
/// Declared defaults of the `--state-probe` float diagnostic: the restricted target population, the
/// fit schedule and the minibatch size. The probe's Adam coefficients are fixed below.
const TOP_K: usize = 1024;
const PROBE_EPOCHS: usize = 6;
const PROBE_LR: f64 = 0.003;
const PROBE_SEED: u64 = 13;
/// Declared initialisation of the refinement probe, and the two accepted values of `--probe-init`.
const PROBE_INIT: &str = PROBE_INIT_RIDGE;
const PROBE_INIT_RIDGE: &str = "ridge";
const PROBE_INIT_ARTIFACT: &str = "artifact";
const PROBE_BATCH: usize = 256;
const ADAM_BETA1: f64 = 0.9;
const ADAM_BETA2: f64 = 0.999;
const ADAM_EPS: f64 = 1e-8;
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const EXPECTED_TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
const E_ARTIFACT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/head-projection-3/corrected/empirical.cpl2";

/// Declared single-token words of the authored grounded world. Every word must encode to exactly one
/// token; the runner fails rather than silently splitting a word across the copy boundary.
const WORDS_TEMPORAL: &str = " now";
const WORDS_COPULA: [&str; 2] = [" it", " is"];
/// Declared **class labels** for the post-copy vocabulary preflight. They are arbitrary declared
/// labels, not semantic claims: the preflight asks only whether the loaded post-copy decision changes
/// with the copied token identity.
const CLASS_LABEL_A: &str = " first";
const CLASS_LABEL_B: &str = " still";
/// Declared entity values of the authored world. The selection rule is declared: a candidate word
/// is admissible only if it encodes to exactly one token under the pinned tokenizer, and the four
/// selected values are drawn from the declared candidate pool below.
const VALUES: [&str; 4] = [" red", " green", " north", " summer"];

struct Args {
    root: PathBuf,
    docs: PathBuf,
    tokenizer: PathBuf,
    source_rev: String,
    steps: u64,
    batch: usize,
    seed: u64,
    probe_words: bool,
    replay: Option<PathBuf>,
    audit: Option<PathBuf>,
    local_channel: Option<PathBuf>,
    /// `--condition <root>`: score one existing sealed artifact under both prose conditionings. The
    /// value is the **new** report root; `--root` is not used by that mode.
    condition: Option<PathBuf>,
    /// `--state-probe <root>`: fit a **float** readout on this artifact's own recorded served states
    /// and compare its development loss with the count reference. The value is the **new** report
    /// root; `--root` is not used by that mode. Diagnostic only: floating point never enters serving.
    state_probe: Option<PathBuf>,
    /// `--artifact <path>`: the `.tlx` file scored in `--condition` mode. Read in place, never copied
    /// and never re-sealed.
    artifact: Option<PathBuf>,
    /// Declared prose formulation of `run()`: the delivered `window` conditioning or the per-position
    /// `position` conditioning.
    objective: String,
    /// Declared batch schedule of `run()`: the delivered three phases or the interleaved mix.
    curriculum: String,
    /// Interleaved-schedule draws per step. Declared default `batch`.
    prose_slots: Option<usize>,
    /// Interleaved-schedule draws per step. Declared default `0`.
    ground_slots: Option<usize>,
    /// Supervision weight of one grounded example. Declared default `GROUND_WEIGHT`.
    ground_weight: Option<f32>,
    /// `--top-k <n>`: restricted target population of `--state-probe`. Declared default `TOP_K`.
    top_k: usize,
    /// `--probe-epochs <n>`: epochs of the `--state-probe` float fit. Declared default `PROBE_EPOCHS`.
    probe_epochs: usize,
    /// `--probe-lr <f>`: Adam learning rate of the `--state-probe` float fit.
    probe_lr: f64,
    /// `--probe-seed <n>`: minibatch order and label permutation of the `--state-probe` float fit.
    probe_seed: u64,
    /// `--probe-init <ridge|artifact>`: where the refinement starts. Declared default `ridge` (the
    /// closed-form least-squares readout). `artifact` starts it from the artifact's own served readout,
    /// dequantised into the probe's feature layout, which is a much better-conditioned point when the
    /// least-squares solution is far from the served one.
    probe_init: String,
    /// `--probe-skip-null`: do not run the null probe in this attempt. The receipt then records that the
    /// control was not run and cites the sealed attempts where the identical machinery was validated.
    probe_skip_null: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut docs = None;
    let mut tokenizer = PathBuf::from(DEFAULT_TOKENIZER);
    let mut source_rev = String::from("unknown");
    let mut steps = 1200u64;
    let mut batch = 4usize;
    let mut seed = 13u64;
    let mut probe_words = false;
    let mut replay = None;
    let mut audit = None;
    let mut local_channel = None;
    let mut condition = None;
    let mut state_probe = None;
    let mut artifact = None;
    let mut objective = String::from("window");
    let mut curriculum = String::from("delivered");
    let mut prose_slots = None;
    let mut ground_slots = None;
    let mut ground_weight = None;
    let mut top_k = TOP_K;
    let mut probe_epochs = PROBE_EPOCHS;
    let mut probe_lr = PROBE_LR;
    let mut probe_seed = PROBE_SEED;
    let mut probe_init = String::from(PROBE_INIT);
    let mut probe_skip_null = false;
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let k = argv[i].as_str();
        if k == "--probe-words" {
            probe_words = true;
            i += 1;
            continue;
        }
        if k == "--probe-skip-null" {
            probe_skip_null = true;
            i += 1;
            continue;
        }
        let v = || -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{k} needs a value"))
        };
        match k {
            "--root" => root = Some(PathBuf::from(v()?)),
            "--replay" => replay = Some(PathBuf::from(v()?)),
            "--audit" => audit = Some(PathBuf::from(v()?)),
            "--local-channel" => local_channel = Some(PathBuf::from(v()?)),
            "--condition" => condition = Some(PathBuf::from(v()?)),
            "--state-probe" => state_probe = Some(PathBuf::from(v()?)),
            "--artifact" => artifact = Some(PathBuf::from(v()?)),
            "--docs" => docs = Some(PathBuf::from(v()?)),
            "--tokenizer" => tokenizer = PathBuf::from(v()?),
            "--source-rev" => source_rev = v()?,
            "--objective" => objective = v()?,
            "--curriculum" => curriculum = v()?,
            "--prose-slots" => {
                prose_slots = Some(v()?.parse().map_err(|e| format!("--prose-slots: {e}"))?)
            }
            "--ground-slots" => {
                ground_slots = Some(v()?.parse().map_err(|e| format!("--ground-slots: {e}"))?)
            }
            "--ground-weight" => {
                ground_weight = Some(v()?.parse().map_err(|e| format!("--ground-weight: {e}"))?)
            }
            "--top-k" => top_k = v()?.parse().map_err(|e| format!("--top-k: {e}"))?,
            "--probe-epochs" => {
                probe_epochs = v()?.parse().map_err(|e| format!("--probe-epochs: {e}"))?
            }
            "--probe-lr" => probe_lr = v()?.parse().map_err(|e| format!("--probe-lr: {e}"))?,
            "--probe-seed" => {
                probe_seed = v()?.parse().map_err(|e| format!("--probe-seed: {e}"))?
            }
            "--probe-init" => probe_init = v()?,
            "--steps" => steps = v()?.parse().map_err(|e| format!("--steps: {e}"))?,
            "--batch" => batch = v()?.parse().map_err(|e| format!("--batch: {e}"))?,
            "--seed" => seed = v()?.parse().map_err(|e| format!("--seed: {e}"))?,
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    if objective != "window" && objective != "position" {
        return Err(format!(
            "--objective must be window or position, got {objective:?}"
        ));
    }
    if curriculum != "delivered" && curriculum != "interleaved" {
        return Err(format!(
            "--curriculum must be delivered or interleaved, got {curriculum:?}"
        ));
    }
    if top_k == 0 || top_k > VOCAB {
        return Err(format!("--top-k must be within 1..={VOCAB}, got {top_k}"));
    }
    if probe_epochs == 0 {
        return Err("--probe-epochs must be at least 1".into());
    }
    if !(probe_lr.is_finite() && probe_lr > 0.0) {
        return Err(format!(
            "--probe-lr must be positive and finite, got {probe_lr}"
        ));
    }
    if probe_init != PROBE_INIT_RIDGE && probe_init != PROBE_INIT_ARTIFACT {
        return Err(format!(
            "--probe-init must be {} or {}, got {probe_init:?}",
            PROBE_INIT_RIDGE, PROBE_INIT_ARTIFACT
        ));
    }
    Ok(Args {
        root: root.unwrap_or_else(|| PathBuf::from("")),
        docs: docs.unwrap_or_else(|| PathBuf::from("")),
        tokenizer,
        source_rev,
        steps,
        batch,
        seed,
        probe_words,
        replay,
        audit,
        local_channel,
        condition,
        state_probe,
        artifact,
        objective,
        curriculum,
        prose_slots,
        ground_slots,
        ground_weight,
        top_k,
        probe_epochs,
        probe_lr,
        probe_seed,
        probe_init,
        probe_skip_null,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The declared single-token words, resolved against the tokenizer.
struct Words {
    temporal: u32,
    copula: [u32; 2],
    class_a: u32,
    class_b: u32,
    values: Vec<u32>,
}

fn read_words(tokenizer: &HfBpeTokenizer) -> Result<Words, String> {
    let one = |w: &str| -> Result<u32, String> {
        let t = tokenizer.encode(w);
        if t.len() != 1 {
            return Err(format!("declared word {w:?} encodes to {} tokens", t.len()));
        }
        Ok(t[0])
    };
    let mut values = Vec::new();
    for v in VALUES.iter() {
        values.push(one(v)?);
    }
    Ok(Words {
        temporal: one(WORDS_TEMPORAL)?,
        copula: [one(WORDS_COPULA[0])?, one(WORDS_COPULA[1])?],
        class_a: one(CLASS_LABEL_A)?,
        class_b: one(CLASS_LABEL_B)?,
        values,
    })
}

const PROBE_CANDIDATES: [&str; 24] = [
    " red", " blue", " green", " north", " south", " east", " west", " gold", " silver", " stone",
    " river", " summer", " winter", " spring", " autumn", " alpha", " beta", " gamma", " delta",
    " cell", " wave", " prime", " set", " tree",
];

fn probe_words(tokenizer: &HfBpeTokenizer) {
    let mut all: Vec<&str> = vec![WORDS_TEMPORAL, CLASS_LABEL_A, CLASS_LABEL_B];
    all.extend(WORDS_COPULA.iter().copied());
    all.extend(VALUES.iter().copied());
    all.extend(PROBE_CANDIDATES.iter().copied());
    for w in all {
        let t = tokenizer.encode(w);
        println!("{w:>10?} -> {t:?}  ({} token)", t.len());
    }
    println!("decode: {:?}", tokenizer.decode(&tokenizer.encode(" Alma")));
}

#[derive(Default, Clone)]
struct Eval {
    per_doc: Vec<(f64, usize)>,
}

impl Eval {
    fn total(&self) -> (f64, usize) {
        self.per_doc
            .iter()
            .fold((0.0, 0usize), |(a, b), (l, n)| (a + l, b + n))
    }
    fn micro(&self) -> f64 {
        let (l, n) = self.total();
        if n == 0 {
            f64::NAN
        } else {
            l / n as f64
        }
    }
}

/// Paired document-cluster bootstrap of a loss *difference* `a - b`.
fn paired_interval(a: &Eval, b: &Eval, seed: u64) -> (f64, f64, f64) {
    let n = a.per_doc.len().min(b.per_doc.len());
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let d: Vec<(f64, f64, f64, f64)> = (0..n)
        .map(|i| {
            let (la, na) = a.per_doc[i];
            let (lb, nb) = b.per_doc[i];
            (la, na as f64, lb, nb as f64)
        })
        .collect();
    let point = (d.iter().map(|x| x.0).sum::<f64>() / d.iter().map(|x| x.1).sum::<f64>())
        - (d.iter().map(|x| x.2).sum::<f64>() / d.iter().map(|x| x.3).sum::<f64>());
    let mut st = seed | 1;
    let mut boots = Vec::with_capacity(BOOTSTRAP_DRAWS);
    for _ in 0..BOOTSTRAP_DRAWS {
        let (mut sa, mut na, mut sb, mut nb) = (0f64, 0f64, 0f64, 0f64);
        for _ in 0..n {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            let (la, n1, lb, n2) = d[(st as usize) % n];
            sa += la;
            na += n1;
            sb += lb;
            nb += n2;
        }
        if na > 0.0 && nb > 0.0 {
            boots.push(sa / na - sb / nb);
        }
    }
    if boots.len() < BOOTSTRAP_DRAWS / 2 {
        return (point, f64::NAN, f64::NAN);
    }
    boots.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let lo = boots[(boots.len() as f64 * 0.025) as usize];
    let hi = boots[(boots.len() as f64 * 0.975) as usize];
    (point, lo, hi)
}

/// One ordinary-text window, cut so a window that ends a document carries the terminal `Stop`.
#[derive(Clone)]
struct ProseWindow {
    doc: usize,
    tokens: Vec<u32>,
    terminal: bool,
}

fn prose_windows(tokens: &[u32], doc: usize) -> Vec<ProseWindow> {
    let mut out = Vec::new();
    let mut start = 0usize;
    while start + PREFIX + 2 <= tokens.len() {
        if tokens.len() - start <= WINDOW {
            out.push(ProseWindow {
                doc,
                tokens: tokens[start..].to_vec(),
                terminal: true,
            });
            break;
        }
        out.push(ProseWindow {
            doc,
            tokens: tokens[start..start + WINDOW].to_vec(),
            terminal: false,
        });
        start += WINDOW;
    }
    out
}

fn prose_example(w: &ProseWindow) -> TlExample {
    let mut actions: Vec<TlAction> = w.tokens[PREFIX..]
        .iter()
        .map(|t| TlAction::Generate(*t))
        .collect();
    if w.terminal {
        actions.push(TlAction::Stop);
    }
    TlExample {
        sel: Vec::new(),
        res: Vec::new(),
        facts: SlFacts::default(),
        observed: w.tokens[..PREFIX].to_vec(),
        actions,
        weight: 1.0,
        doc: w.doc,
        grounded: false,
        terminal_stop: w.terminal,
    }
}

/// One authored grounded case: the exact owned span, the evidence, the typed provenance and the
/// accepted action script.
#[derive(Clone)]
struct Grounded {
    name: &'static str,
    sel: Vec<u32>,
    res: Vec<u32>,
    facts: SlFacts,
    accepted: Vec<TlAction>,
    /// Declared preflight role, empty for training cases.
    role: &'static str,
}

impl Grounded {
    fn example(&self, weight: f32, doc: usize) -> TlExample {
        TlExample {
            sel: self.sel.clone(),
            res: self.res.clone(),
            facts: self.facts,
            observed: Vec::new(),
            actions: self.accepted.clone(),
            weight,
            doc,
            grounded: true,
            terminal_stop: true,
        }
    }
}

/// The authored grounded world: the four typed regimes of the temporal preflight, the two post-copy
/// class cases, and two **held-out relation compositions** that combine typed dimensions in a way the
/// training cross-product never presents.
#[allow(clippy::too_many_arguments)]
fn authored_world(w: &Words) -> (Vec<Grounded>, Vec<Grounded>, Vec<Grounded>) {
    let copy_stop = vec![TlAction::Copy, TlAction::Stop];
    let copy_temporal = vec![
        TlAction::Copy,
        TlAction::Generate(w.temporal),
        TlAction::Stop,
    ];
    let mut train: Vec<Grounded> = Vec::new();

    // Regime 1: a different derived address with an equal value. `key_changed` is true and the
    // authored meaning is **not** temporal.
    for v in &w.values {
        train.push(Grounded {
            name: "regime1_changed_key_equal_value",
            sel: vec![*v],
            res: vec![*v],
            facts: SlFacts {
                derived: true,
                key_changed: true,
                ..SlFacts::default()
            },
            accepted: copy_stop.clone(),
            role: "preflight_temporal",
        });
    }
    // Regime 2: the same address, a different value, and **no committed mutation**.
    for v in &w.values {
        for u in &w.values {
            if v == u {
                continue;
            }
            train.push(Grounded {
                name: "regime2_changed_value_no_commit",
                sel: vec![*v],
                res: vec![*u],
                facts: SlFacts {
                    derived: true,
                    prior_differs: true,
                    ..SlFacts::default()
                },
                accepted: copy_stop.clone(),
                role: "preflight_temporal",
            });
        }
    }
    // Regime 3: a plain read with no committed correction.
    for v in &w.values {
        train.push(Grounded {
            name: "regime3_no_commit",
            sel: vec![*v],
            res: Vec::new(),
            facts: SlFacts::default(),
            accepted: copy_stop.clone(),
            role: "preflight_temporal",
        });
    }
    // Regime 4: a value was actually committed at the answered address and an earlier value
    // differed. Only this regime carries the temporal word.
    for v in &w.values {
        for u in &w.values {
            if v == u {
                continue;
            }
            train.push(Grounded {
                name: "regime4_committed_correction",
                sel: vec![*v],
                res: vec![*u],
                facts: SlFacts {
                    derived: true,
                    committed: true,
                    prior_differs: true,
                    key_changed: false,
                    ..SlFacts::default()
                },
                accepted: copy_temporal.clone(),
                role: "preflight_temporal",
            });
        }
    }
    // Post-copy class cases: the request, typed facts, copy length and prefixes are identical; only
    // the copied token identity differs and the accepted post-copy word is authored from it.
    // The class cases carry a distinct declared requested view (`history = 2`), which no typed
    // regime uses, so they never present the same input as a regime with a different target.
    let class_facts = SlFacts {
        history: 2,
        ..SlFacts::default()
    };
    let mut class_cases: Vec<Grounded> = Vec::new();
    for (token, label, name) in [
        (w.values[0], w.class_a, "class_a"),
        (w.values[2], w.class_b, "class_b"),
    ] {
        class_cases.push(Grounded {
            name,
            sel: vec![token],
            res: Vec::new(),
            facts: class_facts,
            accepted: vec![TlAction::Copy, TlAction::Generate(label), TlAction::Stop],
            role: "preflight_class",
        });
    }
    train.extend(class_cases.clone());
    // The class cases are supervised in both directions so neither token is a shortcut for a label.
    class_cases.push(Grounded {
        name: "class_a_swap",
        sel: vec![w.values[1]],
        res: Vec::new(),
        facts: class_facts,
        accepted: vec![
            TlAction::Copy,
            TlAction::Generate(w.class_a),
            TlAction::Stop,
        ],
        role: "preflight_class",
    });
    class_cases.push(Grounded {
        name: "class_b_swap",
        sel: vec![w.values[3]],
        res: Vec::new(),
        facts: class_facts,
        accepted: vec![
            TlAction::Copy,
            TlAction::Generate(w.class_b),
            TlAction::Stop,
        ],
        role: "preflight_class",
    });
    for extra in class_cases.iter().skip(2) {
        train.push(extra.clone());
    }

    // Held-out compositions: history view and commit facts combined in a way no training case shows.
    let held_out = vec![
        Grounded {
            name: "heldout_previous_view_committed_correction",
            sel: vec![w.values[1]],
            res: vec![w.values[3]],
            facts: SlFacts {
                history: 1,
                derived: true,
                committed: true,
                prior_differs: true,
                key_changed: false,
            },
            accepted: copy_temporal.clone(),
            role: "heldout_composition",
        },
        Grounded {
            name: "heldout_initial_view_committed_via_changed_key",
            sel: vec![w.values[2]],
            res: vec![w.values[2]],
            facts: SlFacts {
                history: 3,
                derived: true,
                committed: true,
                prior_differs: true,
                key_changed: true,
            },
            accepted: copy_temporal.clone(),
            role: "heldout_composition",
        },
        Grounded {
            name: "heldout_previous_view_observed_change_without_commit",
            sel: vec![w.values[3]],
            res: vec![w.values[0]],
            facts: SlFacts {
                history: 1,
                derived: true,
                committed: false,
                prior_differs: true,
                key_changed: false,
            },
            accepted: copy_stop.clone(),
            role: "heldout_composition",
        },
    ];
    (train, class_cases, held_out)
}

/// The served grounded panel: every authored case is executed through the loaded artifact.
fn grounded_panel(model: &TlModel, cases: &[Grounded]) -> Vec<serde_json::Value> {
    cases
        .iter()
        .map(|c| {
            let r = model.rollout(
                &c.sel,
                &c.res,
                c.facts,
                &[],
                &c.sel,
                c.accepted.len() + 2,
                false,
                false,
            );
            serde_json::json!({
                "name": c.name,
                "role": c.role,
                "accepted_actions": c.accepted.iter().map(action_name).collect::<Vec<_>>(),
                "served_actions": r.actions.iter().map(action_name).collect::<Vec<_>>(),
                "served_tokens": r.tokens,
                "stopped": r.stopped,
                "state_digest": r.state_digest,
                "exact": r.actions == c.accepted,
                "temporal_meaning": TlModel::truthful_temporal_meaning(c.facts),
            })
        })
        .collect()
}

fn action_name(a: &TlAction) -> String {
    match a {
        TlAction::Generate(v) => format!("Generate({v})"),
        TlAction::Copy => "Copy".into(),
        TlAction::Stop => "Stop".into(),
    }
}

fn run(args: Args) -> Result<ExitCode, String> {
    if args.root.as_os_str().is_empty() || args.docs.as_os_str().is_empty() {
        return Err("--root and --docs are required".into());
    }
    claim(&args.root).map_err(|e| format!("claim {}: {e}", args.root.display()))?;
    let started = Instant::now();
    let root = args.root.clone();
    println!("=== ordinary-lexical: one shared Generate/Copy/Stop path ===");

    // ---- tokenizer identity ------------------------------------------------
    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&Sha256::digest(&tok_bytes));
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer sha256 {tok_sha} != expected {EXPECTED_TOKENIZER_SHA256}"
        ));
    }
    let derived_bytes =
        uor_r4_core::transformerless::bpe_derive::derive_tokenizer_json(&tok_bytes, VOCAB)
            .map_err(|e| format!("derive bytes: {e}"))?;
    let derived_sha = sha256_hex(&Sha256::digest(&derived_bytes));
    let tokenizer = derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive: {e}"))?;
    write_checked(&root, "tokenizer_source.json", &tok_bytes)?;
    write_checked(&root, "tokenizer_derived_v4096.json", &derived_bytes)?;
    println!("tokenizer source {tok_sha}\ntokenizer derived {derived_sha}");
    let words = read_words(&tokenizer)?;
    println!(
        "authored words: temporal={} copula={:?} class_a={} class_b={} values={:?}",
        words.temporal, words.copula, words.class_a, words.class_b, words.values
    );

    // ---- corpus, source-separated ------------------------------------------
    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
    let mut groups: HashMap<String, usize> = HashMap::new();
    let mut fit_docs = Vec::new();
    let mut tune_docs = Vec::new();
    let mut dev_pool = Vec::new();
    for (i, d) in uniq.iter().enumerate() {
        let group = d.path.split('/').next().unwrap_or("?").to_string();
        *groups.entry(group).or_insert(0) += 1;
        match d.split {
            Split::Fit => fit_docs.push(i),
            Split::Tune => tune_docs.push(i),
            Split::Dev => dev_pool.push(i),
        }
    }
    let mut group_list: Vec<(String, usize)> = groups.into_iter().collect();
    group_list.sort();
    println!(
        "corpus collected={collected} unique={} duplicates_grouped={duplicates} \
         fit_docs={} tune_docs={} dev_pool={}",
        uniq.len(),
        fit_docs.len(),
        tune_docs.len(),
        dev_pool.len()
    );
    println!("source groups: {group_list:?}");
    if fit_docs.is_empty() || dev_pool.is_empty() {
        return Err("corpus split has an empty fit or development side".into());
    }

    // ---- windows -----------------------------------------------------------
    let mut fit_windows: Vec<ProseWindow> = Vec::new();
    for i in &fit_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), *i) {
            fit_windows.push(w);
        }
    }
    let all_fit_windows = fit_windows.len();
    if fit_windows.len() > FIT_MAX_WINDOWS {
        let stride = fit_windows.len() as f64 / FIT_MAX_WINDOWS as f64;
        fit_windows = (0..FIT_MAX_WINDOWS)
            .map(|k| {
                let idx = (k as f64 * stride) as usize;
                fit_windows[idx].clone()
            })
            .collect();
    }
    // Development: stratify the dev pool by document token length.
    let mut with_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    with_len.sort_by_key(|(_, n)| *n);
    let take = DEV_MAX_DOCS.min(with_len.len());
    let mut dev_windows: Vec<ProseWindow> = Vec::new();
    let mut dev_names: Vec<String> = Vec::new();
    for k in 0..take {
        let (doc_idx, _) = with_len[k * with_len.len() / take.max(1)];
        dev_names.push(uniq[doc_idx].path.clone());
        for w in prose_windows(&tokenizer.encode(&uniq[doc_idx].text), k)
            .into_iter()
            .take(DEV_WINDOWS_PER_DOC)
        {
            dev_windows.push(w);
        }
    }
    let dev_doc_count = dev_windows
        .iter()
        .map(|w| w.doc)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    // The per-position all-`OBSERVE` conditioning of the same development windows: one `Generate`
    // target per position from `PREFIX` onward, over the identical 5,376 target identities. The
    // window conditioning in `development.*` is unchanged and remains the served comparison.
    let position_dev_examples = position_examples(&dev_windows, 0);
    let dev_targets: usize = dev_windows
        .iter()
        .map(|w| w.tokens.len().saturating_sub(PREFIX))
        .sum();
    let fit_targets: usize = fit_windows
        .iter()
        .map(|w| w.tokens.len().saturating_sub(PREFIX))
        .sum();
    println!(
        "fit windows={} (of {all_fit_windows}) scored_targets={fit_targets} | \
         development docs={} windows={} scored_targets={dev_targets}",
        fit_windows.len(),
        dev_names.len(),
        dev_windows.len()
    );
    let mut tune_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), 0) {
            tune_windows.push(w.tokens);
        }
    }

    // ---- count references fitted on the fit split only ---------------------
    let mut uni = Uni {
        counts: vec![0u64; VOCAB],
        total: 0,
    };
    let mut c1 = Cond::default();
    let mut c2 = Cond::default();
    for w in &fit_windows {
        for k in PREFIX..w.tokens.len() {
            let prev = if k == 1 {
                VOCAB
            } else {
                (w.tokens[k - 2] as usize).min(VOCAB - 1)
            };
            let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
            let next = w.tokens[k];
            uni.counts[next as usize] += 1;
            uni.total += 1;
            c1.observe(cur as u64, next);
            c2.observe(ctx2(prev, cur), next);
        }
    }
    let (lambdas, tune_bits) = tune_lambdas(&c1, &c2, &uni, &tune_windows);
    println!(
        "count references: unigram total={} tuned lambdas={lambdas:?} tune bits/target={tune_bits:.4}",
        uni.total
    );
    let e_bytes = std::fs::read(E_ARTIFACT).map_err(|e| format!("E artifact: {e}"))?;
    let e_core = PriorCore::from_bytes(&e_bytes).map_err(|e| format!("E load: {e}"))?;
    let e_sha = sha256_hex(&Sha256::digest(&e_bytes));
    println!("loaded donor prior E: {} B sha256 {e_sha}", e_bytes.len());

    // ---- the authored grounded world --------------------------------------
    let (grounded_train, class_cases, held_out) = authored_world(&words);
    let content_tokens = model_content_tokens(&grounded_train);
    println!("grounded content tokens (declared identity set): {content_tokens}");
    println!(
        "grounded supervision: {} training cases, {} class cases, {} held-out compositions",
        grounded_train.len(),
        class_cases.len(),
        held_out.len()
    );

    // ---- fit ---------------------------------------------------------------
    let cfg = TlConfig::new(VOCAB);
    let mut trainer = TlTrainer::new(
        cfg,
        TlTrainConfig {
            lr: 0.02,
            seed: args.seed,
            ..Default::default()
        },
    )?;
    // The declared marginal covers the extended alphabet: the two action rows are treated as two
    // further symbols with the add-one prior, so `Stop` does not start with probability one.
    let denom = uni.total as f64 + (VOCAB + 2) as f64;
    let mut log2_p: Vec<f64> = (0..VOCAB)
        .map(|t| ((uni.counts[t] as f64 + 1.0) / denom).log2())
        .collect();
    let action_prior = (1.0 / denom).log2();
    log2_p.push(action_prior);
    log2_p.push(action_prior);
    trainer.set_output_bias(&log2_p, BIAS_SCALE)?;
    let untrained = trainer.model()?;
    // Declared prose formulation. `window` (default) is the delivered example builder, unchanged;
    // `position` scores each window position from its whole preceding context under `OBSERVE`.
    let prose_examples: Vec<TlExample> = match args.objective.as_str() {
        "position" => position_examples(&fit_windows, 0),
        _ => fit_windows.iter().map(prose_example).collect(),
    };
    // Effective grounded supervision weight. `None` is the declared constant, so the delivered path
    // is unchanged.
    let ground_weight = args.ground_weight.unwrap_or(GROUND_WEIGHT);
    // Declared phased curriculum into one served artifact: a prose warm-up, then a rehearsal mix in
    // which grounded supervision is interleaved so neither responsibility is forgotten.
    let phase1 = (args.steps as f64 * 0.45) as u64;
    let phase2_end = (args.steps as f64 * 0.85) as u64;
    let base_lr = trainer.tcfg.lr;
    // The interleaved curriculum is a different declared schedule on the same examples: the first
    // 45% of steps is the delivered prose warm-up, every later step draws `prose_slots` prose and
    // `ground_slots` grounded examples with the learning rate held at `base_lr` (no phase-3 scale).
    // Declared defaults (`prose_slots = batch`, `ground_slots = 0`) make that schedule pure prose.
    let interleaved = args.curriculum == "interleaved";
    let iw_prose_slots = args.prose_slots.unwrap_or(args.batch);
    let iw_ground_slots = args.ground_slots.unwrap_or(0);
    let grounded_gradient_weight_share = {
        let g = iw_ground_slots as f64 * ground_weight as f64;
        let d = iw_prose_slots as f64 + g;
        if d == 0.0 {
            0.0
        } else {
            g / d
        }
    };
    let probe_windows: Vec<ProseWindow> = dev_windows.iter().take(8).cloned().collect();
    let probe_docs = probe_windows
        .iter()
        .map(|w| w.doc)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let mut order: Vec<usize> = (0..prose_examples.len()).collect();
    let mut cursor = 0usize;
    let mut schedule_rng = args.seed ^ 0x5DEE_CE66;
    let t_fit = Instant::now();
    let mut grounded_rot = 0usize;
    for step in 0..args.steps {
        let mut batch: Vec<TlExample> = Vec::new();
        let (prose_slots, ground_slots) = if interleaved {
            if step < phase1 {
                (args.batch, 0)
            } else {
                (iw_prose_slots, iw_ground_slots)
            }
        } else if step < phase1 {
            (args.batch, 0)
        } else if step < phase2_end {
            (args.batch.saturating_sub(2).max(1), 6)
        } else {
            // Final grounded lock-in on the same artifact: the shared prose maps are already fitted,
            // and the grounded supervision is pinned before the loaded panel is executed.
            trainer.tcfg.lr = base_lr * 0.6;
            (0, 8)
        };
        for _ in 0..prose_slots {
            if cursor >= order.len() {
                shuffle(&mut order, &mut schedule_rng);
                cursor = 0;
            }
            batch.push(prose_examples[order[cursor]].clone());
            cursor += 1;
        }
        for _ in 0..ground_slots {
            let g = &grounded_train[grounded_rot % grounded_train.len()];
            grounded_rot += 1;
            batch.push(g.example(ground_weight, 0));
        }
        let _report = trainer.train_batch(&batch);
        if step % 100 == 0 || step + 1 == args.steps {
            if let Ok(m) = trainer.model() {
                let probe = evaluate_single(&m, &probe_windows);
                let probe_position =
                    eval_examples(&m, &position_examples(&probe_windows, 0), probe_docs).micro();
                let examples: Vec<TlExample> = grounded_train
                    .iter()
                    .map(|g| g.example(ground_weight, 0))
                    .collect();
                let total: usize = grounded_train.iter().map(|g| g.accepted.len()).sum();
                let (exact, _) = m.teacher_forced_agreement(&examples);
                println!(
                    "step {step:>5} dev-probe bits/target window {probe:.4} position {probe_position:.4} grounded actions {exact}/{total}"
                );
            }
        }
    }
    let fit_seconds = t_fit.elapsed().as_secs_f64();
    let model = trainer.model()?;
    let artifact = model.to_bytes();
    write_checked(&root, "artifacts/model.tlx", &artifact)?;
    println!(
        "fitted {:.3}s; artifact {} B; table {} B; nonzero reads/step {}",
        fit_seconds,
        artifact.len(),
        model.table_bytes(),
        model.nonzero_per_step()
    );
    let collisions = model.embedding_collisions();
    println!(
        "quantised embedding collisions: {} pairs over {} tokens (witness {:?})",
        collisions.colliding_pairs, collisions.colliding_tokens, collisions.witness
    );

    // ---- preflight A: authored temporal meaning ---------------------------
    let panel = grounded_panel(&model, &grounded_train);
    let panel_held_out = grounded_panel(&model, &held_out);
    let panel_class = grounded_panel(&model, &class_cases);
    let temporal_cases: Vec<&serde_json::Value> = panel
        .iter()
        .filter(|v| v["role"] == "preflight_temporal")
        .collect();
    let temporal_exact = temporal_cases
        .iter()
        .filter(|v| v["exact"] == serde_json::json!(true))
        .count();
    // Observation separation: the typed blocks of the four regimes must be pairwise distinct.
    // The typed block must separate the four regimes *between* them; within a regime the block is
    // deliberately identical, which is exactly what the identity-blinding control relies on.
    let mut by_regime: std::collections::BTreeMap<String, Vec<Vec<i32>>> =
        std::collections::BTreeMap::new();
    for g in grounded_train
        .iter()
        .filter(|g| g.role == "preflight_temporal")
    {
        by_regime
            .entry(g.name.to_string())
            .or_default()
            .push(model.typed_block(&g.sel, &g.res, g.facts));
    }
    let sigs: Vec<Vec<Vec<i32>>> = by_regime.values().cloned().collect();
    let mut separation = sigs.len() == 4;
    for i in 0..sigs.len() {
        for j in (i + 1)..sigs.len() {
            if sigs[i].iter().any(|a| sigs[j].contains(a)) {
                separation = false;
            }
        }
    }
    let within_regime_identical = by_regime
        .values()
        .all(|v| v.iter().all(|b| v.iter().filter(|c| *c == b).count() > 1));
    // The retained route-key rule would answer the *opposite* for the two load-bearing regimes.
    let route_key_failures = grounded_train
        .iter()
        .filter(|g| g.role == "preflight_temporal")
        .filter(|g| g.facts.key_changed != TlModel::truthful_temporal_meaning(g.facts))
        .count();
    let temporal_failures: Vec<String> = temporal_cases
        .iter()
        .filter(|v| v["exact"] != serde_json::json!(true))
        .map(|v| {
            format!(
                "{}: accepted={:?} served={:?}",
                v["name"], v["accepted_actions"], v["served_actions"]
            )
        })
        .collect();
    println!(
        "preflight A: {temporal_exact}/{} authored temporal regimes exact; typed blocks separate \
         regimes={separation} (within-regime identical={within_regime_identical}); retained \
         route-key rule contradicted on {route_key_failures} cases",
        temporal_cases.len()
    );
    for f in &temporal_failures {
        println!("  preflight A miss: {f}");
    }
    println!("held-out compositions: {panel_held_out:?}");

    // ---- preflight B: post-copy vocabulary depends on copied identity ------
    let pb = preflight_post_copy(&model, &class_cases[0], &class_cases[1]);
    println!(
        "preflight B: actual arms exact={} blinded arms alias={} source-disabled loses copy={}",
        pb.actual_exact, pb.blinded_alias, pb.source_disabled_loses
    );

    // ---- development evaluation -------------------------------------------
    let dev = evaluate_layers(&model, &dev_windows, &uni, &c1, &c2, lambdas, &e_core);
    let (m_bits, u_bits, c_bits, e_bits) = (
        dev[0].micro(),
        dev[1].micro(),
        dev[2].micro(),
        dev[3].micro(),
    );
    println!(
        "development bits/target: shared model {m_bits:.4} | unigram {u_bits:.4} | \
         tuned interpolated count {c_bits:.4} | loaded E (exposed) {e_bits:.4}"
    );
    let vs_unigram = paired_interval(&dev[1], &dev[0], 0xA11CE);
    let vs_count = paired_interval(&dev[2], &dev[0], 0xB0B);
    let vs_e = paired_interval(&dev[3], &dev[0], 0xC0FFEE);
    println!("  gain vs unigram  (ref - model) {:?}", vs_unigram);
    println!("  gain vs count    (ref - model) {:?}", vs_count);
    println!("  gain vs E        (ref - model) {:?}", vs_e);

    // ---- the other prose conditioning of the same targets ------------------
    // The token-only references above already share this target population (one Generate target per
    // dev position), so they are reused rather than refitted.
    let position_eval = eval_examples(&model, &position_dev_examples, dev_doc_count);
    let position_bits = position_eval.micro();
    let position_vs_window = paired_interval(&position_eval, &dev[0], 0x9E3779B9);
    let untrained_position_bits =
        eval_examples(&untrained, &position_dev_examples, dev_doc_count).micro();
    println!(
        "development bits/target: position conditioning {position_bits:.4} over {} targets | \
         position minus window {:?} (negative = position lower)",
        position_eval.total().1,
        position_vs_window
    );
    println!("  untrained position conditioning {untrained_position_bits:.4}");

    // ---- loaded generation -------------------------------------------------
    let generation = generate_samples(&model, &tokenizer, &uni, &c1, &c2, lambdas, &dev_windows);
    for g in &generation {
        println!("prompt {}: {:?}", g["prompt_text"], g["model_text"]);
        println!("   count ref: {:?}", g["count_text"]);
    }

    // ---- mixed session + separate-process restore --------------------------
    let prompt: Vec<u32> = dev_windows
        .first()
        .map(|w| w.tokens[..WINDOW.min(w.tokens.len())].to_vec())
        .unwrap_or_default();
    let session = mixed_session(&model, &words, &grounded_train, &held_out, &prompt);
    let panel_json = serde_json::json!({
        "training_panel": panel,
        "held_out_panel": panel_held_out,
        "class_panel": panel_class,
    });
    write_checked(
        &root,
        "grounded_panel.json",
        serde_json::to_string_pretty(&panel_json)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    let reload_ok = separate_process_restore(&root, &args.tokenizer, &artifact)?;
    println!(
        "mixed session: steps={} saved_phases={} separate-process restore={reload_ok}",
        session["steps"], session["saved_phases"]
    );

    // ---- cost --------------------------------------------------------------
    let costs = measure_costs(&model, &tokenizer, &artifact);
    println!(
        "cost: served {:.1} us/token, artifact {} B, table {} B, peak RSS measured externally",
        costs["served_us_per_token"].as_f64().unwrap_or(f64::NAN),
        artifact.len(),
        model.table_bytes()
    );

    // ---- seal ---------------------------------------------------------------
    let receipt = serde_json::json!({
        "schema": "uor-r4.ordinary-lexical/1",
        "source_rev": args.source_rev,
        "tokenizer_source_sha256": tok_sha,
        "tokenizer_derived_sha256": derived_sha,
        "corpus": {
            "collected": collected,
            "unique": uniq.len(),
            "exact_duplicates_grouped": duplicates,
            "fit_docs": fit_docs.len(),
            "tune_docs": tune_docs.len(),
            "dev_docs": dev_names.len(),
            "source_groups": group_list,
            "fit_windows_used": fit_windows.len(),
            "fit_windows_available": all_fit_windows,
            "fit_scored_targets": fit_targets,
            "dev_scored_targets": dev_targets,
            "document_sha256": uniq.iter().map(|d| serde_json::json!({"path": d.path, "sha256": sha256_hex(&d.sha256)})).collect::<Vec<_>>(),
        },
        "config": {
            "vocab": VOCAB, "window": WINDOW, "prefix": PREFIX,
            "h_dim": cfg.h_dim, "h_clamp": cfg.h_clamp, "m_clamp": cfg.m_clamp,
            "recurrent_shift": cfg.recurrent_shift,
            "steps": args.steps, "batch": args.batch, "seed": args.seed,
            "phase1_steps": phase1, "phase2_end": phase2_end, "lr_phase3_scale": 0.6,
            "bias_scale": BIAS_SCALE, "ground_weight": GROUND_WEIGHT,
            "lr": 0.02,
        },
        "count_reference": {"lambdas": [lambdas.0, lambdas.1], "tune_bits_per_target": tune_bits},
        "donor_E": {"path": E_ARTIFACT, "bytes": e_bytes.len(), "sha256": e_sha,
                     "exposure": "trained on the pinned corpus; disclosed, not removed"},
        "fit_seconds": fit_seconds,
        "artifact_bytes": artifact.len(),
        "table_bytes": model.table_bytes(),
        "nonzero_reads_per_step": model.nonzero_per_step(),
        "embedding_collisions": {
            "colliding_pairs": collisions.colliding_pairs,
            "colliding_tokens": collisions.colliding_tokens,
            "witness": collisions.witness.map(|(a, b)| [a, b]),
            "distinct_rows": collisions.distinct_rows,
        },
        "preflight_a": {
            "cases": temporal_cases.len(),
            "exact": temporal_exact,
            "typed_blocks_separate_regimes": separation,
            "typed_blocks_identical_within_a_regime": within_regime_identical,
            "route_key_rule_contradictions": route_key_failures,
            "authored_rule": "committed && prior_differs",
            "authored_rule_revision": "an earlier oracle additionally required !key_changed; a \
                                        held-out composition with a superseded older value at the \
                                        answered address reached through a changed derived key \
                                        showed that conjunct was over-specified, so the rule was \
                                        revised after that exposure and the regimes are unchanged",
        },
        "preflight_b": pb.json,
        "preflight_b_exact_of": [pb.actual_exact, pb.of],
        "development": {
            "model_bits_per_target": m_bits,
            "unigram_bits_per_target": u_bits,
            "count_bits_per_target": c_bits,
            "donor_E_bits_per_target": e_bits,
            "gain_vs_unigram": [vs_unigram.0, vs_unigram.1, vs_unigram.2],
            "gain_vs_count": [vs_count.0, vs_count.1, vs_count.2],
            "gain_vs_E": [vs_e.0, vs_e.1, vs_e.2],
            "per_document_model_bits": dev[0].per_doc.iter().map(|(l, n)| if *n == 0 { f64::NAN } else { l / *n as f64 }).collect::<Vec<f64>>(),
            "documents": dev_names,
        },
        "development_position_conditioned": {
            "model_bits_per_target": position_bits,
            "targets": position_eval.total().1,
            "paired_interval_position_minus_window": [position_vs_window.0, position_vs_window.1, position_vs_window.2],
            "paired_interval_sign": "point = position minus window; negative means the position conditioning scores lower bits",
            "unigram_bits_per_target": u_bits,
            "tuned_two_token_count_bits_per_target": c_bits,
            "donor_E_bits_per_target": e_bits,
            "per_document_model_bits": position_eval.per_doc.iter().map(|(l, n)| if *n == 0 { f64::NAN } else { l / *n as f64 }).collect::<Vec<f64>>(),
            "note": "the two conditionings share target identities (the same development targets, one \
                      Generate target per position from PREFIX onward) but not the conditioning event of \
                      the preceding context: the position conditioning feeds the whole preceding window \
                      context with OBSERVE, so this is a different scoring event, not a rescoring of the \
                      same sequence. The token-only references are shared because their target \
                      population is identical.",
        },
        "untrained_position_bits_per_target": untrained_position_bits,
        "formulation": {
            "objective": args.objective.as_str(),
            "curriculum": args.curriculum.as_str(),
            "prose_slots": iw_prose_slots,
            "ground_slots": iw_ground_slots,
            "ground_weight": ground_weight,
            "grounded_gradient_weight_share": grounded_gradient_weight_share,
            "prose_example_count": prose_examples.len(),
            "declared_note": "the per-position formulation feeds the whole preceding window context with \
                              the OBSERVE event while the window formulation feeds the frozen prefix with \
                              OBSERVE and later targets with GENERATE, so the two conditionings are not \
                              interchangeable and only the window conditioning matches the served \
                              multi-token recurrence; prose_slots and ground_slots are the effective \
                              per-step draws of the interleaved schedule (defaults prose_slots = batch, \
                              ground_slots = 0) and the delivered curriculum keeps its three fixed phases",
        },
        "generation": generation,
        "held_out_composition": panel_held_out,
        "session": session,
        "costs": costs,
        "energy": "UNAVAILABLE",
        "untrained_bits_per_target": evaluate_single(&untrained, &dev_windows),
        "elapsed_seconds": started.elapsed().as_secs_f64(),
    });
    write_checked(
        &root,
        "receipt.json",
        serde_json::to_string_pretty(&receipt)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed {} with {} unlisted files; total {:.1}s",
        root.display(),
        unlisted.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(ExitCode::SUCCESS)
}

fn model_content_tokens(cases: &[Grounded]) -> usize {
    let mut set = std::collections::BTreeSet::new();
    for c in cases {
        set.extend(c.sel.iter().copied());
        set.extend(c.res.iter().copied());
    }
    set.len()
}

fn shuffle(order: &mut [usize], st: &mut u64) {
    for i in (1..order.len()).rev() {
        *st ^= *st << 13;
        *st ^= *st >> 7;
        *st ^= *st << 17;
        let j = (*st as usize) % (i + 1);
        order.swap(i, j);
    }
}

fn write_checked(root: &std::path::Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {name}: {e}"))?;
    }
    std::fs::write(&path, bytes).map_err(|e| format!("write {name}: {e}"))
}

// ---------------------------------------------------------------------------
// Preflight B: the post-copy vocabulary decision must depend on copied identity
// ---------------------------------------------------------------------------

struct PreflightB {
    actual_exact: usize,
    of: usize,
    blinded_alias: bool,
    source_disabled_loses: bool,
    json: serde_json::Value,
}

fn preflight_post_copy(model: &TlModel, a: &Grounded, b: &Grounded) -> PreflightB {
    let roll = |case: &Grounded, blind: bool, disabled: bool| {
        model.rollout(
            &case.sel,
            &case.res,
            case.facts,
            &[],
            &case.sel,
            case.accepted.len() + 2,
            blind,
            disabled,
        )
    };
    let ta = roll(a, false, false);
    let tb = roll(b, false, false);
    let actual_exact =
        usize::from(ta.actions == a.accepted) + usize::from(tb.actions == b.accepted);
    let ba = roll(a, true, false);
    let bb = roll(b, true, false);
    // With identity erased the two cases are byte-identical in every input, so any difference in the
    // post-copy decision would be a nondeterminism artefact rather than content use.
    let blinded_alias = ba.actions.get(1) == bb.actions.get(1);
    let da = roll(a, false, true);
    let source_disabled_loses = da.actions != a.accepted;
    PreflightB {
        actual_exact,
        of: 2,
        blinded_alias,
        source_disabled_loses,
        json: serde_json::json!({
            "case_a": a.name,
            "case_b": b.name,
            "class_labels": {"a": CLASS_LABEL_A, "b": CLASS_LABEL_B},
            "declared": "the post-copy words are declared class labels, not semantic claims; the \
                         preflight isolates causal dependence of the post-copy vocabulary decision on \
                         the copied token identity",
            "actual": {"a_actions": ta.actions.iter().map(action_name).collect::<Vec<_>>(),
                       "b_actions": tb.actions.iter().map(action_name).collect::<Vec<_>>(),
                       "a_tokens": ta.tokens, "b_tokens": tb.tokens,
                       "exact": actual_exact},
            "identity_blinded": {"a_actions": ba.actions.iter().map(action_name).collect::<Vec<_>>(),
                                 "b_actions": bb.actions.iter().map(action_name).collect::<Vec<_>>(),
                                 "post_copy_decision_aliases": blinded_alias},
            "source_disabled": {"a_actions": da.actions.iter().map(action_name).collect::<Vec<_>>(),
                                "a_tokens": da.tokens,
                                "loses_the_exact_copy": source_disabled_loses},
        }),
    }
}

// ---------------------------------------------------------------------------
// Development evaluation against three references on one tokenizer and denominator
// ---------------------------------------------------------------------------

fn evaluate_layers(
    model: &TlModel,
    dev: &[ProseWindow],
    uni: &Uni,
    c1: &Cond,
    c2: &Cond,
    lambdas: (f64, f64),
    e_core: &PriorCore,
) -> Vec<Eval> {
    let docs = dev.iter().map(|w| w.doc).max().map(|m| m + 1).unwrap_or(0);
    let mut out = vec![Eval::default(); 4];
    for e in out.iter_mut() {
        e.per_doc = vec![(0.0, 0usize); docs];
    }
    for w in dev {
        let ex = prose_example(w);
        let score = model.score_example(&ex);
        out[0].per_doc[w.doc].0 += score.bits_generate;
        out[0].per_doc[w.doc].1 += score.generate_targets;
        for k in PREFIX..w.tokens.len() {
            let prev = if k == 1 {
                VOCAB
            } else {
                (w.tokens[k - 2] as usize).min(VOCAB - 1)
            };
            let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
            let target = w.tokens[k];
            out[1].per_doc[w.doc].0 -= uni.p(target).max(1e-300).log2();
            out[1].per_doc[w.doc].1 += 1;
            out[2].per_doc[w.doc].0 -= family_p(c1, c2, uni, prev, cur, target, lambdas)
                .max(1e-300)
                .log2();
            out[2].per_doc[w.doc].1 += 1;
            let tr = e_core.trace(prev, cur, true);
            out[3].per_doc[w.doc].0 += e_core.bits_one(&tr.z, target);
            out[3].per_doc[w.doc].1 += 1;
        }
    }
    out
}

fn evaluate_single(model: &TlModel, dev: &[ProseWindow]) -> f64 {
    let mut bits = 0f64;
    let mut n = 0usize;
    for w in dev {
        let s = model.score_example(&prose_example(w));
        bits += s.bits_generate;
        n += s.generate_targets;
    }
    if n == 0 {
        f64::NAN
    } else {
        bits / n as f64
    }
}

fn generate_samples(
    model: &TlModel,
    tokenizer: &HfBpeTokenizer,
    uni: &Uni,
    c1: &Cond,
    c2: &Cond,
    lambdas: (f64, f64),
    dev: &[ProseWindow],
) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for (i, w) in dev.iter().take(3).enumerate() {
        let prompt_len = w.tokens.len().min(16);
        let prompt = &w.tokens[..prompt_len];
        let r = model.rollout(
            &[],
            &[],
            SlFacts::default(),
            prompt,
            &[],
            GEN_TOKENS,
            false,
            false,
        );
        let mut all = prompt.to_vec();
        all.extend_from_slice(&r.tokens);
        let ref_new =
            uor_r4_core::native_geometric::learner::realtext_support::generate_reference_full(
                c1, c2, uni, lambdas, prompt, GEN_TOKENS,
            );
        let mut ref_all = prompt.to_vec();
        ref_all.extend_from_slice(&ref_new);
        out.push(serde_json::json!({
            "index": i,
            "prompt_text": tokenizer.decode(prompt),
            "model_tokens": r.tokens,
            "model_text": tokenizer.decode(&all),
            "model_stopped": r.stopped,
            "model_steps": r.actions.len(),
            "count_text": tokenizer.decode(&ref_all),
        }));
    }
    out
}

// ---------------------------------------------------------------------------
// A cheap loaded mixed session, saved and restored in another process
// ---------------------------------------------------------------------------

fn mixed_session(
    model: &TlModel,
    words: &Words,
    grounded: &[Grounded],
    held_out: &[Grounded],
    prompt: &[u32],
) -> serde_json::Value {
    let mut phases = Vec::new();
    // 1. Observe ordinary text, then continue vocabulary.
    let observed = model.rollout(&[], &[], SlFacts::default(), prompt, &[], 24, false, false);
    phases.push(serde_json::json!({
        "phase": "observe_and_continue",
        "observed_tokens": prompt,
        "actions": observed.actions.iter().map(action_name).collect::<Vec<_>>(),
        "emitted": observed.tokens,
        "complete": observed.stopped,
    }));
    // 2..n. Grounded questions through the same loaded artifact.
    for (phase, case) in [
        (
            "ask_current_then_copy",
            grounded.iter().find(|g| g.name == "regime3_no_commit"),
        ),
        (
            "correct_then_ask",
            grounded
                .iter()
                .find(|g| g.name == "regime4_committed_correction"),
        ),
        (
            "continue_after_copy",
            held_out
                .iter()
                .find(|g| g.name == "heldout_previous_view_committed_correction"),
        ),
    ] {
        if let Some(c) = case {
            let r = model.rollout(
                &c.sel,
                &c.res,
                c.facts,
                &[],
                &c.sel,
                c.accepted.len() + 2,
                false,
                false,
            );
            phases.push(serde_json::json!({
                "phase": phase,
                "case": c.name,
                "sel": c.sel,
                "actions": r.actions.iter().map(action_name).collect::<Vec<_>>(),
                "emitted": r.tokens,
                "exact": r.actions == c.accepted,
                "temporal_meaning": TlModel::truthful_temporal_meaning(c.facts),
                "state_digest": r.state_digest,
            }));
        }
    }
    // A pinned answer retains its causal view while a new answer sees the correction: the
    // no-commit and committed-correction cases answer the same request differently.
    let pinned = model.rollout(
        &words.values[0..1].to_vec(),
        &[],
        SlFacts::default(),
        &[],
        &words.values[0..1],
        3,
        false,
        false,
    );
    let corrected = model.rollout(
        &[words.values[0]],
        &[words.values[1]],
        SlFacts {
            derived: true,
            committed: true,
            prior_differs: true,
            key_changed: false,
            ..SlFacts::default()
        },
        &[],
        &[words.values[0]],
        4,
        false,
        false,
    );
    phases.push(serde_json::json!({
        "phase": "pinned_retains_view_new_answer_sees_correction",
        "pinned_actions": pinned.actions.iter().map(action_name).collect::<Vec<_>>(),
        "corrected_actions": corrected.actions.iter().map(action_name).collect::<Vec<_>>(),
        "differ": pinned.actions != corrected.actions,
    }));
    let steps: usize = phases
        .iter()
        .map(|p| p["actions"].as_array().map(|a| a.len()).unwrap_or(0))
        .sum();
    serde_json::json!({
        "phases": phases,
        "steps": steps,
        "saved_phases": phases.len(),
        "note": "exact ownership, version authority and the copy cursor stay outside the compressed \
                 state; the model only chooses the action",
    })
}

fn separate_process_restore(
    root: &std::path::Path,
    tokenizer_path: &std::path::Path,
    artifact: &[u8],
) -> Result<bool, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let out = std::process::Command::new(exe)
        .arg("--replay")
        .arg(root)
        .arg("--tokenizer")
        .arg(tokenizer_path)
        .output()
        .map_err(|e| format!("spawn child: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if !out.status.success() {
        return Err(format!(
            "child replay failed: {} {}",
            stdout,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    println!("  child replay: {}", stdout.trim());
    let _ = artifact;
    Ok(stdout.contains("REPLAY_OK"))
}

// ---------------------------------------------------------------------------
// Whole-path cost on the named M1 machine
// ---------------------------------------------------------------------------

fn measure_costs(
    model: &TlModel,
    tokenizer: &HfBpeTokenizer,
    artifact: &[u8],
) -> serde_json::Value {
    let prompt = tokenizer.encode(" The model is");
    let reps = 40usize;
    let tokens_per_rep = 32usize;
    let t0 = Instant::now();
    let mut emitted = 0usize;
    for _ in 0..reps {
        let r = model.rollout(
            &[],
            &[],
            SlFacts::default(),
            &prompt,
            &[],
            tokens_per_rep,
            false,
            false,
        );
        emitted += r.tokens.len();
    }
    let secs = t0.elapsed().as_secs_f64();
    let us_per_token = if emitted == 0 {
        f64::NAN
    } else {
        secs * 1e6 / emitted as f64
    };
    serde_json::json!({
        "served_reps": reps,
        "served_tokens": emitted,
        "served_us_per_token": us_per_token,
        "served_tokens_per_second": if secs > 0.0 { emitted as f64 / secs } else { f64::NAN },
        "artifact_bytes": artifact.len(),
        "table_bytes": model.table_bytes(),
        "nonzero_reads_per_step": model.nonzero_per_step(),
        "declared_heap_allocations_per_served_token": 8,
        "allocation_note": "structural count of the served call path (init/readout/transition vectors); \
                            not counter-measured, because a counting allocator needs unsafe code and this \
                            binary is forbid(unsafe_code)",
        "peak_rss": "MEASURED_EXTERNALLY: run this binary under /usr/bin/time -l and read maximum resident set size",
        "energy": "UNAVAILABLE",
        "multiplier_in_kernel": false,
        "float_in_kernel": false,
    })
}

// ---------------------------------------------------------------------------
// Separate-process restore
// ---------------------------------------------------------------------------

fn replay_mode(
    root: &std::path::Path,
    tokenizer_path: &std::path::Path,
) -> Result<ExitCode, String> {
    let artifact = std::fs::read(root.join("artifacts/model.tlx"))
        .map_err(|e| format!("child artifact: {e}"))?;
    let model = TlModel::from_bytes(&artifact)?;
    let tok_bytes = std::fs::read(tokenizer_path).map_err(|e| format!("child tokenizer: {e}"))?;
    let tokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("child derive: {e}"))?;
    let words = read_words(&tokenizer)?;
    let (train, class_cases, held_out) = authored_world(&words);
    let mut cases = train.clone();
    cases.extend(held_out.clone());
    cases.extend(class_cases.clone());
    let panel = grounded_panel(&model, &cases);
    let stored: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("grounded_panel.json"))
            .map_err(|e| format!("child panel: {e}"))?,
    )
    .map_err(|e| format!("child panel parse: {e}"))?;
    let stored_train = stored["training_panel"]
        .as_array()
        .ok_or("stored panel missing training_panel")?;
    let mut ok = stored_train.len() == train.len();
    for (i, c) in train.iter().enumerate() {
        let fresh = &panel[i];
        match stored_train.get(i) {
            Some(s)
                if s["served_actions"] == fresh["served_actions"]
                    && s["served_tokens"] == fresh["served_tokens"]
                    && s["state_digest"] == fresh["state_digest"] => {}
            _ => {
                let _ = c;
                ok = false;
            }
        }
    }
    if ok {
        println!("REPLAY_OK {} cases", train.len());
        Ok(ExitCode::SUCCESS)
    } else {
        Err("child replay panel differs from the sealed panel".into())
    }
}

// ---------------------------------------------------------------------------
// Audit mode: fresh process, deserialized artifact
// ---------------------------------------------------------------------------
//
// One decision is `a_t = readout(h_t, m, f, event)` restricted to the legal rows, and the only way
// the token an action **actually emitted** can influence the next decision is through
// `h_(t+1) = transition(h_t, event, token, m, f)`, where `token` enters solely as the embedding row
// `e[row(token)]`. This mode therefore isolates the two channels that the delivered preflight changed
// together: the selected-source fingerprint `m` and the emitted-token feedback.

fn prob_of(logits: &[i32], rows: &[usize], target: usize, score_shift: u32) -> f64 {
    let scale = (-(score_shift as f64)).exp2();
    let max = rows
        .iter()
        .map(|r| logits[*r])
        .fold(i32::MIN, |a, b| a.max(b));
    let mut z = 0f64;
    for r in rows {
        z += ((logits[*r] - max) as f64 * scale).exp2();
    }
    if z <= 0.0 {
        return 0.0;
    }
    ((logits[target] - max) as f64 * scale).exp2() / z
}

fn count_clamp(v: &[i32], bound: i32) -> (usize, usize) {
    let mut entries = 0usize;
    let mut at = 0usize;
    for x in v {
        entries += 1;
        if x.unsigned_abs() as i32 == bound {
            at += 1;
        }
    }
    (entries, at)
}

/// One walk of a supervised example that additionally reports the Stop mass and hidden clamp
/// saturation at each decision.
#[derive(Default)]
struct WalkStats {
    stop_mass: f64,
    generate_conditional_bits: f64,
    generate_targets: usize,
    decisions: usize,
    stop_chosen: usize,
    h_entries: usize,
    h_clamped: usize,
    m_entries: usize,
    m_clamped: usize,
}

fn walk_stats(model: &TlModel, ex: &TlExample) -> WalkStats {
    let owned: &[u32] = if ex.grounded { &ex.sel } else { &[] };
    let m = if ex.grounded {
        model.content_feature(&ex.sel, &ex.res)
    } else {
        vec![0i32; model.h_dim]
    };
    let f = model.typed_block(&ex.sel, &ex.res, ex.facts);
    let (me, mc) = count_clamp(&m, model.m_clamp);
    let mut st = WalkStats {
        m_entries: me,
        m_clamped: mc,
        ..WalkStats::default()
    };
    let mut h = model.init_state(&m, &f);
    let (he, hc) = count_clamp(&h, model.h_clamp);
    st.h_entries += he;
    st.h_clamped += hc;
    for t in &ex.observed {
        h = model.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
        let (he, hc) = count_clamp(&h, model.h_clamp);
        st.h_entries += he;
        st.h_clamped += hc;
    }
    for i in 0..ex.actions.len() {
        let copy_legal = ex.copy_legal(i, owned);
        let event = ex.prior_event(i);
        let logits = model.readout(&h, &m, &f, event);
        let rows = model.legal_rows(copy_legal);
        let p_stop = prob_of(&logits, &rows, model.stop_row(), model.score_shift);
        st.stop_mass += p_stop;
        if let TlAction::Generate(token) = ex.actions[i] {
            let p_action = prob_of(&logits, &rows, model.token_row(token), model.score_shift);
            st.generate_conditional_bits -=
                (p_action / (1.0 - p_stop).max(1e-300)).max(1e-300).log2();
            st.generate_targets += 1;
        }
        st.decisions += 1;
        if matches!(model.decide(&h, &m, &f, event, copy_legal), TlAction::Stop) {
            st.stop_chosen += 1;
        }
        let emitted = if ex.grounded {
            ex.emitted(i, owned)
        } else {
            match ex.actions[i] {
                TlAction::Generate(v) => Some(v),
                TlAction::Copy => ex.emitted(i, owned),
                TlAction::Stop => None,
            }
        };
        h = model.transition(&h, ex.actions[i].event(), emitted, &m, &f);
        let (he, hc) = count_clamp(&h, model.h_clamp);
        st.h_entries += he;
        st.h_clamped += hc;
    }
    st
}

/// Build a state by observing `ctx` as source tokens and read the next-token decision. Used for the
/// local-identity probes; the event is `OBSERVE`, exactly as the first prose decision uses.
fn next_top1(model: &TlModel, ctx: &[u32]) -> (u32, u64) {
    let m = vec![0i32; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let mut h = model.init_state(&m, &f);
    for t in ctx {
        h = model.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
    }
    let logits = model.readout(&h, &m, &f, TL_EV_OBSERVE);
    let mut best = 0usize;
    let mut bs = i32::MIN;
    for r in 0..model.vocab {
        if logits[r] > bs {
            bs = logits[r];
            best = r;
        }
    }
    (best as u32, state_digest(&h))
}

fn cmp_panel(fresh: &[serde_json::Value], stored: &[serde_json::Value]) -> (usize, Vec<String>) {
    let mut matched = 0usize;
    let mut mismatches = Vec::new();
    if fresh.len() != stored.len() {
        mismatches.push(format!(
            "panel length {} != stored {}",
            fresh.len(),
            stored.len()
        ));
    }
    for i in 0..fresh.len().min(stored.len()) {
        let a = &fresh[i];
        let b = &stored[i];
        if a["served_actions"] == b["served_actions"]
            && a["served_tokens"] == b["served_tokens"]
            && a["state_digest"] == b["state_digest"]
        {
            matched += 1;
        } else {
            mismatches.push(format!(
                "{}: fresh actions={} tokens={} digest={} | stored actions={} tokens={} digest={}",
                a["name"],
                a["served_actions"],
                a["served_tokens"],
                a["state_digest"],
                b["served_actions"],
                b["served_tokens"],
                b["state_digest"]
            ));
        }
    }
    (matched, mismatches)
}

/// The single-variable copied-token feedback intervention.
///
/// All four arms share the same typed facts, the same forced `Copy` action, the same learned
/// initialization maps and the same readout. The only quantities that move are the evidence
/// fingerprint `m` and the token supplied to `transition` (the emitted-token feedback).
fn crossed_feedback(model: &TlModel, words: &Words) -> serde_json::Value {
    let facts = SlFacts {
        history: 2,
        ..SlFacts::default()
    };
    let tok_a = words.values[0];
    let tok_b = words.values[2];
    let sel_a = vec![tok_a];
    let sel_b = vec![tok_b];
    let m_a = model.content_feature(&sel_a, &[]);
    let m_b = model.content_feature(&sel_b, &[]);
    let f_a = model.typed_block(&sel_a, &[], facts);
    let f_b = model.typed_block(&sel_b, &[], facts);
    let f_is_fixed = f_a == f_b;
    let m_is_fixed = m_a == m_b;

    // The post-copy decision: force the Copy action's transition with an explicit feedback token,
    // then read the next vocabulary distribution. Copy is illegal after a length-1 owned span.
    let post = |h0: &[i32], m: &[i32], f: &[i32], event: usize, feed: Option<u32>| {
        let h1 = model.transition(h0, event, feed, m, f);
        let logits = model.readout(&h1, m, f, event);
        let action = model.decide(&h1, m, f, event, false);
        (action, logits, h1)
    };
    let h0_a = model.init_state(&m_a, &f_a);
    let h0_b = model.init_state(&m_b, &f_b);

    let (act_a_own, _, _) = post(&h0_a, &m_a, &f_a, TL_EV_COPY, Some(tok_a));
    let (act_b_own, _, _) = post(&h0_b, &m_b, &f_b, TL_EV_COPY, Some(tok_b));
    // Arm FB: fingerprint fixed at A, only the feedback token changes A -> B.
    let (act_a_xfb, _, _) = post(&h0_a, &m_a, &f_a, TL_EV_COPY, Some(tok_b));
    // Arm M: feedback fixed at A, only the fingerprint changes A -> B.
    let (act_b_xm, _, _) = post(&h0_b, &m_b, &f_b, TL_EV_COPY, Some(tok_a));
    // No-copy control: identical feedback token, but the preceding action is Generate, not Copy.
    let (act_nocopy, _, _) = post(&h0_a, &m_a, &f_a, TL_EV_GENERATE, Some(tok_a));
    // Identity-erased control: fingerprint blinded and feedback removed (reserved row), for both cases.
    let m_blind_a = model.content_feature_blind(&sel_a, &[]);
    let m_blind_b = model.content_feature_blind(&sel_b, &[]);
    let (act_blind_a, _, _) = post(
        &model.init_state(&m_blind_a, &f_a),
        &m_blind_a,
        &f_a,
        TL_EV_COPY,
        None,
    );
    let (act_blind_b, _, _) = post(
        &model.init_state(&m_blind_b, &f_b),
        &m_blind_b,
        &f_b,
        TL_EV_COPY,
        None,
    );

    // The decision tracks the fingerprint iff swapping only the fingerprint moves it away from the
    // fingerprint-matching baseline; it tracks the feedback iff swapping only the feedback does.
    let decision_tracks_fingerprint = act_b_xm != act_a_own;
    let decision_tracks_feedback = act_a_xfb != act_a_own;
    // Whether each single-variable arm reproduces the decision of the case it now matches.
    let feedback_arm_follows_b = act_a_xfb == act_b_own;
    let fingerprint_arm_follows_b = act_b_xm == act_b_own;
    serde_json::json!({
        "tok_a": tok_a,
        "tok_b": tok_b,
        "typed_facts_fixed_across_cases": f_is_fixed,
        "fingerprint_identical_across_cases": m_is_fixed,
        "forced_action": "Copy",
        "arm_actual_a": action_name(&act_a_own),
        "arm_actual_b": action_name(&act_b_own),
        "arm_swapped_feedback_only": action_name(&act_a_xfb),
        "arm_swapped_fingerprint_only": action_name(&act_b_xm),
        "arm_no_copy_event": action_name(&act_nocopy),
        "arm_identity_erased_a": action_name(&act_blind_a),
        "arm_identity_erased_b": action_name(&act_blind_b),
        "classification": {
            "decision_tracks_fingerprint": decision_tracks_fingerprint,
            "decision_tracks_feedback": decision_tracks_feedback,
            "feedback_arm_follows_b": feedback_arm_follows_b,
            "fingerprint_arm_follows_b": fingerprint_arm_follows_b,
            "no_copy_event_matches_actual": act_nocopy == act_a_own,
            "identity_erased_alias": act_blind_a == act_blind_b,
        },
        "interpretation": "single-variable: the feedback-only arm holds the evidence fingerprint, typed \
                           facts, prefix and the chosen Copy action fixed and changes only the token \
                           handed to transition; the fingerprint-only arm holds the feedback token fixed \
                           and changes only the evidence fingerprint m",
        "declared": "the post-copy words remain declared class labels, not semantic claims",
    })
}

fn dev_denominators(
    model: &TlModel,
    dev: &[ProseWindow],
    uni: &Uni,
    c1: &Cond,
    c2: &Cond,
    lambdas: (f64, f64),
    e_core: &PriorCore,
) -> serde_json::Value {
    let mut gen = (0f64, 0usize);
    let mut all = (0f64, 0usize);
    let mut stop_targets = 0usize;
    let mut correct_stops = 0usize;
    let mut correct = 0usize;
    let mut stop_mass = 0f64;
    let mut generate_conditional_bits = 0f64;
    let mut generate_conditional_targets = 0usize;
    let mut decisions = 0usize;
    let mut stop_chosen = 0usize;
    let mut h_entries = 0usize;
    let mut h_clamped = 0usize;
    let mut m_entries = 0usize;
    let mut m_clamped = 0usize;
    for w in dev {
        let ex = prose_example(w);
        let s = model.score_example(&ex);
        gen.0 += s.bits_generate;
        gen.1 += s.generate_targets;
        all.0 += s.bits_all;
        all.1 += s.scored;
        stop_targets += s.stop_targets;
        correct_stops += s.correct_stops;
        correct += s.correct;
        let ws = walk_stats(model, &ex);
        stop_mass += ws.stop_mass;
        generate_conditional_bits += ws.generate_conditional_bits;
        generate_conditional_targets += ws.generate_targets;
        decisions += ws.decisions;
        stop_chosen += ws.stop_chosen;
        h_entries += ws.h_entries;
        h_clamped += ws.h_clamped;
        m_entries += ws.m_entries;
        m_clamped += ws.m_clamped;
    }
    // The token-only references share the Generate denominator only; report them on that boundary.
    let refs = evaluate_layers(model, dev, uni, c1, c2, lambdas, e_core);
    serde_json::json!({
        "full_action_bits_per_target": if all.1 == 0 { f64::NAN } else { all.0 / all.1 as f64 },
        "full_action_scored": all.1,
        "generate_action_bits_per_target": if gen.1 == 0 { f64::NAN } else { gen.0 / gen.1 as f64 },
        "generate_action_targets": gen.1,
        "generate_conditional_bits_per_target": if generate_conditional_targets == 0 { f64::NAN } else { generate_conditional_bits / generate_conditional_targets as f64 },
        "generate_conditional_targets": generate_conditional_targets,
        "generated_actions": gen.1,
        "action_accuracy": if all.1 == 0 { f64::NAN } else { correct as f64 / all.1 as f64 },
        "stop_targets": stop_targets,
        "correct_stops": correct_stops,
        "mean_stop_probability": if decisions == 0 { f64::NAN } else { stop_mass / decisions as f64 },
        "stop_chosen_fraction": if decisions == 0 { f64::NAN } else { stop_chosen as f64 / decisions as f64 },
        "hidden_clamp_saturation": if h_entries == 0 { f64::NAN } else { h_clamped as f64 / h_entries as f64 },
        "fingerprint_clamp_saturation": if m_entries == 0 { f64::NAN } else { m_clamped as f64 / m_entries as f64 },
        "token_only_references_same_target_population": {
            "unigram_bits_per_target": refs[1].micro(),
            "tuned_two_token_count_bits_per_target": refs[2].micro(),
            "donor_E_bits_per_target": refs[3].micro(),
        },
        "denominator_note": "full-action NLL scores every supervised action over legal rows; \
                              generate-action NLL selects Generate targets but still normalizes over \
                              Generate and Stop rows; generate-conditional NLL renormalizes over \
                              Generate rows. Count and donor references are token-only on the same \
                              target population as generate-conditional NLL.",
    })
}

/// Token-conditional loss stratified by position within the window and by the fit-split frequency of
/// the exact `(previous, current)` context. A deficit concentrated at low-frequency contexts is
/// evidence for a local-identity/order problem; a deficit spread across frequent contexts is not.
fn dev_stratified(model: &TlModel, dev: &[ProseWindow], c2: &Cond) -> serde_json::Value {
    let mut by_pos: Vec<(f64, usize)> = vec![(0.0, 0usize); 8];
    let mut by_freq: Vec<(f64, usize)> = vec![(0.0, 0usize); 4];
    for w in dev {
        let ex = prose_example(w);
        let m = vec![0i32; model.h_dim];
        let f = model.typed_block(&ex.sel, &ex.res, ex.facts);
        let mut h = model.init_state(&m, &f);
        for t in &ex.observed {
            h = model.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
        }
        for i in 0..ex.actions.len() {
            let copy_legal = ex.copy_legal(i, &[]);
            let event = ex.prior_event(i);
            let logits = model.readout(&h, &m, &f, event);
            let rows = model.legal_rows(copy_legal);
            let target = model.action_row(ex.actions[i]);
            if let TlAction::Generate(_) = ex.actions[i] {
                let bits = -prob_of(&logits, &rows, target, model.score_shift)
                    .max(1e-300)
                    .log2();
                let pb = (i / 8).min(by_pos.len() - 1);
                by_pos[pb].0 += bits;
                by_pos[pb].1 += 1;
                let k = PREFIX + i;
                let prev = if k == 1 {
                    VOCAB
                } else {
                    (w.tokens[k - 2] as usize).min(VOCAB - 1)
                };
                let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
                let observed = c2.total(ctx2(prev, cur));
                let fb = match observed {
                    0 => 0,
                    1 => 1,
                    2..=4 => 2,
                    _ => 3,
                };
                by_freq[fb].0 += bits;
                by_freq[fb].1 += 1;
            }
            let emitted = match ex.actions[i] {
                TlAction::Generate(v) => Some(v),
                _ => None,
            };
            h = model.transition(&h, ex.actions[i].event(), emitted, &m, &f);
        }
    }
    let rate = |v: &Vec<(f64, usize)>| -> Vec<f64> {
        v.iter()
            .map(|(b, n)| if *n == 0 { f64::NAN } else { b / *n as f64 })
            .collect()
    };
    serde_json::json!({
        "generate_action_bits_by_position_bucket_8": rate(&by_pos),
        "position_bucket_counts": by_pos.iter().map(|(_, n)| *n).collect::<Vec<usize>>(),
        "generate_action_bits_by_context_frequency": rate(&by_freq),
        "context_frequency_counts": by_freq.iter().map(|(_, n)| *n).collect::<Vec<usize>>(),
        "context_frequency_buckets": "context = (previous token, current token), bucketed by its fit-split \
                                       occurrence count: 0, 1, 2-4, 5+",
        "position_buckets": "scored position i within the window, bucket i/8",
    })
}

/// Local-identity diagnostic: does the state carry the exact last two token identities, and does the
/// next-token decision move when only the penultimate (or only the last) token changes?
fn local_identity_report(model: &TlModel, dev: &[ProseWindow]) -> serde_json::Value {
    use std::collections::HashMap;
    let mut ctxs: Vec<(u32, u32)> = Vec::new();
    for w in dev {
        for k in 2..w.tokens.len() {
            ctxs.push((w.tokens[k - 2], w.tokens[k - 1]));
        }
    }
    let mut top1: HashMap<(u32, u32), u32> = HashMap::new();
    let mut digest: HashMap<(u32, u32), u64> = HashMap::new();
    for (p, q) in &ctxs {
        if top1.contains_key(&(*p, *q)) {
            continue;
        }
        let (t, d) = next_top1(model, &[*p, *q]);
        top1.insert((*p, *q), t);
        digest.insert((*p, *q), d);
    }
    let mut penults_by_last: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut lasts_by_penult: HashMap<u32, Vec<u32>> = HashMap::new();
    for (p, q) in &ctxs {
        penults_by_last.entry(*q).or_default().push(*p);
        lasts_by_penult.entry(*p).or_default().push(*q);
    }
    for v in penults_by_last.values_mut() {
        v.sort_unstable();
        v.dedup();
    }
    for v in lasts_by_penult.values_mut() {
        v.sort_unstable();
        v.dedup();
    }
    let mut penult_groups = 0usize;
    let mut penult_top1_changes = 0usize;
    let mut penult_digest_changes = 0usize;
    for (q, ps) in &penults_by_last {
        if ps.len() < 2 {
            continue;
        }
        penult_groups += 1;
        let tops: std::collections::BTreeSet<u32> = ps
            .iter()
            .filter_map(|p| top1.get(&(*p, *q)).copied())
            .collect();
        let digs: std::collections::BTreeSet<u64> = ps
            .iter()
            .filter_map(|p| digest.get(&(*p, *q)).copied())
            .collect();
        if tops.len() > 1 {
            penult_top1_changes += 1;
        }
        if digs.len() > 1 {
            penult_digest_changes += 1;
        }
    }
    let mut last_groups = 0usize;
    let mut last_top1_changes = 0usize;
    for (p, qs) in &lasts_by_penult {
        if qs.len() < 2 {
            continue;
        }
        last_groups += 1;
        let tops: std::collections::BTreeSet<u32> = qs
            .iter()
            .filter_map(|q| top1.get(&(*p, *q)).copied())
            .collect();
        if tops.len() > 1 {
            last_top1_changes += 1;
        }
    }
    serde_json::json!({
        "distinct_two_token_contexts": top1.len(),
        "penultimate_groups_same_last_token": penult_groups,
        "penultimate_swap_changes_top1_fraction": frac(penult_top1_changes, penult_groups),
        "penultimate_swap_changes_state_digest_fraction": frac(penult_digest_changes, penult_groups),
        "last_token_groups_same_penultimate": last_groups,
        "last_swap_changes_top1_fraction": frac(last_top1_changes, last_groups),
        "interpretation": "a next-token decision insensitive to the penultimate token cannot recover \
                           local order beyond the single last token; this is a local-identity probe of \
                           the loaded artifact, not a language claim",
    })
}

fn frac(a: usize, b: usize) -> f64 {
    if b == 0 {
        f64::NAN
    } else {
        a as f64 / b as f64
    }
}

fn audit_mode(args: &Args, src: &std::path::Path) -> Result<ExitCode, String> {
    if args.root.as_os_str().is_empty() || args.docs.as_os_str().is_empty() {
        return Err("--root (new attempt) and --docs are required with --audit".into());
    }
    claim(&args.root).map_err(|e| format!("claim {}: {e}", args.root.display()))?;
    let started = Instant::now();
    let root = args.root.clone();
    println!("=== ordinary-lexical audit: fresh-process deserialized artifact ===");

    // ---- tokenizer identity ------------------------------------------------
    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&Sha256::digest(&tok_bytes));
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer sha256 {tok_sha} != expected {EXPECTED_TOKENIZER_SHA256}"
        ));
    }
    let derived_bytes =
        uor_r4_core::transformerless::bpe_derive::derive_tokenizer_json(&tok_bytes, VOCAB)
            .map_err(|e| format!("derive bytes: {e}"))?;
    let derived_sha = sha256_hex(&Sha256::digest(&derived_bytes));
    let tokenizer = derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive: {e}"))?;
    write_checked(&root, "tokenizer_source.json", &tok_bytes)?;
    write_checked(&root, "tokenizer_derived_v4096.json", &derived_bytes)?;
    let words = read_words(&tokenizer)?;

    // ---- tested executable and source identity -----------------------------
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let exe_bytes = std::fs::read(&exe).map_err(|e| format!("read exe: {e}"))?;
    let exe_sha = sha256_hex(&Sha256::digest(&exe_bytes));
    let src_artifact = src.join("artifacts/model.tlx");
    let artifact = std::fs::read(&src_artifact)
        .map_err(|e| format!("source artifact {}: {e}", src_artifact.display()))?;
    let artifact_sha = sha256_hex(&Sha256::digest(&artifact));
    let model = TlModel::from_bytes(&artifact).map_err(|e| format!("deserialize: {e}"))?;
    write_checked(&root, "artifacts/model.tlx", &artifact)?;
    println!(
        "audit loaded {} B artifact sha256 {artifact_sha}; executable sha256 {exe_sha}",
        artifact.len()
    );

    // ---- corpus, source-separated (mirrors the training run) ----------------
    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
    let mut fit_docs = Vec::new();
    let mut tune_docs = Vec::new();
    let mut dev_pool = Vec::new();
    for (i, d) in uniq.iter().enumerate() {
        match d.split {
            Split::Fit => fit_docs.push(i),
            Split::Tune => tune_docs.push(i),
            Split::Dev => dev_pool.push(i),
        }
    }
    let mut fit_windows: Vec<ProseWindow> = Vec::new();
    for i in &fit_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), *i) {
            fit_windows.push(w);
        }
    }
    if fit_windows.len() > FIT_MAX_WINDOWS {
        let stride = fit_windows.len() as f64 / FIT_MAX_WINDOWS as f64;
        fit_windows = (0..FIT_MAX_WINDOWS)
            .map(|k| {
                let idx = (k as f64 * stride) as usize;
                fit_windows[idx].clone()
            })
            .collect();
    }
    let mut with_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    with_len.sort_by_key(|(_, n)| *n);
    let take = DEV_MAX_DOCS.min(with_len.len());
    let mut dev_windows: Vec<ProseWindow> = Vec::new();
    let mut dev_names: Vec<String> = Vec::new();
    for k in 0..take {
        let (doc_idx, _) = with_len[k * with_len.len() / take.max(1)];
        dev_names.push(uniq[doc_idx].path.clone());
        for w in prose_windows(&tokenizer.encode(&uniq[doc_idx].text), k)
            .into_iter()
            .take(DEV_WINDOWS_PER_DOC)
        {
            dev_windows.push(w);
        }
    }
    let mut tune_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), 0) {
            tune_windows.push(w.tokens);
        }
    }
    let mut uni = Uni {
        counts: vec![0u64; VOCAB],
        total: 0,
    };
    let mut c1 = Cond::default();
    let mut c2 = Cond::default();
    for w in &fit_windows {
        for k in PREFIX..w.tokens.len() {
            let prev = if k == 1 {
                VOCAB
            } else {
                (w.tokens[k - 2] as usize).min(VOCAB - 1)
            };
            let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
            let next = w.tokens[k];
            uni.counts[next as usize] += 1;
            uni.total += 1;
            c1.observe(cur as u64, next);
            c2.observe(ctx2(prev, cur), next);
        }
    }
    let (lambdas, tune_bits) = tune_lambdas(&c1, &c2, &uni, &tune_windows);
    let e_bytes = std::fs::read(E_ARTIFACT).map_err(|e| format!("E artifact: {e}"))?;
    let e_core = PriorCore::from_bytes(&e_bytes).map_err(|e| format!("E load: {e}"))?;
    let e_sha = sha256_hex(&Sha256::digest(&e_bytes));
    println!(
        "corpus collected={collected} unique={} dev_docs={}",
        uniq.len(),
        dev_names.len()
    );
    println!("count references tuned lambdas={lambdas:?} tune bits/target={tune_bits:.4}");

    // ---- stored evidence to compare against ---------------------------------
    let stored_panel: serde_json::Value = serde_json::from_slice(
        &std::fs::read(src.join("grounded_panel.json"))
            .map_err(|e| format!("stored panel: {e}"))?,
    )
    .map_err(|e| format!("stored panel parse: {e}"))?;
    let stored_receipt: serde_json::Value = serde_json::from_slice(
        &std::fs::read(src.join("receipt.json")).map_err(|e| format!("stored receipt: {e}"))?,
    )
    .map_err(|e| format!("stored receipt parse: {e}"))?;

    // ---- every authored panel, executed on the deserialized artifact --------
    let (grounded_train, class_cases, held_out) = authored_world(&words);
    let panel = grounded_panel(&model, &grounded_train);
    let panel_held = grounded_panel(&model, &held_out);
    let panel_class = grounded_panel(&model, &class_cases);
    let (ok_t, mism_t) = cmp_panel(
        &panel,
        stored_panel["training_panel"]
            .as_array()
            .ok_or("stored training_panel")?,
    );
    let (ok_h, mism_h) = cmp_panel(
        &panel_held,
        stored_panel["held_out_panel"]
            .as_array()
            .ok_or("stored held_out_panel")?,
    );
    let (ok_c, mism_c) = cmp_panel(
        &panel_class,
        stored_panel["class_panel"]
            .as_array()
            .ok_or("stored class_panel")?,
    );
    println!(
        "panel replay on deserialized artifact: training {ok_t}/{} held_out {ok_h}/{} class {ok_c}/{}",
        panel.len(),
        panel_held.len(),
        panel_class.len()
    );

    // preflight A recomputed
    let temporal: Vec<&serde_json::Value> = panel
        .iter()
        .filter(|v| v["role"] == "preflight_temporal")
        .collect();
    let temporal_exact = temporal
        .iter()
        .filter(|v| v["exact"] == serde_json::json!(true))
        .count();
    let route_key_failures = grounded_train
        .iter()
        .filter(|g| g.role == "preflight_temporal")
        .filter(|g| g.facts.key_changed != TlModel::truthful_temporal_meaning(g.facts))
        .count();
    // preflight B recomputed
    let pb = preflight_post_copy(&model, &class_cases[0], &class_cases[1]);
    let stored_b = &stored_receipt["preflight_b"];
    let pb_matches = pb.json["actual"] == stored_b["actual"]
        && pb.json["identity_blinded"] == stored_b["identity_blinded"];
    println!(
        "preflight A {temporal_exact}/{}; preflight B actual {}/2 blinded-alias {} source-disabled-loses {}; matches stored {} ",
        temporal.len(),
        pb.actual_exact,
        pb.blinded_alias,
        pb.source_disabled_loses,
        pb_matches
    );

    // ---- prose, denominators and references on the loaded artifact ----------
    let dev = evaluate_layers(&model, &dev_windows, &uni, &c1, &c2, lambdas, &e_core);
    let stored_dev = &stored_receipt["development"];
    let model_bits = dev[0].micro();
    let stored_model_bits = stored_dev["model_bits_per_target"]
        .as_f64()
        .unwrap_or(f64::NAN);
    let prose_matches = (model_bits - stored_model_bits).abs() < 1e-9;
    let denominators = dev_denominators(&model, &dev_windows, &uni, &c1, &c2, lambdas, &e_core);
    println!(
        "prose: Generate-action {model_bits:.4} (stored {stored_model_bits:.4}, equal={prose_matches}); full-action {:.4} over {} scored; Generate-conditional {:.4}",
        denominators["full_action_bits_per_target"].as_f64().unwrap_or(f64::NAN),
        denominators["full_action_scored"],
        denominators["generate_conditional_bits_per_target"].as_f64().unwrap_or(f64::NAN)
    );
    let local = local_identity_report(&model, &dev_windows);
    println!("local identity: {local}");

    // ---- crossed feedback intervention -------------------------------------
    let crossed = crossed_feedback(&model, &words);
    println!("crossed feedback: {crossed}");

    // ---- generation and continuation on the loaded artifact ----------------
    let generation = generate_samples(&model, &tokenizer, &uni, &c1, &c2, lambdas, &dev_windows);
    let stored_gen = stored_receipt["generation"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let gen_matches = generation.len() == stored_gen.len()
        && generation
            .iter()
            .zip(stored_gen.iter())
            .all(|(a, b)| a["model_tokens"] == b["model_tokens"]);
    for g in &generation {
        println!(
            "prompt {:?} -> model {:?}",
            g["prompt_text"], g["model_text"]
        );
    }

    // ---- mixed session on the loaded artifact ------------------------------
    let prompt: Vec<u32> = dev_windows
        .first()
        .map(|w| w.tokens[..WINDOW.min(w.tokens.len())].to_vec())
        .unwrap_or_default();
    let session = mixed_session(&model, &words, &grounded_train, &held_out, &prompt);
    let stored_session = &stored_receipt["session"];
    let session_matches = session["phases"] == stored_session["phases"];
    println!(
        "mixed session steps={} matches stored={}",
        session["steps"], session_matches
    );

    // ---- cost: rollout-only and whole-path, kept separate ------------------
    let costs = measure_costs(&model, &tokenizer, &artifact);
    let whole_path = {
        let prompt = tokenizer.encode(" The model is");
        let t0 = Instant::now();
        let mut emitted = 0usize;
        for _ in 0..40 {
            let enc = tokenizer.encode(" The model is");
            let r = model.rollout(&[], &[], SlFacts::default(), &enc, &[], 32, false, false);
            let _ = tokenizer.decode(&r.tokens);
            emitted += r.tokens.len();
        }
        let secs = t0.elapsed().as_secs_f64();
        serde_json::json!({
            "whole_path_us_per_token": if emitted == 0 { f64::NAN } else { secs * 1e6 / emitted as f64 },
            "whole_path_tokens": emitted,
            "includes": "tokenization + rollout + decode; excludes source selection and session bookkeeping",
            "prompt": tokenizer.decode(&prompt),
        })
    };
    let h = model.h_dim;
    let f = TL_F_DIM;
    let packed_inspections_per_step =
        h * h + h * (h + f + TL_EVENTS) + model.action_rows() * (2 * h + f + TL_EVENTS);

    // ---- seal ---------------------------------------------------------------
    let receipt = serde_json::json!({
        "schema": "uor-r4.ordinary-lexical-audit/1",
        "source_root": src.display().to_string(),
        "source_artifact_sha256": artifact_sha,
        "source_artifact_bytes": artifact.len(),
        "tested_executable_sha256": exe_sha,
        "tested_source_rev": args.source_rev,
        "tokenizer_source_sha256": tok_sha,
        "tokenizer_derived_sha256": derived_sha,
        "donor_E_sha256": e_sha,
        "replay": {
            "training_panel_matched": ok_t,
            "training_panel_total": panel.len(),
            "training_panel_mismatches": mism_t,
            "held_out_panel_matched": ok_h,
            "held_out_panel_total": panel_held.len(),
            "held_out_panel_mismatches": mism_h,
            "class_panel_matched": ok_c,
            "class_panel_total": panel_class.len(),
            "class_panel_mismatches": mism_c,
            "preflight_a_exact": temporal_exact,
            "preflight_a_total": temporal.len(),
            "preflight_a_route_key_contradictions": route_key_failures,
            "preflight_b_matches_stored": pb_matches,
            "prose_generate_action_matches_stored": prose_matches,
            "prose_model_bits_per_target": model_bits,
            "prose_stored_bits_per_target": stored_model_bits,
            "generation_matches_stored": gen_matches,
            "mixed_session_matches_stored": session_matches,
        },
        "panels": {
            "training_panel": panel,
            "held_out_panel": panel_held,
            "class_panel": panel_class,
        },
        "prose_denominators": denominators,
        "prose_stratified": dev_stratified(&model, &dev_windows, &c2),
        "local_identity": local,
        "crossed_feedback": crossed,
        "generation": generation,
        "session": session,
        "costs": {
            "rollout_only": costs,
            "whole_path": whole_path,
            "packed_coefficient_inspections_per_step": packed_inspections_per_step,
            "packed_inspection_note": "wi is init-only; per generated step the recurrent, fact and readout \
                                        maps scan every packed coefficient including zero codes",
            "energy": "UNAVAILABLE",
        },
        "count_reference": {"lambdas": [lambdas.0, lambdas.1], "tune_bits_per_target": tune_bits},
        "corpus": {
            "collected": collected,
            "unique": uniq.len(),
            "exact_duplicates_grouped": duplicates,
            "fit_docs": fit_docs.len(),
            "tune_docs": tune_docs.len(),
            "dev_docs": dev_names.len(),
            "dev_scored_targets": dev_windows.iter().map(|w| w.tokens.len().saturating_sub(PREFIX)).sum::<usize>(),
            "document_sha256": uniq.iter().map(|d| serde_json::json!({"path": d.path, "sha256": sha256_hex(&d.sha256)})).collect::<Vec<_>>(),
        },
        "elapsed_seconds": started.elapsed().as_secs_f64(),
    });
    write_checked(
        &root,
        "receipt.json",
        serde_json::to_string_pretty(&receipt)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed audit {} with {} unlisted files; total {:.1}s",
        root.display(),
        unlisted.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Matched local-channel comparator
// ---------------------------------------------------------------------------
//
// One ordinary learner (the retained `TlModel`/`TlTrainer`), one data split, one optimizer schedule,
// one seed and one architecture. The **only** difference between the two arms is the bounded local
// channel handed to the readout: the last token alone, or the last two tokens with the fixed order
// rotation of the retained evidence-fingerprint construction. That construction is a clamped sum of
// the served embedding rows, so the channel stays inside the 4-bit/ternary serving envelope.

fn position_examples(windows: &[ProseWindow], local: usize) -> Vec<TlExample> {
    let mut out = Vec::new();
    for w in windows {
        for k in PREFIX..w.tokens.len() {
            let next = w.tokens[k];
            let res = match local {
                0 => Vec::new(),
                1 => vec![w.tokens[k - 1]],
                _ => vec![w.tokens[k - 2], w.tokens[k - 1]],
            };
            out.push(TlExample {
                sel: Vec::new(),
                res,
                facts: SlFacts::default(),
                observed: w.tokens[..k].to_vec(),
                actions: vec![TlAction::Generate(next)],
                weight: 1.0,
                doc: w.doc,
                grounded: true,
                terminal_stop: false,
            });
        }
    }
    out
}

fn train_channel_learner(
    examples: &[TlExample],
    log2_p: &[f64],
    seed: u64,
    steps: u64,
    batch: usize,
) -> Result<TlModel, String> {
    let cfg = TlConfig::new(VOCAB);
    let mut trainer = TlTrainer::new(
        cfg,
        TlTrainConfig {
            lr: 0.02,
            seed,
            ..Default::default()
        },
    )?;
    trainer.set_output_bias(log2_p, BIAS_SCALE)?;
    let mut order: Vec<usize> = (0..examples.len()).collect();
    let mut rng = seed ^ 0x5DEE_CE66;
    let mut cursor = 0usize;
    for step in 0..steps {
        let mut b: Vec<TlExample> = Vec::with_capacity(batch);
        for _ in 0..batch {
            if cursor >= order.len() {
                shuffle(&mut order, &mut rng);
                cursor = 0;
            }
            b.push(examples[order[cursor]].clone());
            cursor += 1;
        }
        let r = trainer.train_batch(&b);
        if step % 500 == 0 || step + 1 == steps {
            println!(
                "    step {step:>6} batch bits/target {:.4}",
                r.bits_per_target()
            );
        }
    }
    trainer.model()
}

fn eval_examples(model: &TlModel, examples: &[TlExample], docs: usize) -> Eval {
    let mut e = Eval::default();
    e.per_doc = vec![(0.0, 0usize); docs];
    for ex in examples {
        let s = model.score_example(ex);
        e.per_doc[ex.doc].0 += s.bits_generate;
        e.per_doc[ex.doc].1 += s.generate_targets;
    }
    e
}

fn reference_evals(
    dev: &[ProseWindow],
    uni: &Uni,
    c1: &Cond,
    c2: &Cond,
    lambdas: (f64, f64),
) -> (Eval, Eval) {
    let docs = dev.iter().map(|w| w.doc).max().map(|m| m + 1).unwrap_or(0);
    let mut u = Eval::default();
    u.per_doc = vec![(0.0, 0usize); docs];
    let mut c = Eval::default();
    c.per_doc = vec![(0.0, 0usize); docs];
    for w in dev {
        for k in PREFIX..w.tokens.len() {
            let prev = if k == 1 {
                VOCAB
            } else {
                (w.tokens[k - 2] as usize).min(VOCAB - 1)
            };
            let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
            let target = w.tokens[k];
            u.per_doc[w.doc].0 -= uni.p(target).max(1e-300).log2();
            u.per_doc[w.doc].1 += 1;
            c.per_doc[w.doc].0 -= family_p(c1, c2, uni, prev, cur, target, lambdas)
                .max(1e-300)
                .log2();
            c.per_doc[w.doc].1 += 1;
        }
    }
    (u, c)
}

fn local_channel_mode(args: &Args, root: &std::path::Path) -> Result<ExitCode, String> {
    if args.docs.as_os_str().is_empty() {
        return Err("--docs is required with --local-channel".into());
    }
    claim(root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    println!("=== ordinary-lexical local-channel comparator ===");

    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&Sha256::digest(&tok_bytes));
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer sha256 {tok_sha} != expected {EXPECTED_TOKENIZER_SHA256}"
        ));
    }
    let tokenizer = derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive: {e}"))?;

    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let exe_bytes = std::fs::read(&exe).map_err(|e| format!("read exe: {e}"))?;
    let exe_sha = sha256_hex(&Sha256::digest(&exe_bytes));

    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
    let mut fit_docs = Vec::new();
    let mut tune_docs = Vec::new();
    let mut dev_pool = Vec::new();
    for (i, d) in uniq.iter().enumerate() {
        match d.split {
            Split::Fit => fit_docs.push(i),
            Split::Tune => tune_docs.push(i),
            Split::Dev => dev_pool.push(i),
        }
    }
    let mut fit_windows: Vec<ProseWindow> = Vec::new();
    for i in &fit_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), *i) {
            fit_windows.push(w);
        }
    }
    if fit_windows.len() > FIT_MAX_WINDOWS {
        let stride = fit_windows.len() as f64 / FIT_MAX_WINDOWS as f64;
        fit_windows = (0..FIT_MAX_WINDOWS)
            .map(|k| {
                let idx = (k as f64 * stride) as usize;
                fit_windows[idx].clone()
            })
            .collect();
    }
    let mut with_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    with_len.sort_by_key(|(_, n)| *n);
    let take = DEV_MAX_DOCS.min(with_len.len());
    let mut dev_windows: Vec<ProseWindow> = Vec::new();
    for k in 0..take {
        let (doc_idx, _) = with_len[k * with_len.len() / take.max(1)];
        for w in prose_windows(&tokenizer.encode(&uniq[doc_idx].text), k)
            .into_iter()
            .take(DEV_WINDOWS_PER_DOC)
        {
            dev_windows.push(w);
        }
    }
    let mut tune_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), 0) {
            tune_windows.push(w.tokens);
        }
    }
    let mut uni = Uni {
        counts: vec![0u64; VOCAB],
        total: 0,
    };
    let mut c1 = Cond::default();
    let mut c2 = Cond::default();
    for w in &fit_windows {
        for k in PREFIX..w.tokens.len() {
            let prev = if k == 1 {
                VOCAB
            } else {
                (w.tokens[k - 2] as usize).min(VOCAB - 1)
            };
            let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
            let next = w.tokens[k];
            uni.counts[next as usize] += 1;
            uni.total += 1;
            c1.observe(cur as u64, next);
            c2.observe(ctx2(prev, cur), next);
        }
    }
    let (lambdas, tune_bits) = tune_lambdas(&c1, &c2, &uni, &tune_windows);
    let denom = uni.total as f64 + (VOCAB + 2) as f64;
    let mut log2_p: Vec<f64> = (0..VOCAB)
        .map(|t| ((uni.counts[t] as f64 + 1.0) / denom).log2())
        .collect();
    log2_p.push((1.0 / denom).log2());
    log2_p.push((1.0 / denom).log2());

    let docs = dev_windows
        .iter()
        .map(|w| w.doc)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let fit0 = position_examples(&fit_windows, 0);
    let fit2 = position_examples(&fit_windows, 2);
    let dev0 = position_examples(&dev_windows, 0);
    let dev2 = position_examples(&dev_windows, 2);
    println!(
        "corpus collected={collected} unique={} fit_windows={} dev_docs={} dev_targets={} tune_lambdas={lambdas:?} tune_bits={tune_bits:.4}",
        uniq.len(),
        fit_windows.len(),
        take,
        dev0.len()
    );
    let (uni_e, cnt_e) = reference_evals(&dev_windows, &uni, &c1, &c2, lambdas);

    println!("arm recurrence-only (full preceding context, no direct local channel):");
    let t1 = Instant::now();
    let m1 = train_channel_learner(&fit0, &log2_p, args.seed, args.steps, args.batch)?;
    let secs1 = t1.elapsed().as_secs_f64();
    let e1 = eval_examples(&m1, &dev0, docs);
    println!(
        "  arm recurrence-only dev bits/target {:.4} in {secs1:.1}s",
        e1.micro()
    );

    println!("arm recurrence + direct two-token channel (same task, same targets):");
    let t2 = Instant::now();
    let m2 = train_channel_learner(&fit2, &log2_p, args.seed, args.steps, args.batch)?;
    let secs2 = t2.elapsed().as_secs_f64();
    let e2 = eval_examples(&m2, &dev2, docs);
    println!(
        "  arm recurrence + two-token dev bits/target {:.4} in {secs2:.1}s",
        e2.micro()
    );

    let u1 = paired_interval(&uni_e, &e1, 0xA11CE);
    let c1g = paired_interval(&cnt_e, &e1, 0xB0B);
    let u2 = paired_interval(&uni_e, &e2, 0xA11CE);
    let c2g = paired_interval(&cnt_e, &e2, 0xB0B);
    let d21 = paired_interval(&e2, &e1, 0xC0FFEE);
    println!(
        "recurrence-only: bits {:.4} | gain vs unigram {:?} | gain vs count (ref-model) {:?}",
        e1.micro(),
        u1,
        c1g
    );
    println!(
        "recurrence+two-token: bits {:.4} | gain vs unigram {:?} | gain vs count (ref-model) {:?}",
        e2.micro(),
        u2,
        c2g
    );
    println!(
        "two-token arm minus recurrence-only (two-token bits - recurrence-only bits) {:?}",
        d21
    );

    let art1 = m1.to_bytes();
    let art2 = m2.to_bytes();
    let receipt = serde_json::json!({
        "schema": "uor-r4.ordinary-lexical-local-channel/2",
        "tested_source_rev": args.source_rev,
        "tested_executable_sha256": exe_sha,
        "tokenizer_source_sha256": tok_sha,
        "corpus": {
            "collected": collected,
            "unique": uniq.len(),
            "exact_duplicates_grouped": duplicates,
            "fit_docs": fit_docs.len(),
            "tune_docs": tune_docs.len(),
            "dev_docs": take,
            "fit_windows": fit_windows.len(),
            "dev_targets": dev0.len(),
        },
        "schedule": {"steps": args.steps, "batch": args.batch, "seed": args.seed, "lr": 0.02},
        "count_reference": {"lambdas": [lambdas.0, lambdas.1], "tune_bits_per_target": tune_bits},
        "arms": {
            "recurrence_only": {
                "dev_bits_per_target": e1.micro(),
                "gain_vs_unigram": [u1.0, u1.1, u1.2],
                "gain_vs_count": [c1g.0, c1g.1, c1g.2],
                "artifact_bytes": art1.len(),
                "table_bytes": m1.table_bytes(),
                "nonzero_per_step": m1.nonzero_per_step(),
                "fit_seconds": secs1,
            },
            "recurrence_plus_two_token": {
                "dev_bits_per_target": e2.micro(),
                "gain_vs_unigram": [u2.0, u2.1, u2.2],
                "gain_vs_count": [c2g.0, c2g.1, c2g.2],
                "artifact_bytes": art2.len(),
                "table_bytes": m2.table_bytes(),
                "nonzero_per_step": m2.nonzero_per_step(),
                "fit_seconds": secs2,
            },
            "two_token_minus_recurrence_only": [d21.0, d21.1, d21.2],
        },
        "unigram_bits_per_target": uni_e.micro(),
        "count_bits_per_target": cnt_e.micro(),
        "declared": "one ordinary learner, one data split, one schedule, one seed, one architecture; the \
                     only change between arms is the bounded local channel (none vs the last two tokens \
                     with a fixed order rotation) on the same full-context task and the same scored \
                     targets. Positive gain_vs_count means the reference has more bits than the arm (the \
                     arm is better). Positive two_token_minus_recurrence_only means the two-token arm \
                     has more bits (is worse).",
        "elapsed_seconds": started.elapsed().as_secs_f64(),
    });
    write_checked(root, "artifacts/recurrence_only.tlx", &art1)?;
    write_checked(root, "artifacts/recurrence_plus_two_token.tlx", &art2)?;
    write_checked(
        root,
        "receipt.json",
        serde_json::to_string_pretty(&receipt)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    seal(root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed {} with {} unlisted files; total {:.1}s",
        root.display(),
        unlisted.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Condition mode: score one existing sealed artifact under both prose conditionings
// ---------------------------------------------------------------------------
//
// Two prose conditionings of the *same* development target identities, on an artifact that was
// trained elsewhere. No training happens here, the source root and the artifact's own directory are
// never written or re-sealed, and the artifact is read, hashed and scored in place: its identity is
// recorded in the receipt instead of being copied into the new report root.

fn condition_mode(args: &Args) -> Result<ExitCode, String> {
    let root = args.condition.clone().unwrap_or_default();
    if root.as_os_str().is_empty() {
        return Err("--condition needs a new report root".into());
    }
    if args.docs.as_os_str().is_empty() {
        return Err("--docs is required with --condition".into());
    }
    let artifact_path = match args.artifact.clone() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => return Err("--artifact (path to a .tlx file) is required with --condition".into()),
    };
    // Claimed exclusively before any model work: an existing report is never reused or overwritten.
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    println!("=== ordinary-lexical condition: both prose conditionings of one sealed artifact ===");

    // ---- tokenizer identity ------------------------------------------------
    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&Sha256::digest(&tok_bytes));
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer sha256 {tok_sha} != expected {EXPECTED_TOKENIZER_SHA256}"
        ));
    }
    let derived_bytes =
        uor_r4_core::transformerless::bpe_derive::derive_tokenizer_json(&tok_bytes, VOCAB)
            .map_err(|e| format!("derive bytes: {e}"))?;
    let derived_sha = sha256_hex(&Sha256::digest(&derived_bytes));
    let tokenizer = derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive: {e}"))?;
    write_checked(&root, "tokenizer_source.json", &tok_bytes)?;
    write_checked(&root, "tokenizer_derived_v4096.json", &derived_bytes)?;

    // ---- the artifact under test: read in place, never copied --------------
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let exe_bytes = std::fs::read(&exe).map_err(|e| format!("read exe: {e}"))?;
    let exe_sha = sha256_hex(&Sha256::digest(&exe_bytes));
    let artifact_bytes = std::fs::read(&artifact_path)
        .map_err(|e| format!("artifact {}: {e}", artifact_path.display()))?;
    let artifact_sha = sha256_hex(&Sha256::digest(&artifact_bytes));
    let model = TlModel::from_bytes(&artifact_bytes).map_err(|e| format!("deserialize: {e}"))?;

    // ---- corpus, source-separated (mirrors the training run) ---------------
    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
    let mut fit_docs = Vec::new();
    let mut tune_docs = Vec::new();
    let mut dev_pool = Vec::new();
    for (i, d) in uniq.iter().enumerate() {
        match d.split {
            Split::Fit => fit_docs.push(i),
            Split::Tune => tune_docs.push(i),
            Split::Dev => dev_pool.push(i),
        }
    }
    if fit_docs.is_empty() || dev_pool.is_empty() {
        return Err("corpus split has an empty fit or development side".into());
    }
    let mut fit_windows: Vec<ProseWindow> = Vec::new();
    for i in &fit_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), *i) {
            fit_windows.push(w);
        }
    }
    let all_fit_windows = fit_windows.len();
    if fit_windows.len() > FIT_MAX_WINDOWS {
        let stride = fit_windows.len() as f64 / FIT_MAX_WINDOWS as f64;
        fit_windows = (0..FIT_MAX_WINDOWS)
            .map(|k| {
                let idx = (k as f64 * stride) as usize;
                fit_windows[idx].clone()
            })
            .collect();
    }
    // Development: the same length-stratified document draw, in the same document order.
    let mut with_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    with_len.sort_by_key(|(_, n)| *n);
    let take = DEV_MAX_DOCS.min(with_len.len());
    let mut dev_windows: Vec<ProseWindow> = Vec::new();
    let mut dev_names: Vec<String> = Vec::new();
    for k in 0..take {
        let (doc_idx, _) = with_len[k * with_len.len() / take.max(1)];
        dev_names.push(uniq[doc_idx].path.clone());
        for w in prose_windows(&tokenizer.encode(&uniq[doc_idx].text), k)
            .into_iter()
            .take(DEV_WINDOWS_PER_DOC)
        {
            dev_windows.push(w);
        }
    }
    let mut tune_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), 0) {
            tune_windows.push(w.tokens);
        }
    }
    // Count references fitted on the fit split only, exactly as the training run fits them.
    let mut uni = Uni {
        counts: vec![0u64; VOCAB],
        total: 0,
    };
    let mut c1 = Cond::default();
    let mut c2 = Cond::default();
    for w in &fit_windows {
        for k in PREFIX..w.tokens.len() {
            let prev = if k == 1 {
                VOCAB
            } else {
                (w.tokens[k - 2] as usize).min(VOCAB - 1)
            };
            let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
            let next = w.tokens[k];
            uni.counts[next as usize] += 1;
            uni.total += 1;
            c1.observe(cur as u64, next);
            c2.observe(ctx2(prev, cur), next);
        }
    }
    let (lambdas, tune_bits) = tune_lambdas(&c1, &c2, &uni, &tune_windows);
    let e_bytes = std::fs::read(E_ARTIFACT).map_err(|e| format!("E artifact: {e}"))?;
    let e_core = PriorCore::from_bytes(&e_bytes).map_err(|e| format!("E load: {e}"))?;
    let e_sha = sha256_hex(&Sha256::digest(&e_bytes));

    let docs = dev_windows
        .iter()
        .map(|w| w.doc)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let dev_targets: usize = dev_windows
        .iter()
        .map(|w| w.tokens.len().saturating_sub(PREFIX))
        .sum();
    let position_dev_examples = position_examples(&dev_windows, 0);

    // ---- both conditionings over the same development windows --------------
    let layers = evaluate_layers(&model, &dev_windows, &uni, &c1, &c2, lambdas, &e_core);
    let window_eval = &layers[0];
    let position_eval = eval_examples(&model, &position_dev_examples, docs);
    let (u_bits, c_bits, e_bits) = (layers[1].micro(), layers[2].micro(), layers[3].micro());
    let window_bits = window_eval.micro();
    let position_bits = position_eval.micro();
    let paired = paired_interval(&position_eval, window_eval, 0x9E3779B9);
    let per_doc = |e: &Eval| -> Vec<f64> {
        e.per_doc
            .iter()
            .map(|(l, n)| if *n == 0 { f64::NAN } else { l / *n as f64 })
            .collect()
    };
    let (window_per_doc, position_per_doc) = (per_doc(window_eval), per_doc(&position_eval));

    println!(
        "1. artifact {} sha256 {artifact_sha} ({} B); tested executable sha256 {exe_sha}",
        artifact_path.display(),
        artifact_bytes.len()
    );
    println!(
        "2. window   conditioning bits/target {window_bits:.4} over {} targets",
        window_eval.total().1
    );
    println!(
        "3. position conditioning bits/target {position_bits:.4} over {} targets",
        position_eval.total().1
    );
    println!(
        "4. paired position minus window [point, lo, hi] = [{:.6}, {:.6}, {:.6}] (negative = the position conditioning scores fewer bits)",
        paired.0, paired.1, paired.2
    );
    println!(
        "5. token-only references unigram {u_bits:.4} tuned-count {c_bits:.4} donor E {e_bits:.4}; \
         dev docs {docs} over {dev_targets} targets; fit windows {} (of {all_fit_windows})",
        fit_windows.len()
    );

    let receipt = serde_json::json!({
        "schema": "uor-r4.ordinary-lexical-condition/1",
        "source_rev": args.source_rev.as_str(),
        "artifact": {
            "path": artifact_path.display().to_string(),
            "sha256": artifact_sha,
            "bytes": artifact_bytes.len(),
            "copied_into_report_root": false,
            "note": "the artifact is read, hashed and scored in place; this attempt records its \
                     identity and neither writes to nor re-seals the artifact's own directory",
        },
        "tested_executable_sha256": exe_sha,
        "tokenizer_source_sha256": tok_sha,
        "tokenizer_derived_sha256": derived_sha,
        "corpus": {
            "collected": collected,
            "unique": uniq.len(),
            "exact_duplicates_grouped": duplicates,
            "fit_docs": fit_docs.len(),
            "tune_docs": tune_docs.len(),
            "dev_docs": dev_names.len(),
            "fit_windows_used": fit_windows.len(),
            "fit_windows_available": all_fit_windows,
            "dev_windows": dev_windows.len(),
            "dev_scored_targets": dev_targets,
            "document_sha256": uniq.iter().map(|d| serde_json::json!({"path": d.path, "sha256": sha256_hex(&d.sha256)})).collect::<Vec<_>>(),
        },
        "dev_documents": dev_names,
        "window_conditioning": {
            "definition": "frozen prefix advanced with OBSERVE and later positions supervised as \
                           GENERATE: the served multi-token recurrence",
            "model_bits_per_target": window_bits,
            "targets": window_eval.total().1,
            "per_document_bits": window_per_doc,
        },
        "position_conditioning": {
            "definition": "one Generate target per position from PREFIX onward with the whole \
                           preceding window context advanced as OBSERVE",
            "model_bits_per_target": position_bits,
            "targets": position_eval.total().1,
            "per_document_bits": position_per_doc,
        },
        "paired_interval_position_minus_window": [paired.0, paired.1, paired.2],
        "paired_interval_sign": "point = position minus window; negative means the position conditioning scores lower bits",
        "token_only_references_same_target_population": {
            "unigram_bits_per_target": u_bits,
            "tuned_two_token_count_bits_per_target": c_bits,
            "donor_E_bits_per_target": e_bits,
        },
        "conditioning_note": "the two conditionings share target identities (one Generate target per \
                              development position, the same scored targets) but not the conditioning \
                              event of the preceding context: the position conditioning advances the \
                              whole preceding window context with the OBSERVE event, so the two are not \
                              interchangeable and only the window conditioning matches the served \
                              multi-token recurrence. The token-only references share the target \
                              population and are therefore reported once.",
        "count_reference": {"lambdas": [lambdas.0, lambdas.1], "tune_bits_per_target": tune_bits},
        "donor_E": {"path": E_ARTIFACT, "bytes": e_bytes.len(), "sha256": e_sha,
                    "exposure": "trained on the pinned corpus; disclosed, not removed"},
        "elapsed_seconds": started.elapsed().as_secs_f64(),
    });
    write_checked(
        &root,
        "receipt.json",
        serde_json::to_string_pretty(&receipt)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "6. sealed {} with {} unlisted files; total {:.1}s",
        root.display(),
        unlisted.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// State probe mode: a float readout fitted on an artifact's own recorded served states
// ---------------------------------------------------------------------------
//
// One instrument, one question. The shared artifact scores ~6.69 bits per development target while a
// fit/tune-separated tuned two-token count reference scores 5.12 on the same targets. Either the
// served recurrent state does not carry the information that would close the gap, or it carries it
// and the trained ternary readout cannot use it.
//
// `--state-probe` decides between those two readings by recording the **served state** at every
// supervised `Generate` decision of the fit and development splits — the very vector the artifact's
// readout consumes, recorded by the same walk `TlModel::score_example` performs — and fitting an
// unrestricted float multinomial logistic readout on the fit split's states. The probe's development
// loss is therefore an upper bound on what *any* readout could achieve from that state.
//
// The probe is a diagnostic instrument, not a serving path: it is fitted and evaluated offline in
// `f64` over recorded states, it is never serialised into an artifact, never loaded by a session and
// never consulted while serving. The artifact is read, hashed and probed in place: nothing here
// writes to or re-seals the artifact's own directory.

/// One recorded served state: exactly the vector the artifact's readout consumes at one `Generate`
/// decision, with the decision's event, its target token, its document group, the served bits of that
/// same decision, and the tokens the two transitions before it consumed.
struct ServedState {
    h: Vec<i32>,
    target: u32,
    doc: usize,
    bits: f64,
    /// `ex.prior_event(i)`: the event symbol the readout was given at this decision.
    event: usize,
    /// The token consumed by the transition that produced `h` (`None` when nothing was consumed —
    /// a `Stop`, or a state straight after `init_state`). For ordinary prose this is `x_(t-1)`.
    cur: Option<u32>,
    /// The token consumed by the transition before that; `None` when there is none. For ordinary
    /// prose this is `x_(t-2)`.
    prev: Option<u32>,
}

/// Walk one example exactly as [`TlModel::score_example`] walks it, recording the served state at
/// every `Generate` decision.
///
/// For a non-grounded example `score_example` takes `owned = &[]`, `m = vec![0i32; h_dim]` and
/// `f = typed_block(&[], &[], SlFacts::default())`, observes `ex.observed` with `TL_EV_OBSERVE`, then
/// scores step `i` on `(h, m, f, ex.prior_event(i))` and advances with
/// `(ex.actions[i].event(), emitted)` where `emitted` is the literal non-grounded branch: `Some(v)`
/// for `Generate`, `ex.emitted(i, owned)` for `Copy`, `None` for `Stop`. Every step is reproduced
/// here, so the recorded `h` is the state the readout receives by construction; `Stop` steps are not
/// recorded but do advance it. The recorded `bits` come from the very logits `score_example`
/// normalises, so their sum reproduces its `bits_generate` (checked against it in `state_probe_mode`).
///
/// `cur`/`prev` are read off the walk's own consumption history — the token each transition was
/// given, in walk order, with `None` for a `Stop` — not off any window indexing, so the same function
/// is correct for windows, for synthetic examples and for a `Stop` in mid-script. `state_probe_mode`
/// additionally checks them against the window token sequence, which is what the count reference
/// conditions on, and the unit test re-derives a recorded state from `(previous state, event, cur)`.
fn record_example_served_states(model: &TlModel, ex: &TlExample) -> Vec<ServedState> {
    let owned: &[u32] = &[];
    let m = vec![0i32; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let mut out = Vec::new();
    let mut h = model.init_state(&m, &f);
    // One entry per transition performed so far, newest last: exactly the token that transition was
    // given, `None` when it was given none (a `Stop`, or a blinded diagnostic).
    let mut consumed: Vec<Option<u32>> = Vec::new();
    for t in &ex.observed {
        h = model.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
        consumed.push(Some(*t));
    }
    for i in 0..ex.actions.len() {
        let copy_legal = ex.copy_legal(i, owned);
        let event = ex.prior_event(i);
        if let TlAction::Generate(v) = ex.actions[i] {
            let logits = model.readout(&h, &m, &f, event);
            let rows = model.legal_rows(copy_legal);
            let p = prob_of(
                &logits,
                &rows,
                model.action_row(ex.actions[i]),
                model.score_shift,
            );
            let n = consumed.len();
            out.push(ServedState {
                h: h.clone(),
                target: v,
                doc: ex.doc,
                bits: -p.max(f64::MIN_POSITIVE).log2(),
                event,
                cur: consumed.get(n.wrapping_sub(1)).copied().flatten(),
                prev: consumed.get(n.wrapping_sub(2)).copied().flatten(),
            });
        }
        let emitted = match ex.actions[i] {
            TlAction::Generate(v) => Some(v),
            TlAction::Copy => ex.emitted(i, owned),
            TlAction::Stop => None,
        };
        h = model.transition(&h, ex.actions[i].event(), emitted, &m, &f);
        consumed.push(emitted);
    }
    out
}

/// The same recording over a split of prose windows: the fit and development splits go through one
/// code path, so the two populations are recorded in exactly the same way.
fn record_served_states(model: &TlModel, windows: &[ProseWindow]) -> Vec<ServedState> {
    let mut out = Vec::new();
    for w in windows {
        out.extend(record_example_served_states(model, &prose_example(w)));
    }
    out
}

/// Declared byte-stream fingerprint of one split's recorded states: for every state in walk order,
/// `doc` (u64 LE), `target` (u32 LE), `event` (u32 LE), `cur` (u32 LE, `u32::MAX` for none),
/// `prev` (same), then every `h` entry (i32 LE).
fn recorded_states_sha256(states: &[ServedState]) -> String {
    let none = u32::MAX;
    let mut hasher = Sha256::new();
    for s in states {
        hasher.update(s.doc.to_le_bytes());
        hasher.update(s.target.to_le_bytes());
        hasher.update((s.event as u32).to_le_bytes());
        hasher.update(s.cur.unwrap_or(none).to_le_bytes());
        hasher.update(s.prev.unwrap_or(none).to_le_bytes());
        for v in &s.h {
            hasher.update(v.to_le_bytes());
        }
    }
    sha256_hex(&hasher.finalize())
}

/// A bits/target total over one position population, kept both restricted to the top-`k` targets and
/// over every scored `Generate` target, so the restriction never changes a reference definition.
#[derive(Default, Clone, Copy)]
struct BitsPair {
    restricted_bits: f64,
    restricted_n: usize,
    all_bits: f64,
    all_n: usize,
}

impl BitsPair {
    fn add(&mut self, bits: f64, keep: bool) {
        self.all_bits += bits;
        self.all_n += 1;
        if keep {
            self.restricted_bits += bits;
            self.restricted_n += 1;
        }
    }
    fn restricted(&self) -> f64 {
        if self.restricted_n == 0 {
            f64::NAN
        } else {
            self.restricted_bits / self.restricted_n as f64
        }
    }
    fn all(&self) -> f64 {
        if self.all_n == 0 {
            f64::NAN
        } else {
            self.all_bits / self.all_n as f64
        }
    }
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "restricted_bits_per_target": self.restricted(),
            "restricted_targets": self.restricted_n,
            "unrestricted_bits_per_target": self.all(),
            "unrestricted_targets": self.all_n,
        })
    }
}

/// `(prev, cur, target)` of one prose position, with `prev`/`cur` derived exactly as `reference_evals`
/// derives them.
fn prose_position(w: &ProseWindow, k: usize) -> (usize, usize, u32) {
    let prev = if k == 1 {
        VOCAB
    } else {
        (w.tokens[k - 2] as usize).min(VOCAB - 1)
    };
    let cur = (w.tokens[k - 1] as usize).min(VOCAB - 1);
    (prev, cur, w.tokens[k])
}

/// The `E1` level: unigram interpolated with the **single-token** bigram of `cur` only, with its weight
/// tuned on the tune split. This is the count reference a *local* one-token channel can reach, so it
/// bounds what the served recurrence would need in order to be redundant with a one-token channel.
/// The interpolation is `l * p(next | cur) + (1 - l) * p_unigram(next)`, tuned over the same declared
/// grid `tune_lambdas` searches.
fn tune_e1_weight(c1: &Cond, uni: &Uni, tune: &[Vec<u32>]) -> (f64, f64) {
    let e1_p = |cur: u64, target: u32, l: f64| -> f64 {
        let t = c1.total(cur);
        let p1 = if t == 0 {
            uni.p(target)
        } else {
            c1.count_of(cur, target) as f64 / t as f64
        };
        l * p1 + (1.0 - l) * uni.p(target)
    };
    let mut best = (0.5, f64::INFINITY);
    for &l in LAMBDA_GRID.iter() {
        let mut bits = 0.0;
        let mut n = 0usize;
        for w in tune {
            for k in PREFIX..w.len() {
                // Unigram + bigram(`cur`): the context is the single preceding token, exactly the one
                // `prose_position` uses, so `E1` and the `(prev, cur)` reference share a definition.
                bits -= e1_p(w[k - 1] as u64, w[k], l).max(1e-300).log2();
                n += 1;
            }
        }
        if n > 0 {
            let per = bits / n as f64;
            if per < best.1 {
                best = (l, per);
            }
        }
    }
    best
}

/// The `E1` bits of one split, restricted and unrestricted, at the tuned weight.
fn e1_reference(
    windows: &[ProseWindow],
    c1: &Cond,
    uni: &Uni,
    weight: f64,
    in_top: &[bool],
) -> BitsPair {
    let mut p = BitsPair::default();
    for w in windows {
        for k in PREFIX..w.tokens.len() {
            let (_, cur, target) = prose_position(w, k);
            let t = c1.total(cur as u64);
            let p1 = if t == 0 {
                uni.p(target)
            } else {
                c1.count_of(cur as u64, target) as f64 / t as f64
            };
            let prob = weight * p1 + (1.0 - weight) * uni.p(target);
            p.add(-prob.max(1e-300).log2(), in_top[target as usize]);
        }
    }
    p
}

/// The fit-only token references on one split under the same top-`k` restriction, through the
/// existing helpers so the definitions are unchanged: the unigram, the tuned interpolated
/// `(prev, cur)` count model, and the unigram renormalised over the restricted classes (the level a
/// `k`-class readout reaches by learning only the marginal, which is the null probe's expectation).
fn token_references(
    windows: &[ProseWindow],
    uni: &Uni,
    c1: &Cond,
    c2: &Cond,
    lambdas: (f64, f64),
    in_top: &[bool],
    top_mass: f64,
) -> (BitsPair, BitsPair, BitsPair) {
    let (mut u, mut c, mut r) = (
        BitsPair::default(),
        BitsPair::default(),
        BitsPair::default(),
    );
    for w in windows {
        for k in PREFIX..w.tokens.len() {
            let (prev, cur, target) = prose_position(w, k);
            let keep = in_top[target as usize];
            let pu = uni.p(target).max(1e-300);
            u.add(-pu.log2(), keep);
            c.add(
                -family_p(c1, c2, uni, prev, cur, target, lambdas)
                    .max(1e-300)
                    .log2(),
                keep,
            );
            r.add(-(pu / top_mass).max(1e-300).log2(), keep);
        }
    }
    (u, c, r)
}

/// The artifact's own served readout loss over already recorded states, under the same restriction.
fn served_reference(states: &[ServedState], in_top: &[bool]) -> BitsPair {
    let mut p = BitsPair::default();
    for s in states {
        p.add(s.bits, in_top[s.target as usize]);
    }
    p
}

/// The declared probe input: `[h / h_clamp, event one-hot]`, `d = h_dim + TL_EVENTS`.
///
/// `score_example` feeds its readout `[h, m, f, event_onehot]`. The probe keeps `h` and the event and
/// **drops `m` and `f`**, verified before relying on it:
///
/// * `m`: for a non-grounded example `score_example` takes `m = vec![0i32; h_dim]` literally, and
///   `prose_example` is the only example constructor this mode uses, with `grounded: false`. So `m` is
///   exactly all zeros; `state_probe_mode` re-checks that no window example is grounded and that
///   `sel`/`res` are empty and `facts` is `SlFacts::default()`, which is the premise of that branch.
/// * `f = typed_block(&[], &[], SlFacts::default())` is the **same declared constant block at every
///   prose position** (it is `[1, 0, ...]`: `typed_causal_block` sets one `history` slot and every
///   other entry is a flag or a count that is zero for empty `sel`/`res`). `state_probe_mode` computes
///   it per window and asserts it is identical for every window.
///
/// Dropping constant coordinates costs a *linear* readout nothing: the zero block contributes nothing
/// and the constant block contributes one constant per class, which any fitted bias absorbs. The
/// event, by contrast, varies per decision and is kept, so the probe is never given less than the
/// served readout receives.
const PROBE_EVENT_DIMS: usize = TL_EVENTS;

/// The float probe's population: the recorded served states of one split, normalised by the declared
/// scaling `1 / h_clamp` with the decision's event one-hot appended, restricted to the top-`k`
/// targets.
struct ProbeSet {
    /// Row-major `n x d`, `d = h_dim + TL_EVENTS`, each entry `h / h_clamp` then the event one-hot.
    x: Vec<f64>,
    d: usize,
    /// Class index of each retained target in the top-`k` id list.
    y: Vec<usize>,
    target: Vec<u32>,
    doc: Vec<usize>,
    /// The two tokens consumed before this state, as recorded.
    cur: Vec<Option<u32>>,
    prev: Vec<Option<u32>>,
    /// Served bits of the same decision, for the restricted served reference.
    bits: Vec<f64>,
    /// Index into the recorded state list this position came from, so the integer (servable) path can
    /// recover the exact integers the served readout would be given.
    state_index: Vec<usize>,
    /// Every recorded `Generate` decision of the split, before the restriction.
    all_recorded: usize,
    n: usize,
}

fn probe_set(states: &[ServedState], class_of: &[usize], h_clamp: i32) -> ProbeSet {
    let scale = 1.0 / h_clamp as f64;
    let d = states
        .first()
        .map(|s| s.h.len() + PROBE_EVENT_DIMS)
        .unwrap_or(PROBE_EVENT_DIMS);
    let mut set = ProbeSet {
        x: Vec::new(),
        d,
        y: Vec::new(),
        target: Vec::new(),
        doc: Vec::new(),
        cur: Vec::new(),
        prev: Vec::new(),
        bits: Vec::new(),
        state_index: Vec::new(),
        all_recorded: states.len(),
        n: 0,
    };
    for (state_index, s) in states.iter().enumerate() {
        let class = match class_of.get(s.target as usize) {
            Some(c) if *c != usize::MAX => *c,
            _ => continue,
        };
        set.x.extend(s.h.iter().map(|v| *v as f64 * scale));
        for e in 0..PROBE_EVENT_DIMS {
            set.x.push(if e == s.event.min(PROBE_EVENT_DIMS - 1) {
                1.0
            } else {
                0.0
            });
        }
        set.y.push(class);
        set.target.push(s.target);
        set.doc.push(s.doc);
        set.cur.push(s.cur);
        set.prev.push(s.prev);
        set.bits.push(s.bits);
        set.state_index.push(state_index);
        set.n += 1;
    }
    set
}

/// A float multinomial logistic readout over recorded probe inputs: `K x d` weights plus `K`
/// biases, fitted by Adam from a declared initialisation. Diagnostic only.
struct FloatProbe {
    k: usize,
    d: usize,
    w: Vec<f64>,
    b: Vec<f64>,
    /// First and second Adam moments of the weights and the biases.
    mw: Vec<f64>,
    vw: Vec<f64>,
    mb: Vec<f64>,
    vb: Vec<f64>,
    steps: u32,
}

impl FloatProbe {
    fn zeros(k: usize, d: usize) -> Self {
        Self {
            k,
            d,
            w: vec![0.0; k * d],
            b: vec![0.0; k],
            mw: vec![0.0; k * d],
            vw: vec![0.0; k * d],
            mb: vec![0.0; k],
            vb: vec![0.0; k],
            steps: 0,
        }
    }

    /// Initialise from a closed-form readout: the refinement starts at the ridge solution with fresh
    /// Adam moments, so the first epochs refine a converged readout instead of fitting a marginal.
    fn from_readout(k: usize, d: usize, w: &[f64], b: &[f64]) -> Result<Self, String> {
        if w.len() != k * d || b.len() != k {
            return Err(format!(
                "readout shape {}x{} does not match the probe's {k}x{d}",
                w.len() / d.max(1),
                d
            ));
        }
        Ok(Self {
            w: w.to_vec(),
            b: b.to_vec(),
            ..Self::zeros(k, d)
        })
    }

    /// Row scores of one state, returning the stabilising maximum. Zero coordinates are skipped.
    fn scores(&self, x: &[f64], out: &mut [f64]) -> f64 {
        let mut max = f64::NEG_INFINITY;
        for r in 0..self.k {
            let row = &self.w[r * self.d..(r + 1) * self.d];
            let mut acc = self.b[r];
            for (wv, xv) in row.iter().zip(x.iter()) {
                if *xv != 0.0 {
                    acc += wv * xv;
                }
            }
            out[r] = acc;
            if acc > max {
                max = acc;
            }
        }
        max
    }

    /// Top-1 accuracy over the given positions, with the same deterministic lowest-row tie-break the
    /// served path uses.
    fn top1(&self, x: &[f64], y: &[usize], idx: &[usize], scratch: &mut [f64]) -> f64 {
        if idx.is_empty() {
            return f64::NAN;
        }
        let mut correct = 0usize;
        for &i in idx {
            let xi = &x[i * self.d..(i + 1) * self.d];
            self.scores(xi, scratch);
            let mut best = 0usize;
            for r in 1..self.k {
                if scratch[r] > scratch[best] {
                    best = r;
                }
            }
            if best == y[i] {
                correct += 1;
            }
        }
        correct as f64 / idx.len() as f64
    }

    /// One Adam step on the batch-mean gradient of the mean NLL (nats) of the batch.
    fn step(&mut self, x: &[f64], y: &[usize], idx: &[usize], lr: f64) {
        if idx.is_empty() {
            return;
        }
        let inv = 1.0 / idx.len() as f64;
        let mut gw = vec![0.0; self.w.len()];
        let mut gb = vec![0.0; self.k];
        let mut logits = vec![0.0; self.k];
        for &i in idx {
            let xi = &x[i * self.d..(i + 1) * self.d];
            let max = self.scores(xi, &mut logits);
            let mut z = 0.0;
            for v in logits.iter() {
                z += (*v - max).exp();
            }
            let target = y[i];
            for r in 0..self.k {
                let d = (logits[r] - max).exp() / z - if r == target { 1.0 } else { 0.0 };
                if d == 0.0 {
                    continue;
                }
                gb[r] += d * inv;
                let grow = &mut gw[r * self.d..(r + 1) * self.d];
                for (g, xv) in grow.iter_mut().zip(xi.iter()) {
                    if *xv != 0.0 {
                        *g += d * *xv * inv;
                    }
                }
            }
        }
        self.steps += 1;
        let b1t = 1.0 - ADAM_BETA1.powi(self.steps as i32);
        let b2t = 1.0 - ADAM_BETA2.powi(self.steps as i32);
        for i in 0..self.w.len() {
            let g = gw[i];
            let m = ADAM_BETA1 * self.mw[i] + (1.0 - ADAM_BETA1) * g;
            let v = ADAM_BETA2 * self.vw[i] + (1.0 - ADAM_BETA2) * g * g;
            self.mw[i] = m;
            self.vw[i] = v;
            self.w[i] -= lr * (m / b1t) / ((v / b2t).sqrt() + ADAM_EPS);
        }
        for r in 0..self.k {
            let g = gb[r];
            let m = ADAM_BETA1 * self.mb[r] + (1.0 - ADAM_BETA1) * g;
            let v = ADAM_BETA2 * self.vb[r] + (1.0 - ADAM_BETA2) * g * g;
            self.mb[r] = m;
            self.vb[r] = v;
            self.b[r] -= lr * (m / b1t) / ((v / b2t).sqrt() + ADAM_EPS);
        }
    }

    /// A fingerprint of the fitted parameters, declared: `k` (u32 LE), `h_dim` (u32 LE), then every
    /// weight and bias (f64 LE, in storage order). It records that this attempt's float fit is the one
    /// described, without storing the parameters themselves.
    fn weights_sha256(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update((self.k as u32).to_le_bytes());
        hasher.update((self.d as u32).to_le_bytes());
        for v in self.w.iter().chain(self.b.iter()) {
            hasher.update(v.to_le_bytes());
        }
        sha256_hex(&hasher.finalize())
    }

    /// Mean NLL in bits per target over the given positions, with a numerically stable softmax.
    fn nll_bits(&self, x: &[f64], y: &[usize], idx: &[usize], scratch: &mut [f64]) -> f64 {
        if idx.is_empty() {
            return f64::NAN;
        }
        let mut total = 0.0;
        for &i in idx {
            let xi = &x[i * self.d..(i + 1) * self.d];
            let max = self.scores(xi, scratch);
            let mut z = 0.0;
            for v in scratch.iter() {
                z += (*v - max).exp();
            }
            let p = ((scratch[y[i]] - max).exp() / z).max(f64::MIN_POSITIVE);
            total -= p.log2();
        }
        total / idx.len() as f64
    }
}

/// One fitted float probe: its fit/development loss at the declared initialisation (before any epoch,
/// which for the artifact initialisation *is* the artifact's own readout in float form at the probe's
/// class normalisation) and after every epoch.
struct FloatProbeFit {
    probe: FloatProbe,
    init_fit_bits: f64,
    init_dev_bits: f64,
    per_epoch: Vec<(f64, f64)>,
}

/// Fit the probe and report its fit and development loss after every epoch, so a reader can see that
/// it has converged rather than merely being under-trained. The probe starts from the caller's
/// declared initialisation (the closed-form ridge readout for the refinement, the permuted-label ridge
/// readout for the null control) and each epoch visits every fit position once, in minibatches of
/// `PROBE_BATCH` drawn without replacement from a per-epoch shuffle.
fn fit_float_probe(
    mut probe: FloatProbe,
    x: &[f64],
    y: &[usize],
    n: usize,
    dev: &ProbeSet,
    epochs: usize,
    lr: f64,
    seed: u64,
    label: &str,
) -> FloatProbeFit {
    let k = probe.k;
    let mut order: Vec<usize> = (0..n).collect();
    let all_fit: Vec<usize> = (0..n).collect();
    let all_dev: Vec<usize> = (0..dev.n).collect();
    let mut scratch = vec![0.0; k];
    let init_fit_bits = probe.nll_bits(x, y, &all_fit, &mut scratch);
    let init_dev_bits = probe.nll_bits(&dev.x, &dev.y, &all_dev, &mut scratch);
    println!(
        "    {label} init (no epoch yet)  fit {init_fit_bits:.4} bits/target   dev {init_dev_bits:.4} \
         bits/target"
    );
    let mut st = seed ^ 0x5DEE_CE66;
    let mut per_epoch = Vec::with_capacity(epochs);
    for epoch in 0..epochs {
        shuffle(&mut order, &mut st);
        for batch in order.chunks(PROBE_BATCH) {
            probe.step(x, y, batch, lr);
        }
        let fb = probe.nll_bits(x, y, &all_fit, &mut scratch);
        let db = probe.nll_bits(&dev.x, &dev.y, &all_dev, &mut scratch);
        println!(
            "    {label} epoch {:>2}/{epochs}  fit {fb:.4} bits/target   dev {db:.4} bits/target",
            epoch + 1
        );
        per_epoch.push((fb, db));
    }
    FloatProbeFit {
        probe,
        init_fit_bits,
        init_dev_bits,
        per_epoch,
    }
}

/// The declared rule that decides whether the fit is still moving at the last epoch, in bits/target.
const PROBE_CONVERGED_BITS: f64 = 0.01;
/// The declared margin below the unigram-over-`k` level at which the null probe is called a leak.
const NULL_LEAK_BITS: f64 = 0.05;

// ---------------------------------------------------------------------------
// Closed-form ridge readout, its servable repricings, and the state-side decode ladder
// ---------------------------------------------------------------------------
//
// The iterative probe answers slow questions slowly. A linear readout has a closed form, so the
// converged readout — the thing the "state versus readout" question actually needs — can be had in one
// pass over the fit states, and the *servable* question (can a bounded ≤4-bit readout realise that
// gain?) can then be asked directly by requantising it and rescoring it in integer arithmetic.

/// The declared ridge search grid, scaled by `trace(HtH) / d`.
const RIDGE_LAMBDAS: [f64; 5] = [1e-6, 1e-4, 1e-2, 1.0, 100.0];
/// Declared fraction of the fit states held out for the ridge-`lambda` and temperature choices.
const RIDGE_SELECT_FRACTION: f64 = 0.1;
/// Declared cap on the held-out population actually scored per `lambda` candidate.
const RIDGE_SELECT_CAP: usize = 20_000;
/// Declared cap on the population the temperature is fitted on.
const RIDGE_TEMP_CAP: usize = 10_000;
/// Declared first-stage population for the coarse temperature scan.
const RIDGE_TEMP_COARSE_CAP: usize = 2_000;
/// Declared Cholesky pivot tolerance, relative to the mean diagonal of the system.
const CHOLESKY_REL_TOL: f64 = 1e-12;
/// Declared bounded search interval for `log2(T)`.
const TEMP_LOG2_RANGE: f64 = 12.0;
/// Declared per-row scale offsets searched by the fitted repricing policy; `0` is the scheme's rule.
const REPRICE_OFFSETS: [i32; 4] = [0, -1, -2, -3];
/// The artifact's declared upper bound on its dyadic score scale (`TlConfig::score_shift`).
const SS_MAX: u32 = 20;
/// Declared cap on the fit states the nearest-embedding decode scans (dev is always complete).
const EMBED_DECODE_FIT_CAP: usize = 50_000;
/// Declared stride of the served-bit control on the fit windows: every window whose index is a
/// multiple of this is checked, plus every terminal window. Development is always checked completely.
const FIT_CONTROL_STRIDE: usize = 20;

/// The refined probe's last reported fit loss, for choosing which float readout to reprice.
/// The artifact's own served readout, rearranged into the probe's feature layout.
///
/// The probe's input is `[h / h_clamp, event one-hot]` (`d = h_dim + PROBE_EVENT_DIMS`). The artifact's
/// readout input is `[h, m, f, event]` (`2*h_dim + f_dim + TL_EVENTS`) through a per-row scaled ternary
/// map plus an `i32` bias:
///
/// * `h` state columns (`0 .. h_dim`), divided by `h_clamp` in the probe's input, so
///   `W[c][j] = (w_(r_c, j) << shift_(r_c)) * h_clamp`;
/// * `m` columns (`h_dim .. 2*h_dim`) contribute **exactly zero** for ordinary prose, because `m` is
///   identically zero, so nothing is taken from them;
/// * `f` columns (`2*h_dim .. 2*h_dim + f_dim`) are a **constant**, so their weighted contribution is one
///   constant per output row, folded into `b` as `Σ_f' (w_(r_c, 2h+f') << shift_(r_c)) * f0[f']`;
/// * event columns (`2*h_dim + f_dim ..`) are unscaled: `W[c][h_dim + e] = w_(r_c, 2h+f+e) << shift_(r_c)`;
/// * the artifact's `i32` bias `bo` of the same readout row is added into `b`;
/// * every coefficient is multiplied by `2^-score_shift`, the artifact's declared dyadic score scale, so
///   the initialised probe's logits are the **served** logits in float form.
///
/// `r_c = token_row(top_ids[c])` is the artifact readout row of the `c`-th restricted class. Every
/// coefficient is read from the artifact's **own** float evaluation of its packed map
/// (`TlLinear::forward_reference` on unit vectors), so no packing or dequantisation convention is
/// reimplemented here. `state_column_scale` folds the probe's `1 / h_clamp` input scaling back in.
struct ArtifactReadoutInit {
    /// `k x d`, class-major.
    w: Vec<f64>,
    b: Vec<f64>,
    state_columns: usize,
    m_columns_dropped: usize,
    f_columns_folded: usize,
    f_bias_from_constant_block: f64,
    event_columns: usize,
    score_scale: f64,
}

fn artifact_readout_init(
    model: &TlModel,
    top_ids: &[u32],
    f_block: &[i32],
    h_clamp: i32,
) -> Result<ArtifactReadoutInit, String> {
    let (k, h_dim, f_dim) = (top_ids.len(), model.h_dim, model.f_dim);
    let d = h_dim + PROBE_EVENT_DIMS;
    let cols = model.wo.cols;
    if cols != 2 * h_dim + f_dim + TL_EVENTS {
        return Err(format!(
            "artifact readout width {cols} is not 2*h_dim + f_dim + TL_EVENTS = {}",
            2 * h_dim + f_dim + TL_EVENTS
        ));
    }
    if f_block.len() != f_dim {
        return Err(format!(
            "typed block width {} is not the artifact fact width {f_dim}",
            f_block.len()
        ));
    }
    let rows: Vec<usize> = top_ids.iter().map(|t| model.token_row(*t)).collect();
    let score_scale = (2.0f64).powi(-(model.score_shift as i32));
    let mut w = vec![0.0f64; k * d];
    let mut b = vec![0.0f64; k];
    let mut unit = vec![0.0f64; cols];
    // State block: one unit-vector probe per column, through the artifact's own dequantisation.
    for j in 0..h_dim {
        unit[j] = 1.0;
        let out = model.wo.forward_reference(&unit);
        unit[j] = 0.0;
        for (c, r) in rows.iter().enumerate() {
            w[c * d + j] = out[*r] * h_clamp as f64 * score_scale;
        }
    }
    // Event block: unscaled, the probe's one-hot is the artifact's one-hot.
    for e in 0..PROBE_EVENT_DIMS {
        let col = 2 * h_dim + f_dim + e;
        unit[col] = 1.0;
        let out = model.wo.forward_reference(&unit);
        unit[col] = 0.0;
        for (c, r) in rows.iter().enumerate() {
            w[c * d + h_dim + e] = out[*r] * score_scale;
        }
    }
    // Constant typed block: a per-row constant, folded into the bias.
    let mut f_const = vec![0.0f64; k];
    let mut f_columns_folded = 0usize;
    for (fi, fv) in f_block.iter().enumerate() {
        if *fv == 0 {
            continue;
        }
        f_columns_folded += 1;
        let col = 2 * h_dim + fi;
        unit[col] = 1.0;
        let out = model.wo.forward_reference(&unit);
        unit[col] = 0.0;
        for (c, r) in rows.iter().enumerate() {
            f_const[c] += out[*r] * (*fv as f64);
        }
    }
    for (c, r) in rows.iter().enumerate() {
        b[c] = (f_const[c] + model.bo[*r] as f64) * score_scale;
    }
    let f_bias_from_constant_block = f_const.iter().map(|v| v.abs()).fold(0.0f64, f64::max);
    Ok(ArtifactReadoutInit {
        w,
        b,
        state_columns: h_dim,
        m_columns_dropped: h_dim,
        f_columns_folded,
        f_bias_from_constant_block,
        event_columns: PROBE_EVENT_DIMS,
        score_scale,
    })
}

/// The control that the rearrangement above is the artifact's own readout: on real recorded states the
/// initialised probe's logits must equal `TlModel::readout`'s logits at the artifact's declared dyadic
/// scale, entry by entry. Returns the number of positions checked, the number of compared values and the
/// largest absolute deviation seen.
fn verify_artifact_init(
    model: &TlModel,
    init: &ArtifactReadoutInit,
    rows: &[usize],
    states: &[ServedState],
    idx: &[usize],
    f_block: &[i32],
) -> Result<(usize, usize, f64), String> {
    let (k, h_dim) = (init.b.len(), model.h_dim);
    let d = h_dim + PROBE_EVENT_DIMS;
    let m = vec![0i32; h_dim];
    let mut worst = 0.0f64;
    let mut compared = 0usize;
    for &i in idx {
        let s = states
            .get(i)
            .ok_or("initialisation control: state index out of range")?;
        let served = model.readout(&s.h, &m, f_block, s.event);
        let mut x = vec![0.0f64; d];
        for c in 0..h_dim {
            x[c] = s.h[c] as f64 / model.h_clamp as f64;
        }
        x[h_dim + s.event.min(PROBE_EVENT_DIMS - 1)] = 1.0;
        for c in 0..k {
            let mut acc = init.b[c];
            for j in 0..d {
                acc += init.w[c * d + j] * x[j];
            }
            let want = served[rows[c]] as f64 * init.score_scale;
            let dev = (acc - want).abs();
            if dev > worst {
                worst = dev;
            }
            compared += 1;
        }
    }
    Ok((idx.len(), compared, worst))
}

/// The refined probe's last reported fit loss, for choosing which float readout to reprice.
fn probe_fit_last_of(fit: &FloatProbeFit) -> f64 {
    fit.per_epoch.last().map(|(f, _)| *f).unwrap_or(f64::NAN)
}

/// The declared integers of the servable path for the positions in `idx` (row `j` <-> `idx[j]`): the
/// raw clipped state block followed by the decision's event one-hot. This is exactly what the artifact's
/// integer maps would be handed, so the repricing is scored on the served input, not on the float one.
fn integer_inputs(states: &[ServedState], set: &ProbeSet, idx: &[usize]) -> Vec<i32> {
    let d = set.d;
    let mut out = Vec::with_capacity(idx.len() * d);
    for &i in idx {
        let s = &states[set.state_index[i]];
        out.extend_from_slice(&s.h);
        for e in 0..PROBE_EVENT_DIMS {
            out.push(i32::from(e == s.event.min(PROBE_EVENT_DIMS - 1)));
        }
    }
    out
}

/// First- and second-order moments of one population, enough for ridge: `HtH` (upper triangle only),
/// `HtY`, the feature sums and the class counts. Centering these is algebraically identical to fitting
/// an unpenalised intercept, so the system stays exactly `d x d`.
struct Moments {
    d: usize,
    k: usize,
    n: f64,
    ht_h: Vec<f64>,
    ht_y: Vec<f64>,
    sx: Vec<f64>,
    sy: Vec<f64>,
}

impl Moments {
    fn zeros(d: usize, k: usize) -> Self {
        Self {
            d,
            k,
            n: 0.0,
            ht_h: vec![0.0; d * d],
            ht_y: vec![0.0; d * k],
            sx: vec![0.0; d],
            sy: vec![0.0; k],
        }
    }

    /// Add one `(x, class)` observation. `HtH` keeps only `r <= c`, mirrored when the system is built.
    fn add(&mut self, x: &[f64], class: usize) {
        self.n += 1.0;
        self.sy[class] += 1.0;
        for r in 0..self.d {
            let xr = x[r];
            if xr != 0.0 {
                self.sx[r] += xr;
                self.ht_y[r * self.k + class] += xr;
                for c in r..self.d {
                    let xc = x[c];
                    if xc != 0.0 {
                        self.ht_h[r * self.d + c] += xr * xc;
                    }
                }
            }
        }
    }
}

/// Solve `A X = B` for symmetric positive-definite `A` (`n x n`, row-major) and `B` (`n x k`) by
/// Cholesky: `Err` on a pivot below the declared tolerance instead of a panic or a silent NaN.
fn cholesky_solve(a: &[f64], n: usize, b: &[f64], k: usize) -> Result<Vec<f64>, String> {
    let mean_diag = (0..n).map(|i| a[i * n + i]).sum::<f64>() / n as f64;
    let tol = CHOLESKY_REL_TOL * mean_diag.abs().max(1.0);
    let mut l = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..=i {
            let mut sum = a[i * n + j];
            for p in 0..j {
                sum -= l[i * n + p] * l[j * n + p];
            }
            if i == j {
                if !(sum > tol) {
                    return Err(format!(
                        "ridge system is not positive definite at pivot {i}: {sum} (tolerance {tol})"
                    ));
                }
                l[i * n + i] = sum.sqrt();
            } else {
                l[i * n + j] = sum / l[j * n + j];
            }
        }
    }
    let mut x = b.to_vec();
    for c in 0..k {
        for i in 0..n {
            let mut s = x[i * k + c];
            for p in 0..i {
                s -= l[i * n + p] * x[p * k + c];
            }
            x[i * k + c] = s / l[i * n + i];
        }
        for i in (0..n).rev() {
            let mut s = x[i * k + c];
            for p in i + 1..n {
                s -= l[p * n + i] * x[p * k + c];
            }
            x[i * k + c] = s / l[i * n + i];
        }
    }
    Ok(x)
}

/// A closed-form readout: `logits = w x + b` with `w` class-major (`k x d`), its selected ridge weight,
/// its fitted temperature and the loss it reached on the held-out selection.
struct RidgeReadout {
    k: usize,
    d: usize,
    w: Vec<f64>,
    b: Vec<f64>,
    lambda_rel: f64,
    lambda_abs: f64,
    temperature: f64,
    selection_n: usize,
    selection_bits: f64,
    lambda_table: Vec<(f64, f64)>,
}

impl RidgeReadout {
    fn logits_into(&self, x: &[f64], out: &mut [f64]) {
        for c in 0..self.k {
            let row = &self.w[c * self.d..(c + 1) * self.d];
            let mut acc = self.b[c];
            for (wv, xv) in row.iter().zip(x.iter()) {
                if *xv != 0.0 {
                    acc += wv * xv;
                }
            }
            out[c] = acc;
        }
    }

    fn nll_bits(&self, x: &[f64], y: &[usize], idx: &[usize], scratch: &mut [f64]) -> f64 {
        if idx.is_empty() {
            return f64::NAN;
        }
        let mut total = 0.0;
        for &i in idx {
            self.logits_into(&x[i * self.d..(i + 1) * self.d], scratch);
            let max = scratch.iter().fold(f64::NEG_INFINITY, |m, v| m.max(*v));
            let mut z = 0.0;
            for v in scratch.iter() {
                z += (*v - max).exp();
            }
            let p = ((scratch[y[i]] - max).exp() / z).max(f64::MIN_POSITIVE);
            total -= p.log2();
        }
        total / idx.len() as f64
    }

    fn top1(&self, x: &[f64], y: &[usize], idx: &[usize], scratch: &mut [f64]) -> f64 {
        if idx.is_empty() {
            return f64::NAN;
        }
        let mut correct = 0usize;
        for &i in idx {
            self.logits_into(&x[i * self.d..(i + 1) * self.d], scratch);
            let mut best = 0usize;
            for r in 1..self.k {
                if scratch[r] > scratch[best] {
                    best = r;
                }
            }
            if best == y[i] {
                correct += 1;
            }
        }
        correct as f64 / idx.len() as f64
    }

    /// Every logit of the readout over `idx`, row-major `idx.len() x k`, for the temperature search.
    fn logit_matrix(&self, x: &[f64], idx: &[usize]) -> Vec<f64> {
        let mut out = vec![0.0; idx.len() * self.k];
        for (j, &i) in idx.iter().enumerate() {
            self.logits_into(
                &x[i * self.d..(i + 1) * self.d],
                &mut out[j * self.k..(j + 1) * self.k],
            );
        }
        out
    }
}

/// Mean NLL in bits of stored logits after multiplying by `scale` (`1/T`), over the matrix rows in
/// `rows` (row `j` of the matrix belongs to position `idx[j]`).
fn nll_bits_at_scale(
    logits: &[f64],
    k: usize,
    y: &[usize],
    idx: &[usize],
    rows: &[usize],
    scale: f64,
    scratch: &mut [f64],
) -> f64 {
    if rows.is_empty() {
        return f64::NAN;
    }
    let mut total = 0.0;
    for &j in rows {
        let i = idx[j];
        let row = &logits[j * k..(j + 1) * k];
        let mut max = f64::NEG_INFINITY;
        for c in 0..k {
            let v = row[c] * scale;
            scratch[c] = v;
            if v > max {
                max = v;
            }
        }
        let mut z = 0.0;
        for v in scratch.iter() {
            z += (*v - max).exp();
        }
        let p = ((scratch[y[i]] - max).exp() / z).max(f64::MIN_POSITIVE);
        total -= p.log2();
    }
    total / rows.len() as f64
}

/// A declared capped sample of `0..n` row indices, every-`stride`.
fn declared_rows(n: usize, cap: usize) -> Vec<usize> {
    if n <= cap {
        return (0..n).collect();
    }
    let stride = n as f64 / cap as f64;
    (0..cap).map(|k| (k as f64 * stride) as usize).collect()
}

/// Fit one scalar temperature on stored logits: a declared coarse scan of `log2(T)` over the whole
/// declared range on a small row sample, then a golden-section refinement inside the best bracket on
/// the full temperature subset. Returns `(T, bits)`.
fn fit_temperature(
    logits: &[f64],
    k: usize,
    y: &[usize],
    idx: &[usize],
    scratch: &mut [f64],
) -> (f64, f64) {
    if idx.is_empty() {
        return (1.0, f64::NAN);
    }
    let coarse_rows = declared_rows(idx.len(), RIDGE_TEMP_COARSE_CAP);
    let all_rows = declared_rows(idx.len(), idx.len());
    let steps = 49usize;
    let mut best_u = 0.0;
    let mut best_bits = f64::INFINITY;
    for s in 0..=steps {
        let u = -TEMP_LOG2_RANGE + 2.0 * TEMP_LOG2_RANGE * s as f64 / steps as f64;
        let b = nll_bits_at_scale(logits, k, y, idx, &coarse_rows, (-u).exp2(), scratch);
        if b < best_bits {
            best_bits = b;
            best_u = u;
        }
    }
    // Refinement: golden section on `log2(T)` inside the coarse bracket, on the full subset.
    let w = 2.0 * TEMP_LOG2_RANGE / steps as f64;
    let mut lo = (best_u - w).max(-TEMP_LOG2_RANGE);
    let mut hi = (best_u + w).min(TEMP_LOG2_RANGE);
    let phi = 0.5 * (5.0f64.sqrt() - 1.0);
    let mut c = hi - phi * (hi - lo);
    let mut dd = lo + phi * (hi - lo);
    let mut fc = nll_bits_at_scale(logits, k, y, idx, &all_rows, (-c).exp2(), scratch);
    let mut fd = nll_bits_at_scale(logits, k, y, idx, &all_rows, (-dd).exp2(), scratch);
    for _ in 0..30 {
        if fc < fd {
            hi = dd;
            dd = c;
            fd = fc;
            c = hi - phi * (hi - lo);
            fc = nll_bits_at_scale(logits, k, y, idx, &all_rows, (-c).exp2(), scratch);
        } else {
            lo = c;
            c = dd;
            fc = fd;
            dd = lo + phi * (hi - lo);
            fd = nll_bits_at_scale(logits, k, y, idx, &all_rows, (-dd).exp2(), scratch);
        }
        if (hi - lo).abs() < 1e-6 {
            break;
        }
    }
    let u = 0.5 * (lo + hi);
    let bits = nll_bits_at_scale(logits, k, y, idx, &all_rows, (-u).exp2(), scratch);
    (u.exp2(), bits)
}

/// The declared temperature subset: a deterministic every-`stride` stride of `idx`, capped.
fn declared_subset(idx: &[usize], cap: usize) -> Vec<usize> {
    if idx.len() <= cap {
        return idx.to_vec();
    }
    let stride = idx.len() as f64 / cap as f64;
    (0..cap)
        .map(|k| idx[(k as f64 * stride) as usize])
        .collect()
}

/// Fit the closed-form readout for one label column set. The ridge weight is chosen by the declared
/// grid on the held-out part of the fit states only (never on dev); the final system is solved on
/// `refit_idx` at that weight; one temperature is fitted on a declared capped subset of the fit states.
#[allow(clippy::too_many_arguments)]
fn fit_ridge_readout(
    x: &[f64],
    y: &[usize],
    d: usize,
    k: usize,
    train_idx: &[usize],
    held_idx: &[usize],
    temp_idx: &[usize],
    refit_idx: &[usize],
) -> Result<RidgeReadout, String> {
    let mut train = Moments::zeros(d, k);
    for &i in train_idx {
        train.add(&x[i * d..(i + 1) * d], y[i]);
    }
    let trace = (0..d).map(|r| train.ht_h[r * d + r]).sum::<f64>() / d as f64;
    let mut best: Option<(f64, f64, Vec<f64>, Vec<f64>, f64)> = None;
    let mut lambda_table = Vec::with_capacity(RIDGE_LAMBDAS.len());
    for &lambda_rel in RIDGE_LAMBDAS.iter() {
        let lambda_abs = lambda_rel * trace;
        let (w, b) = solve_moments(&train, lambda_abs)?;
        let probe = RidgeReadout {
            k,
            d,
            lambda_rel,
            lambda_abs,
            temperature: 1.0,
            selection_n: held_idx.len(),
            selection_bits: f64::NAN,
            lambda_table: Vec::new(),
            w: transpose_to_class_major(&w, d, k),
            b,
        };
        let mut scratch = vec![0.0; k];
        let bits = probe.nll_bits(x, y, held_idx, &mut scratch);
        lambda_table.push((lambda_rel, bits));
        if best.as_ref().map(|v| bits < v.4).unwrap_or(true) {
            best = Some((
                lambda_rel,
                lambda_abs,
                probe.w.clone(),
                probe.b.clone(),
                bits,
            ));
        }
    }
    let (lambda_rel, lambda_abs, _, _, selection_bits) =
        best.ok_or_else(|| "no ridge candidate was evaluated".to_string())?;
    // The reported readout is refitted at the selected weight on `refit_idx`.
    let mut all = Moments::zeros(d, k);
    for &i in refit_idx {
        all.add(&x[i * d..(i + 1) * d], y[i]);
    }
    let (w, b) = solve_moments(&all, lambda_abs)?;
    let mut readout = RidgeReadout {
        k,
        d,
        w: transpose_to_class_major(&w, d, k),
        b,
        lambda_rel,
        lambda_abs,
        temperature: 1.0,
        selection_n: held_idx.len(),
        selection_bits,
        lambda_table,
    };
    // One scalar temperature, fitted on the declared capped subset of the fit states.
    let matrix = readout.logit_matrix(x, temp_idx);
    let mut scratch = vec![0.0; k];
    let (t, bits) = fit_temperature(&matrix, k, y, temp_idx, &mut scratch);
    // The fitted temperature is baked into the reported readout, so every loss below is on the
    // calibrated scale (the argmax, and therefore top-1, is unchanged by it).
    if t.is_finite() && t > 0.0 {
        for v in readout.w.iter_mut() {
            *v /= t;
        }
        for v in readout.b.iter_mut() {
            *v /= t;
        }
    }
    readout.temperature = t;
    readout.selection_bits = bits;
    Ok(readout)
}

/// Solve the centred ridge system: `(HtH - n x̄x̄ᵀ + λI) W = HtY - n x̄ȳᵀ`, with `b = ȳ - Wᵀx̄`.
/// Returns `(W, b)` with `W` feature-major (`d x k`).
fn solve_moments(m: &Moments, lambda_abs: f64) -> Result<(Vec<f64>, Vec<f64>), String> {
    let (d, k) = (m.d, m.k);
    let xbar: Vec<f64> = m.sx.iter().map(|v| v / m.n).collect();
    let ybar: Vec<f64> = m.sy.iter().map(|v| v / m.n).collect();
    let mut a = vec![0.0; d * d];
    for r in 0..d {
        for c in 0..d {
            let h = if r <= c {
                m.ht_h[r * d + c]
            } else {
                m.ht_h[c * d + r]
            };
            a[r * d + c] = h - m.n * xbar[r] * xbar[c] + if r == c { lambda_abs } else { 0.0 };
        }
    }
    let mut b = vec![0.0; d * k];
    for r in 0..d {
        for c in 0..k {
            b[r * k + c] = m.ht_y[r * k + c] - m.n * xbar[r] * ybar[c];
        }
    }
    let w = cholesky_solve(&a, d, &b, k)?;
    let mut bias = vec![0.0; k];
    for c in 0..k {
        let mut acc = 0.0;
        for r in 0..d {
            acc += w[r * k + c] * xbar[r];
        }
        bias[c] = ybar[c] - acc;
    }
    Ok((w, bias))
}

fn transpose_to_class_major(w: &[f64], d: usize, k: usize) -> Vec<f64> {
    let mut out = vec![0.0; d * k];
    for r in 0..d {
        for c in 0..k {
            out[c * d + r] = w[r * k + c];
        }
    }
    out
}

/// The servable alphabets: the artifact's ternary readout scheme and its signed 4-bit embedding scheme.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CodeAlphabet {
    Ternary,
    FourBit,
}

impl CodeAlphabet {
    fn name(self) -> &'static str {
        match self {
            CodeAlphabet::Ternary => "ternary",
            CodeAlphabet::FourBit => "four_bit",
        }
    }
    /// Bits of information per stored weight.
    fn bits_per_entry(self) -> usize {
        match self {
            CodeAlphabet::Ternary => 2,
            CodeAlphabet::FourBit => 4,
        }
    }
    fn bound(self) -> i32 {
        match self {
            CodeAlphabet::Ternary => 1,
            CodeAlphabet::FourBit => TL_EMB_BOUND,
        }
    }
    fn max_shift(self) -> u32 {
        match self {
            CodeAlphabet::Ternary => TL_MAX_SHIFT,
            CodeAlphabet::FourBit => TL_EMB_MAX_SHIFT,
        }
    }

    /// The artifact's own per-row scale rule, reimplemented because `quantize_ternary` and
    /// `TlEmbed::quantize` are private: the largest power-of-two scale that does not clip the row,
    /// capped at the scheme's declared shift bound.
    fn rule_shift(self, row: &[f64]) -> u32 {
        let amax = row.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        match self {
            CodeAlphabet::Ternary => {
                if amax > 0.0 {
                    (amax.log2().floor().max(0.0) as u32).min(self.max_shift())
                } else {
                    0
                }
            }
            CodeAlphabet::FourBit => {
                let bound = self.bound() as f64;
                if amax > bound {
                    ((amax / bound).log2().floor().max(0.0) as u32).min(self.max_shift())
                } else {
                    0
                }
            }
        }
    }

    fn quantize_row(self, row: &[f64], shift: u32) -> Vec<i8> {
        let scale = (1u32 << shift) as f64;
        let bound = self.bound() as f64;
        row.iter()
            .map(|v| (v / scale).round().clamp(-bound, bound) as i8)
            .collect()
    }
}

/// The declared grid of power-of-two pre-scales the repricing searches before quantisation.
const PRE_SCALE_LOG2_RANGE: std::ops::RangeInclusive<i32> = -24..=20;

/// Choose the global pre-scale a servable build would use, by the alphabet's own reconstruction error.
///
/// For every power of two in the declared range the rows are quantised at the scheme's rule (and, when
/// fitted, at their best per-row offset) and the relative error `Σ (w λ - q 2^(s+o))² / Σ (w λ)²` is
/// scored; the smallest wins. This is scale-*variant* in exactly the way that matters: the shift rule's
/// clamp at zero, the shift cap and code saturation all show up in it.
fn choose_pre_scale(
    w_int: &[f64],
    k: usize,
    d: usize,
    alphabet: CodeAlphabet,
    fitted_offsets: bool,
) -> (f64, i32, f64, f64) {
    let mut best = (1.0f64, 0i32, f64::INFINITY, 0.0f64);
    for log2_scale in PRE_SCALE_LOG2_RANGE {
        let scale = (2.0f64).powi(log2_scale);
        let mut err = 0.0f64;
        let mut power = 0.0f64;
        let mut nonzero = 0usize;
        let mut entries = 0usize;
        for c in 0..k {
            let row: Vec<f64> = w_int[c * d..(c + 1) * d]
                .iter()
                .map(|v| v * scale)
                .collect();
            let base = alphabet.rule_shift(&row);
            let mut best_row = (f64::INFINITY, base);
            for (oi, off) in REPRICE_OFFSETS.iter().enumerate() {
                if !fitted_offsets && oi > 0 {
                    break;
                }
                let s = (base as i32 + off).max(0) as u32;
                let q = alphabet.quantize_row(&row, s);
                let e: f64 = row
                    .iter()
                    .zip(q.iter())
                    .map(|(w, code)| {
                        let r = w - *code as f64 * (1u32 << s) as f64;
                        r * r
                    })
                    .sum();
                if e < best_row.0 {
                    best_row = (e, s);
                }
            }
            err += best_row.0;
            for v in row.iter() {
                power += v * v;
            }
            let q = alphabet.quantize_row(&row, best_row.1);
            nonzero += q.iter().filter(|c| **c != 0).count();
            entries += q.len();
        }
        let rel = if power > 0.0 {
            err / power
        } else {
            f64::INFINITY
        };
        if rel < best.2 {
            best = (
                scale,
                log2_scale,
                rel,
                nonzero as f64 / entries.max(1) as f64,
            );
        }
    }
    best
}

/// The artifact's 2-bits-per-weight packing, reimplemented from its documented rule (four weights per
/// byte, lowest slot first, `+1 = 1`, `-1 = 2`, `0 = 0`) because `pack_ternary` is private. Checked
/// against the artifact's own `TlLinear::forward_i32` before use (see `verify_ternary_packing`).
fn pack_ternary_codes(codes: &[i8]) -> Vec<u8> {
    let mut packed = vec![0u8; codes.len().div_ceil(4)];
    for (i, c) in codes.iter().enumerate() {
        let code: u8 = match *c {
            1 => 1,
            -1 => 2,
            _ => 0,
        };
        packed[i >> 2] |= code << ((i & 3) as u32 * 2);
    }
    packed
}

/// Read the packed codes back through the artifact's own integer kernel and compare against a direct sum
/// of the codes this module intended to store, row by row over a sample. This is the control that the
/// reimplemented packing is the artifact's packing.
fn verify_ternary_packing(
    linear: &TlLinear,
    codes: &[i8],
    d: usize,
    x: &[i32],
) -> Result<(), String> {
    let got = linear.forward_i32(x)?;
    for r in 0..linear.rows {
        let mut direct: i32 = 0;
        for c in 0..d {
            direct += codes[r * d + c] as i32 * x[c];
        }
        let expect = direct << linear.shift[r];
        if got[r] != expect {
            return Err(format!(
                "reimplemented ternary packing disagrees with the artifact kernel at row {r}: \
                 {} vs {expect}",
                got[r]
            ));
        }
    }
    Ok(())
}

/// One servable repricing of a float readout: integer codes with one power-of-two scale per row, an
/// integer bias in the served dyadic units, and the dyadic score scale that was fitted.
struct Repricing {
    alphabet: CodeAlphabet,
    fitted_offsets: bool,
    k: usize,
    d: usize,
    codes: Vec<i8>,
    shift: Vec<u32>,
    offset_histogram: Vec<usize>,
    /// The dyadic score scale chosen on the fit population, and the fit/dev NLL in bits at it.
    score_shift: u32,
    dyadic_fit_bits: f64,
    dev_bits: f64,
    /// The alphabet's ceiling: one free (non-dyadic) logit scale on the same integer logits, its fit and
    /// dev NLL, and the fractional `score_shift` that scale would imply.
    free_scale: f64,
    free_scale_fit_bits: f64,
    implied_free_score_shift: f64,
    dev_bits_free_scale: f64,
    /// Integers of the served path: the widest bias, the widest row accumulator and its D0-b check.
    max_abs_bias: i64,
    max_row_bound: i128,
    envelope_ok: bool,
    /// Observed integer-logit range on dev, before the dyadic scale.
    dev_logit_min: i64,
    dev_logit_max: i64,
    /// The float readout's own dev NLL on the same population, for the gain that is not realised.
    float_dev_bits: f64,
    /// The fitted global pre-scale of the readout before quantisation, its power of two, the relative
    /// reconstruction error the alphabet reaches at it, and the fraction of nonzero codes.
    pre_scale: f64,
    pre_scale_log2: i32,
    pre_scale_relative_error: f64,
    code_density: f64,
}

/// Reprice one float readout into one servable alphabet and rescore it in integer arithmetic.
///
/// `score_shift` is the artifact's declared dyadic score scale (`TlConfig::score_shift`): the served
/// score is `(integer logit) * 2^-score_shift`, so a fitted temperature *is* a servable parameter here,
/// and the integer search runs over the artifact's own declared range `0..=SS_MAX`. Nothing here writes
/// to or modifies any sealed artifact: the repriced readout lives only in this attempt's own report.
#[allow(clippy::too_many_arguments)]
fn reprice(
    readout: &RidgeReadout,
    float_dev_bits: f64,
    h_dim: usize,
    h_clamp: i32,
    alphabet: CodeAlphabet,
    fitted_offsets: bool,
    fit_u: &[i32],
    fit_y: &[usize],
    dev_u: &[i32],
    dev_y: &[usize],
    fit_idx: &[usize],
    dev_idx: &[usize],
) -> Result<Repricing, String> {
    let (k, d) = (readout.k, readout.d);
    // The integer path sees the raw clipped state and the event one-hot, so the float readout's
    // `h / h_clamp` scaling is folded into the weights of the state block.
    let scale = 1.0 / h_clamp as f64;
    let mut w_int = readout.w.clone();
    for c in 0..k {
        for r in 0..h_dim {
            w_int[c * d + r] *= scale;
        }
    }
    // (i) global pre-scale. The artifact's per-row rule never scales *down* (`s = floor(log2 amax)`
    // clamped at 0), so a readout whose rows are narrower than one unit of the served input quantises to
    // all zeros -- the alphabet would be reported as useless when the readout merely sits at a different
    // scale. A real artifact build sets that scale once, so one is fitted here: the power of two whose
    // quantised rows have the smallest relative reconstruction error, with the fitted dyadic score
    // scale absorbing it (see `score_shift`), and the integer bias built in the same units.
    let (pre_scale, pre_scale_log2, pre_scale_error, code_density) =
        choose_pre_scale(&w_int, k, d, alphabet, fitted_offsets);
    for v in w_int.iter_mut() {
        *v *= pre_scale;
    }
    let b_scaled: Vec<f64> = readout.b.iter().map(|v| v * pre_scale).collect();
    let mut codes = vec![0i8; k * d];
    let mut shift = vec![0u32; k];
    let mut offset_histogram = vec![0usize; REPRICE_OFFSETS.len()];
    for c in 0..k {
        let row = &w_int[c * d..(c + 1) * d];
        let base = alphabet.rule_shift(row);
        let mut best = (base, 0usize);
        let mut best_score = f64::NEG_INFINITY;
        for (oi, &off) in REPRICE_OFFSETS.iter().enumerate() {
            if !fitted_offsets && oi > 0 {
                break;
            }
            let s = (base as i32 + off).max(0) as u32;
            let q = alphabet.quantize_row(row, s);
            // Declared per-row policy: the offset whose quantised row is best aligned with the float
            // row, scale-free, `sum_c q_c w_c / ||q||`; `o = 0` is the scheme's own rule.
            let dot: f64 = q.iter().zip(row.iter()).map(|(a, b)| *a as f64 * b).sum();
            let norm = q
                .iter()
                .map(|a| (*a as f64) * (*a as f64))
                .sum::<f64>()
                .sqrt();
            let score = if norm > 0.0 {
                dot / norm
            } else {
                f64::NEG_INFINITY
            };
            if score > best_score {
                best_score = score;
                best = (s, oi);
            }
        }
        shift[c] = best.0;
        offset_histogram[best.1] += 1;
        let q = alphabet.quantize_row(row, best.0);
        codes[c * d..(c + 1) * d].copy_from_slice(&q);
    }
    // Integer scores of the fit and dev populations. The ternary path runs through the artifact's own
    // `TlLinear::forward_i32`; the 4-bit path accumulates `TlEmbed::value` the way that table stores it.
    let (fit_z, dev_z): (Vec<i64>, Vec<i64>) = match alphabet {
        CodeAlphabet::Ternary => {
            let linear = TlLinear {
                rows: k,
                cols: d,
                packed: pack_ternary_codes(&codes),
                shift: shift.clone(),
            };
            let probe_x: Vec<i32> = (0..d).map(|c| ((c % 7) as i32) - 3).collect();
            verify_ternary_packing(&linear, &codes, d, &probe_x)?;
            let mut fit_z: Vec<i64> = Vec::with_capacity(fit_idx.len() * k);
            for (j, _) in fit_idx.iter().enumerate() {
                fit_z.extend(
                    linear
                        .forward_i32(&fit_u[j * d..(j + 1) * d])?
                        .into_iter()
                        .map(i64::from),
                );
            }
            let mut dev_z: Vec<i64> = Vec::with_capacity(dev_idx.len() * k);
            for (j, _) in dev_idx.iter().enumerate() {
                dev_z.extend(
                    linear
                        .forward_i32(&dev_u[j * d..(j + 1) * d])?
                        .into_iter()
                        .map(i64::from),
                );
            }
            (fit_z, dev_z)
        }
        CodeAlphabet::FourBit => {
            let table = TlEmbed {
                rows: k,
                cols: d,
                codes: codes.clone(),
                shift: shift.clone(),
            };
            let score = |u: &[i32], out: &mut Vec<i64>| {
                for r in 0..k {
                    let mut acc: i64 = 0;
                    for c in 0..d {
                        acc += table.value(r, c) as i64 * u[c] as i64;
                    }
                    out.push(acc);
                }
            };
            let (mut fit_z, mut dev_z) = (Vec::new(), Vec::new());
            for j in 0..fit_idx.len() {
                score(&fit_u[j * d..(j + 1) * d], &mut fit_z);
            }
            for j in 0..dev_idx.len() {
                score(&dev_u[j * d..(j + 1) * d], &mut dev_z);
            }
            (fit_z, dev_z)
        }
    };
    // Fit the dyadic score scale over the artifact's declared range on the fit population, and one free
    // (non-dyadic) temperature as the alphabet's ceiling.
    let mut best_ss = (0u32, f64::INFINITY);
    let mut table = Vec::with_capacity(SS_MAX as usize + 1);
    for ss in 0..=SS_MAX {
        let bits = nll_bits_int(&fit_z, k, fit_y, fit_idx, &b_scaled, ss);
        table.push((ss, bits));
        if bits < best_ss.1 {
            best_ss = (ss, bits);
        }
    }
    let free_scale = fit_free_scale(&fit_z, k, fit_y, fit_idx, &b_scaled);
    let dev_bits = nll_bits_int(&dev_z, k, dev_y, dev_idx, &b_scaled, best_ss.0);
    let dev_bits_free = free_scale_shift(free_scale.0, &dev_z, k, dev_y, dev_idx, &b_scaled);
    let (mut lo, mut hi) = (i64::MAX, i64::MIN);
    for (j, &i) in dev_idx.iter().enumerate() {
        let _ = i;
        for r in 0..k {
            let v = dev_z[j * k + r] + (b_scaled[r] * (1i64 << best_ss.0) as f64).round() as i64;
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    let bound = max_row_bound(&codes, &shift, d, k, h_clamp);
    Ok(Repricing {
        alphabet,
        fitted_offsets,
        k,
        d,
        codes,
        shift,
        offset_histogram,
        score_shift: best_ss.0,
        dyadic_fit_bits: best_ss.1,
        dev_bits,
        free_scale: free_scale.0,
        free_scale_fit_bits: free_scale.1,
        implied_free_score_shift: -(free_scale.0).log2(),
        dev_bits_free_scale: dev_bits_free,
        max_abs_bias: b_scaled.iter().fold(0i64, |m, v| {
            m.max((v.abs() * (1i64 << best_ss.0) as f64).round() as i64)
        }),
        max_row_bound: bound,
        envelope_ok: bound <= (i32::MAX / 4) as i128,
        dev_logit_min: lo,
        dev_logit_max: hi,
        float_dev_bits,
        pre_scale,
        pre_scale_log2,
        pre_scale_relative_error: pre_scale_error,
        code_density,
    })
}

/// The dev NLL of the same integer logits at a free (non-dyadic) scale: `2^-log2(T)` written as a
/// fractional dyadic exponent, which is why it is a ceiling and not a servable configuration.
fn free_scale_shift(
    scale: f64,
    z: &[i64],
    k: usize,
    y: &[usize],
    idx: &[usize],
    bias: &[f64],
) -> f64 {
    // `nll_bits_int` takes an integral `score_shift`; for the free scale we rescore directly here.
    let mut total = 0.0;
    let mut vals = vec![0.0f64; k];
    for (j, &i) in idx.iter().enumerate() {
        let row = &z[j * k..(j + 1) * k];
        let mut max = f64::NEG_INFINITY;
        for r in 0..k {
            let v = (row[r] as f64 + bias[r]) * scale;
            vals[r] = v;
            if v > max {
                max = v;
            }
        }
        let mut denom = 0.0;
        for v in vals.iter() {
            denom += (*v - max).exp();
        }
        let p = ((vals[y[i]] - max).exp() / denom).max(f64::MIN_POSITIVE);
        total -= p.log2();
    }
    if idx.is_empty() {
        f64::NAN
    } else {
        total / idx.len() as f64
    }
}

/// The declared D0-b envelope of one repriced map: the widest row accumulator, `(Σ_c |code| * input
/// bound) << shift`, with the state block bounded by `h_clamp` and the event block by 1.
fn max_row_bound(codes: &[i8], shift: &[u32], d: usize, k: usize, h_clamp: i32) -> i128 {
    let h_dim = d.saturating_sub(PROBE_EVENT_DIMS);
    let mut worst: i128 = 0;
    for r in 0..k {
        let mut sum: i128 = 0;
        for c in 0..d {
            let bound = if c < h_dim { h_clamp as i128 } else { 1 };
            sum += (codes[r * d + c] as i128).abs() * bound;
        }
        let shifted = sum << shift[r].min(31);
        worst = worst.max(shifted);
    }
    worst
}

/// NLL in bits of integer logits plus an integer bias, at the declared dyadic scale `2^-score_shift`.
fn nll_bits_int(
    z: &[i64],
    k: usize,
    y: &[usize],
    idx: &[usize],
    bias: &[f64],
    score_shift: u32,
) -> f64 {
    if idx.is_empty() {
        return f64::NAN;
    }
    let scale = (-(score_shift as f64)).exp2();
    let mut total = 0.0;
    for (j, &i) in idx.iter().enumerate() {
        let row = &z[j * k..(j + 1) * k];
        let mut max = f64::NEG_INFINITY;
        let mut best = Vec::with_capacity(k);
        for r in 0..k {
            let v = (row[r] as f64 + bias[r]) * scale;
            best.push(v);
            if v > max {
                max = v;
            }
        }
        let target = (row[y[i]] as f64 + bias[y[i]]) * scale;
        let mut denom = 0.0;
        for v in best.iter() {
            denom += (*v - max).exp();
        }
        let p = ((target - max).exp() / denom).max(f64::MIN_POSITIVE);
        total -= p.log2();
    }
    total / idx.len() as f64
}

/// One free (non-dyadic) logit scale for integer logits, fitted by the declared golden-section search on
/// `log2 ` of the scale, as the ceiling the servable dyadic scale is compared against. Returns the
/// multiplier applied to the integer logits (so the served logit is `(z + bo) * multiplier`) and the fit
/// NLL in bits at it.
fn fit_free_scale(z: &[i64], k: usize, y: &[usize], idx: &[usize], bias: &[f64]) -> (f64, f64) {
    let mut vals = vec![0.0f64; k];
    let bits_at = |u: f64, vals: &mut [f64]| -> f64 {
        let scale = (-u).exp2();
        let mut total = 0.0;
        for (j, &i) in idx.iter().enumerate() {
            let row = &z[j * k..(j + 1) * k];
            let mut max = f64::NEG_INFINITY;
            for r in 0..k {
                let v = (row[r] as f64 + bias[r]) * scale;
                vals[r] = v;
                if v > max {
                    max = v;
                }
            }
            let mut denom = 0.0;
            for v in vals.iter() {
                denom += (*v - max).exp();
            }
            let p = ((vals[y[i]] - max).exp() / denom).max(f64::MIN_POSITIVE);
            total -= p.log2();
        }
        total / idx.len() as f64
    };
    let mut lo = -TEMP_LOG2_RANGE;
    let mut hi = TEMP_LOG2_RANGE;
    let phi = 0.5 * (5.0f64.sqrt() - 1.0);
    let mut c = hi - phi * (hi - lo);
    let mut dd = lo + phi * (hi - lo);
    let mut fc = bits_at(c, &mut vals);
    let mut fd = bits_at(dd, &mut vals);
    for _ in 0..24 {
        if fc < fd {
            hi = dd;
            dd = c;
            fd = fc;
            c = hi - phi * (hi - lo);
            fc = bits_at(c, &mut vals);
        } else {
            lo = c;
            c = dd;
            fc = fd;
            dd = lo + phi * (hi - lo);
            fd = bits_at(dd, &mut vals);
        }
        if (hi - lo).abs() < 1e-4 {
            break;
        }
    }
    let u = 0.5 * (lo + hi);
    let scale = (-u).exp2();
    (scale, bits_at(u, &mut vals))
}

/// The state-side decode ladder: what the recorded `h` alone decodes to, by nearest embedding row and
/// by a closed-form ridge readout, for the two tokens that were consumed before it and for the token
/// the decision then generates.
#[derive(Default, Clone, Copy)]
struct DecodeScore {
    n: usize,
    l2_top1: f64,
    cos_top1: f64,
    /// The accuracy of a constant predictor on this population: the frequency of its commonest class.
    majority: f64,
    /// The population's collision probability `Σ p²`, the accuracy a predictor that ignores the state
    /// entirely reaches when it guesses from the target distribution.
    collision: f64,
    /// The accuracy of always predicting the vocabulary row of smallest `||E||`: the confound the raw
    /// L2 decode has to beat, because `||h - E||² = ||h||² - 2h·E + ||E||²` favours short rows.
    min_norm_row: f64,
}

impl DecodeScore {
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "positions": self.n,
            "nearest_l2_top1": self.l2_top1,
            "nearest_cosine_top1": self.cos_top1,
            "smallest_norm_row_top1": self.min_norm_row,
            "majority_class_frequency": self.majority,
            "collision_probability_sum_p_squared": self.collision,
        })
    }
}

/// Nearest-row and cosine decode accuracy of one truth vector over `idx` (indices into the state list).
fn decode_accuracy(
    winners: &[u32],
    decode: &EmbeddingDecode,
    truth: &[u32],
    idx: &[usize],
    class_of: &[usize],
) -> DecodeScore {
    let k = class_of.len();
    let mut counts = vec![0usize; k];
    let mut score = DecodeScore {
        n: idx.len(),
        ..DecodeScore::default()
    };
    if idx.is_empty() {
        return score;
    }
    let mut l2 = 0usize;
    let mut cos = 0usize;
    let mut min_norm = 0usize;
    for (j, &i) in idx.iter().enumerate() {
        let truth_id = truth[i];
        if winners[2 * j] == truth_id {
            l2 += 1;
        }
        if winners[2 * j + 1] == truth_id {
            cos += 1;
        }
        if decode.min_norm_row == truth_id {
            min_norm += 1;
        }
        let class = class_of.get(truth_id as usize).copied().unwrap_or(k - 1);
        counts[class.min(k - 1)] += 1;
    }
    let total = idx.len() as f64;
    score.l2_top1 = l2 as f64 / total;
    score.cos_top1 = cos as f64 / total;
    score.min_norm_row = min_norm as f64 / total;
    score.majority = counts.iter().max().copied().unwrap_or(0) as f64 / total;
    score.collision = counts
        .iter()
        .map(|c| (*c as f64 / total).powi(2))
        .sum::<f64>();
    score
}

/// Decode every state's consumed-token history by nearest embedding row: argmin `||h - E[v]||` and
/// argmax cosine, over the whole vocabulary, using `h` only.
///
/// `E[v][c] = code << shift[v]`, so both decodes come from one pass: with `||E||` precomputed,
/// `||h - E||² = ||h||² - 2 h·E + ||E||²`. The smallest-norm row is recorded because a raw L2 decode is
/// biased towards it, which is why the caller reports `min_norm_row` accuracy alongside.
struct EmbeddingDecode {
    /// Per scanned state, in the order of the scanned index list: `[l2 winner, cosine winner]`.
    winners: Vec<u32>,
    min_norm_row: u32,
    enorm2: Vec<i64>,
    enorm: Vec<f64>,
}

fn embedding_decode(model: &TlModel, states: &[ServedState], idx: &[usize]) -> EmbeddingDecode {
    let d = model.h_dim;
    let rows = model.e.rows;
    let mut enorm2 = vec![0i64; rows];
    let mut enorm = vec![0f64; rows];
    for v in 0..rows {
        let mut acc: i64 = 0;
        for c in 0..d {
            let e = model.e.value(v, c) as i64;
            acc += e * e;
        }
        enorm2[v] = acc;
        enorm[v] = (acc as f64).sqrt();
    }
    let min_norm_row = (0..rows)
        .min_by(|a, b| enorm2[*a].cmp(&enorm2[*b]))
        .unwrap_or(0) as u32;
    let mut winners = Vec::with_capacity(2 * idx.len());
    for &i in idx {
        let s = &states[i];
        let mut hn2: i64 = 0;
        for c in 0..d {
            let v = s.h[c] as i64;
            hn2 += v * v;
        }
        let hnorm = (hn2 as f64).sqrt();
        let mut best_l2: i64 = i64::MAX;
        let mut best_cos = f64::NEG_INFINITY;
        let mut win_l2 = 0u32;
        let mut win_cos = 0u32;
        for v in 0..rows {
            let shift = model.e.shift[v];
            let mut dot: i64 = 0;
            for c in 0..d {
                dot += model.e.codes[v * d + c] as i64 * s.h[c] as i64;
            }
            let dot = dot << shift.min(20);
            let l2 = hn2 - 2 * dot + enorm2[v];
            if l2 < best_l2 {
                best_l2 = l2;
                win_l2 = v as u32;
            }
            let denom = hnorm * enorm[v];
            let cos = if denom > 0.0 { dot as f64 / denom } else { 0.0 };
            if cos > best_cos {
                best_cos = cos;
                win_cos = v as u32;
            }
        }
        winners.push(win_l2);
        winners.push(win_cos);
    }
    EmbeddingDecode {
        winners,
        min_norm_row,
        enorm2,
        enorm,
    }
}

fn state_probe_mode(args: &Args) -> Result<ExitCode, String> {
    let root = args.state_probe.clone().unwrap_or_default();
    if root.as_os_str().is_empty() {
        return Err("--state-probe needs a new report root".into());
    }
    if args.docs.as_os_str().is_empty() {
        return Err("--docs is required with --state-probe".into());
    }
    let artifact_path = match args.artifact.clone() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => {
            return Err("--artifact (path to a .tlx file) is required with --state-probe".into());
        }
    };
    // Claimed exclusively before any model work: an existing report is never reused or overwritten.
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    println!(
        "=== ordinary-lexical state probe: a float readout on an artifact's own recorded served states ==="
    );

    // ---- tokenizer identity ------------------------------------------------
    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&Sha256::digest(&tok_bytes));
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer sha256 {tok_sha} != expected {EXPECTED_TOKENIZER_SHA256}"
        ));
    }
    let derived_bytes =
        uor_r4_core::transformerless::bpe_derive::derive_tokenizer_json(&tok_bytes, VOCAB)
            .map_err(|e| format!("derive bytes: {e}"))?;
    let derived_sha = sha256_hex(&Sha256::digest(&derived_bytes));
    let tokenizer = derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive: {e}"))?;
    write_checked(&root, "tokenizer_source.json", &tok_bytes)?;
    write_checked(&root, "tokenizer_derived_v4096.json", &derived_bytes)?;

    // ---- the artifact under test: read in place, never copied, never re-sealed ----
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let exe_bytes = std::fs::read(&exe).map_err(|e| format!("read exe: {e}"))?;
    let exe_sha = sha256_hex(&Sha256::digest(&exe_bytes));
    let artifact_bytes = std::fs::read(&artifact_path)
        .map_err(|e| format!("artifact {}: {e}", artifact_path.display()))?;
    let artifact_sha = sha256_hex(&Sha256::digest(&artifact_bytes));
    let model = TlModel::from_bytes(&artifact_bytes).map_err(|e| format!("deserialize: {e}"))?;
    if model.vocab != VOCAB {
        return Err(format!(
            "artifact vocabulary {} != the {VOCAB}-token corpus vocabulary; the probe's class ids and \
             the token references would not be comparable",
            model.vocab
        ));
    }

    // ---- corpus, source-separated; the split, windows and count references of run() ----
    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
    let mut fit_docs = Vec::new();
    let mut tune_docs = Vec::new();
    let mut dev_pool = Vec::new();
    for (i, d) in uniq.iter().enumerate() {
        match d.split {
            Split::Fit => fit_docs.push(i),
            Split::Tune => tune_docs.push(i),
            Split::Dev => dev_pool.push(i),
        }
    }
    if fit_docs.is_empty() || dev_pool.is_empty() {
        return Err("corpus split has an empty fit or development side".into());
    }
    let mut fit_windows: Vec<ProseWindow> = Vec::new();
    for i in &fit_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), *i) {
            fit_windows.push(w);
        }
    }
    let all_fit_windows = fit_windows.len();
    if fit_windows.len() > FIT_MAX_WINDOWS {
        let stride = fit_windows.len() as f64 / FIT_MAX_WINDOWS as f64;
        fit_windows = (0..FIT_MAX_WINDOWS)
            .map(|k| {
                let idx = (k as f64 * stride) as usize;
                fit_windows[idx].clone()
            })
            .collect();
    }
    let mut with_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    with_len.sort_by_key(|(_, n)| *n);
    let take = DEV_MAX_DOCS.min(with_len.len());
    let mut dev_windows: Vec<ProseWindow> = Vec::new();
    let mut dev_names: Vec<String> = Vec::new();
    for k in 0..take {
        let (doc_idx, _) = with_len[k * with_len.len() / take.max(1)];
        dev_names.push(uniq[doc_idx].path.clone());
        for w in prose_windows(&tokenizer.encode(&uniq[doc_idx].text), k)
            .into_iter()
            .take(DEV_WINDOWS_PER_DOC)
        {
            dev_windows.push(w);
        }
    }
    let mut tune_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_docs {
        for w in prose_windows(&tokenizer.encode(&uniq[*i].text), 0) {
            tune_windows.push(w.tokens);
        }
    }
    // Every reference below indexes the fixed-vocabulary tables directly, so a token id outside the
    // declared vocabulary is an input-integrity failure rather than something to clamp silently.
    for w in fit_windows.iter().chain(dev_windows.iter()) {
        if w.tokens.iter().any(|t| *t as usize >= VOCAB) {
            return Err(format!(
                "corpus token id outside the {VOCAB}-token vocabulary; the token references are \
                 undefined on it"
            ));
        }
    }
    let mut uni = Uni {
        counts: vec![0u64; VOCAB],
        total: 0,
    };
    let mut c1 = Cond::default();
    let mut c2 = Cond::default();
    for w in &fit_windows {
        for k in PREFIX..w.tokens.len() {
            let (prev, cur, next) = prose_position(w, k);
            uni.counts[next as usize] += 1;
            uni.total += 1;
            c1.observe(cur as u64, next);
            c2.observe(ctx2(prev, cur), next);
        }
    }
    let (lambdas, tune_bits) = tune_lambdas(&c1, &c2, &uni, &tune_windows);

    // ---- restricted target population: the top-k fit-position targets ----
    let mut freq = vec![0u64; VOCAB];
    for w in &fit_windows {
        for k in PREFIX..w.tokens.len() {
            freq[w.tokens[k] as usize] += 1;
        }
    }
    let mut ranked: Vec<u32> = (0..VOCAB as u32).collect();
    ranked.sort_by(|a, b| {
        freq[*b as usize]
            .cmp(&freq[*a as usize])
            .then_with(|| a.cmp(b))
    });
    let k_used = args.top_k.min(VOCAB);
    let top_ids: Vec<u32> = ranked[..k_used].to_vec();
    let mut in_top = vec![false; VOCAB];
    let mut class_of = vec![usize::MAX; VOCAB];
    for (class, t) in top_ids.iter().enumerate() {
        in_top[*t as usize] = true;
        class_of[*t as usize] = class;
    }
    let top_mass: f64 = top_ids.iter().map(|t| uni.p(*t)).sum();
    if !(top_mass > 0.0) {
        return Err("the restricted unigram mass is zero; the null reference is undefined".into());
    }

    // ---- record the served states of both splits ----
    let t_rec = Instant::now();
    let fit_states = record_served_states(&model, &fit_windows);
    let dev_states = record_served_states(&model, &dev_windows);
    let rec_secs = t_rec.elapsed().as_secs_f64();
    let fit_states_sha = recorded_states_sha256(&fit_states);
    let dev_states_sha = recorded_states_sha256(&dev_states);

    // The recording control: the sum of the recorded per-position bits must equal `score_example`'s
    // own `bits_generate` over the same windows, which ties the recorded states to the served walk.
    // The control is a second full readout pass, so it runs on **every** dev window (the decision
    // population) and on a declared sample of the fit windows: every `FIT_CONTROL_STRIDE`-th window
    // plus every terminal window, so both window shapes are covered. `stride = 0` means every window.
    let control = |windows: &[ProseWindow],
                   states: &[ServedState],
                   stride: usize|
     -> (f64, f64, usize, usize, usize) {
        let mut sx_bits = 0.0;
        let mut rec_bits = 0.0;
        let mut n = 0usize;
        let mut states_seen = 0usize;
        let mut checked_windows = 0usize;
        let mut cursor = 0usize;
        for (i, w) in windows.iter().enumerate() {
            let here = w.tokens.len() - PREFIX;
            let selected = stride == 0 || i % stride == 0 || w.terminal;
            if selected {
                let s = model.score_example(&prose_example(w));
                sx_bits += s.bits_generate;
                n += s.generate_targets;
                for j in 0..here {
                    rec_bits += states[cursor + j].bits;
                }
                states_seen += here;
                checked_windows += 1;
            }
            cursor += here;
        }
        (sx_bits, rec_bits, n, states_seen, checked_windows)
    };
    let (fit_sx_bits, fit_rec_bits, fit_sx_n, fit_control_states, fit_control_windows) =
        control(&fit_windows, &fit_states, FIT_CONTROL_STRIDE);
    let (dev_sx_bits, dev_rec_bits, dev_sx_n, dev_control_states, dev_control_windows) =
        control(&dev_windows, &dev_states, 0);
    let mismatch = |a: f64, b: f64| (a - b).abs() > 1e-9 * (1.0 + a.abs());
    if mismatch(fit_sx_bits, fit_rec_bits) || mismatch(dev_sx_bits, dev_rec_bits) {
        return Err(format!(
            "served-state recording control failed: recorded bit sums {fit_rec_bits} / {dev_rec_bits} \
             vs score_example {fit_sx_bits} / {dev_sx_bits}"
        ));
    }
    if fit_sx_n != fit_control_states || dev_sx_n != dev_control_states {
        return Err(format!(
            "served-state recording control failed: {fit_sx_n}/{dev_sx_n} scored Generate targets vs \
             {fit_control_states}/{dev_control_states} recorded states in the checked windows"
        ));
    }
    if dev_control_states != dev_states.len() {
        return Err("the development served-bit control must cover every recorded state".into());
    }
    let fit_served = served_reference(&fit_states, &in_top);
    let dev_served = served_reference(&dev_states, &in_top);

    // ---- control: the consumed-token bookkeeping is the window's own (prev, cur) ----
    // For every window the recorded states are the `Generate` actions in walk order, so state `j` of a
    // window is position `k = PREFIX + j` and must carry `cur = tokens[k-1]` and `prev = tokens[k-2]`,
    // which is exactly the pair the tuned count reference conditions on. Any off-by-one in the
    // recorder's consumption history fails here, on every position of both splits.
    let cur_prev_control =
        |windows: &[ProseWindow], states: &[ServedState]| -> Result<usize, String> {
            let mut cursor = 0usize;
            for w in windows {
                let generates = w.tokens.len() - PREFIX;
                for j in 0..generates {
                    let k = PREFIX + j;
                    let s = &states[cursor + j];
                    let want_cur = Some(w.tokens[k - 1]);
                    let want_prev = if k >= 2 { Some(w.tokens[k - 2]) } else { None };
                    if s.cur != want_cur || s.prev != want_prev {
                        return Err(format!(
                            "recorded consumed-token history is off at window position {k}: got \
                             (cur {:?}, prev {:?}), the window says ({want_cur:?}, {want_prev:?})",
                            s.cur, s.prev
                        ));
                    }
                }
                cursor += generates;
            }
            if cursor != states.len() {
                return Err(format!(
                    "window walk covers {cursor} positions but {} states were recorded",
                    states.len()
                ));
            }
            Ok(cursor)
        };
    let fit_cur_prev = cur_prev_control(&fit_windows, &fit_states)?;
    let dev_cur_prev = cur_prev_control(&dev_windows, &dev_states)?;

    // ---- control: the declared premise of dropping `m` and `f` from the probe input ----
    // `score_example` feeds its readout `[h, m, f, event_onehot]`. The probe keeps `h` and the event and
    // drops `m` (all zeros in the non-grounded branch) and `f` (the same constant block everywhere).
    // Both premises are checked here against every window example instead of asserted in prose.
    let mut f_block: Option<Vec<i32>> = None;
    for w in fit_windows.iter().chain(dev_windows.iter()) {
        let ex = prose_example(w);
        if ex.grounded || !ex.sel.is_empty() || !ex.res.is_empty() || ex.facts != SlFacts::default()
        {
            return Err(
                "a prose example is grounded or carries evidence, so m and f would not be the \
                 declared constants and dropping them from the probe input would lose information"
                    .into(),
            );
        }
        let f = model.typed_block(&ex.sel, &ex.res, ex.facts);
        match &f_block {
            None => f_block = Some(f),
            Some(previous) if *previous != f => {
                return Err(
                    "the typed block differs between prose windows; the probe input cannot drop it"
                        .into(),
                );
            }
            Some(_) => {}
        }
    }
    let f_block = f_block.unwrap_or_default();

    // The artifact's own readout rearranged into the probe's layout, built once here and used either as
    // the refinement's starting point (`--probe-init artifact`) or only for the control below.
    let artifact_init = artifact_readout_init(&model, &top_ids, &f_block, model.h_clamp)?;
    let artifact_init_rows: Vec<usize> = top_ids.iter().map(|t| model.token_row(*t)).collect();

    let fit = probe_set(&fit_states, &class_of, model.h_clamp);
    let dev = probe_set(&dev_states, &class_of, model.h_clamp);
    let d_probe = fit.d;
    if fit.x.len() != fit.n * d_probe
        || dev.x.len() != dev.n * d_probe
        || d_probe != model.h_dim + PROBE_EVENT_DIMS
    {
        return Err("recorded state width does not match the artifact state width".into());
    }

    // ---- decode ladder inputs, gathered while the state vectors are still resident ----
    // The decode population is the recorded states whose `cur`, `prev` **and** target are all in the
    // restricted class set, so the two consumed tokens and the generated token are compared on one
    // population. The fit side is capped at a declared stride for cost; dev is complete.
    let decode_population = |states: &[ServedState]| -> Vec<usize> {
        (0..states.len())
            .filter(|i| {
                let s = &states[*i];
                let in_set =
                    |t: Option<u32>| t.is_some_and(|v| (v as usize) < VOCAB && in_top[v as usize]);
                in_set(s.cur) && in_set(s.prev) && in_top[s.target as usize]
            })
            .collect()
    };
    let truth_of = |states: &[ServedState], pick: u8| -> Vec<u32> {
        states
            .iter()
            .map(|s| match pick {
                0 => s.cur.unwrap_or(u32::MAX),
                1 => s.prev.unwrap_or(u32::MAX),
                _ => s.target,
            })
            .collect()
    };
    let fit_decode_all = decode_population(&fit_states);
    let fit_decode = declared_subset(&fit_decode_all, EMBED_DECODE_FIT_CAP);
    let dev_decode = decode_population(&dev_states);
    let (fit_truth_cur, fit_truth_prev, fit_truth_next) = (
        truth_of(&fit_states, 0),
        truth_of(&fit_states, 1),
        truth_of(&fit_states, 2),
    );
    let (dev_truth_cur, dev_truth_prev, dev_truth_next) = (
        truth_of(&dev_states, 0),
        truth_of(&dev_states, 1),
        truth_of(&dev_states, 2),
    );
    let t_embed = Instant::now();
    let fit_embed = embedding_decode(&model, &fit_states, &fit_decode);
    let dev_embed = embedding_decode(&model, &dev_states, &dev_decode);
    let embed_secs = t_embed.elapsed().as_secs_f64();

    // Control for the artefact rearrangement: on real recorded states the initialised readout must
    // reproduce the artifact's own logits entry by entry, at the artifact's declared dyadic scale.
    let (init_check_fit, init_check_fit_values, init_check_fit_worst) = verify_artifact_init(
        &model,
        &artifact_init,
        &artifact_init_rows,
        &fit_states,
        &declared_subset(&(0..fit_states.len()).collect::<Vec<usize>>(), 128),
        &f_block,
    )?;
    let (init_check_dev, init_check_dev_values, init_check_dev_worst) = verify_artifact_init(
        &model,
        &artifact_init,
        &artifact_init_rows,
        &dev_states,
        &(0..dev_states.len().min(128)).collect::<Vec<usize>>(),
        &f_block,
    )?;
    if init_check_fit_worst > 1e-6 || init_check_dev_worst > 1e-6 {
        return Err(format!(
            "artifact-readout rearrangement control failed: worst deviations {init_check_fit_worst} \
             (fit) and {init_check_dev_worst} (dev) from the artifact's own logits"
        ));
    }

    // ---- the integers the served readout would be given, for the servable repricing ----
    let fit_ids: Vec<usize> = (0..fit.n).collect();
    let dev_ids: Vec<usize> = (0..dev.n).collect();
    let fit_temp_idx = declared_subset(&fit_ids, RIDGE_TEMP_CAP);
    let fit_reprice_u = integer_inputs(&fit_states, &fit, &fit_temp_idx);
    let dev_u = integer_inputs(&dev_states, &dev, &dev_ids);
    drop(fit_states);
    drop(dev_states);

    let dev_canonical = evaluate_single(&model, &dev_windows);
    println!(
        "1. served states recorded: {fit_total} fit / {dev_total} dev Generate decisions in \
         {rec_secs:.1}s (h_dim {}, h_clamp {}); recorded bit sums equal score_example's bits_generate \
         and every recorded state carries its window's (cur, prev)",
        model.h_dim,
        model.h_clamp,
        fit_total = fit.all_recorded,
        dev_total = dev.all_recorded,
    );
    println!(
        "   restricted to the top-{k_used} targets: fit {} of {} ({:.2}%), dev {} of {} ({:.2}%)",
        fit.n,
        fit.all_recorded,
        frac(fit.n, fit.all_recorded) * 100.0,
        dev.n,
        dev.all_recorded,
        frac(dev.n, dev.all_recorded) * 100.0
    );
    println!(
        "   artifact served loss: restricted {:.4} / unrestricted {:.12} (canonical score_example \
         {:.12}); windows fit {} of {} / dev {}",
        dev_served.restricted(),
        dev_served.all(),
        dev_canonical,
        fit_windows.len(),
        all_fit_windows,
        dev_windows.len()
    );
    println!(
        "   controls: {fit_cur_prev} fit + {dev_cur_prev} dev recorded states carry cur = x_(t-1) and \
         prev = x_(t-2); m is all zeros and f is the constant [{}] block (checked per window)",
        f_block
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!(
        "   probe input: x = [h / h_clamp ({}), event one-hot ({PROBE_EVENT_DIMS})], d = {d_probe}",
        model.h_dim
    );
    println!(
        "   decode population (cur, prev and target all in the top-{k_used}): fit {} of {} recorded \
         states (capped at {EMBED_DECODE_FIT_CAP} for the nearest-row decode), dev {} of {}; {embed_secs:.1}s",
        fit_decode_all.len(),
        fit.all_recorded,
        dev_decode.len(),
        dev.all_recorded
    );

    // ---- the closed-form (ridge) readout, and the refinement started from it ----
    // The ridge weight is chosen on a held-out subset of the fit states only; the reported readout is
    // refitted on every fit state at the selected weight; one temperature is fitted on a capped subset.
    let mut sel_seed = args.probe_seed ^ 0xA5A5_5A5A_1234_5678;
    let mut shuffled_ids = fit_ids.clone();
    shuffle(&mut shuffled_ids, &mut sel_seed);
    let held_len = ((fit.n as f64 * RIDGE_SELECT_FRACTION).round() as usize).clamp(1, fit.n);
    let held_ids: Vec<usize> = shuffled_ids[..held_len].to_vec();
    let held_score_ids = declared_subset(&held_ids, RIDGE_SELECT_CAP);
    let train_ids: Vec<usize> = shuffled_ids[held_len..].to_vec();
    let t_ridge = Instant::now();
    println!(
        "2. closed-form ridge readout ({k_used} classes, d = {d_probe}, {} parameters, lambda grid \
         {RIDGE_LAMBDAS:?} scaled by trace(HtH)/d, chosen on {} held-out fit states = {:.0}% seeded by \
         {:x}):",
        k_used * d_probe + k_used,
        held_score_ids.len(),
        RIDGE_SELECT_FRACTION * 100.0,
        args.probe_seed ^ 0xA5A5_5A5A_1234_5678,
    );
    let ridge = fit_ridge_readout(
        &fit.x,
        &fit.y,
        d_probe,
        k_used,
        &train_ids,
        &held_score_ids,
        &fit_temp_idx,
        &fit_ids,
    )?;
    let mut ridge_scratch = vec![0.0; k_used];
    let ridge_fit_bits = ridge.nll_bits(&fit.x, &fit.y, &fit_ids, &mut ridge_scratch);
    let ridge_dev_bits = ridge.nll_bits(&dev.x, &dev.y, &dev_ids, &mut ridge_scratch);
    let ridge_dev_top1 = ridge.top1(&dev.x, &dev.y, &dev_ids, &mut ridge_scratch);
    let ridge_secs = t_ridge.elapsed().as_secs_f64();
    println!(
        "   lambda {:.0e} (absolute {:.6e}); fit {ridge_fit_bits:.4} / dev {ridge_dev_bits:.4} \
         bits/target, dev top-1 {:.4}; temperature {:.6} fitted on {} fit states",
        ridge.lambda_rel,
        ridge.lambda_abs,
        ridge_dev_top1,
        ridge.temperature,
        fit_temp_idx.len()
    );

    // ---- the float probe, initialised from the declared starting point ----
    let t_probe = Instant::now();
    println!(
        "3. softmax refinement from the {}-initialised readout (Adam lr {}):",
        args.probe_init, args.probe_lr
    );
    let refinement_init = if args.probe_init == PROBE_INIT_ARTIFACT {
        FloatProbe::from_readout(k_used, d_probe, &artifact_init.w, &artifact_init.b)?
    } else {
        FloatProbe::from_readout(k_used, d_probe, &ridge.w, &ridge.b)?
    };
    let probe_fit = fit_float_probe(
        refinement_init,
        &fit.x,
        &fit.y,
        fit.n,
        &dev,
        args.probe_epochs,
        args.probe_lr,
        args.probe_seed,
        "probe",
    );
    let probe_dev_top1 = probe_fit
        .probe
        .top1(&dev.x, &dev.y, &dev_ids, &mut ridge_scratch);
    let probe_secs = t_probe.elapsed().as_secs_f64();

    // ---- the null probe: the identical procedure on permuted labels, evaluated on the real dev pairs ----
    // It can be skipped (--probe-skip-null), in which case the receipt says the control was not run in
    // this attempt and cites the sealed attempts where the identical machinery was validated.
    let mut permuted = fit.y.clone();
    let mut pst = args.probe_seed ^ 0x9E37_79B9_7F4A_7C15;
    shuffle(&mut permuted, &mut pst);
    let t_null = Instant::now();
    let (null_ridge, null_fit, null_secs) = if args.probe_skip_null {
        println!(
            "4. null probe skipped (--probe-skip-null): the same machinery is already validated in \
             olx-probe-3 at K=1024 and /tmp/olx-sp-smoke-2 at K=64"
        );
        (None, None, 0.0)
    } else {
        println!("4. null probe (the same states, labels permuted by the probe seed):");
        let mut null_ids = fit_ids.clone();
        let mut nst = args.probe_seed ^ 0x2718_2818_2845_9045;
        shuffle(&mut null_ids, &mut nst);
        let null_train: Vec<usize> = null_ids[held_len..].to_vec();
        let ridge = fit_ridge_readout(
            &fit.x,
            &permuted,
            d_probe,
            k_used,
            &null_train,
            &held_score_ids,
            &fit_temp_idx,
            &fit_ids,
        )?;
        let fit = fit_float_probe(
            FloatProbe::from_readout(k_used, d_probe, &ridge.w, &ridge.b)?,
            &fit.x,
            &permuted,
            fit.n,
            &dev,
            args.probe_epochs,
            args.probe_lr,
            args.probe_seed,
            "null ",
        );
        (Some(ridge), Some(fit), t_null.elapsed().as_secs_f64())
    };
    let null_ridge_dev = null_ridge
        .as_ref()
        .map(|r| r.nll_bits(&dev.x, &dev.y, &dev_ids, &mut ridge_scratch))
        .unwrap_or(f64::NAN);

    // ---- matched references on the identical restricted population ----
    let (fit_uni, fit_cnt, _) =
        token_references(&fit_windows, &uni, &c1, &c2, lambdas, &in_top, top_mass);
    let (dev_uni, dev_cnt, dev_uni_topk) =
        token_references(&dev_windows, &uni, &c1, &c2, lambdas, &in_top, top_mass);
    if dev_uni.all_n != dev_served.all_n || dev_uni.restricted_n != dev_served.restricted_n {
        return Err(format!(
            "position populations disagree: token references {} / {} vs served states {} / {}",
            dev_uni.all_n, dev_uni.restricted_n, dev_served.all_n, dev_served.restricted_n
        ));
    }
    // ---- the `E1` level: unigram interpolated with the single-token bigram of `cur` ----
    let (e1_weight, e1_tune_bits) = tune_e1_weight(&c1, &uni, &tune_windows);
    let fit_e1 = e1_reference(&fit_windows, &c1, &uni, e1_weight, &in_top);
    let dev_e1 = e1_reference(&dev_windows, &c1, &uni, e1_weight, &in_top);

    // ---- servable repricings of the better float readout ----
    // `P_softmax` beats `P_ridge` on the fit states here, so the repricings start from it.
    let softmax_fit_bits = probe_fit_last_of(&probe_fit);
    let repricing_source = if softmax_fit_bits <= ridge_fit_bits {
        "P_softmax"
    } else {
        "P_ridge"
    };
    let source_readout = if repricing_source == "P_softmax" {
        RidgeReadout {
            k: k_used,
            d: d_probe,
            w: probe_fit.probe.w.clone(),
            b: probe_fit.probe.b.clone(),
            lambda_rel: f64::NAN,
            lambda_abs: f64::NAN,
            temperature: 1.0,
            selection_n: fit.n,
            selection_bits: probe_fit_last_of(&probe_fit),
            lambda_table: Vec::new(),
        }
    } else {
        RidgeReadout {
            k: ridge.k,
            d: ridge.d,
            w: ridge.w.clone(),
            b: ridge.b.clone(),
            lambda_rel: ridge.lambda_rel,
            lambda_abs: ridge.lambda_abs,
            temperature: ridge.temperature,
            selection_n: ridge.selection_n,
            selection_bits: ridge.selection_bits,
            lambda_table: ridge.lambda_table.clone(),
        }
    };
    let source_dev_bits = source_readout.nll_bits(&dev.x, &dev.y, &dev_ids, &mut ridge_scratch);
    let t_reprice = Instant::now();
    let mut repricings = Vec::new();
    for (alphabet, fitted) in [
        (CodeAlphabet::Ternary, false),
        (CodeAlphabet::Ternary, true),
        (CodeAlphabet::FourBit, false),
        (CodeAlphabet::FourBit, true),
    ] {
        repricings.push(reprice(
            &source_readout,
            source_dev_bits,
            model.h_dim,
            model.h_clamp,
            alphabet,
            fitted,
            &fit_reprice_u,
            &fit.y,
            &dev_u,
            &dev.y,
            &fit_temp_idx,
            &dev_ids,
        )?);
    }
    let reprice_secs = t_reprice.elapsed().as_secs_f64();

    // ---- the state-side decode ladder: ridge decode of cur, prev and the generated token ----
    let t_decode = Instant::now();
    let fit_decode_ids: Vec<usize> = (0..fit.n)
        .filter(|i| {
            let ok = |t: Option<u32>| t.is_some_and(|v| (v as usize) < VOCAB && in_top[v as usize]);
            ok(fit.cur[*i]) && ok(fit.prev[*i])
        })
        .collect();
    let mut decode_sel_seed = args.probe_seed ^ 0x0F0F_0F0F_1234_ABCD;
    let mut decode_shuffled = fit_decode_ids.clone();
    shuffle(&mut decode_shuffled, &mut decode_sel_seed);
    let decode_held_len =
        ((fit_decode_ids.len() as f64 * RIDGE_SELECT_FRACTION).round() as usize).max(1);
    let decode_train: Vec<usize> = decode_shuffled[decode_held_len..].to_vec();
    let decode_held = declared_subset(&decode_shuffled[..decode_held_len], 10_000);
    let decode_temp = declared_subset(&fit_decode_ids, RIDGE_TEMP_CAP);
    let dev_decode_ids: Vec<usize> = (0..dev.n)
        .filter(|i| {
            let ok = |t: Option<u32>| t.is_some_and(|v| (v as usize) < VOCAB && in_top[v as usize]);
            ok(dev.cur[*i]) && ok(dev.prev[*i])
        })
        .collect();
    let class_y = |set: &ProbeSet, pick: u8| -> Vec<usize> {
        (0..set.n)
            .map(|i| {
                let t = match pick {
                    0 => set.cur[i],
                    1 => set.prev[i],
                    _ => Some(set.target[i]),
                };
                t.and_then(|v| class_of.get(v as usize).copied())
                    .filter(|c| *c != usize::MAX)
                    .unwrap_or(0)
            })
            .collect()
    };
    let (fit_cur_y, fit_prev_y) = (class_y(&fit, 0), class_y(&fit, 1));
    let (dev_cur_y, dev_prev_y) = (class_y(&dev, 0), class_y(&dev, 1));
    let ridge_decode = |y: &[usize]| -> Result<RidgeReadout, String> {
        fit_ridge_readout(
            &fit.x,
            y,
            d_probe,
            k_used,
            &decode_train,
            &decode_held,
            &decode_temp,
            &fit_decode_ids,
        )
    };
    let decode_cur = ridge_decode(&fit_cur_y)?;
    let decode_prev = ridge_decode(&fit_prev_y)?;
    let decode_next = ridge_decode(&fit.y)?;
    let mut decode_scratch = vec![0.0; k_used];
    let dev_cur_bits =
        decode_cur.nll_bits(&dev.x, &dev_cur_y, &dev_decode_ids, &mut decode_scratch);
    let dev_prev_bits =
        decode_prev.nll_bits(&dev.x, &dev_prev_y, &dev_decode_ids, &mut decode_scratch);
    let dev_next_bits = decode_next.nll_bits(&dev.x, &dev.y, &dev_decode_ids, &mut decode_scratch);
    let dev_cur_top1 = decode_cur.top1(&dev.x, &dev_cur_y, &dev_decode_ids, &mut decode_scratch);
    let dev_prev_top1 = decode_prev.top1(&dev.x, &dev_prev_y, &dev_decode_ids, &mut decode_scratch);
    let dev_next_top1 = decode_next.top1(&dev.x, &dev.y, &dev_decode_ids, &mut decode_scratch);
    let decode_secs = t_decode.elapsed().as_secs_f64();

    let nearest_fit = (
        decode_accuracy(
            &fit_embed.winners,
            &fit_embed,
            &fit_truth_cur,
            &fit_decode,
            &class_of,
        ),
        decode_accuracy(
            &fit_embed.winners,
            &fit_embed,
            &fit_truth_prev,
            &fit_decode,
            &class_of,
        ),
        decode_accuracy(
            &fit_embed.winners,
            &fit_embed,
            &fit_truth_next,
            &fit_decode,
            &class_of,
        ),
    );
    let nearest_dev = (
        decode_accuracy(
            &dev_embed.winners,
            &dev_embed,
            &dev_truth_cur,
            &dev_decode,
            &class_of,
        ),
        decode_accuracy(
            &dev_embed.winners,
            &dev_embed,
            &dev_truth_prev,
            &dev_decode,
            &class_of,
        ),
        decode_accuracy(
            &dev_embed.winners,
            &dev_embed,
            &dev_truth_next,
            &dev_decode,
            &class_of,
        ),
    );

    let probe_dev = probe_fit
        .per_epoch
        .last()
        .map(|(_, d)| *d)
        .unwrap_or(f64::NAN);
    let probe_fit_last = probe_fit
        .per_epoch
        .last()
        .map(|(f, _)| *f)
        .unwrap_or(f64::NAN);
    let probe_fit_delta = if probe_fit.per_epoch.len() >= 2 {
        probe_fit.per_epoch[probe_fit.per_epoch.len() - 2].0 - probe_fit_last
    } else {
        f64::NAN
    };
    let probe_converged =
        probe_fit_delta.is_finite() && probe_fit_delta.abs() < PROBE_CONVERGED_BITS;
    let null_dev = null_fit
        .as_ref()
        .and_then(|f| f.per_epoch.last())
        .map(|(_, d)| *d)
        .filter(|_| !args.probe_skip_null);
    let null_gap = null_dev.map(|d| d - dev_uni_topk.restricted());
    let null_leaks = null_gap.map(|g| g < -NULL_LEAK_BITS).unwrap_or(false);
    if let Some(gap) = null_gap.filter(|g| *g < -NULL_LEAK_BITS) {
        println!(
            "   WARNING: the null probe is {:.4} bits below the unigram-over-{k_used} level; the \
             instrument is leaking information and this run's dev number must not be read as a state \
             bound",
            -gap
        );
    }

    println!("5. losses in bits/target (restricted = the top-{k_used} target population):");
    println!("   {:<28} {:>10} {:>10}", "", "restricted", "unrestricted");
    println!(
        "   {:<28} {:>10.4} {:>10.4}",
        "artifact served readout",
        dev_served.restricted(),
        dev_served.all()
    );
    println!(
        "   {:<28} {:>10.4} {:>10.4}",
        "tuned 2-token count",
        dev_cnt.restricted(),
        dev_cnt.all()
    );
    println!(
        "   {:<28} {:>10.4} {:>10.4}",
        "E1 unigram + bigram(cur)",
        dev_e1.restricted(),
        dev_e1.all()
    );
    println!(
        "   {:<28} {:>10.4} {:>10.4}",
        "fit-only unigram",
        dev_uni.restricted(),
        dev_uni.all()
    );
    println!(
        "   {:<28} {:>10} {:>10}",
        "--- P_ridge (dev) ---",
        format!("{ridge_dev_bits:.4}"),
        "-"
    );
    println!(
        "   {:<28} {:>10} {:>10}",
        "--- P_softmax (dev) ---",
        format!("{probe_dev:.4}"),
        "-"
    );
    println!(
        "   {:<28} {:>10} {:>10}",
        "--- null probe (dev) ---",
        null_dev
            .map(|d| format!("{d:.4}"))
            .unwrap_or_else(|| "not run".into()),
        "-"
    );
    println!(
        "   A {:.4} | P_ridge {ridge_dev_bits:.4} (top-1 {ridge_dev_top1:.4}) | P_softmax {probe_dev:.4} \
         (top-1 {probe_dev_top1:.4}) | E1 {:.4} | C {:.4} | U {:.4}",
        dev_served.restricted(),
        dev_e1.restricted(),
        dev_cnt.restricted(),
        dev_uni.restricted()
    );
    println!(
        "   P_ridge - A {:+.4}, P_softmax - A {:+.4}, P_softmax - P_ridge {:+.4}, best P - E1 {:+.4}, best P - C {:+.4} \
         (negative favours the readout)",
        ridge_dev_bits - dev_served.restricted(),
        probe_dev - dev_served.restricted(),
        probe_dev - ridge_dev_bits,
        probe_dev.min(ridge_dev_bits) - dev_e1.restricted(),
        probe_dev.min(ridge_dev_bits) - dev_cnt.restricted()
    );
    println!(
        "9. servable repricing of {repricing_source} (integer path over the recorded states; no sealed \
         artifact is read or written):"
    );
    println!(
        "   {:<22} {:>10} {:>10} {:>9} {:>10} {:>9}",
        "readout", "dev bits", "fit bits", "score_sh", "free-scale", "bytes"
    );
    for r in repricings.iter() {
        println!(
            "   {:<22} {:>10.4} {:>10.4} {:>9} {:>10.4} {:>9}",
            format!(
                "{}{}",
                r.alphabet.name(),
                if r.fitted_offsets {
                    "+offsets"
                } else {
                    " rule"
                }
            ),
            r.dev_bits,
            r.dyadic_fit_bits,
            r.score_shift,
            r.dev_bits_free_scale,
            r.k * r.d / r.alphabet.bits_per_entry() + 4 * r.k + 4 * r.k
        );
    }
    println!(
        "   float {repricing_source} dev {source_dev_bits:.4}; each repricing keeps its own fitted \
         per-row scale policy, dyadic score scale and integer bias (envelopes ok: {})",
        repricings.iter().all(|r| r.envelope_ok)
    );
    for r in repricings.iter() {
        println!(
            "   {}: pre-scale 2^{} (rel. error {:.4}, code density {:.4}, nonzero {}), score_shift {} \
             (free-scale shift {:+.3}), logits [{}, {}]",
            format!(
                "{}{}",
                r.alphabet.name(),
                if r.fitted_offsets { "+offsets" } else { " rule" }
            ),
            r.pre_scale_log2,
            r.pre_scale_relative_error,
            r.code_density,
            r.codes.iter().filter(|c| **c != 0).count(),
            r.score_shift,
            r.implied_free_score_shift,
            r.dev_logit_min,
            r.dev_logit_max
        );
    }
    println!(
        "10. state-side decode ladder, dev ({} positions with cur, prev and target all in the top-{k_used}):",
        dev_decode.len()
    );
    println!(
        "   {:<24} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "target", "l2 top1", "cos top1", "min-norm", "majority", "sum p^2"
    );
    for (label, s) in [
        ("cur = x_(t-1)", nearest_dev.0),
        ("prev = x_(t-2)", nearest_dev.1),
        ("x_(t+1) [generated]", nearest_dev.2),
    ] {
        println!(
            "   {:<24} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4}",
            label, s.l2_top1, s.cos_top1, s.min_norm_row, s.majority, s.collision
        );
    }
    println!(
        "   ridge decode from h (dev): cur {dev_cur_bits:.4} bits / {dev_cur_top1:.4} top-1; \
         prev {dev_prev_bits:.4} / {dev_prev_top1:.4}; x_(t+1) {dev_next_bits:.4} / {dev_next_top1:.4}"
    );
    println!(
        "   fit-side nearest-row decode over {} of {} recorded states: cur l2 {:.4} / cos {:.4}, \
         prev l2 {:.4} / cos {:.4}, x_(t+1) l2 {:.4} / cos {:.4}",
        fit_decode.len(),
        fit.all_recorded,
        nearest_fit.0.l2_top1,
        nearest_fit.0.cos_top1,
        nearest_fit.1.l2_top1,
        nearest_fit.1.cos_top1,
        nearest_fit.2.l2_top1,
        nearest_fit.2.cos_top1
    );
    println!(
        "6. declared reading rule: P is a linear readout of the recorded state, so it is bounded by the \
         rank of the linear class and is not expected to reach C. The actionable comparison is P versus \
         A: a converged P below A means the served ternary readout is the bottleneck at this state; a P \
         that stays at A means any readout over this state is no better, so the state is the bound."
    );
    match (null_dev, null_gap) {
        (Some(d), Some(g)) => println!(
            "7. null probe {d:.4} (its own ridge init {null_ridge_dev:.4}) vs unigram-over-{k_used} \
             {:.4} (null - unigram {g:+.4}, leak={null_leaks})",
            dev_uni_topk.restricted()
        ),
        _ => println!(
            "7. null probe NOT RUN in this attempt (--probe-skip-null); the identical machinery was \
             validated on the sealed olx-probe-3 at K=1024 and on /tmp/olx-sp-smoke-2 at K=64"
        ),
    }
    println!(
        "8. E1 weight {e1_weight} (tune {e1_tune_bits:.4}); probe refinement converged={probe_converged} \
         (last-epoch fit delta {probe_fit_delta:+.4} bits)"
    );
    println!(
        "   phase seconds: record {rec_secs:.1}, embed-decode {embed_secs:.1}, ridge {ridge_secs:.1}, \
         refinement {probe_secs:.1}, null {null_secs:.1}, reprice {reprice_secs:.1}, decode-ridge {decode_secs:.1}"
    );

    // ---- receipt ----
    // Declared once so the receipt can carry it: the distinct per-row power-of-two scales each
    // repricing actually used.
    let repricing_scales: Vec<Vec<u32>> = repricings
        .iter()
        .map(|r| {
            let mut v = r.shift.clone();
            v.sort_unstable();
            v.dedup();
            v
        })
        .collect();
    let receipt = serde_json::json!({
        "schema": "uor-r4.ordinary-lexical-state-probe/1",
        "source_rev": args.source_rev.as_str(),
        "artifact": {
            "path": artifact_path.display().to_string(),
            "sha256": artifact_sha,
            "bytes": artifact_bytes.len(),
            "vocab": model.vocab,
            "h_dim": model.h_dim,
            "h_clamp": model.h_clamp,
            "copied_into_report_root": false,
            "note": "the artifact is read, hashed and probed in place; this attempt records its \
                     identity and neither writes to nor re-seals the artifact's own directory"
        },
        "tested_executable_sha256": exe_sha,
        "tokenizer_source_sha256": tok_sha,
        "tokenizer_derived_sha256": derived_sha,
        "corpus": {
            "collected": collected,
            "unique": uniq.len(),
            "exact_duplicates_grouped": duplicates,
            "fit_docs": fit_docs.len(),
            "tune_docs": tune_docs.len(),
            "dev_docs": dev_names.len(),
            "fit_windows_used": fit_windows.len(),
            "fit_windows_available": all_fit_windows,
            "dev_windows": dev_windows.len(),
            "fit_generate_targets_unrestricted": fit_served.all_n,
            "dev_generate_targets_unrestricted": dev_served.all_n,
            "document_sha256": uniq.iter().map(|d| serde_json::json!({"path": d.path, "sha256": sha256_hex(&d.sha256)})).collect::<Vec<_>>(),
        },
        "dev_documents": dev_names,
        "count_reference": {"lambdas": [lambdas.0, lambdas.1], "tune_bits_per_target": tune_bits,
                            "definition": "fit-only unigram, (cur) and (prev, cur) levels interpolated \
                                           with the fit/tune-separated lambdas, exactly as run() fits them"},
        "state_recording": {
            "method": "one walk per window that mirrors TlModel::score_example for a non-grounded \
                       example: owned empty, m = vec![0; h_dim], f = typed_block(&[], &[], default); \
                       observed tokens advance with TL_EV_OBSERVE; each step scores on \
                       (h, m, f, prior_event(i)) and advances with (actions[i].event(), emitted) where \
                       emitted is Some(v) for Generate, ex.emitted(i, owned) for Copy and None for Stop",
            "m_is_all_zero": true,
            "f_is_the_declared_constant_block": true,
            "recorded_states_sha256": {"fit": fit_states_sha, "dev": dev_states_sha},
            "recorded_states_sha256_layout": "walk order; per state doc (u64 LE), target (u32 LE), then \
                                              h (i32 LE) per coordinate",
            "control": {
                "definition": "the sum of the recorded per-position bits equals score_example's own \
                               bits_generate over the same windows, and the recorded state count equals \
                               its scored Generate targets",
                "fit_control_stride": FIT_CONTROL_STRIDE,
                "fit_windows_checked": fit_control_windows,
                "fit_states_checked": fit_control_states,
                "dev_windows_checked": dev_control_windows,
                "fit_score_example_bits": fit_sx_bits,
                "fit_recorded_bits": fit_rec_bits,
                "fit_scored_targets": fit_sx_n,
                "dev_score_example_bits": dev_sx_bits,
                "dev_recorded_bits": dev_rec_bits,
                "dev_scored_targets": dev_sx_n,
                "coverage": "development is complete; the fit side is a declared stride sample plus \
                             every terminal window, because the control is a second full readout pass",
                "passed": true,
            },
            "served_aggregate_unrestricted_bits_per_target": dev_served.all(),
            "served_aggregate_canonical_score_example": dev_canonical,
        },
        "restricted_population": {
            "k": k_used,
            "source": "top-K token ids by fit-position target frequency over the fit windows actually \
                       used, count descending then lowest id first",
            "top_ids": top_ids,
            "fit_retained_targets": fit.n,
            "fit_unrestricted_targets": fit.all_recorded,
            "fit_coverage": frac(fit.n, fit.all_recorded),
            "dev_retained_targets": dev.n,
            "dev_unrestricted_targets": dev.all_recorded,
            "dev_coverage": frac(dev.n, dev.all_recorded),
            "fit_unigram_mass_in_restricted_set": top_mass,
        },
        "probe_initialisation": {
            "chosen": args.probe_init,
            "values": [PROBE_INIT_RIDGE, PROBE_INIT_ARTIFACT],
            "ridge": "the closed-form least-squares readout of the recorded states, fitted on this \
                      run's fit split (the P_ridge row above)",
            "artifact": "the artifact's own served readout, dequantised into the probe's feature layout. \
                         Mapping, exactly: the probe's input is [h / h_clamp, event one-hot] (d) and the \
                         artifact's readout input is [h, m, f, event] (2*h_dim + f_dim + 4) through a \
                         per-row-scaled ternary map plus an i32 bias bo. (i) state columns 0..h_dim are \
                         taken as (w << shift) * h_clamp, the factor h_clamp undoing the probe's own \
                         1 / h_clamp input scaling; (ii) event columns 2*h_dim + f_dim .. + 4 are taken \
                         unscaled; (iii) the h_dim m columns are dropped entirely because m is \
                         identically zero for ordinary prose, so they contribute exactly zero; (iv) the \
                         f_dim columns of the constant typed block are folded into b as \
                         sum_f (w_(r,f) << shift_(r)) * f0[f]; (v) the artifact's bo of the same row is \
                         added into b; (vi) every coefficient is multiplied by 2^-score_shift, the \
                         artifact's declared dyadic score scale, so the initial logits are the SERVED \
                         logits in float form. Every coefficient is read from the artifact's own float \
                         evaluation of its packed map (TlLinear::forward_reference on unit vectors), so \
                         no packing or dequantisation convention is reimplemented",
            "row_of_class": "token_row(top_ids[c]): the artifact readout row of the c-th restricted class",
            "state_columns_taken": artifact_init.state_columns,
            "state_column_scale": format!("h_clamp = {} then 2^-score_shift", model.h_clamp),
            "m_columns_dropped_as_zero": artifact_init.m_columns_dropped,
            "f_columns_folded_into_bias": artifact_init.f_columns_folded,
            "f_constant_block_max_weighted_magnitude": artifact_init.f_bias_from_constant_block,
            "event_columns_taken": artifact_init.event_columns,
            "artifact_bo_entries_added": k_used,
            "dyadic_score_scale": artifact_init.score_scale,
            "artifact_score_shift": model.score_shift,
            "rearrangement_control": {
                "definition": "on real recorded states the initialised readout's logits must equal \
                               TlModel::readout's logits at the artifact's declared dyadic scale, entry \
                               by entry; a deviation above 1e-6 aborts the run",
                "fit_positions_checked": init_check_fit,
                "fit_values_compared": init_check_fit_values,
                "fit_worst_absolute_deviation": init_check_fit_worst,
                "dev_positions_checked": init_check_dev,
                "dev_values_compared": init_check_dev_values,
                "dev_worst_absolute_deviation": init_check_dev_worst,
                "passed": true,
            },
            "epoch_zero_note": "the refinement prints and records its fit/dev loss at the declared \
                                initialisation before any epoch; with --probe-init artifact that number \
                                is the artifact's own readout evaluated in the probe's K-class softmax \
                                at the artifact's declared dyadic scale, which is the directly \
                                comparable baseline for the probe's own losses (the artifact's served \
                                reference A is normalised over its full legal action set instead)",
        },
        "probe_input": {
            "form": "[h / h_clamp, event one-hot]",
            "d": d_probe,
            "h_dim": model.h_dim,
            "event_dims": PROBE_EVENT_DIMS,
            "m_dropped": "all zeros for a non-grounded example: score_example's non-grounded branch \
                          takes m = vec![0i32; h_dim] literally, and every window example is \
                          non-grounded with empty sel/res (checked at run time)",
            "f_dropped": "the same declared constant block at every prose position; the block actually \
                          observed on this corpus is recorded below and was checked to be identical \
                          for every window",
            "f_block_observed_first_15": f_block,
            "why_dropping_costs_nothing": "a linear readout is unchanged by dropping a zero block and by \
                                           absorbing a constant block into its bias, so the probe is \
                                           never given less than the served readout receives",
            "verification": "for every fit and dev window the runner asserts the example is not \
                             grounded, sel and res are empty and facts == SlFacts::default(), and that \
                             typed_block returns the identical vector; a violation aborts the run",
        },
        "consumed_token_bookkeeping": {
            "definition": "cur = the token consumed by the transition that produced h, prev = the token \
                           consumed by the transition before it, both read off the walk's own \
                           consumption history with None for a Stop",
            "fit_positions_checked": fit_cur_prev,
            "dev_positions_checked": dev_cur_prev,
            "control": "for every position of both splits, cur must equal tokens[k-1] and prev must \
                        equal tokens[k-2] of the containing window, the exact pair the tuned count \
                        reference conditions on; any off-by-one aborts the run",
            "off_by_one_risk": "a state at the window head has eight observed tokens behind it and so \
                                still has both; only a walk that skipped the unrecorded Stop advance, \
                                or that read the emitted token instead of the consumed one, can shift \
                                the pair, and both of those fail this control",
        },
        "decode_population": {
            "definition": "recorded states whose cur, prev and target are all in the restricted class \
                           set, so the two consumed tokens and the generated token are compared on one \
                           population",
            "fit_states": fit_decode_all.len(),
            "fit_scanned": fit_decode.len(),
            "fit_scan_cap": EMBED_DECODE_FIT_CAP,
            "dev_states": dev_decode.len(),
            "fit_decode_ridge_positions": fit_decode_ids.len(),
            "dev_decode_ridge_positions": dev_decode_ids.len(),
        },
        "ridge_readout": {
            "definition": "closed-form (Tikhonov) least-squares readout over the restricted classes: \
                           (HtH - n x̄x̄ᵀ + λI) W = HtY - n x̄ȳᵀ, b = ȳ - Wᵀx̄",
            "ht_h_shape": [d_probe, d_probe],
            "ht_y_shape": [d_probe, k_used],
            "intercept": "the intercept is handled by centering, which is algebraically identical to \
                          fitting an unpenalised intercept, so the ridge penalty applies to the slopes \
                          only and the system stays exactly d x d",
            "lambda_grid": RIDGE_LAMBDAS,
            "lambda_scaling": "each grid value is multiplied by trace(HtH)/d of the training moments",
            "selection_fraction": RIDGE_SELECT_FRACTION,
            "selection_seed": args.probe_seed ^ 0xA5A5_5A5A_1234_5678,
            "selection_n": held_score_ids.len(),
            "selection_cap": RIDGE_SELECT_CAP,
            "solve": "Cholesky with a relative pivot tolerance, returning Result rather than panicking",
            "lambda_table": ridge.lambda_table.iter().map(|(l, b)| serde_json::json!({"lambda_rel": l, "held_out_bits_per_target": b})).collect::<Vec<_>>(),
            "chosen_lambda_rel": ridge.lambda_rel,
            "chosen_lambda_abs": ridge.lambda_abs,
            "refit_population": fit.n,
            "temperature": ridge.temperature,
            "temperature_fitted_on": fit_temp_idx.len(),
            "temperature_implied_score_shift": -(ridge.temperature).log2(),
            "fit_bits_per_target": ridge_fit_bits,
            "dev_bits_per_target": ridge_dev_bits,
            "dev_top1": ridge_dev_top1,
            "softmax_refinement_dev_top1": probe_dev_top1,
            "rank_note": "the readout is linear in the input, so the (target x input) score matrix has \
                          rank at most d and cannot in general represent the full 4096x4096 \
                          interpolated count table",
        },
        "null_ridge_readout": {
            "definition": "the same closed-form procedure applied to the permuted labels, so the null \
                           probe starts from a converged readout of its own data",
            "run": null_ridge.is_some(),
            "dev_bits_per_target": if null_ridge.is_some() { serde_json::json!(null_ridge_dev) } else { serde_json::json!(null) },
            "temperature": null_ridge.as_ref().map(|r| r.temperature),
            "chosen_lambda_rel": null_ridge.as_ref().map(|r| r.lambda_rel),
        },
        "e1_reference": {
            "definition": "unigram interpolated with the single-token bigram of cur only, fitted on the \
                           fit split and tuned on the tune split over the declared grid",
            "weight": e1_weight,
            "tune_bits_per_target": e1_tune_bits,
            "dev": dev_e1.json(),
            "fit": fit_e1.json(),
        },
        "servable_repricing": {
            "source": repricing_source,
            "source_dev_bits_per_target": source_dev_bits,
            "schemes": repricings.iter().enumerate().map(|(ri, r)| serde_json::json!({
                "alphabet": r.alphabet.name(),
                "fitted_offsets": r.fitted_offsets,
                "per_row_offsets_tried": REPRICE_OFFSETS,
                "offset_histogram": r.offset_histogram,
                "score_shift": r.score_shift,
                "dyadic_fit_bits_per_target": r.dyadic_fit_bits,
                "dev_bits_per_target": r.dev_bits,
                "free_scale": r.free_scale,
                "free_scale_fit_bits_per_target": r.free_scale_fit_bits,
                "implied_free_score_shift": r.implied_free_score_shift,
                "dev_bits_at_free_scale": r.dev_bits_free_scale,
                "float_source_dev_bits_per_target": r.float_dev_bits,
                "pre_scale": r.pre_scale,
                "pre_scale_log2": r.pre_scale_log2,
                "pre_scale_relative_reconstruction_error": r.pre_scale_relative_error,
                "code_density": r.code_density,
                "nonzero_codes": r.codes.iter().filter(|c| **c != 0).count(),
                "code_entries": r.codes.len(),
                "distinct_row_scales": repricing_scales[ri],
                "max_abs_bias": r.max_abs_bias,
                "max_row_accumulator_bound": r.max_row_bound.to_string(),
                "envelope_ok": r.envelope_ok,
                "dev_integer_logit_min": r.dev_logit_min,
                "dev_integer_logit_max": r.dev_logit_max,
                "bytes_packed_information": r.k * r.d / r.alphabet.bits_per_entry(),
                "bytes_artifact_container": r.k * r.d / r.alphabet.bits_per_entry() + 4 * r.k + 4 * r.k,
                "bytes_note": "packed information is the declared alphabet at 2 or 4 bits per weight; \
                               artifact container adds the per-row scales and the per-class i32 bias",
            })).collect::<Vec<_>>(),
            "integer_path": "the ternary scheme is rescored through the artifact's own \
                             TlLinear::forward_i32 (built from its public fields; the packing is \
                             reimplemented and verified against that kernel before use); the 4-bit \
                             scheme accumulates TlEmbed::value, which is how that table stores its \
                             entries. This evaluates the served integers over recorded states; it does \
                             not modify, rewrite or re-seal any artifact",
            "score_shift_is_the_temperature": "the served score is (integer logit) * 2^-score_shift, so \
                                               the fitted dyadic scale is a servable parameter, and \
                                               the free-scale column is the ceiling that the alphabets \
                                               are compared against",
        },
        "decode_ladder": {
            "nearest_embedding_decode": "argmin ||h - E[v]||_2 and argmax cosine over all rows, using h \
                                         only (no event row)",
            "nearest_l2_confound_control": "the L2 decode is biased towards short rows because \
                                            ||h - E||² = ||h||² - 2h·E + ||E||²; the accuracy of a \
                                            constant smallest-norm-row predictor is reported beside it, \
                                            and the cosine decode is reported as the scale-free variant",
            "expected_chance": "the majority-class frequency and the collision probability sum p^2 of \
                                the target population are reported per target",
            "x_next_note": "x_(t+1) is the token the decision generates, which the served readout \
                            already predicts at ~6 bits, so it is not a chance control but the \
                            prediction target; the chance floors above are reported explicitly",
            "dev": {
                "cur_nearest": nearest_dev.0.json(),
                "prev_nearest": nearest_dev.1.json(),
                "next_nearest": nearest_dev.2.json(),
                "ridge_cur_dev_bits": dev_cur_bits,
                "ridge_cur_dev_top1": dev_cur_top1,
                "ridge_prev_dev_bits": dev_prev_bits,
                "ridge_prev_dev_top1": dev_prev_top1,
                "ridge_next_dev_bits": dev_next_bits,
                "ridge_next_dev_top1": dev_next_top1,
            },
            "embedding_norm_statistics": {
                "note": "the raw L2 decode favours short rows, so the spread of ||E[v]|| is reported                           with it; the smallest-norm row is the constant predictor compared against",
                "min_norm2": dev_embed.enorm2.iter().min().copied().unwrap_or(0),
                "max_norm2": dev_embed.enorm2.iter().max().copied().unwrap_or(0),
                "min_norm": dev_embed.enorm.iter().cloned().fold(f64::INFINITY, f64::min),
                "max_norm": dev_embed.enorm.iter().cloned().fold(0.0f64, f64::max),
                "smallest_norm_row": dev_embed.min_norm_row,
                "rows": dev_embed.enorm.len(),
            },
            "fit_nearest": {
                "cur_l2_top1": nearest_fit.0.l2_top1,
                "cur_cos_top1": nearest_fit.0.cos_top1,
                "prev_l2_top1": nearest_fit.1.l2_top1,
                "prev_cos_top1": nearest_fit.1.cos_top1,
                "next_l2_top1": nearest_fit.2.l2_top1,
                "next_cos_top1": nearest_fit.2.cos_top1,
            },
        },
        "declared_comparisons": {
            "ladder": "A = the artifact's own served readout (restricted and unrestricted); P_ridge and \
                       P_softmax = converged float linear readouts of the recorded state; P_ternary and \
                       P_4bit = their servable repricings, each with its own fitted per-row scale policy \
                       and dyadic score scale, rescored in integer arithmetic; E1 = unigram + \
                       bigram(cur); C = the tuned (prev, cur) count reference; U = unigram; plus the \
                       state-side decode ladder.",
            "decision": "P versus A is the actionable comparison. P is a linear readout of h, so its \
                         score matrix has rank at most d and it is not expected to reach C, a full \
                         interpolated table; C is a context anchor, not the threshold.",
            "rank_sentence": "P is bounded by the rank of the linear class, so it cannot in general \
                              reach the count reference C.",
        },
        "float_probe": {
            "classes": k_used,
            "parameters": k_used * model.h_dim + k_used,
            "input": "x = h as f64 / h_clamp, the declared scaling of the recorded served state",
            "init": if args.probe_init == PROBE_INIT_ARTIFACT {
                "W and b start at the artifact's own served readout, rearranged into the probe's feature \
                 layout (see probe_initialisation), with fresh Adam moments"
            } else {
                "W and b start at the closed-form ridge solution fitted on this split's states, with \
                 fresh Adam moments"
            },
            "init_fit_bits_per_target": probe_fit.init_fit_bits,
            "init_dev_bits_per_target": probe_fit.init_dev_bits,
            "optimizer": "Adam(beta1 0.9, beta2 0.999, eps 1e-8), one step per minibatch on the \
                          batch-mean gradient",
            "epochs": args.probe_epochs,
            "lr": args.probe_lr,
            "seed": args.probe_seed,
            "minibatch": PROBE_BATCH,
            "batch_order": "every fit position once per epoch, in minibatches drawn without \
                            replacement from a per-epoch shuffle of the fit population",
            "loss_unit": "NLL in bits per target (loss / ln 2), numerically stable softmax",
            "per_epoch": probe_fit.per_epoch.iter().enumerate().map(|(e, (f, d))| serde_json::json!({
                "epoch": e + 1, "fit_bits_per_target": f, "dev_bits_per_target": d,
            })).collect::<Vec<_>>(),
            "final_dev_bits_per_target": probe_dev,
            "final_fit_bits_per_target": probe_fit_last,
            "weights_sha256": probe_fit.probe.weights_sha256(),
            "weights_sha256_layout": "k (u32 LE), h_dim (u32 LE), then W row-major and b (f64 LE)",
            "last_epoch_fit_delta_bits": if probe_fit_delta.is_finite() { serde_json::json!(probe_fit_delta) } else { serde_json::json!(null) },
            "fit_converged": probe_converged,
            "converged_rule": "the last epoch changes the fit loss by less than 0.01 bits/target",
            "convergence_note": if probe_converged {
                "the fit loss is not still falling materially at the last epoch".to_string()
            } else {
                format!("the fit loss is still falling by {probe_fit_delta:.4} bits/target at the last \
                         epoch, so the probe dev number is a lower bound on what this readout would \
                         reach with a longer schedule")
            },
        },
        "null_probe": {
            "definition": "the identical fit on the same recorded states with the fit target labels \
                           permuted by the probe seed, evaluated on the real development pairs",
            "permutation_seed": args.probe_seed ^ 0x9E37_79B9_7F4A_7C15,
            "run": !args.probe_skip_null,
            "per_epoch": null_fit.as_ref().map(|f| f.per_epoch.iter().enumerate().map(|(e, (fb, d))| serde_json::json!({
                "epoch": e + 1, "fit_bits_per_target_permuted": fb, "real_dev_bits_per_target": d,
            })).collect::<Vec<_>>()).unwrap_or_default(),
            "init_fit_bits_per_target_permuted": null_fit.as_ref().map(|f| f.init_fit_bits),
            "init_dev_bits_per_target": null_fit.as_ref().map(|f| f.init_dev_bits),
            "dev_bits_per_target": null_dev,
            "weights_sha256": null_fit.as_ref().map(|f| f.probe.weights_sha256()),
            "unigram_over_k_renormalised_dev_bits_per_target": dev_uni_topk.restricted(),
            "null_minus_unigram_over_k": null_gap,
            "leak_margin_bits": NULL_LEAK_BITS,
            "leaking": if null_leaks { Some(true) } else { null_dev.map(|_| false) },
            "not_run_in_this_attempt": args.probe_skip_null,
            "not_run_declaration": if args.probe_skip_null {
                serde_json::json!("--probe-skip-null was set: the null control was NOT run in this \
                                   attempt, so this receipt carries no leak check of its own. The \
                                   field is present and marked run=false rather than omitted.")
            } else {
                serde_json::json!(null)
            },
            "validated_elsewhere": if args.probe_skip_null {
                serde_json::json!({
                    "olx_probe_3": {
                        "root": "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/olx-probe-3",
                        "k": 1024,
                        "null_dev_bits_per_target": 8.5662034244338,
                        "unigram_over_k_renormalised_dev_bits_per_target": 8.526355693409007,
                        "null_minus_unigram_over_k": 0.039847731024792665,
                        "leaking": false,
                    },
                    "smoke_k64": {
                        "root": "/tmp/olx-sp-smoke-2 (temporary; the sealed receipt is not under the \
                                 owner's model directory)",
                        "k": 64,
                        "null_dev_bits_per_target": 5.6089,
                        "unigram_over_k_renormalised_dev_bits_per_target": 5.6009,
                        "null_minus_unigram_over_k": 0.0080,
                        "leaking": false,
                    },
                    "declaration": "the identical null machinery (permuted labels, its own ridge \
                                    initialisation, the same epochs and lr) reached the unigram-over-k \
                                    level at a larger and at a smaller K, so skipping it here removes no \
                                    validated coverage of the instrument; it does mean this attempt's \
                                    own headline number is unaccompanied by a leak check.",
                })
            } else {
                serde_json::json!(null)
            },
            "note": "a null probe at the unigram-over-k level is the expected outcome: the probe can \
                     only learn the permuted labels' marginal, and the k-class softmax renormalises \
                     the unigram over the restricted set. A null probe materially below that level \
                     would mean the instrument leaks information it should not have",
        },
        "references_dev": {
            "artifact_served_readout": dev_served.json(),
            "tuned_two_token_count": dev_cnt.json(),
            "fit_only_unigram": dev_uni.json(),
            "unigram_renormalised_over_k": dev_uni_topk.json(),
        },
        "references_fit": {
            "artifact_served_readout": fit_served.json(),
            "tuned_two_token_count": fit_cnt.json(),
            "fit_only_unigram": fit_uni.json(),
        },
        "declared": {
            "probe_is_a_diagnostic": "the float probe is an offline diagnostic instrument over states \
                                      the artifact itself recorded. It is fitted and evaluated in f64 \
                                      outside serving, is never serialised into an artifact, never \
                                      loaded by a session and never consulted while serving. Nothing \
                                      here is model capability, and no float path is proposed for \
                                      serving",
            "state_identity": "h is the exact vector the artifact's readout consumes at that decision, \
                               recorded by the same walk score_example performs; the probe's dev loss \
                               is therefore an upper bound on what any readout can achieve from this \
                               state",
            "normalisation_caveat": "the probe's softmax is normalised over the k retained classes \
                                     while the count and unigram references are normalised over all \
                                     4096 tokens, so the probe starts with the mass the restriction \
                                     removes (fit unigram mass in the set is recorded above). The \
                                     null probe's unigram-over-k level is the leak-free comparison and \
                                     includes that same advantage",
            "readout_definition": "the artifact's own served readout loss is the served integer path: \
                                   bits_generate of score_example, whose legality set and dyadic score \
                                   scale are unchanged. Its unrestricted development value is the \
                                   stored arm score",
        },
        "timings": {
            "record_served_states_seconds": rec_secs,
            "nearest_embedding_decode_seconds": embed_secs,
            "ridge_readout_seconds": ridge_secs,
            "softmax_refinement_seconds": probe_secs,
            "null_probe_seconds": if args.probe_skip_null { serde_json::json!(null) } else { serde_json::json!(null_secs) },
            "servable_repricing_seconds": reprice_secs,
            "decode_ridge_seconds": decode_secs,
        },
        "elapsed_seconds": started.elapsed().as_secs_f64(),
    });
    write_checked(
        &root,
        "receipt.json",
        serde_json::to_string_pretty(&receipt)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "8. sealed {} with {} unlisted files; total {:.1}s",
        root.display(),
        unlisted.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    if args.probe_words {
        let bytes = match std::fs::read(&args.tokenizer) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: tokenizer: {e}");
                return ExitCode::from(1);
            }
        };
        match derive_tokenizer(&bytes, VOCAB) {
            Ok(t) => probe_words(&t),
            Err(e) => {
                eprintln!("error: derive: {e}");
                return ExitCode::from(1);
            }
        }
        return ExitCode::SUCCESS;
    }
    if let Some(root) = args.replay.clone() {
        return match replay_mode(&root, &args.tokenizer) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: replay: {e}");
                ExitCode::from(1)
            }
        };
    }
    if let Some(src) = args.audit.clone() {
        return match audit_mode(&args, &src) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: audit: {e}");
                ExitCode::from(1)
            }
        };
    }
    if let Some(root) = args.local_channel.clone() {
        return match local_channel_mode(&args, &root) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: local-channel: {e}");
                ExitCode::from(1)
            }
        };
    }
    if args.condition.is_some() {
        return match condition_mode(&args) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: condition: {e}");
                ExitCode::from(1)
            }
        };
    }
    if args.state_probe.is_some() {
        return match state_probe_mode(&args) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: state-probe: {e}");
                ExitCode::from(1)
            }
        };
    }
    match run(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// A small hand-built artifact whose embedding, recurrent, fact and readout maps are all non-zero,
    /// so the served state actually moves between decisions and the policy's own choice depends on it.
    /// The control below is only informative on such a model: on a model with no readout dependence on
    /// `h`, deciding at the right and the wrong state would agree equally often.
    fn probe_control_model(vocab: usize, h: usize) -> Result<TlModel, String> {
        let sign = |i: usize, salt: u64| -> f32 {
            let mut s = (i as u64)
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(salt);
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            if s & 1 == 0 {
                1.0
            } else {
                -1.0
            }
        };
        let cfg = TlConfig {
            vocab,
            h_dim: h,
            h_clamp: 256,
            m_clamp: 4096,
            recurrent_shift: 3,
            score_shift: 4,
        };
        let e: Vec<f32> = (0..(vocab + 1) * h).map(|i| sign(i, 0xE)).collect();
        let wi: Vec<f32> = (0..h * (h + TL_F_DIM)).map(|i| sign(i, 0x1)).collect();
        let wh: Vec<f32> = (0..h * h).map(|i| sign(i, 0x2)).collect();
        let wf: Vec<f32> = (0..h * (h + TL_F_DIM + TL_EVENTS))
            .map(|i| sign(i, 0x3))
            .collect();
        let wo: Vec<f32> = (0..(vocab + 2) * (2 * h + TL_F_DIM + TL_EVENTS))
            .map(|i| sign(i, 0x4))
            .collect();
        TlModel::build(
            &cfg,
            &e,
            &wi,
            &wh,
            &wf,
            &wo,
            &vec![0.0; h],
            &vec![0.0; vocab + 2],
        )
    }

    /// The policy's own choice at each recorded state, with the state displaced by `shift` positions in
    /// the recorded sequence and, optionally, the event replaced by a bare `OBSERVE`.
    ///
    /// Returns `(decision index, supervised action, chosen action)` per `Generate` decision for which a
    /// state is available.
    fn choices_at(
        model: &TlModel,
        ex: &TlExample,
        m: &[i32],
        f: &[i32],
        states: &[ServedState],
        shift: usize,
        force_observe: bool,
    ) -> Vec<(usize, TlAction, TlAction)> {
        let mut out = Vec::new();
        let mut step = 0usize;
        for i in 0..ex.actions.len() {
            if !matches!(ex.actions[i], TlAction::Generate(_)) {
                continue;
            }
            let event = if force_observe {
                TL_EV_OBSERVE
            } else {
                ex.prior_event(i)
            };
            if let Some(s) = states.get(step + shift) {
                let chosen = model.decide(&s.h, m, f, event, ex.copy_legal(i, &[]));
                out.push((i, ex.actions[i], chosen));
            }
            step += 1;
        }
        out
    }

    /// `score_example`'s own per-decision view of one example, as the indicator of whether each
    /// decision was the action it chose.
    ///
    /// The walk up to step `i` depends only on the actions before it, so `score_example` on the prefix
    /// of length `i + 1` makes exactly the same decisions as the full example up to step `i`, and
    /// `correct(prefix i + 1) - correct(prefix i)` is exactly "decision `i` was reproduced". This
    /// recovers the served path's own choice per decision through public API only.
    fn served_decision_indicators(model: &TlModel, ex: &TlExample) -> Vec<bool> {
        let mut out = Vec::with_capacity(ex.actions.len());
        let mut previous = 0usize;
        for k in 1..=ex.actions.len() {
            let mut prefix = ex.clone();
            prefix.actions.truncate(k);
            let correct = model.score_example(&prefix).correct;
            assert!(
                correct == previous || correct == previous + 1,
                "one more decision adds at most one correct action"
            );
            out.push(correct > previous);
            previous = correct;
        }
        out
    }

    /// The instrument reads the served state at the right step, on the right event, or it does not.
    ///
    /// `record_example_served_states` must return, at every `Generate` decision, exactly the state
    /// `TlModel::score_example` makes that decision on. The check is the one the served path itself
    /// exposes: deciding at each recorded state must reproduce `score_example`'s own chosen action for
    /// that decision — recovered per decision from action prefixes — and the reproduced count must
    /// equal its own `correct`. Negative controls then show the check has power over the two delicate
    /// parts of the walk: the alignment of a state with its decision, and the advance across a `Stop`
    /// that is never recorded. Every control here is a property of this fixed model and script rather
    /// than a sampled result.
    #[test]
    fn recorded_served_states_reproduce_score_example_decisions() {
        let model = probe_control_model(24, 8).expect("hand model builds");
        let cases = [
            // Prose-like: eight Generate decisions and the document-terminal `Stop`.
            TlExample {
                sel: Vec::new(),
                res: Vec::new(),
                facts: SlFacts::default(),
                observed: vec![3, 4, 5, 6],
                actions: vec![
                    TlAction::Generate(7),
                    TlAction::Generate(11),
                    TlAction::Generate(2),
                    TlAction::Generate(19),
                    TlAction::Generate(4),
                    TlAction::Generate(13),
                    TlAction::Generate(6),
                    TlAction::Generate(21),
                    TlAction::Stop,
                ],
                weight: 1.0,
                doc: 0,
                grounded: false,
                terminal_stop: true,
            },
            // A mid-script `Stop`: never recorded, but it advances the state that the next decision
            // reads, so an instrument that skipped its advance is caught here.
            TlExample {
                sel: Vec::new(),
                res: Vec::new(),
                facts: SlFacts::default(),
                observed: vec![1, 2],
                actions: vec![
                    TlAction::Generate(9),
                    TlAction::Stop,
                    TlAction::Generate(5),
                    TlAction::Generate(17),
                ],
                weight: 1.0,
                doc: 1,
                grounded: false,
                terminal_stop: false,
            },
        ];
        let m = vec![0i32; model.h_dim];
        let f = model.typed_block(&[], &[], SlFacts::default());
        for (case, ex) in cases.iter().enumerate() {
            let states = record_example_served_states(&model, ex);
            let generates = ex
                .actions
                .iter()
                .filter(|a| matches!(a, TlAction::Generate(_)))
                .count();
            assert_eq!(
                states.len(),
                generates,
                "case {case}: one served state per Generate decision, Stop decisions not recorded"
            );
            assert!(
                states.iter().all(|s| s.h.len() == model.h_dim),
                "case {case}: a recorded state must be a full served state vector"
            );
            // For ordinary prose every recorded decision follows a Generate, so `prior_event(i)` and
            // the action's own event `actions[i].event()` coincide on every one of them: that event
            // convention is not observable on a prose-like script, and only a `Stop` followed by a
            // decision can distinguish the two. Asserted here rather than assumed.
            for i in 1..ex.actions.len() {
                if matches!(ex.actions[i], TlAction::Generate(_))
                    && matches!(ex.actions[i - 1], TlAction::Generate(_))
                {
                    assert_eq!(
                        ex.prior_event(i),
                        ex.actions[i].event(),
                        "case {case}: consecutive Generate decisions share one event"
                    );
                }
            }
            // The required control, decision by decision.
            let served = served_decision_indicators(&model, ex);
            let faithful = choices_at(&model, ex, &m, &f, &states, 0, false);
            assert_eq!(faithful.len(), generates, "case {case}");
            let mut reproduced = 0usize;
            for (i, supervised, chosen) in faithful.iter() {
                assert_eq!(
                    chosen == supervised,
                    served[*i],
                    "case {case}: deciding at the recorded state of decision {i} chose {chosen:?}, \
                     where score_example chose {supervised:?} itself as the reproduced action"
                );
                if chosen == supervised {
                    reproduced += 1;
                }
            }
            assert_eq!(
                reproduced,
                model.score_example(ex).correct,
                "case {case}: the decisions reproduced at the recorded states must equal \
                 score_example's own count of correct actions"
            );
            let distinct: BTreeSet<String> =
                faithful.iter().map(|(_, _, c)| format!("{c:?}")).collect();
            assert!(
                distinct.len() > 1,
                "case {case}: the control is only informative when the policy's choice depends on the \
                 state; this model chose {distinct:?} at every recorded state"
            );
            // Negative control 1: a state used one recorded step out of order must change a decision.
            let shifted = choices_at(&model, ex, &m, &f, &states, 1, false);
            assert!(
                shifted.iter().any(|(i, _, shifted_choice)| faithful
                    .iter()
                    .any(|(j, _, c)| i == j && c != shifted_choice)),
                "case {case}: displacing the recorded state by one step changed nothing, so this \
                 control cannot detect a misaligned walk"
            );
            // Negative control 2: after a mid-script `Stop`, the recorded state is the state advanced
            // across that unrecorded `Stop`; a walk that skipped the advance must change a decision.
            for i in 1..ex.actions.len() {
                if !matches!(ex.actions[i - 1], TlAction::Stop) {
                    continue;
                }
                // `step` is the index of this decision among the scored decisions, so it indexes the
                // recorded states directly; the last Generate before the `Stop` is the previous one.
                let previous = (0..i)
                    .rev()
                    .find(|j| matches!(ex.actions[*j], TlAction::Generate(_)));
                let Some(previous) = previous else { continue };
                let step = ex.actions[..i]
                    .iter()
                    .filter(|a| matches!(a, TlAction::Generate(_)))
                    .count();
                if step == 0 || step >= states.len() {
                    continue;
                }
                let emitted = match ex.actions[previous] {
                    TlAction::Generate(v) => Some(v),
                    _ => None,
                };
                let before_stop =
                    model.transition(&states[step - 1].h, TL_EV_GENERATE, emitted, &m, &f);
                let after_stop =
                    model.transition(&before_stop, TlAction::Stop.event(), None, &m, &f);
                // The recorded state of this decision must be exactly the state advanced across the
                // unrecorded `Stop`: an instrument that lost that advance records `before_stop` here.
                assert_eq!(
                    after_stop, states[step].h,
                    "case {case}: the state recorded at decision {i} must be the state advanced \
                     across the unrecorded Stop"
                );
                let skipped = model.decide(
                    &before_stop,
                    &m,
                    &f,
                    ex.prior_event(i),
                    ex.copy_legal(i, &[]),
                );
                let advanced = model.decide(
                    &after_stop,
                    &m,
                    &f,
                    ex.prior_event(i),
                    ex.copy_legal(i, &[]),
                );
                assert_ne!(
                    advanced, skipped,
                    "case {case}: skipping the unrecorded Stop's advance changed nothing, so this \
                     control cannot detect a lost Stop advance"
                );
            }
            // Negative control 3: on the script with a `Stop` followed by a decision, that decision
            // reads event `STOP`; forcing a bare `OBSERVE` there must change it.
            let wrong_event = choices_at(&model, ex, &m, &f, &states, 0, true);
            let replayed: Vec<(usize, TlAction, TlAction)> = wrong_event
                .iter()
                .filter(|(i, _, _)| ex.prior_event(*i) != TL_EV_OBSERVE)
                .cloned()
                .collect();
            if case == 1 {
                assert!(
                    !replayed.is_empty(),
                    "case {case}: the fixture must contain a decision after a non-OBSERVE event"
                );
                assert!(
                    replayed.iter().any(|(i, _, wrong_choice)| faithful
                        .iter()
                        .any(|(j, _, c)| i == j && c != wrong_choice)),
                    "case {case}: forcing the event to OBSERVE changed nothing, so this control \
                     cannot detect a wrong prior event"
                );
            }
        }
    }

    /// A tiny ridge regression whose closed-form solution is checkable by hand.
    ///
    /// Four observations, `d = 2` features and `k = 2` classes: `x = (1,0)` twice labelled class 0 and
    /// `x = (0,1)` twice labelled class 1. Centring gives `HtH - n x̄x̄ᵀ = [[1,-1],[-1,1]]` and
    /// `HtY - n x̄ȳᵀ = [[1,-1],[-1,1]]`, so at `lambda = 1` the system is `[[2,-1],[-1,2]] W = B` and
    /// `W = [[1/3,-1/3],[-1/3,1/3]]`, `b = ȳ - Wᵀx̄ = (1/2, 1/2)`. Every number below is that
    /// hand-computed answer, and the two points must be classified correctly by the resulting readout.
    #[test]
    fn ridge_solve_reproduces_a_hand_computed_solution() {
        let (d, k) = (2usize, 2usize);
        let mut moments = Moments::zeros(d, k);
        for (x, class) in [
            (vec![1.0, 0.0], 0usize),
            (vec![1.0, 0.0], 0usize),
            (vec![0.0, 1.0], 1usize),
            (vec![0.0, 1.0], 1usize),
        ] {
            moments.add(&x, class);
        }
        assert_eq!(moments.n, 4.0);
        let (w, b) = solve_moments(&moments, 1.0).expect("the ridge system is positive definite");
        let expect_w = [1.0 / 3.0, -1.0 / 3.0, -1.0 / 3.0, 1.0 / 3.0];
        for (got, want) in w.iter().zip(expect_w.iter()) {
            assert!(
                (got - want).abs() < 1e-12,
                "W must equal the hand solution: got {w:?}, want {expect_w:?}"
            );
        }
        for value in b.iter() {
            assert!(
                (value - 0.5).abs() < 1e-12,
                "b must equal ybar - W^T xbar = (1/2, 1/2): got {b:?}"
            );
        }
        // The readout classifies both points correctly, which is what the machinery is used for.
        let readout = RidgeReadout {
            k,
            d,
            w: transpose_to_class_major(&w, d, k),
            b: b.clone(),
            lambda_rel: 1.0,
            lambda_abs: 1.0,
            temperature: 1.0,
            selection_n: 2,
            selection_bits: f64::NAN,
            lambda_table: Vec::new(),
        };
        let x = [1.0, 0.0, 0.0, 1.0];
        let y = [0usize, 1usize];
        let mut scratch = vec![0.0; k];
        assert_eq!(readout.top1(&x, &y, &[0, 1], &mut scratch), 1.0);
        assert!(readout.nll_bits(&x, &y, &[0, 1], &mut scratch) < 0.7);
    }

    /// The ridge solve must report a bad system instead of panicking on it (the runner's contract is
    /// `Result`, never an unwrap on a recoverable path).
    #[test]
    fn ridge_solve_rejects_a_non_positive_definite_system() {
        // A symmetric matrix with a negative eigenvalue: pivots must fail, not produce NaN.
        let a = vec![1.0, 3.0, 3.0, 1.0];
        let b = vec![1.0, 0.0];
        let err = cholesky_solve(&a, 2, &b, 1).expect_err("a non-PD system must be an error");
        assert!(err.contains("positive definite"), "{err}");
    }

    /// The recorder's `cur`/`prev` bookkeeping, checked against the walk's own transition.
    ///
    /// The two tokens are read off the consumption history, so an off-by-one would show up as either a
    /// wrong token or a wrong `None`. The cases below cover the three ways the history can start or be
    /// interrupted: a full observed prefix, a `Stop` that consumed nothing, and an empty prefix where
    /// the state really is the window head and both entries must be `None`. The control is mechanical:
    /// re-running `transition` from the previous recorded state with the recorded token must land
    /// exactly on the next recorded state, which pins the token to the transition that consumed it.
    #[test]
    fn recorder_bookkeeping_pins_the_consumed_tokens() {
        let model = probe_control_model(24, 8).expect("hand model builds");
        let m = vec![0i32; model.h_dim];
        let f = model.typed_block(&[], &[], SlFacts::default());
        // (observed, actions, expected cur/prev per Generate decision)
        let cases: Vec<(Vec<u32>, Vec<TlAction>, Vec<(Option<u32>, Option<u32>)>)> = vec![
            (
                vec![11, 12, 13],
                vec![
                    TlAction::Generate(4),
                    TlAction::Generate(5),
                    TlAction::Generate(6),
                ],
                vec![
                    (Some(13), Some(12)),
                    (Some(4), Some(13)),
                    (Some(5), Some(4)),
                ],
            ),
            (
                vec![21, 22],
                vec![TlAction::Generate(7), TlAction::Stop, TlAction::Generate(8)],
                // After the Stop nothing was consumed, so `cur` is None and `prev` is the generated 7.
                vec![(Some(22), Some(21)), (None, Some(7))],
            ),
            (Vec::new(), vec![TlAction::Generate(9)], vec![(None, None)]),
        ];
        for (case, (observed, actions, want)) in cases.into_iter().enumerate() {
            let ex = TlExample {
                sel: Vec::new(),
                res: Vec::new(),
                facts: SlFacts::default(),
                observed,
                actions,
                weight: 1.0,
                doc: case,
                grounded: false,
                terminal_stop: false,
            };
            let states = record_example_served_states(&model, &ex);
            assert_eq!(states.len(), want.len(), "case {case}");
            for (i, (cur, prev)) in want.iter().enumerate() {
                assert_eq!(
                    (states[i].cur, states[i].prev),
                    (*cur, *prev),
                    "case {case}: decision {i} must carry the consumed tokens ({cur:?}, {prev:?})"
                );
            }
            // Mechanical control: re-derive the whole consumed-token history from the action script on
            // an independent path and check both that each recorded state is the state at its decision
            // and that `cur`/`prev` are exactly the tokens the last two transitions consumed. A `Stop`
            // consumes nothing, so a recorder that advanced its history there fails here, and a
            // recorder reading the wrong history slot fails on `prev`.
            let mut h = model.init_state(&m, &f);
            let mut history: Vec<Option<u32>> = Vec::new();
            for t in &ex.observed {
                h = model.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
                history.push(Some(*t));
            }
            let mut recorded = 0usize;
            for i in 0..ex.actions.len() {
                if matches!(ex.actions[i], TlAction::Generate(_)) {
                    let n = history.len();
                    assert_eq!(
                        h, states[recorded].h,
                        "case {case}: decision {i} must be scored on the replayed state"
                    );
                    assert_eq!(
                        states[recorded].cur,
                        history.get(n.wrapping_sub(1)).copied().flatten(),
                        "case {case}: decision {i} cur must be the token the last transition consumed"
                    );
                    assert_eq!(
                        states[recorded].prev,
                        history.get(n.wrapping_sub(2)).copied().flatten(),
                        "case {case}: decision {i} prev must be the token before that"
                    );
                    recorded += 1;
                }
                let emitted = match ex.actions[i] {
                    TlAction::Generate(v) => Some(v),
                    TlAction::Copy => ex.emitted(i, &[]),
                    TlAction::Stop => None,
                };
                h = model.transition(&h, ex.actions[i].event(), emitted, &m, &f);
                history.push(emitted);
            }
            assert_eq!(
                recorded,
                states.len(),
                "case {case}: every decision was replayed"
            );
        }
    }

    /// The artifact initialisation is a rearrangement, not an approximation: at every decision the
    /// initialised probe's logits must equal the artifact's own `readout` logits at the artifact's
    /// declared dyadic scale, entry by entry over the restricted classes, and its argmax must therefore
    /// agree with `TlModel::decide` whenever the served decision is a `Generate` of a restricted class.
    ///
    /// The fixture takes every vocabulary row as a class so the probe's class set is the artifact's whole
    /// generation vocabulary, and exercises several event symbols so the event block is really used.
    #[test]
    fn artifact_initialised_probe_reproduces_the_artifact_readout() {
        let model = probe_control_model(24, 8).expect("hand model builds");
        let top_ids: Vec<u32> = (0..model.vocab as u32).collect();
        let f_block = model.typed_block(&[], &[], SlFacts::default());
        let init = artifact_readout_init(&model, &top_ids, &f_block, model.h_clamp)
            .expect("the artifact readout rearranges");
        let rows: Vec<usize> = top_ids.iter().map(|t| model.token_row(*t)).collect();
        assert_eq!(init.state_columns, model.h_dim);
        assert_eq!(init.m_columns_dropped, model.h_dim);
        assert_eq!(init.event_columns, TL_EVENTS);
        assert!(
            init.f_columns_folded >= 1,
            "the declared typed block is not the zero block, so at least one f column must be folded"
        );
        let m = vec![0i32; model.h_dim];
        // A few states built by the served walk itself, with every event symbol represented.
        let mut states = Vec::new();
        for event in 0..TL_EVENTS {
            let mut h = model.init_state(&m, &f_block);
            for t in [3u32, 5, 8] {
                h = model.transition(&h, event, Some(t), &m, &f_block);
            }
            states.push(ServedState {
                h,
                target: 4,
                doc: 0,
                bits: 0.0,
                event,
                cur: Some(5),
                prev: Some(3),
            });
        }
        let idx: Vec<usize> = (0..states.len()).collect();
        let (checked, compared, worst) =
            verify_artifact_init(&model, &init, &rows, &states, &idx, &f_block)
                .expect("the control runs");
        assert_eq!(checked, states.len());
        assert_eq!(compared, states.len() * model.vocab);
        assert!(
            worst < 1e-9,
            "the initialised readout must equal the artifact's logits exactly, worst deviation {worst}"
        );
        // Argmax agreement with the served decision, wherever that decision is a Generate.
        let mut agreements = 0usize;
        let mut generations = 0usize;
        for s in states.iter() {
            let served = model.readout(&s.h, &m, &f_block, s.event);
            let mut chosen = 0usize;
            for c in 0..model.vocab {
                if served[rows[c]] > served[rows[chosen]] {
                    chosen = c;
                }
            }
            let decision = model.decide(&s.h, &m, &f_block, s.event, false);
            if let TlAction::Generate(v) = decision {
                generations += 1;
                assert_eq!(
                    chosen, v as usize,
                    "the initialised probe's argmax must agree with the served decision"
                );
                agreements += 1;
            }
        }
        assert_eq!(agreements, generations);
        assert!(
            generations > 0,
            "the fixture must produce at least one Generate decision to compare against"
        );
    }
}
