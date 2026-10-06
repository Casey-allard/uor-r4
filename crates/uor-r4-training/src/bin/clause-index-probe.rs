//! Phase 0 clause-retrieval probe (#820). Pre-registration:
//! `docs/research/minilm-phase0/PREREG.md`.
//!
//! ```text
//! clause-index-probe parity model=MINILM_DIR
//! clause-index-probe run model=MINILM_DIR out=NEW_REPORT_ROOT \
//!   v3=data/panels/conversational-v3.json v3_checks=data/panels/conversational-v3-checks.tsv \
//!   dev=GENERATOR_ROOT/dev/dialogues.jsonl
//! ```
//!
//! `MINILM_DIR` holds `model.safetensors` and `tokenizer.json` of
//! `sentence-transformers/all-MiniLM-L6-v2`, an offline teacher/comparator
//! only. `parity` checks the Rust forward against the cosine matrix published
//! in the sentence-transformers quickstart. `run` scores every arm on the v3
//! memory rows and the generator dev questions and writes `report.json` and
//! `rows.jsonl` into a newly claimed, sealed report root. Paths naming
//! `conversational-v4` are refused (held-out panel).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_training::clause_probe::{
    argmax_recent_by, argmax_recent_f64, clause_hit, clause_mixed, cosine, e8_cmp, e8_dequantize,
    e8_quantize, e8_roots, hopf_blocks, hopf_score, int_dot, lexical_choice, segment_history,
    sum_codes, sum_rows, MiniLm, Result, ValueSet, WordPiece, BLOCKS, E8_SCALE_K, HIDDEN,
};

const SCHEMA: &str = "uor-r4.clause-index-probe/1";
const ARMS: [&str; 8] = ["L", "M", "S", "S_raw", "E", "H", "H_F", "H_E8root"];

/// Published by sentence-transformers (sbert.net quickstart) for
/// all-MiniLM-L6-v2, rounded to four decimals.
const PARITY_SENTENCES: [&str; 3] = [
    "The weather is lovely today.",
    "It's so sunny outside!",
    "He drove to the stadium.",
];
const PARITY_COSINES: [[f64; 3]; 3] = [
    [1.0000, 0.6660, 0.1046],
    [0.6660, 1.0000, 0.1411],
    [0.1046, 0.1411, 1.0000],
];
const PARITY_TOLERANCE: f64 = 1e-3;

fn main() {
    if let Err(e) = real_main() {
        eprintln!("clause-index-probe: {e}");
        std::process::exit(1);
    }
}

fn parse_args(args: &[String]) -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    for a in args {
        let (k, v) = a
            .split_once('=')
            .ok_or(format!("argument {a} is not key=value"))?;
        if out.insert(k.to_string(), v.to_string()).is_some() {
            return Err(format!("duplicate argument {k}"));
        }
    }
    Ok(out)
}

fn need<'a>(m: &'a HashMap<String, String>, k: &str) -> Result<&'a str> {
    m.get(k).map(String::as_str).ok_or(format!("missing {k}="))
}

fn refuse_v4(p: &str) -> Result<()> {
    if p.contains("conversational-v4") {
        return Err(format!(
            "{p}: the v4 panel is held out and never read by this probe"
        ));
    }
    Ok(())
}

fn sha256_file(p: &Path) -> Result<String> {
    let bytes = fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

fn real_main() -> Result<()> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (cmd, rest) = argv
        .split_first()
        .ok_or("usage: clause-index-probe parity|run key=value...")?;
    let args = parse_args(rest)?;
    match cmd.as_str() {
        "parity" => parity(&args),
        "run" => run(&args),
        other => Err(format!("unknown command {other}")),
    }
}

fn load(model_dir: &Path) -> Result<(MiniLm, WordPiece)> {
    let tok = WordPiece::from_tokenizer_json(&model_dir.join("tokenizer.json"))?;
    let model = MiniLm::load(&model_dir.join("model.safetensors"))?;
    Ok((model, tok))
}

