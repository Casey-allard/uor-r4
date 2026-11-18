//! GPU fit and blind span/connector replay on two frozen D11 releases.
#[path = "../span_gpu.rs"]
mod gpu;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::Path,
};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    content_selector::{select, Choice, Prototypes, BOUNDARY, WIDTH},
    discrete_d11::IntegerDiscreteModel,
    span_selector::{resolve, Arms, Bank, Relation, CAP, CONT, NONWORD, START},
};
type Fail = Box<dyn std::error::Error>;
const MODELS: [&str; 2] = [
    "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
    "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
];
const TOKENIZERS: [&str; 2] = [
    "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377",
    "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e",
];
const BASELINES: [&str; 2] = [
    "a9e340c153975b7ac3d8d9157283d78e3c5040d053e052f0fefec570f35b9317",
    "41ff82f106f3e051dfa55406aea4f0fb5e63804210b5a339fe99dfc59b4cf72d",
];
const TRAIN: &str = "edcb27bfda2b469b009a49e11b957f0c58e9d29077f149ad606feb44d078f743";
const SCHEMA: &str = "uor-r4.anchored-entity-spans/1";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    prefix: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Training {
    prefix: Vec<u32>,
    query_start: u32,
    query_end: u32,
    fact_start: u32,
    fact_end: u32,
    source: u32,
}
#[derive(Deserialize)]
struct Tokenizer {
    model: Vocab,
}
#[derive(Deserialize)]
struct Vocab {
    vocab: BTreeMap<String, usize>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema: String,
    artifact_sha256: String,
    tokenizer_sha256: String,
    train_sha256: String,
    bank: Bank,
}
#[derive(Deserialize)]
struct LegacyState {
    prototypes: Prototypes,
}
#[derive(Deserialize)]
struct LegacyPicks {
    artifact_sha256: String,
    state_sha256: String,
    arm: String,
    rows: Vec<LegacyRow>,
}
#[derive(Deserialize)]
struct LegacyRow {
    prefix_sha256: String,
    #[serde(flatten)]
    choice: Choice,
}
#[derive(Serialize)]
struct Picks {
    artifact_sha256: String,
    state_sha256: String,
    baseline_state_sha256: String,
    baseline_predictions_sha256: String,
    rows: Vec<Row>,
}
#[derive(Serialize)]
struct Row {
    prefix_sha256: String,
    #[serde(flatten)]
    choices: Arms,
}
fn write(path: &Path, value: &impl Serialize) -> Result<(), Fail> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(&bytes)?;
    Ok(())
}
fn model(path: &Path) -> Result<(IntegerDiscreteModel, usize), Fail> {
    let m = IntegerDiscreteModel::load(path)?;
    let i = MODELS
        .iter()
        .position(|&s| s == m.artifact_sha256())
        .ok_or("unregistered D11")?;
    Ok((m, i))
}
fn class(piece: &str) -> u32 {
    let word = piece.strip_prefix('Ġ').unwrap_or(piece);
    if word.is_empty() || !word.bytes().all(|b| b.is_ascii_alphabetic()) {
        return NONWORD;
    }
    if piece == word {
        CONT
    } else {
        START
    }
}
fn classes(bytes: &[u8]) -> Result<Vec<u32>, Fail> {
    let t: Tokenizer = serde_json::from_slice(bytes)?;
    let mut out = vec![BOUNDARY; 4096];
    for (piece, id) in t.model.vocab {
        let slot = out.get_mut(id).ok_or("bad vocab ID")?;
        if *slot != BOUNDARY {
            return Err("duplicate vocab ID".into());
        }
        *slot = class(&piece);
    }
    if out.contains(&BOUNDARY) {
        return Err("missing vocab ID".into());
    }
    Ok(out)
}
fn pack(ids: &[u32], a: u32, b: u32) -> Result<[u32; gpu::STRIDE], Fail> {
    if ids.first() != Some(&0)
        || ids.len() > WIDTH
        || ids[1..].iter().any(|id| !(2..4096).contains(id))
    {
        return Err("invalid prefix".into());
    }
    let mut row = [0; gpu::STRIDE];
    row[..ids.len()].copy_from_slice(ids);
    row[WIDTH..].copy_from_slice(&[ids.len() as u32, a, b]);
    Ok(row)
}
fn fit(model_path: &Path, tokenizer: &Path, train: &Path, out: &Path) -> Result<(), Fail> {
    let (m, seed) = model(model_path)?;
    let t = fs::read(tokenizer)?;
    let bytes = fs::read(train)?;
    if sha256(&t) != TOKENIZERS[seed] || sha256(&bytes) != TRAIN {
        return Err("unregistered tokenizer or training data".into());
    }
    let labels: Vec<Training> = serde_json::from_slice(&bytes)?;
    if labels.len() != 1728 {
        return Err("unexpected training row count".into());
    }
    let mut data = Vec::with_capacity(gpu::STRIDE * labels.len());
    for r in &labels {
        let row = pack(&r.prefix, r.fact_end, r.source)?;
        let span = |a: u32, b: u32| {
            r.prefix
                .get(a as usize..=b as usize)
                .filter(|s| !s.is_empty())
        };
        let q = span(r.query_start, r.query_end).ok_or("invalid query span")?;
        let f = span(r.fact_start, r.fact_end).ok_or("invalid fact span")?;
        if q != f
            || r.fact_end >= r.source
            || r.source >= r.query_start
            || !(2..=4).contains(&(r.source - r.fact_end))
        {
            return Err("inconsistent TRAIN entity/connector".into());
        }
        data.extend(row);
    }
    let roots = m.packed_codebook_roots();
    let relations = gpu::Gpu::new()?.fit(&data, &roots)?;
    // Check the actual fitted device order, dedup and label endpoints independently.
    let mut seen = BTreeSet::new();
    let mut reference = Vec::new();
    for r in labels {
        let mut word = [BOUNDARY; 5];
        let connector = &r.prefix[r.fact_end as usize + 1..r.source as usize];
        word[0] = connector.len() as u32;
        word[4] = roots[r.prefix[r.source as usize] as usize];
        for (j, &id) in connector.iter().enumerate() {
            word[j + 1] = roots[id as usize];
        }
        if seen.insert(word) {
            reference.push(word);
        }
    }
    if relations != reference
        || relations.iter().map(|r| r[0]).collect::<BTreeSet<_>>() != BTreeSet::from([1, 2, 3])
    {
        return Err("CPU/GPU fitted bank mismatch or missing length".into());
    }
    let bank = Bank {
        classes: classes(&t)?,
        relations,
    };
    bank.validate()?;
    write(
        out,
        &State {
            schema: SCHEMA.into(),
            artifact_sha256: MODELS[seed].into(),
            tokenizer_sha256: TOKENIZERS[seed].into(),
            train_sha256: TRAIN.into(),
            bank,
        },
    )
}
fn words(a: Arms) -> [u32; 21] {
    let mut out = [BOUNDARY; 21];
    for (dst, arm) in out
        .chunks_exact_mut(7)
        .zip([a.span, a.endpoint_only, a.connector_only])
    {
        dst.copy_from_slice(&arm.words());
    }
    out
}
fn predict(m: &Path, s: &Path, l: &Path, p: &Path, b: &Path, out: &Path) -> Result<(), Fail> {
    let (m, seed) = model(m)?;
    let bytes = fs::read(s)?;
    let state: State = serde_json::from_slice(&bytes)?;
    let old = fs::read(l)?;
    let bare = fs::read(p)?;
    let base = fs::read(b)?;
    let picks: LegacyPicks = serde_json::from_slice(&base)?;
    if state.schema != SCHEMA
        || state.artifact_sha256 != MODELS[seed]
        || state.train_sha256 != TRAIN
        || state.tokenizer_sha256 != TOKENIZERS[seed]
        || sha256(&old) != BASELINES[seed]
        || picks.arm != "trained"
        || picks.state_sha256 != BASELINES[seed]
        || picks.artifact_sha256 != MODELS[seed]
    {
        return Err("invalid state or frozen baseline provenance".into());
    }
    let baseline: LegacyState = serde_json::from_slice(&old)?;
    baseline.prototypes.validate()?;
    let [legacy] = baseline.prototypes.relation.as_slice() else {
        return Err("invalid frozen relation".into());
    };
    if legacy[0] != 1 {
        return Err("unexpected frozen bridge length".into());
    }
    let queries: Vec<Query> = serde_json::from_slice(&bare)?;
    if !(1..=CAP).contains(&queries.len()) || queries.len() != picks.rows.len() {
        return Err("invalid prediction row count".into());
    }
    let mut data = Vec::with_capacity(gpu::STRIDE * queries.len());
    let mut seen = BTreeSet::new();
    let roots = m.packed_codebook_roots();
    for (q, b) in queries.iter().zip(&picks.rows) {
        let hash = sha256(
            &q.prefix
                .iter()
                .flat_map(|i| i.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        if hash != b.prefix_sha256
            || !seen.insert(hash)
            || select(&q.prefix, &roots, &baseline.prototypes, false)? != b.choice
        {
            return Err("frozen baseline does not match bare prefix".into());
        }
        data.extend(pack(
            &q.prefix,
            b.choice.query.unwrap_or(BOUNDARY),
            b.choice.fact.unwrap_or(BOUNDARY),
        )?);
    }
    let result = gpu::Gpu::new()?.predict(&data, &roots, &state.bank, legacy[1])?;
    let mut rows = Vec::new();
    for ((q, b), device) in queries.into_iter().zip(picks.rows).zip(result) {
        let choices = resolve(&q.prefix, &roots, &state.bank, legacy[1], b.choice)?;
        if words(choices) != device {
            return Err(format!("span CUDA/Rust word mismatch for {}", b.prefix_sha256).into());
        }
        rows.push(Row {
            prefix_sha256: b.prefix_sha256,
            choices,
        });
    }
    write(
        out,
        &Picks {
            artifact_sha256: state.artifact_sha256,
            state_sha256: sha256(&bytes),
            baseline_state_sha256: BASELINES[seed].into(),
            baseline_predictions_sha256: sha256(&base),
            rows,
        },
    )
}
fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let Some((cmd, paths)) = args.split_first() else {
        return Err("missing span command".into());
    };
    let paths: Vec<_> = paths.iter().map(Path::new).collect();
    match (cmd.to_str(),paths.as_slice()){
        (Some("fit"),[m,t,data,out])=>fit(m,t,data,out),
        (Some("predict"),[m,s,l,p,b,out])=>predict(m,s,l,p,b,out),
        (Some("fixture"),[])=>fixture(),
        _=>Err("span-selector fixture | fit MODEL TOKENIZER TRAIN OUT | predict MODEL STATE BASELINE_STATE BARE BASELINE_PICKS OUT".into()),
    }
}
fn fixture() -> Result<(), Fail> {
    if [
        class("ĠName"),
        class("ame"),
        class("."),
        class("Ġ"),
        class("é"),
        class("x2"),
    ] != [START, CONT, NONWORD, NONWORD, NONWORD, NONWORD]
    {
        return Err("token word-class contract".into());
    }
    let roots: Vec<_> = (0..4096).map(|i| (i * 53) & 65535).collect();
    let mut classes = vec![NONWORD; 4096];
    for id in [3, 7, 11, 12, 13, 20] {
        classes[id] = START;
    }
    for id in [4, 5, 6] {
        classes[id] = CONT;
    }
    let mut prefixes = Vec::new();
    for n in 1..=3 {
        let mut ids = vec![0, 3, 4, 5];
        ids.extend_from_slice(&[11, 12, 13][..n]);
        ids.extend_from_slice(&[20, 16, 3, 4, 5, 16]);
        prefixes.push(ids);
    }
    let gpu = gpu::Gpu::new()?;
    let mut data = Vec::new();
    let mut expected = Vec::<Relation>::new();
    for (i, ids) in prefixes.iter().enumerate() {
        let n = i + 1;
        let row = pack(ids, 3, (4 + n) as u32)?;
        data.extend(row);
        data.extend(row);
        let mut r = [BOUNDARY; 5];
        r[0] = n as u32;
        r[4] = roots[20];
        for j in 0..n {
            r[j + 1] = roots[11 + j];
        }
        expected.push(r);
    }
    let relations = gpu.fit(&data, &roots)?;
    if relations != expected {
        return Err("known GPU training/displacement/dedup inventory".into());
    }
    let bank = Bank { classes, relations };
    let malformed = Choice {
        query: Some(4),
        fact: Some(u32::MAX),
        source: None,
    };
    if resolve(&prefixes[0], &roots, &bank, roots[11], malformed).is_ok()
        || gpu
            .predict(&pack(&prefixes[0], 4, u32::MAX)?, &roots, &bank, roots[11])
            .is_ok()
    {
        return Err("invalid maximum fact address accepted".into());
    }
    let check = |ids: &[u32], q: u32, f: u32, bank: &Bank, roots: &[u32]| -> Result<Arms, Fail> {
        let oldroot = roots[11];
        let out = gpu.predict(&pack(ids, q, f)?, roots, bank, oldroot)?;
        let as_pos = |p: u32| (p != BOUNDARY).then_some(p);
        let a = resolve(
            ids,
            roots,
            bank,
            oldroot,
            Choice {
                query: as_pos(q),
                fact: as_pos(f),
                source: None,
            },
        )?;
        if out != [words(a)] {
            return Err("finite CPU/device arm disagreement".into());
        }
        Ok(a)
    };
    for (i, ids) in prefixes.iter().enumerate() {
        let n = i as u32 + 1;
        let qs = n + 6;
        let s = 4 + n;
        for delta in 0..3 {
            let a = check(ids, qs + delta, 1 + delta, &bank, &roots)?;
            if a.span.words() != [qs + delta, 1 + delta, qs, qs + 2, 1, 3, s]
                || a.endpoint_only.source != Some(5)
                || (delta == 2 && a.connector_only.source != Some(s))
            {
                return Err(format!(
                    "known entity or connector boundary changed: n={n}, delta={delta}, {a:?}"
                )
                .into());
            }
        }
    }
    let partial = check(&[0, 3, 4, 5, 11, 20, 16, 3, 4, 6, 16], 7, 1, &bank, &roots)?;
    if partial.span.words() != [7, 1, 7, 9, 1, 3, BOUNDARY]
        || partial.endpoint_only.source.is_some()
    {
        return Err("partial-name collision escaped".into());
    }
    for id in [7, 16] {
        let a = check(&[0, id, 11, 20, 16, id, 16], 5, 1, &bank, &roots)?;
        if a.span.words() != [5, 1, 5, 5, 1, 1, 3]
            || a.span != a.connector_only
            || a.span != a.endpoint_only
        {
            return Err("singleton/punctuation boundary changed".into());
        }
    }
    let near = check(&[0, 16, 11, 16], 3, 1, &bank, &roots)?;
    if near.span.source.is_some()
        || near.connector_only.source.is_some()
        || near.endpoint_only.source != Some(2)
    {
        return Err("query fence or legacy clamp failure".into());
    }
    if check(&[0, 7, 11, 20, 16], BOUNDARY, BOUNDARY, &bank, &roots)? != Arms::default() {
        return Err("missing anchor created source".into());
    }
    let zeros = vec![0; 4096];
    let tied = Bank {
        classes: bank.classes.clone(),
        relations: gpu.fit(&data, &zeros)?,
    };
    if check(&prefixes[2], 9, 1, &tied, &zeros)?.span.source != Some(7) {
        return Err("exact tie must choose longer connector".into());
    }
    let ids = &prefixes[2];
    for source in [5, 6, 7] {
        let moved = Bank {
            classes: bank.classes.clone(),
            relations: gpu.fit(&pack(ids, 3, source)?, &roots)?,
        };
        if check(ids, 9, 1, &moved, &roots)?.span.source != Some(source) {
            return Err("displacing training value did not move output".into());
        }
    }
    println!("PASS device vs serving Rust, 21 words each; 1/2/3 IDs with shared prefix, start/interior/end anchors, incompatible full words, singleton/nonword/null/fence, longest exact tie and 3 shifted labels; cubin {}",sha256(gpu::CUBIN));
    Ok(())
}
