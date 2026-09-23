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

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use sha2::{Digest, Sha256};

use uor_r4_core::native_geometric::learner::prior_learning::PriorCore;
use uor_r4_core::native_geometric::learner::realtext_support::{
    ctx2, family_p, reconstruct_corpus, tune_lambdas, Cond, Split, Uni, VOCAB,
};
use uor_r4_core::native_geometric::learner::state_lexical::SlFacts;
use uor_r4_core::native_geometric::learner::transferable_lexical::{
    state_digest, TlAction, TlConfig, TlExample, TlModel, TlTrainConfig, TlTrainer, TL_EVENTS,
    TL_EV_COPY, TL_EV_GENERATE, TL_EV_OBSERVE, TL_F_DIM,
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
    let mut artifact = None;
    let mut objective = String::from("window");
    let mut curriculum = String::from("delivered");
    let mut prose_slots = None;
    let mut ground_slots = None;
    let mut ground_weight = None;
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let k = argv[i].as_str();
        if k == "--probe-words" {
            probe_words = true;
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
        artifact,
        objective,
        curriculum,
        prose_slots,
        ground_slots,
        ground_weight,
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
    match run(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
