//! Source-separated loaded-native sparse-read gate. One frozen artifact, one fitted four-bit gate.
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::realtext_support::{
    ctx2, family_p, reconstruct_corpus, tune_lambdas, Cond, Split, Uni, VOCAB,
};
use uor_r4_core::native_geometric::learner::sparse_native_read::{
    Candidate, GateExample, PairMemory, SparsePolicy, SparseReadGate, SparseSession,
};
use uor_r4_core::native_geometric::learner::state_lexical::SlFacts;
use uor_r4_core::native_geometric::learner::transferable_lexical::{TlModel, TL_EV_OBSERVE};
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const MODEL_SHA: &str = "69e8b88b41bb09d9149be1ac7e83e5e50db2974f470dce05405a55cc1f0bfb7e";
const TOKENIZER_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
const PREFIX: usize = 8;
const MARKDOWN_FIT: usize = 12;
const MARKDOWN_TUNE: usize = 4;
const MARKDOWN_TOKENS: usize = 512;
const OTHER_TOKENS: usize = 1024;
const PROSE_TOKENS: usize = 1024;
const MAX_NEW: usize = 8;

#[derive(Clone, Serialize)]
struct Doc {
    family: String,
    path: String,
    sha256: String,
    original_bytes: usize,
    tokens: Vec<u32>,
}

#[derive(Clone, Serialize)]
struct Row {
    doc: usize,
    position: usize,
    target: u32,
    model_top: u32,
    latest_top: u32,
    gate_top: u32,
    count_top: u32,
    recent: Option<Candidate>,
    previous_different: Option<Candidate>,
    selected_source: Option<usize>,
}

#[derive(Default, Serialize)]
struct Metrics {
    n: usize,
    model_correct: usize,
    latest_correct: usize,
    gate_correct: usize,
    count_correct: usize,
    recent_hits: usize,
    older_available: usize,
    learned_reads: usize,
    learned_older_reads: usize,
    older_rescues_from_recent: usize,
    gate_rescues_from_model: usize,
    gate_harms_to_model: usize,
    distant_hits: usize,
    candidate_slot_reads: usize,
}

impl Metrics {
    fn add(&mut self, row: &Row) {
        self.n += 1;
        self.model_correct += usize::from(row.model_top == row.target);
        self.latest_correct += usize::from(row.latest_top == row.target);
        self.gate_correct += usize::from(row.gate_top == row.target);
        self.count_correct += usize::from(row.count_top == row.target);
        self.recent_hits += usize::from(row.recent.is_some());
        self.older_available += usize::from(row.previous_different.is_some());
        self.candidate_slot_reads +=
            usize::from(row.recent.is_some()) + usize::from(row.previous_different.is_some());
        self.distant_hits += usize::from(row.recent.is_some_and(|c| c.age >= 64));
        self.learned_reads += usize::from(row.selected_source.is_some());
        self.learned_older_reads += usize::from(row.selected_source == Some(1));
        self.older_rescues_from_recent += usize::from(
            row.selected_source == Some(1)
                && row.gate_top == row.target
                && row.latest_top != row.target,
        );
        self.gate_rescues_from_model +=
            usize::from(row.gate_top == row.target && row.model_top != row.target);
        self.gate_harms_to_model +=
            usize::from(row.gate_top != row.target && row.model_top == row.target);
    }
}

struct Counts {
    c1: Cond,
    c2: Cond,
    uni: Uni,
    one: HashMap<u32, BTreeSet<u32>>,
    two: HashMap<(u32, u32), BTreeSet<u32>>,
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn read_bound(path: &Path, expected: &str) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if sha(&bytes) != expected {
        return Err(format!("{}: SHA-256 mismatch", path.display()));
    }
    Ok(bytes)
}

fn one_doc(
    path: &Path,
    family: &str,
    worktree: &Path,
    tok: &HfBpeTokenizer,
    cap: usize,
    gutenberg_body: bool,
) -> Result<Doc, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let text = if gutenberg_body {
        let start = text
            .find("*** START OF")
            .and_then(|p| text[p..].find('\n').map(|i| p + i + 1))
            .ok_or("missing Gutenberg start marker")?;
        let end = text[start..]
            .find("*** END OF")
            .map(|i| start + i)
            .ok_or("missing Gutenberg end marker")?;
        &text[start..end]
    } else {
        text
    };
    let mut tokens = tok.encode(text);
    tokens.truncate(cap);
    if tokens.len() <= PREFIX {
        return Err(format!("{}: too few tokens", path.display()));
    }
    Ok(Doc {
        family: family.into(),
        path: path
            .strip_prefix(worktree)
            .unwrap_or(path)
            .display()
            .to_string(),
        sha256: sha(&bytes),
        original_bytes: bytes.len(),
        tokens,
    })
}

