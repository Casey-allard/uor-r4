//! GPU registration of ordered connector IDs and blind comparison of value addresses.
#[path = "../value_gpu.rs"]
mod gpu;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Write, path::Path};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    content_selector::{select, Prototypes, BOUNDARY, WIDTH},
    discrete_d11::IntegerDiscreteModel,
    span_selector::{resolve, Bank, SpanChoice, CAP, CONT, NONWORD, START},
    value_selector::{decide, Decisions, ExactBank},
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
const SCHEMA: &str = "uor-r4.value-connectors/1";
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

const SPANS: [&str; 2] = [
    "3451b35c018687cd4ae37f80f306ffe2af3724eb4969d2f7271828cf1c67f870",
    "090c53c07a3c16e7d017c931b02e6749bca98affe71c5f8ca19e89f55d57436f",
];
#[derive(Deserialize)]
struct SpanState {
    artifact_sha256: String,
    tokenizer_sha256: String,
    bank: Bank,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema: String,
    train_sha256: String,
    artifact_sha256: String,
    span_state_sha256: String,
    bank: ExactBank,
}
#[derive(Deserialize)]
struct Legacy {
    prototypes: Prototypes,
}
#[derive(Deserialize)]
struct SpanPicks {
    artifact_sha256: String,
    state_sha256: String,
    baseline_state_sha256: String,
    rows: Vec<SpanRow>,
}
#[derive(Deserialize)]
struct SpanRow {
    prefix_sha256: String,
    span: SpanChoice,
}
#[derive(Serialize)]
struct Picks {
    artifact_sha256: String,
    state_sha256: String,
    span_state_sha256: String,
    span_predictions_sha256: String,
    rows: Vec<Row>,
}
#[derive(Serialize)]
struct Row {
    prefix_sha256: String,
    frozen: SpanChoice,
    #[serde(flatten)]
    decisions: Decisions,
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
fn span_state(path: &Path, seed: usize) -> Result<SpanState, Fail> {
    let bytes = fs::read(path)?;
    if sha256(&bytes) != SPANS[seed] {
        return Err("changed span state".into());
    }
    let s: SpanState = serde_json::from_slice(&bytes)?;
    if s.artifact_sha256 != MODELS[seed] || s.tokenizer_sha256 != TOKENIZERS[seed] {
        return Err("span artifact/tokenizer mismatch".into());
    }
    s.bank.validate()?;
    Ok(s)
}

fn fit(m: &Path, span: &Path, train: &Path, out: &Path) -> Result<(), Fail> {
    let (_, seed) = model(m)?;
    span_state(span, seed)?;
    let bytes = fs::read(train)?;
    if sha256(&bytes) != TRAIN {
        return Err("unregistered TRAIN".into());
    }
    let labels: Vec<Training> = serde_json::from_slice(&bytes)?;
    if labels.len() != 1728 {
        return Err("wrong training row count".into());
    }
    let mut data = Vec::with_capacity(gpu::STRIDE * labels.len());
    let mut seen = BTreeSet::new();
    let mut expected = Vec::new();
    for r in labels {
        let ids = &r.prefix;
        let fact = ids
            .get(r.fact_start as usize..=r.fact_end as usize)
            .filter(|s| !s.is_empty());
        let query = ids.get(r.query_start as usize..=r.query_end as usize);
        if fact.is_none()
            || fact != query
            || r.source >= r.query_start
            || !(2..=4).contains(&r.source.saturating_sub(r.fact_end))
        {
            return Err("invalid training positions".into());
        }
        data.extend(pack(ids, r.fact_end, r.source)?);
        let slice = &ids[r.fact_end as usize + 1..r.source as usize];
        let mut c = [BOUNDARY; 4];
        c[0] = slice.len() as u32;
        c[1..1 + slice.len()].copy_from_slice(slice);
        if seen.insert(c) {
            expected.push(c);
        }
    }
    let connectors = gpu::Gpu::new()?.fit(&data)?;
    if connectors != expected {
        return Err("GPU fitted ID bank differs from native reference".into());
    }
    let bank = ExactBank { connectors };
    bank.validate()?;
    write(
        out,
        &State {
            schema: SCHEMA.into(),
            train_sha256: TRAIN.into(),
            artifact_sha256: MODELS[seed].into(),
            span_state_sha256: SPANS[seed].into(),
            bank,
        },
    )
}

fn predict(
    m: &Path,
    l: &Path,
    s: &Path,
    e: &Path,
    p: &Path,
    b: &Path,
    out: &Path,
) -> Result<(), Fail> {
    let (model, seed) = model(m)?;
    let roots = model.packed_codebook_roots();
    let span = span_state(s, seed)?;
    let state_bytes = fs::read(e)?;
    let state: State = serde_json::from_slice(&state_bytes)?;
    let old_bytes = fs::read(l)?;
    let pick_bytes = fs::read(b)?;
    let picks: SpanPicks = serde_json::from_slice(&pick_bytes)?;
    if state.schema != SCHEMA
        || state.train_sha256 != TRAIN
        || state.artifact_sha256 != MODELS[seed]
        || state.span_state_sha256 != SPANS[seed]
        || sha256(&old_bytes) != BASELINES[seed]
        || picks.artifact_sha256 != MODELS[seed]
        || picks.state_sha256 != SPANS[seed]
        || picks.baseline_state_sha256 != BASELINES[seed]
    {
        return Err("invalid prediction/fit provenance".into());
    }
    let old: Legacy = serde_json::from_slice(&old_bytes)?;
    old.prototypes.validate()?;
    let [legacy] = old.prototypes.relation.as_slice() else {
        return Err("unexpected #4087 connector count".into());
    };
    if legacy[0] != 1 {
        return Err("unexpected #4087 connector shape".into());
    }
    let queries: Vec<Query> = serde_json::from_slice(&fs::read(p)?)?;
    if queries.len() != picks.rows.len() || !(1..=CAP).contains(&queries.len()) {
        return Err("invalid prediction count".into());
    }
    let mut data = Vec::with_capacity(gpu::STRIDE * queries.len());
    let mut seen = BTreeSet::new();
    for (q, b) in queries.iter().zip(&picks.rows) {
        let hash = sha256(
            &q.prefix
                .iter()
                .flat_map(|id| id.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        if hash != b.prefix_sha256 || !seen.insert(hash) {
            return Err("invalid prefix join".into());
        }
        let anchor = select(&q.prefix, &roots, &old.prototypes, false)?;
        let frozen = resolve(&q.prefix, &roots, &span.bank, legacy[1], anchor)?.span;
        if frozen != b.span {
            return Err("frozen #4089 endpoints or source differed".into());
        }
        let (end, limit) = if frozen.source.is_some() {
            (
                frozen.fact_end.ok_or("missing fact end")?,
                frozen.query_start.ok_or("missing query start")?,
            )
        } else {
            (BOUNDARY, BOUNDARY)
        };
        data.extend(pack(&q.prefix, end, limit)?);
    }
    let device = gpu::Gpu::new()?.predict(&data, &roots, &span.bank, &state.bank)?;
    let mut rows = Vec::with_capacity(queries.len());
    for ((q, b), d) in queries.into_iter().zip(picks.rows).zip(device) {
        let decisions = decide(&q.prefix, &roots, &span.bank, &state.bank, b.span)?;
        if decisions.words() != d {
            return Err("value source/mask device mismatch".into());
        }
        rows.push(Row {
            prefix_sha256: b.prefix_sha256,
            frozen: b.span,
            decisions,
        });
    }
    write(
        out,
        &Picks {
            artifact_sha256: MODELS[seed].into(),
            state_sha256: sha256(&state_bytes),
            span_state_sha256: SPANS[seed].into(),
            span_predictions_sha256: sha256(&pick_bytes),
            rows,
        },
    )
}

fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else {
        return Err("missing value command".into());
    };
    let paths: Vec<_> = rest.iter().map(Path::new).collect();
    match (cmd.to_str(),paths.as_slice()) {
        (Some("fit"),[m,s,t,o])=>fit(m,s,t,o),
        (Some("predict"),[m,l,s,e,p,b,o])=>predict(m,l,s,e,p,b,o),
        (Some("fixture"),[])=>fixture(),
        _=>Err("value-selector fixture | fit MODEL SPAN_STATE TRAIN OUT | predict MODEL LEGACY_STATE SPAN_STATE EXACT_STATE BARE SPAN_PICKS OUT".into()),
    }
}

fn fixture() -> Result<(), Fail> {
    let gpu = gpu::Gpu::new()?;
    // All roots collide. The old score chooses the longest available length on a tie.
    let roots = vec![0; 4096];
    let mut classes = vec![NONWORD; 4096];
    classes[10] = START;
    classes[11] = START;
    classes[12] = CONT;
    classes[13] = CONT;
    classes[14] = START;
    let relations = (1..=3)
        .map(|n| {
            let mut r = [BOUNDARY; 5];
            r[0] = n;
            r[1..=n as usize].fill(0);
            r[4] = 0;
            r
        })
        .collect();
    let span = Bank { classes, relations };
    let ids = [0, 9, 10, 11, 12, 13, 14, 9];
    let mut data = Vec::new();
    for source in [3, 5, 3] {
        data.extend(pack(&ids, 1, source)?);
    }
    let exact = ExactBank {
        connectors: gpu.fit(&data)?,
    };
    if exact.connectors != [[1, 10, BOUNDARY, BOUNDARY], [3, 10, 11, 12]] {
        return Err("exact bank order/dedup failed".into());
    }
    let check = |ids: &[u32], frozen: SpanChoice, want: [u32; 7]| -> Result<(), Fail> {
        let (end, limit) = if frozen.source.is_some() {
            (frozen.fact_end.unwrap(), frozen.query_start.unwrap())
        } else {
            (BOUNDARY, BOUNDARY)
        };
        let device = gpu.predict(&pack(ids, end, limit)?, &roots, &span, &exact)?;
        let native = decide(ids, &roots, &span, &exact, frozen)?;
        if native.words() != want || device != [want] {
            return Err(format!(
                "wrong source/masks: wanted {want:?}, got {device:?} {:?}",
                native.words()
            )
            .into());
        }
        Ok(())
    };
    let frozen = SpanChoice {
        query: Some(7),
        fact: Some(1),
        query_start: Some(7),
        query_end: Some(7),
        fact_start: Some(1),
        fact_end: Some(1),
        source: Some(5),
    };
    check(&ids, frozen, [5, 3, 5, 3, 7, 1, 5])?;
    // A shorter registered connector is still an exact hypothesis when the longer one is absent.
    let mut different = ids;
    different[4] = 10;
    check(&different, frozen, [5, 4, 3, 3, 7, 3, 1])?;
    different[2] = 14; // equal geometric root, different first raw ID
    check(&different, frozen, [5, 4, BOUNDARY, BOUNDARY, 7, 3, 0])?;
    // A permutation of exactly the same three IDs must lose the length-three witness.
    let mut swapped = ids;
    swapped.swap(3, 4);
    check(&swapped, frozen, [5, 4, 3, BOUNDARY, 7, 2, 1])?;
    // Here the query fence leaves exactly the length-one connector candidate.
    check(
        &ids,
        SpanChoice {
            query: Some(4),
            query_start: Some(4),
            query_end: Some(4),
            source: Some(3),
            ..frozen
        },
        [3, 3, 3, 3, 1, 1, 1],
    )?;
    let mut no_start = ids;
    no_start[3] = 13;
    check(&no_start, frozen, [5, BOUNDARY, 3, BOUNDARY, 7, 0, 1])?;
    let mut good = ids;
    good[5] = 14;
    check(&good, frozen, [5, 5, 5, 5, 7, 5, 5])?;
    check(
        &ids,
        SpanChoice::default(),
        [BOUNDARY, BOUNDARY, BOUNDARY, BOUNDARY, 0, 0, 0],
    )?;
    check(
        &ids,
        SpanChoice {
            source: None,
            ..frozen
        },
        [BOUNDARY, BOUNDARY, BOUNDARY, BOUNDARY, 0, 0, 0],
    )?;
    println!("PASS exact TRAIN bank, 7 device/Rust words per fixture: full vs partial raw IDs, shorter registered prefix, colliding roots, START/CONT, reordered/replaced token, no eligible choice, frozen no-source and longest tie. Cubin {}",sha256(gpu::CUBIN));
    Ok(())
}
