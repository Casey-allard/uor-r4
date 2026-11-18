//! Bounded native occurrence-selector fit, controls and label-free inference.
//! See codex-3/SELECTOR-SHEET.md for the eligible domain and fixed arms.
#[path = "../occurrence_gpu.rs"]
mod gpu;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{discrete_d11::IntegerDiscreteModel, occurrence_selector::select};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};
type Fail = Box<dyn std::error::Error>;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    prefix: Vec<u32>,
}
impl Query {
    fn validate(&self) -> Result<(), Fail> {
        if self.prefix.first() != Some(&0)
            || self.prefix.len() > 192
            || self.prefix.iter().skip(1).any(|&t| !(2..4096).contains(&t))
        {
            return Err("invalid selector prefix".into());
        }
        Ok(())
    }
    fn hash(&self) -> String {
        sha256(
            &self
                .prefix
                .iter()
                .flat_map(|t| t.to_le_bytes())
                .collect::<Vec<_>>(),
        )
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    pair: String,
    text: String,
    query: Query,
    asked: u32,
    other: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus {
    split: String,
    rows: Vec<Case>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema: String,
    artifact_sha256: String,
    train_sha256: String,
    arm: String,
    lookback: u8,
    anchor_back: u8,
    training_rows: u32,
    coordinate_hits: Vec<u32>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Prediction {
    prefix_sha256: String,
    source: Option<u32>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Predictions {
    artifact_sha256: String,
    state_sha256: String,
    arm: String,
    lookback: u8,
    anchor_back: u8,
    rows: Vec<Prediction>,
}
fn write(path: &Path, data: &impl Serialize) -> Result<(), Fail> {
    use std::io::Write;
    let bytes = serde_json::to_vec(data)?;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    f.write_all(&bytes)?;
    f.write_all(b"\n")?;
    Ok(())
}
fn load_model(path: &Path) -> Result<IntegerDiscreteModel, Fail> {
    let model = IntegerDiscreteModel::load(path)?;
    let h = model.artifact_sha256();
    if !matches!(
        h,
        "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d"
            | "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de"
    ) {
        return Err("unregistered D11 artifact".into());
    }
    Ok(model)
}
fn unique_span(prefix: &[u32], span: &[u32]) -> Result<u32, Fail> {
    if span.is_empty() {
        return Err("empty value encoding".into());
    }
    let mut it = prefix
        .windows(span.len())
        .enumerate()
        .filter(|(_, s)| *s == span);
    let (at, _) = it.next().ok_or("complete TRAIN value absent")?;
    if it.next().is_some() {
        return Err("ambiguous complete TRAIN value".into());
    }
    Ok(at as u32)
}
fn generate(tok: &ByteBpeTokenizer, held: bool) -> Result<Corpus, Fail> {
    let encoder = DialogueProtocol::literal_roles_v2(tok)?.bind(tok)?;
    let names: &[(&str, &str)] = if held {
        &[("Joy", "Vic"), ("Ted", "Mia")]
    } else {
        &[
            ("May", "Roy"),
            ("Jim", "Sue"),
            ("Leo", "Amy"),
            ("Max", "Eve"),
        ]
    };
    let values = if held {
        [("silver coin", "copper ring"), ("plum", "peach")]
    } else {
        [("stone", "leaf"), ("paper boat", "wooden spoon")]
    };
    let mut rows = Vec::new();
    for (ni, &(left, right)) in names.iter().enumerate() {
        for (vi, &(x, y)) in values.iter().enumerate() {
            for form in 0..2 {
                for swap in [false, true] {
                    for reverse in [false, true] {
                        let (v0, v1) = if swap { (y, x) } else { (x, y) };
                        let (a, b, va, vb) = if reverse {
                            (right, left, v1, v0)
                        } else {
                            (left, right, v0, v1)
                        };
                        for ask_right in [false, true] {
                            let (ask, other, target, distractor) = if ask_right {
                                (right, left, v1, v0)
                            } else {
                                (left, right, v0, v1)
                            };
                            let text = match (held, form) {
                                (false, 0) => format!(
                                    "{a} picked {va}. {b} picked {vb}. What did {ask} pick? Skip {other}."
                                ),
                                (false, _) => format!(
                                    "{a} kept {va}. {b} kept {vb}. Which did {ask} keep? Disregard {other}."
                                ),
                                (true, 0) => format!(
                                    "{a} took {va}. {b} took {vb}. What was {ask} taking? Exclude {other}."
                                ),
                                (true, _) => format!(
                                    "{a} saved {va}. {b} saved {vb}. Which did {ask} save? Omit {other}."
                                ),
                            };
                            let prefix = encoder
                                .encode_assistant_prefix(&[Message {
                                    role: "user",
                                    content: &text,
                                }])
                                .tokens;
                            let asked = unique_span(&prefix, &tok.encode(&format!(" {target}")))?;
                            let other =
                                unique_span(&prefix, &tok.encode(&format!(" {distractor}")))?;
                            if asked == other {
                                return Err("non-disjoint source labels".into());
                            }
                            let query = Query { prefix };
                            query.validate()?;
                            rows.push(Case {
                                pair: format!("{ni}-{vi}-{form}-{swap}-{reverse}"),
                                text,
                                query,
                                asked,
                                other,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(Corpus {
        split: if held { "heldout" } else { "train" }.into(),
        rows,
    })
}
fn prepare(tokenizer: &Path, out: &Path) -> Result<(), Fail> {
    let bytes = fs::read(tokenizer)?;
    if !matches!(
        sha256(&bytes).as_str(),
        "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e"
            | "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377"
    ) {
        return Err("unregistered release tokenizer".into());
    }
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("bad tokenizer")?;
    let tr = generate(&tok, false)?;
    let ho = generate(&tok, true)?;
    let unique: BTreeSet<_> = tr
        .rows
        .iter()
        .chain(&ho.rows)
        .map(|r| r.query.hash())
        .collect();
    if tr.rows.len() != 128 || ho.rows.len() != 64 || unique.len() != 192 {
        return Err("split size, overlap or uniqueness failure".into());
    }
    fs::create_dir(out)?;
    write(&out.join("train.json"), &tr)?;
    write(&out.join("heldout.json"), &ho)?;
    write(
        &out.join("heldout-prefixes.json"),
        &ho.rows.iter().map(|r| &r.query).collect::<Vec<_>>(),
    )?;
    Ok(())
}
const SCHEMA: &str = "uor-r4.native-occurrence-selector/1";
const TRAIN_SHA256: &str = "060ce5f91b5f9a4960d7896200d60f9a3775695031e3a1724b3af77809fcc3d5";
const HELDOUT_SHA256: &str = "65b26c76be0343387cd25e0efb5605f23f2efd9f2f0f9e94892ba51fdff95190";
fn best(counts: &[u32]) -> usize {
    let mut best = 0;
    for at in 1..counts.len() {
        if counts[at] > counts[best] {
            best = at;
        }
    }
    best
}
fn validate_state(s: &State, model: &IntegerDiscreteModel) -> Result<u32, Fail> {
    if s.schema != SCHEMA
        || s.artifact_sha256 != model.artifact_sha256()
        || s.coordinate_hits.len() != 192
        || s.training_rows != 128
        || s.coordinate_hits.iter().any(|&n| n > s.training_rows)
        || s.train_sha256 != TRAIN_SHA256
    {
        return Err("state identity or coordinate inventory mismatch".into());
    }
    let expected = match s.arm.as_str() {
        "no_fit" if s.coordinate_hits.iter().all(|&n| n == 0) => 0,
        "trained" | "swapped" => best(&s.coordinate_hits),
        _ => return Err("unknown or malformed selector arm".into()),
    };
    if s.lookback as usize != 1 + expected % 6 || s.anchor_back as usize != expected / 6 {
        return Err("rule and fitted votes disagree".into());
    }
    Ok(expected as u32)
}
fn fit(model_path: &Path, train_path: &Path, out: &Path) -> Result<(), Fail> {
    let model = load_model(model_path)?;
    let input = fs::read(train_path)?;
    let train_sha256 = sha256(&input);
    if train_sha256 != TRAIN_SHA256 {
        return Err("TRAIN hash mismatch".into());
    }
    let c: Corpus = serde_json::from_slice(&input)?;
    if c.split != "train" || c.rows.len() != 128 {
        return Err("fit accepts only frozen TRAIN".into());
    }
    let queries: Vec<_> = c.rows.iter().map(|r| r.query.clone()).collect();
    let labels: Vec<_> = c.rows.iter().map(|r| [r.asked, r.other]).collect();
    let device = gpu::Gpu::new()?;
    let (positions, hits, chosen) = device.execute(
        &queries,
        &model.packed_codebook_roots(),
        0,
        192,
        Some(&labels),
    )?;
    for arm in 0..2 {
        for i in 0..192 {
            let oracle = labels
                .iter()
                .enumerate()
                .filter(|(r, ab)| positions[r * 192 + i] == ab[arm])
                .count();
            if hits[arm * 192 + i] as usize != oracle {
                return Err("support count disagrees with GPU raw choices".into());
            }
        }
        if chosen[arm] as usize != best(&hits[arm * 192..(arm + 1) * 192]) {
            return Err("GPU argmax disagrees with support counts".into());
        }
    }
    fs::create_dir(out)?;
    for (arm, rule, counts) in [
        ("trained", chosen[0], hits[..192].to_vec()),
        ("swapped", chosen[1], hits[192..].to_vec()),
        ("no_fit", 0, vec![0; 192]),
    ] {
        let s = State {
            schema: SCHEMA.into(),
            artifact_sha256: model.artifact_sha256().into(),
            train_sha256: train_sha256.clone(),
            arm: arm.into(),
            lookback: (1 + rule % 6) as u8,
            anchor_back: (rule / 6) as u8,
            training_rows: 128,
            coordinate_hits: counts,
        };
        validate_state(&s, &model)?;
        write(&out.join(format!("{arm}.json")), &s)?;
    }
    write(
        &out.join("fit-receipt.json"),
        &json!({"model":model.artifact_sha256(),"train":train_sha256,
        "kernel_sha256":sha256(gpu::CUBIN),"raw_grid_choices_sha256":sha256(&positions.iter().flat_map(|w|w.to_le_bytes()).collect::<Vec<_>>())}),
    )?;
    Ok(())
}
fn predict(model_path: &Path, state: &Path, queries: &Path, out: &Path) -> Result<(), Fail> {
    let model = load_model(model_path)?;
    let bytes = fs::read(state)?;
    let s: State = serde_json::from_slice(&bytes)?;
    let rule = validate_state(&s, &model)?;
    let queries: Vec<Query> = serde_json::from_slice(&fs::read(queries)?)?;
    let roots = model.packed_codebook_roots();
    let (positions, _, _) = gpu::Gpu::new()?.execute(&queries, &roots, rule, 1, None)?;
    let mut rows = Vec::with_capacity(queries.len());
    let mut seen = BTreeSet::new();
    for (q, &raw) in queries.iter().zip(&positions) {
        let source = (raw != u32::MAX).then_some(raw);
        let oracle = select(&q.prefix, &roots, s.lookback, s.anchor_back)?;
        if source != oracle {
            return Err("CUDA/Rust source mismatch".into());
        }
        let prefix_sha256 = q.hash();
        if !seen.insert(prefix_sha256.clone()) {
            return Err("duplicate inference prefix".into());
        }
        rows.push(Prediction {
            prefix_sha256,
            source,
        });
    }
    write(
        out,
        &Predictions {
            artifact_sha256: s.artifact_sha256,
            state_sha256: sha256(&bytes),
            arm: s.arm,
            lookback: s.lookback,
            anchor_back: s.anchor_back,
            rows,
        },
    )
}
fn grade(targets: &Path, preds: &Path, out: &Path) -> Result<(), Fail> {
    let bytes = fs::read(targets)?;
    if sha256(&bytes) != HELDOUT_SHA256 {
        return Err("held-out hash mismatch".into());
    }
    let data: Corpus = serde_json::from_slice(&bytes)?;
    let pred: Predictions = serde_json::from_slice(&fs::read(preds)?)?;
    if data.split != "heldout" || data.rows.len() != 64 || pred.rows.len() != 64 {
        return Err("grade requires all 64 held-out rows".into());
    }
    let mut groups: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let (mut asked, mut other) = (0, 0);
    for (c, p) in data.rows.iter().zip(&pred.rows) {
        if c.query.hash() != p.prefix_sha256 {
            return Err("prefix identity mismatch in grade".into());
        }
        let hits = groups.entry(c.pair.clone()).or_default();
        hits.0 += 1;
        if p.source == Some(c.asked) {
            hits.1 += 1;
            asked += 1;
        } else if p.source == Some(c.other) {
            other += 1;
        }
    }
    if groups.len() != 32 || groups.values().any(|x| x.0 != 2) {
        return Err("held-out group shape".into());
    }
    write(
        out,
        &json!({"arm":pred.arm,"artifact_sha256":pred.artifact_sha256,"state_sha256":pred.state_sha256,
        "lookback":pred.lookback,"anchor_back":pred.anchor_back,"rows":64,"asked":asked,"other":other,
        "pairs":32,"both_correct":groups.values().filter(|x|x.1==2).count(),"heldout_sha256":HELDOUT_SHA256}),
    )
}
fn fixture() -> Result<(), Fail> {
    let roots: Vec<u32> = (0..4096).map(|x| x * 13 & 65535).collect();
    let queries = vec![
        Query {
            prefix: vec![0, 2, 3, 4, 2, 9, 2, 4, 10],
        },
        Query {
            prefix: vec![0, 7, 9, 7, 9, 3, 9, 7, 9],
        },
        Query {
            prefix: vec![0, 3, 3, 3, 3],
        },
    ];
    let labels = [[2, 4], [2, 3], [1, 2]];
    let (p, h, w) = gpu::Gpu::new()?.execute(&queries, &roots, 0, 192, Some(&labels))?;
    for (r, q) in queries.iter().enumerate() {
        for i in 0..192 {
            let expected =
                select(&q.prefix, &roots, (1 + i % 6) as u8, (i / 6) as u8)?.unwrap_or(u32::MAX);
            if p[r * 192 + i] != expected {
                return Err("finite CUDA/serving parity".into());
            }
        }
    }
    let mut planted = p.clone();
    planted[0] = planted[0].wrapping_add(1);
    if planted[0] == select(&queries[0].prefix, &roots, 1, 0)?.unwrap_or(u32::MAX)
        || h[..192] == h[192..]
    {
        return Err("selector negative control missed displaced source or swapped votes".into());
    }
    if select(&[0, 3, 3, 3, 3], &roots, 1, 0)? != Some(2)
        || select(&[0, 3], &roots, 1, 31)?.is_some()
    {
        return Err("tie or empty domain control".into());
    }
    println!("CUDA/serving parity on {} rules x {} prefixes; source displacement detected; swapped vote inventory differs; winners {:?}; cubin {}",192,queries.len(),w,sha256(gpu::CUBIN));
    Ok(())
}
fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let Some((op, paths)) = args.split_first() else {
        return Err("missing command".into());
    };
    let p: Vec<_> = paths.iter().map(Path::new).collect();
    match (op.to_str(), p.as_slice()) {
        (Some("prepare"), [tok, dir]) => prepare(tok, dir),
        (Some("fixture"), []) => fixture(),
        (Some("fit"), [model, train, out]) => fit(model, train, out),
        (Some("predict"), [model, state, inputs, out]) => predict(model, state, inputs, out),
        (Some("grade"), [held, pred, out]) => grade(held, pred, out),
        _ => Err(concat!(
            "occurrence-selector prepare TOKENIZER NEW_DIR | fixture | ",
            "fit MODEL TRAIN NEW_DIR | predict MODEL STATE PREFIXES NEW_JSON | ",
            "grade HELDOUT PREDICTIONS NEW_JSON"
        )
        .into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_labels_and_nonunique_spans() {
        assert!(serde_json::from_str::<Query>(r#"{"prefix":[0,2],"asked":1}"#).is_err());
        assert!(unique_span(&[0, 2, 3, 2, 3], &[2, 3]).is_err());
        assert!(Query {
            prefix: vec![0, 2, 0]
        }
        .validate()
        .is_err());
        assert_eq!(best(&[3, 7, 7]), 1);
    }
}