fn parity_check(model: &MiniLm, tok: &WordPiece) -> Result<Value> {
    let ids = tok.encode("This is an example sentence");
    let expected_ids = vec![101u32, 2023, 2003, 2019, 2742, 6251, 102];
    let embs: Vec<Vec<f32>> = PARITY_SENTENCES
        .iter()
        .map(|s| model.sentence_embedding(tok, s))
        .collect::<Result<_>>()?;
    let mut max_dev = 0f64;
    let mut matrix = Vec::new();
    for i in 0..3 {
        let mut row = Vec::new();
        for j in 0..3 {
            let c = cosine(&embs[i], &embs[j]);
            max_dev = max_dev.max((c - PARITY_COSINES[i][j]).abs());
            row.push(c);
        }
        matrix.push(row);
    }
    let pass = ids == expected_ids && max_dev <= PARITY_TOLERANCE;
    Ok(json!({
        "token_ids_example": ids,
        "token_ids_expected": expected_ids,
        "cosines": matrix,
        "published_cosines": PARITY_COSINES,
        "max_abs_deviation": max_dev,
        "tolerance": PARITY_TOLERANCE,
        "first8": embs.iter().map(|e| e[..8].to_vec()).collect::<Vec<_>>(),
        "pass": pass,
    }))
}

fn parity(args: &HashMap<String, String>) -> Result<()> {
    let dir = PathBuf::from(need(args, "model")?);
    let (model, tok) = load(&dir)?;
    let report = parity_check(&model, &tok)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    );
    if report["pass"] != true {
        return Err("parity check failed".into());
    }
    Ok(())
}

struct Row {
    dataset: &'static str,
    id: String,
    category: String,
    history: Vec<String>,
    question: String,
    expected: ValueSet,
    forbidden: ValueSet,
}