fn corpus(
    worktree: &Path,
    inputs: &Path,
    tok: &HfBpeTokenizer,
) -> Result<(Vec<Doc>, Vec<Doc>, Vec<Doc>), String> {
    let (records, _, _) = reconstruct_corpus(&worktree.join("docs"));
    let mut fit = Vec::new();
    let mut tune = Vec::new();
    for record in records {
        if record.path.ends_with("resource-ledger-2026-09-19.md")
            || record.path.contains("native-sparse-read-")
        {
            continue;
        }
        let dest = match record.split {
            Split::Fit if fit.len() < MARKDOWN_FIT => &mut fit,
            Split::Tune if tune.len() < MARKDOWN_TUNE => &mut tune,
            _ => continue,
        };
        let mut tokens = tok.encode(&record.text);
        tokens.truncate(MARKDOWN_TOKENS);
        if tokens.len() > PREFIX {
            dest.push(Doc {
                family: "project_markdown".into(),
                path: format!("docs/{}", record.path),
                sha256: hex::encode(record.sha256),
                original_bytes: record.bytes,
                tokens,
            });
        }
    }
    if fit.len() != MARKDOWN_FIT || tune.len() != MARKDOWN_TUNE {
        return Err("insufficient source-split Markdown files".into());
    }
    for name in ["lowbit.rs", "language_transport.rs"] {
        fit.push(one_doc(
            &worktree
                .join("crates/uor-r4-core/src/native_geometric/learner")
                .join(name),
            "rust_fit",
            worktree,
            tok,
            OTHER_TOKENS,
            false,
        )?);
    }
    tune.push(one_doc(
        &worktree.join("crates/uor-r4-core/src/native_geometric/learner/group_table.rs"),
        "rust_tune",
        worktree,
        tok,
        OTHER_TOKENS,
        false,
    )?);
    let dev = vec![
        one_doc(
            &inputs.join("oz-55.txt"),
            "prose",
            worktree,
            tok,
            PROSE_TOKENS,
            true,
        )?,
        one_doc(
            &inputs.join("garden-113.txt"),
            "prose",
            worktree,
            tok,
            PROSE_TOKENS,
            true,
        )?,
        one_doc(
            &worktree.join("crates/uor-r4-router/src/geometry.rs"),
            "rust_code",
            worktree,
            tok,
            OTHER_TOKENS,
            false,
        )?,
        one_doc(
            &worktree.join("crates/uor-r4-router/src/session_signature.rs"),
            "rust_code",
            worktree,
            tok,
            OTHER_TOKENS,
            false,
        )?,
    ];
    Ok((fit, tune, dev))
}

fn fit_counts(docs: &[Doc]) -> Result<Counts, String> {
    let mut counts = Counts {
        c1: Cond::default(),
        c2: Cond::default(),
        uni: Uni {
            counts: vec![0; VOCAB],
            total: 0,
        },
        one: HashMap::new(),
        two: HashMap::new(),
    };
    for doc in docs {
        for (i, &target) in doc.tokens.iter().enumerate() {
            let count = counts
                .uni
                .counts
                .get_mut(target as usize)
                .ok_or("out-of-vocabulary token")?;
            *count += 1;
            counts.uni.total += 1;
            if i >= 2 {
                let previous = doc.tokens[i - 2];
                let current = doc.tokens[i - 1];
                counts.c1.observe(current as u64, target);
                counts
                    .c2
                    .observe(ctx2(previous as usize, current as usize), target);
                counts.one.entry(current).or_default().insert(target);
                counts
                    .two
                    .entry((previous, current))
                    .or_default()
                    .insert(target);
            }
        }
    }
    Ok(counts)
}