fn v3_rows(panel: &Path, checks: &Path) -> Result<Vec<Row>> {
    let panel_json: Value =
        serde_json::from_str(&fs::read_to_string(panel).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let mut check_map: HashMap<String, (String, String)> = HashMap::new();
    for line in fs::read_to_string(checks)
        .map_err(|e| e.to_string())?
        .lines()
    {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() >= 5 && cols[2] == "recall" {
            check_map.insert(
                cols[0].to_string(),
                (cols[3].to_string(), cols[4].to_string()),
            );
        }
    }
    let mut rows = Vec::new();
    for r in panel_json.as_array().ok_or("v3 panel is not an array")? {
        if r["category"] != "multi_turn_memory" {
            continue;
        }
        let id = r["id"].as_str().ok_or("row id")?.to_string();
        let turns: Vec<String> = r["user_turns"]
            .as_array()
            .ok_or("user_turns")?
            .iter()
            .map(|t| t.as_str().unwrap_or_default().to_string())
            .collect();
        let (question, history) = turns.split_last().ok_or("empty row")?;
        let (terms, forbid) = check_map
            .get(&id)
            .ok_or(format!("{id} has no recall check row"))?;
        rows.push(Row {
            dataset: "v3",
            id,
            category: "multi_turn_memory".into(),
            history: history.to_vec(),
            question: question.clone(),
            expected: ValueSet::from_spellings(&[terms.as_str()]),
            forbidden: ValueSet::from_spellings(&[forbid.as_str()]),
        });
    }
    if rows.len() != 30 {
        return Err(format!("expected 30 v3 memory rows, found {}", rows.len()));
    }
    Ok(rows)
}

fn dev_rows(path: &Path) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    for line in fs::read_to_string(path).map_err(|e| e.to_string())?.lines() {
        let d: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        let id = d["id"].as_str().ok_or("dialogue id")?;
        let msgs: Vec<String> = d["messages"]
            .as_array()
            .ok_or("messages")?
            .iter()
            .map(|m| m["content"].as_str().unwrap_or_default().to_string())
            .collect();
        for (qi, q) in d["questions"]
            .as_array()
            .ok_or("questions")?
            .iter()
            .enumerate()
        {
            let Some(expect) = q["expect"].as_str() else {
                continue; // abstention: no expected value
            };
            let turn = q["turn"].as_u64().ok_or("turn")? as usize;
            if turn == 0 || turn > msgs.len() {
                return Err(format!("{id}: bad question turn {turn}"));
            }
            let forbid: Vec<String> = q["forbid"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            rows.push(Row {
                dataset: "dev",
                id: format!("{id}#q{qi}"),
                category: q["category"].as_str().unwrap_or("?").to_string(),
                history: msgs[..turn - 1].to_vec(),
                question: msgs[turn - 1].clone(),
                expected: ValueSet::from_spellings(&[expect]),
                forbidden: ValueSet::from_spellings(&forbid),
            });
        }
    }
    Ok(rows)
}

struct Tables {
    centered: Vec<f32>,
    raw: Vec<f32>,
    codes: Vec<i8>,
}

fn build_tables(model: &MiniLm, tok: &WordPiece) -> Result<(Tables, Value)> {
    let raw = model.static_table(tok, 512)?;
    let vocab = tok.vocab_size;
    let mut mean = vec![0f64; HIDDEN];
    for r in 0..vocab {
        for i in 0..HIDDEN {
            mean[i] += raw[r * HIDDEN + i] as f64;
        }
    }
    mean.iter_mut().for_each(|m| *m /= vocab as f64);
    let mut centered = raw.clone();
    for r in 0..vocab {
        for i in 0..HIDDEN {
            centered[r * HIDDEN + i] -= mean[i] as f32;
        }
    }
    let ss: f64 = centered.iter().map(|x| (*x as f64).powi(2)).sum();
    let rms = (ss / centered.len() as f64).sqrt() as f32;
    let scale = E8_SCALE_K / rms;
    let mut codes = Vec::with_capacity(centered.len());
    let mut err2 = 0f64;
    let mut rel_sum = 0f64;
    let mut cos_sum = 0f64;
    let mut max_abs = 0i32;
    let mut distinct: HashSet<[i8; 8]> = HashSet::new();
    let mut zero_blocks = 0usize;
    for r in 0..vocab {
        let v = &centered[r * HIDDEN..(r + 1) * HIDDEN];
        let c = e8_quantize(v, scale)?;
        let back = e8_dequantize(&c, scale);
        let e: f64 = v
            .iter()
            .zip(&back)
            .map(|(a, b)| ((a - b) as f64).powi(2))
            .sum();
        let n: f64 = v.iter().map(|a| (*a as f64).powi(2)).sum();
        err2 += e;
        rel_sum += (e / n.max(1e-30)).sqrt();
        cos_sum += cosine(v, &back).max(-1.0);
        for blk in c.chunks(8) {
            let mut k = [0i8; 8];
            k.copy_from_slice(blk);
            if k == [0; 8] {
                zero_blocks += 1;
            }
            distinct.insert(k);
        }
        max_abs = max_abs.max(c.iter().map(|x| (*x as i32).abs()).max().unwrap_or(0));
        codes.extend(c);
    }
    let table_bytes: Vec<u8> = raw.iter().flat_map(|x| x.to_le_bytes()).collect();
    let code_bytes: Vec<u8> = codes.iter().map(|x| *x as u8).collect();
    let stats = json!({
        "vocab": vocab,
        "static_rule": "[CLS] id [SEP], mean over the 3 positions, not normalized; centered by subtracting the vocabulary mean vector",
        "centered_rms": rms,
        "scale_rule": format!("scale = {E8_SCALE_K} / centered_rms"),
        "scale": scale,
        "global_relative_error": (err2 / ss).sqrt(),
        "mean_per_token_relative_error": rel_sum / vocab as f64,
        "mean_per_token_cosine": cos_sum / vocab as f64,
        "max_abs_doubled_coordinate": max_abs,
        "distinct_block_codes": distinct.len(),
        "zero_block_share": zero_blocks as f64 / (vocab * BLOCKS) as f64,
        "pair_table_entries": distinct.len() * distinct.len(),
        "raw_table_sha256": hex::encode(Sha256::digest(&table_bytes)),
        "e8_codes_sha256": hex::encode(Sha256::digest(&code_bytes)),
    });
    Ok((
        Tables {
            centered,
            raw,
            codes,
        },
        stats,
    ))
}

struct Arms<'a> {
    model: &'a MiniLm,
    tok: &'a WordPiece,
    tables: &'a Tables,
    roots: Vec<[f64; 8]>,
    m_cache: HashMap<String, Vec<f32>>,
}

impl Arms<'_> {
    fn m_emb(&mut self, text: &str) -> Result<Vec<f32>> {
        if let Some(v) = self.m_cache.get(text) {
            return Ok(v.clone());
        }
        let v = self.model.sentence_embedding(self.tok, text)?;
        self.m_cache.insert(text.to_string(), v.clone());
        Ok(v)
    }

    fn choices(&mut self, question: &str, clauses: &[String]) -> Result<Vec<Option<usize>>> {
        let n = clauses.len();
        let l = lexical_choice(question, clauses);
        // M: contextual sentence embeddings (full text), cosine.
        let qm = self.m_emb(question)?;
        let mut ms = Vec::with_capacity(n);
        for c in clauses {
            let cm = self.m_emb(c)?;
            ms.push(cosine(&qm, &cm));
        }
        let m = argmax_recent_f64(&ms);
        // Static arms share the content-token selection.
        let q_ids = self.tok.content_token_ids(question);
        let c_ids: Vec<Vec<u32>> = clauses
            .iter()
            .map(|c| self.tok.content_token_ids(c))
            .collect();
        let t = self.tables;
        let qs = sum_rows(&t.centered, &q_ids);
        let cs: Vec<Vec<f32>> = c_ids.iter().map(|ids| sum_rows(&t.centered, ids)).collect();
        let s_scores: Vec<f64> = cs.iter().map(|c| cosine(&qs, c)).collect();
        let s = argmax_recent_f64(&s_scores);
        let qr = sum_rows(&t.raw, &q_ids);
        let sr_scores: Vec<f64> = c_ids
            .iter()
            .map(|ids| cosine(&qr, &sum_rows(&t.raw, ids)))
            .collect();
        let s_raw = argmax_recent_f64(&sr_scores);
        // E: integer E8 codes, exact integer cosine order.
        let qe = sum_codes(&t.codes, &q_ids);
        let keys: Vec<(i64, i64)> = c_ids
            .iter()
            .map(|ids| {
                let ce = sum_codes(&t.codes, ids);
                (int_dot(&qe, &ce), int_dot(&ce, &ce))
            })
            .collect();
        let e = argmax_recent_by(n, |a, b| e8_cmp(keys[a], keys[b]));
        // H family on the centered static vectors.
        let qh = hopf_blocks(&qs, None);
        let ch: Vec<_> = cs.iter().map(|c| hopf_blocks(c, None)).collect();
        let h_scores: Vec<f64> = ch.iter().map(|c| hopf_score(&qh, c, false)).collect();
        let hf_scores: Vec<f64> = ch.iter().map(|c| hopf_score(&qh, c, true)).collect();
        let qhr = hopf_blocks(&qs, Some(&self.roots));
        let hr_scores: Vec<f64> = cs
            .iter()
            .map(|c| hopf_score(&qhr, &hopf_blocks(c, Some(&self.roots)), false))
            .collect();
        Ok(vec![
            l,
            m,
            s,
            s_raw,
            e,
            argmax_recent_f64(&h_scores),
            argmax_recent_f64(&hf_scores),
            argmax_recent_f64(&hr_scores),
        ])
    }
}

/// Frozen decision rule for one candidate arm (PREREG).
fn verdict(cand: f64, l: f64) -> &'static str {
    if cand >= 0.90 && (cand >= l + 0.05 || l < 0.85) {
        "BUILD"
    } else if cand < 0.75 || cand <= l {
        "DROP"
    } else {
        "REPORT-ONLY"
    }
}