fn count_top(counts: &Counts, previous: u32, current: u32, lambdas: (f64, f64)) -> u32 {
    let mut best = counts.uni.argmax();
    let mut score = family_p(
        &counts.c1,
        &counts.c2,
        &counts.uni,
        previous as usize,
        current as usize,
        best,
        lambdas,
    );
    for target in counts
        .one
        .get(&current)
        .into_iter()
        .flatten()
        .chain(counts.two.get(&(previous, current)).into_iter().flatten())
    {
        let p = family_p(
            &counts.c1,
            &counts.c2,
            &counts.uni,
            previous as usize,
            current as usize,
            *target,
            lambdas,
        );
        if p > score || (p == score && *target < best) {
            best = *target;
            score = p;
        }
    }
    best
}

fn model_top(model: &TlModel, h: &[i32], m: &[i32], f: &[i32]) -> u32 {
    let logits = model.readout(h, m, f, TL_EV_OBSERVE);
    let mut best = 0;
    for i in 1..model.vocab {
        if logits[i] > logits[best] {
            best = i;
        }
    }
    best as u32
}

/// Uses the loaded model and only prior observed tokens at each scored position.
fn observed_rows(
    docs: &[Doc],
    model: &TlModel,
    gate: Option<&SparseReadGate>,
    counts: Option<(&Counts, (f64, f64))>,
) -> Vec<Row> {
    let m = vec![0; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let mut rows = Vec::new();
    for (doc_id, doc) in docs.iter().enumerate() {
        let mut h = model.init_state(&m, &f);
        let mut memory = PairMemory::default();
        for (i, &target) in doc.tokens.iter().enumerate() {
            if i >= PREFIX {
                let previous = doc.tokens[i - 2];
                let current = doc.tokens[i - 1];
                let candidates = memory.candidates(previous, current, i);
                let native = model_top(model, &h, &m, &f);
                let selected = gate.and_then(|g| g.choose(current, candidates));
                let count = counts.map_or(native, |(c, l)| count_top(c, previous, current, l));
                rows.push(Row {
                    doc: doc_id,
                    position: i,
                    target,
                    model_top: native,
                    latest_top: candidates[0].map_or(native, |c| c.value),
                    gate_top: selected.map_or(native, |c| c.value),
                    count_top: count,
                    recent: candidates[0],
                    previous_different: candidates[1],
                    selected_source: selected.map(|c| c.source),
                });
            }
            if i >= 2 {
                memory.observe(doc.tokens[i - 2], doc.tokens[i - 1], target, i);
            }
            h = model.transition(&h, TL_EV_OBSERVE, Some(target), &m, &f);
        }
    }
    rows
}

fn examples(rows: &[Row], docs: &[Doc]) -> Vec<GateExample> {
    rows.iter()
        .flat_map(|row| {
            [row.recent, row.previous_different]
                .into_iter()
                .flatten()
                .map(move |candidate| GateExample {
                    candidate,
                    current: docs[row.doc].tokens[row.position - 1],
                    target: row.target,
                    model_top: row.model_top,
                })
        })
        .collect()
}

fn apply_gate(rows: &mut [Row], docs: &[Doc], gate: &SparseReadGate) {
    for row in rows {
        let current = docs[row.doc].tokens[row.position - 1];
        let selected = gate.choose(current, [row.recent, row.previous_different]);
        row.selected_source = selected.map(|c| c.source);
        row.gate_top = selected.map_or(row.model_top, |c| c.value);
    }
}

fn metrics(rows: &[Row]) -> Metrics {
    let mut out = Metrics::default();
    for row in rows {
        out.add(row);
    }
    out
}

fn tune_threshold(rows: &[Row], docs: &[Doc], gate: &mut SparseReadGate) -> Vec<serde_json::Value> {
    let mut trials = Vec::new();
    let mut best = (0usize, usize::MAX, 0i8);
    for threshold in 0..=4 {
        gate.threshold = threshold;
        let mut scored = rows.to_vec();
        apply_gate(&mut scored, docs, gate);
        let result = metrics(&scored);
        trials.push(json!({"threshold":threshold,"correct":result.gate_correct,"reads":result.learned_reads}));
        if result.gate_correct > best.0
            || (result.gate_correct == best.0 && result.learned_reads < best.1)
        {
            best = (result.gate_correct, result.learned_reads, threshold);
        }
    }
    gate.threshold = best.2;
    trials
}

fn count_generate(counts: &Counts, lambdas: (f64, f64), prompt: &[u32]) -> Vec<u32> {
    let mut history = prompt.to_vec();
    let mut output = Vec::new();
    for _ in 0..MAX_NEW {
        let n = history.len();
        if n < 2 {
            break;
        }
        let next = count_top(counts, history[n - 2], history[n - 1], lambdas);
        history.push(next);
        output.push(next);
    }
    output
}

fn generations(
    docs: &[Doc],
    rows: &[Row],
    model: &TlModel,
    gate: &SparseReadGate,
    counts: &Counts,
    lambdas: (f64, f64),
    tok: &HfBpeTokenizer,
) -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>), String> {
    let mut cases = Vec::new();
    let mut edits = Vec::new();
    for (doc_id, doc) in docs.iter().enumerate() {
        let admitted = rows
            .iter()
            .find(|r| r.doc == doc_id && r.position >= 257 && r.selected_source.is_some())
            .map(|r| r.position);
        for (kind, position) in [
            ("fixed_256", Some(256usize)),
            ("first_admitted_after_256", admitted),
        ] {
            let Some(position) = position else {
                cases.push(json!({"doc":doc.path,"prompt_kind":kind,"status":"NONE"}));
                continue;
            };
            let prompt = &doc.tokens[..position.min(doc.tokens.len())];
            let prepared = SparseSession::new(model, prompt)?;
            let mut disabled = prepared.clone();
            let mut latest = prepared.clone();
            let mut learned = prepared.clone();
            let no = disabled.generate(model, SparsePolicy::NoRead, MAX_NEW)?;
            let recent = latest.generate(model, SparsePolicy::Latest, MAX_NEW)?;
            let selected = learned.generate(model, SparsePolicy::Learned(gate), MAX_NEW)?;
            // Independently constructed read-disabled session must match the same NoRead path.
            let mut disabled_again = SparseSession::new(model, prompt)?;
            if disabled_again.generate(model, SparsePolicy::NoRead, MAX_NEW)? != no {
                return Err("read-disabled generation differs from NoRead".into());
            }
            let c = count_generate(counts, lambdas, prompt);
            let truth = &doc.tokens[position..doc.tokens.len().min(position + MAX_NEW)];
            cases.push(
                json!({"doc":doc.path,"family":doc.family,"prompt_kind":kind,
                "position":position,"prompt_last_tokens":prompt[prompt.len().saturating_sub(8)..],
                "truth_tokens":truth,"truth_text":tok.decode(truth),
                "no_read":{"rollout":no,"text":tok.decode(&no.tokens)},
                "latest":{"rollout":recent,"text":tok.decode(&recent.tokens)},
                "learned":{"rollout":selected,"text":tok.decode(&selected.tokens)},
                "count":{"tokens":c,"text":tok.decode(&c)}}),
            );
            if kind == "first_admitted_after_256" {
                let current = prepared.current_token().ok_or("empty prompt")?;
                let original = gate.choose(current, prepared.current_candidates());
                if let Some(candidate) = original.filter(|c| c.value != current) {
                    let mut edited = prepared.clone();
                    let different = (0..model.vocab as u32)
                        .find(|v| {
                            *v != current
                                && *v != candidate.value
                                && prepared.current_candidates()[1 - candidate.source]
                                    .is_none_or(|other| *v != other.value)
                        })
                        .ok_or("no distinct edit token")?;
                    if !edited.replace_current_source(candidate.source, different) {
                        return Err("source edit did not resolve".into());
                    }
                    let after = gate.choose(current, edited.current_candidates());
                    if after.map(|c| c.source) != Some(candidate.source) {
                        return Err("source edit changed fitted route".into());
                    }
                    let mut before = prepared.clone();
                    let mut no_before = prepared.clone();
                    let mut no_after = edited.clone();
                    let before_out = before.generate(model, SparsePolicy::Learned(gate), 1)?;
                    let after_out = edited.generate(model, SparsePolicy::Learned(gate), 1)?;
                    let no_a = no_before.generate(model, SparsePolicy::NoRead, 1)?;
                    let no_b = no_after.generate(model, SparsePolicy::NoRead, 1)?;
                    if no_a.tokens != no_b.tokens {
                        return Err("NoRead changed under source-only edit".into());
                    }
                    edits.push(json!({"doc":doc.path,"position":position,"source":candidate.source,
                        "before_value":candidate.value,"after_value":different,
                        "before_tokens":before_out.tokens,"after_tokens":after_out.tokens,
                        "no_read_before":no_a.tokens,"no_read_after":no_b.tokens,
                        "route_fixed":true,"source_caused_change":before_out.tokens != after_out.tokens}));
                }
            }
        }
    }
    Ok((cases, edits))
}