fn run(args: &HashMap<String, String>) -> Result<()> {
    let model_dir = PathBuf::from(need(args, "model")?);
    let out = PathBuf::from(need(args, "out")?);
    let v3 = need(args, "v3")?;
    let v3_checks = need(args, "v3_checks")?;
    let dev = need(args, "dev")?;
    for p in [v3, v3_checks, dev] {
        refuse_v4(p)?;
    }
    report_output::claim(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let started = std::time::Instant::now();

    let mut rows = v3_rows(Path::new(v3), Path::new(v3_checks))?;
    rows.extend(dev_rows(Path::new(dev))?);

    let (model, tok) = load(&model_dir)?;
    let parity_report = parity_check(&model, &tok)?;
    if parity_report["pass"] != true {
        return Err(format!("parity failed: {parity_report}"));
    }
    let t_tab = std::time::Instant::now();
    let (tables, e8_stats) = build_tables(&model, &tok)?;
    let table_secs = t_tab.elapsed().as_secs_f64();
    let mut arms = Arms {
        model: &model,
        tok: &tok,
        tables: &tables,
        roots: e8_roots(),
        m_cache: HashMap::new(),
    };

    let mut rows_out = fs::File::create(out.join("rows.jsonl")).map_err(|e| e.to_string())?;
    // per dataset: n, reachable, mixed, hits per arm; per dev category
    let mut tally: HashMap<String, (usize, usize, usize, [usize; ARMS.len()])> = HashMap::new();
    let mut clause_counts: HashMap<&str, usize> = HashMap::new();
    for row in &rows {
        let clauses = segment_history(&row.history);
        *clause_counts.entry(row.dataset).or_default() += clauses.len();
        let reachable = clauses
            .iter()
            .any(|c| clause_hit(c, &row.expected, &row.forbidden));
        let mixed = clauses
            .iter()
            .any(|c| clause_mixed(c, &row.expected, &row.forbidden));
        let choices = arms.choices(&row.question, &clauses)?;
        let hits: Vec<bool> = choices
            .iter()
            .map(|c| {
                c.map(|i| clause_hit(&clauses[i], &row.expected, &row.forbidden))
                    .unwrap_or(false)
            })
            .collect();
        for key in [
            row.dataset.to_string(),
            format!("{}/{}", row.dataset, row.category),
        ] {
            let e = tally.entry(key).or_insert((0, 0, 0, [0; ARMS.len()]));
            e.0 += 1;
            e.1 += reachable as usize;
            e.2 += mixed as usize;
            for (k, h) in hits.iter().enumerate() {
                e.3[k] += *h as usize;
            }
        }
        let rec = json!({
            "dataset": row.dataset,
            "id": row.id,
            "category": row.category,
            "question": row.question,
            "clauses": clauses,
            "expected": row.expected.alternatives,
            "forbidden": row.forbidden.alternatives,
            "reachable": reachable,
            "same_clause": mixed,
            "choice": ARMS.iter().zip(&choices).map(|(a, c)| (a.to_string(), json!(c))).collect::<serde_json::Map<_, _>>(),
            "hit": ARMS.iter().zip(&hits).map(|(a, h)| (a.to_string(), json!(h))).collect::<serde_json::Map<_, _>>(),
        });
        writeln!(rows_out, "{rec}").map_err(|e| e.to_string())?;
    }
    drop(rows_out);

    let mut groups = serde_json::Map::new();
    let mut keys: Vec<&String> = tally.keys().collect();
    keys.sort();
    for k in keys {
        let (n, reach, mixed, hits) = tally[k];
        let mut arm_rates = serde_json::Map::new();
        for (i, a) in ARMS.iter().enumerate() {
            arm_rates.insert(
                a.to_string(),
                json!({"hits": hits[i], "rate": hits[i] as f64 / n as f64}),
            );
        }
        groups.insert(
            k.clone(),
            json!({"n": n, "reachable": reach, "same_clause": mixed, "arms": arm_rates}),
        );
    }
    let rate =
        |key: &str, arm: &str| -> f64 { groups[key]["arms"][arm]["rate"].as_f64().unwrap_or(0.0) };
    let l = rate("dev", "L");
    let e = rate("dev", "E");
    let h = rate("dev", "H");
    let ve = verdict(e, l);
    let vh = verdict(h, l);
    let overall = if ve == "BUILD" || vh == "BUILD" {
        "BUILD"
    } else if ve == "DROP" && vh == "DROP" {
        "DROP"
    } else {
        "REPORT-ONLY"
    };
    let report = json!({
        "schema": SCHEMA,
        "model_files": {
            "model.safetensors": sha256_file(&model_dir.join("model.safetensors"))?,
            "tokenizer.json": sha256_file(&model_dir.join("tokenizer.json"))?,
        },
        "inputs": {
            "v3": {"path": v3, "sha256": sha256_file(Path::new(v3))?},
            "v3_checks": {"path": v3_checks, "sha256": sha256_file(Path::new(v3_checks))?},
            "dev": {"path": dev, "sha256": sha256_file(Path::new(dev))?},
        },
        "parity": parity_report,
        "e8": e8_stats,
        "clauses": clause_counts,
        "groups": groups,
        "decision": {
            "rule": "per candidate (E, H) on generator-dev: BUILD if c >= 0.90 and (c >= L + 0.05 or L < 0.85); DROP if c < 0.75 or c <= L; else REPORT-ONLY. Overall: BUILD if any candidate is BUILD; DROP if all are DROP; else REPORT-ONLY.",
            "dev_L": l, "dev_E": e, "dev_H": h,
            "E": ve, "H": vh, "overall": overall,
        },
        "cost": {"static_table_seconds": table_secs, "total_seconds": started.elapsed().as_secs_f64()},
    });
    fs::write(
        out.join("report.json"),
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    report_output::seal(&out).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report["groups"]["dev"]).unwrap_or_default()
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&report["groups"]["v3"]).unwrap_or_default()
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&report["decision"]).unwrap_or_default()
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&report["e8"]).unwrap_or_default()
    );
    Ok(())
}