fn check_causality() -> Result<(), String> {
    let mut memory = PairMemory::default();
    if memory.candidates(1, 2, 2) != [None, None] {
        return Err("pre-write read was nonempty".into());
    }
    memory.observe(1, 2, 3, 2);
    if memory.candidates(1, 2, 3)[0].map(|c| c.value) != Some(3) {
        return Err("first write missing".into());
    }
    memory.observe(1, 2, 4, 5);
    let [recent, old] = memory.candidates(1, 2, 6);
    if recent.map(|c| (c.value, c.age, c.conflicts)) != Some((4, 1, 1))
        || old.map(|c| (c.value, c.age)) != Some((3, 4))
    {
        return Err("previous-different/age semantics wrong".into());
    }
    Ok(())
}

fn write_json(root: &Path, name: &str, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(root.join(name), bytes).map_err(|e| e.to_string())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: native-sparse-read NEW_REPORT_ROOT MODEL.tlx TOKENIZER.json WORKTREE_ROOT INPUT_ROOT".into());
    }
    let root = PathBuf::from(&args[1]);
    let model_path = PathBuf::from(&args[2]);
    let tokenizer_path = PathBuf::from(&args[3]);
    let worktree = PathBuf::from(&args[4]);
    let inputs = PathBuf::from(&args[5]);
    claim(&root).map_err(|e| e.to_string())?;
    check_causality()?;
    let model = TlModel::from_bytes(&read_bound(&model_path, MODEL_SHA)?)?;
    let tok_bytes = read_bound(&tokenizer_path, TOKENIZER_SHA)?;
    let tok = HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes).ok_or("invalid tokenizer")?;
    if model.vocab != tok.vocab_size() || model.vocab != VOCAB {
        return Err("vocabulary mismatch".into());
    }
    let (fit, tune, dev) = corpus(&worktree, &inputs, &tok)?;
    let counts = fit_counts(&fit)?;
    let tune_tokens: Vec<_> = tune.iter().map(|d| d.tokens.clone()).collect();
    let (lambdas, _) = tune_lambdas(&counts.c1, &counts.c2, &counts.uni, &tune_tokens);
    let fit_rows = observed_rows(&fit, &model, None, None);
    let tune_rows = observed_rows(&tune, &model, None, None);
    let fit_identity = serde_json::to_vec(&fit).map_err(|e| e.to_string())?;
    let mut gate = SparseReadGate::fit(
        &examples(&fit_rows, &fit),
        MODEL_SHA.into(),
        TOKENIZER_SHA.into(),
        sha(&fit_identity),
    );
    let trials = tune_threshold(&tune_rows, &tune, &mut gate);
    let gate_bytes = gate.to_bytes()?;
    std::fs::write(root.join("gate.json"), &gate_bytes).map_err(|e| e.to_string())?;
    let loaded = SparseReadGate::from_bytes(
        &std::fs::read(root.join("gate.json")).map_err(|e| e.to_string())?,
    )?;
    if loaded != gate
        || loaded.model_sha256 != MODEL_SHA
        || loaded.tokenizer_sha256 != TOKENIZER_SHA
    {
        return Err("gate reload/binding mismatch".into());
    }
    // Dev is first accessed by the fitted and independently reloaded gate, never by training.
    let rows = observed_rows(&dev, &model, Some(&loaded), Some((&counts, lambdas)));
    let (generated, edits) = generations(&dev, &rows, &model, &loaded, &counts, lambdas, &tok)?;
    let mut csv =
        BufWriter::new(std::fs::File::create(root.join("rows.csv")).map_err(|e| e.to_string())?);
    writeln!(csv, "doc,position,target,model_top,latest_top,gate_top,count_top,recent_value,recent_age,older_value,older_age,selected_source").map_err(|e| e.to_string())?;
    for r in &rows {
        let maybe = |c: Option<Candidate>, age: bool| -> i64 {
            c.map_or(-1, |c| if age { c.age as i64 } else { c.value as i64 })
        };
        writeln!(
            csv,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            r.doc,
            r.position,
            r.target,
            r.model_top,
            r.latest_top,
            r.gate_top,
            r.count_top,
            maybe(r.recent, false),
            maybe(r.recent, true),
            maybe(r.previous_different, false),
            maybe(r.previous_different, true),
            r.selected_source.map_or(-1, |x| x as i64)
        )
        .map_err(|e| e.to_string())?;
    }
    csv.flush().map_err(|e| e.to_string())?;
    let mut per_source = Vec::new();
    for (i, doc) in dev.iter().enumerate() {
        per_source.push(json!({"doc":doc.path,"family":doc.family,"metrics":metrics(&rows.iter().filter(|r|r.doc==i).cloned().collect::<Vec<_>>())}));
    }
    let prose: Vec<_> = rows
        .iter()
        .filter(|r| dev[r.doc].family == "prose")
        .cloned()
        .collect();
    let code: Vec<_> = rows
        .iter()
        .filter(|r| dev[r.doc].family == "rust_code")
        .cloned()
        .collect();
    let p = metrics(&prose);
    let c = metrics(&code);
    let source_witness = edits.iter().any(|e| e["source_caused_change"] == true);
    let passed = p.gate_correct > p.model_correct
        && p.gate_correct > p.latest_correct
        && c.gate_correct > c.model_correct
        && c.gate_correct > c.latest_correct
        && p.older_rescues_from_recent + c.older_rescues_from_recent > 0
        && source_witness;
    let identity = |docs: &[Doc]| {
        docs.iter().map(|d|json!({"family":d.family,"path":d.path,"sha256":d.sha256,"original_bytes":d.original_bytes,"selected_tokens":d.tokens.len()})).collect::<Vec<_>>()
    };
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let receipt = json!({"schema":"uor-r4.native-sparse-read/1",
        "decision":if passed{"USEFUL_NATIVE_SPARSE_READ_COMPONENT"}else{"DO_NOT_PROMOTE_COMPONENT"},
        "fit":identity(&fit),"tune":identity(&tune),"dev":identity(&dev),
        "fit_rows":fit_rows.len(),"tune_rows":tune_rows.len(),"dev_rows":rows.len(),
        "model":{"path":model_path,"sha256":MODEL_SHA,"nonzero_parameter_reads_per_full_step":model.nonzero_per_step(),"readout_nonzero":model.wo.nonzero(),"transition_nonzero":model.wh.nonzero()+model.wf.nonzero()},
        "tokenizer":{"path":tokenizer_path,"sha256":TOKENIZER_SHA},
        "gate":{"sha256":sha(&gate_bytes),"threshold":loaded.threshold,"positive_cells":loaded.positive_cells(),"table_cells":72,"fit_source_sha256":loaded.fit_source_sha256,"tune_trials":trials},
        "count":{"lambdas":lambdas},"prose":p,"rust_code":c,"per_source":per_source,
        "source_edit_witness":source_witness,"source_edits":edits,
        "generated":generated,
        "rows_sha256":sha(&std::fs::read(root.join("rows.csv")).map_err(|e|e.to_string())?),
        "source_sha256":sha(include_bytes!("native-sparse-read.rs")),
        "mechanism_source_sha256":sha(include_bytes!("../native_geometric/learner/sparse_native_read.rs")),
        "executable_sha256":sha(&std::fs::read(exe).map_err(|e|e.to_string())?),
        "scope":"Frozen dense TlModel plus exact-pair episodic memory and fitted 4-bit source table. Dev is source-separated prose/code but now exposed. Top-one and actual generation; no normalized hard-copy loss, learned semantic address, general language, geometry advantage, D5 whole-model compliance or energy result."});
    write_json(&root, "receipt.json", &receipt)?;
    seal(&root).map_err(|e| e.to_string())?;
    let unlisted = verify(&root).map_err(|e| e.to_string())?;
    if !unlisted.is_empty() {
        return Err(format!("unlisted report files: {unlisted:?}"));
    }
    println!(
        "decision={} prose={}/{}/{}/{} code={}/{}/{}/{} gate_threshold={} source_edits={}",
        receipt["decision"],
        p.model_correct,
        p.latest_correct,
        p.gate_correct,
        p.count_correct,
        c.model_correct,
        c.latest_correct,
        c.gate_correct,
        c.count_correct,
        loaded.threshold,
        source_witness
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
