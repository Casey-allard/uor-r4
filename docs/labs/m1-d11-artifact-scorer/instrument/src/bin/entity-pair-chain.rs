//! Blind query/fact pair ranking followed by the frozen span, WordStart and
//! continuation operators. The optional oracle command accepts only two sites.
#[path = "../content_gpu.rs"]
#[allow(dead_code)]
mod content;
#[path = "../continuation_gpu.rs"]
mod continuation;
#[path = "../span_gpu.rs"]
#[allow(dead_code)]
mod span;
#[path = "../value_gpu.rs"]
#[allow(dead_code)]
mod value;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Write, path::Path};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    content_selector::{self, Choice, Prototypes, BOUNDARY, WIDTH},
    continuation_selector::{self, Continuation},
    discrete_d11::IntegerDiscreteModel,
    span_selector::{self, Bank, SpanChoice},
    value_selector::{self, Decisions, ExactBank},
};
type Fail = Box<dyn std::error::Error>;
const PINS: [[&str; 4]; 2] = [
    [
        "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
        "a9e340c153975b7ac3d8d9157283d78e3c5040d053e052f0fefec570f35b9317",
        "3451b35c018687cd4ae37f80f306ffe2af3724eb4969d2f7271828cf1c67f870",
        "c208bdc55a9df266e59ab19c4e930813c63522619a6734936d58040b73c3f726",
    ],
    [
        "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
        "41ff82f106f3e051dfa55406aea4f0fb5e63804210b5a339fe99dfc59b4cf72d",
        "090c53c07a3c16e7d017c931b02e6749bca98affe71c5f8ca19e89f55d57436f",
        "fbf10ca11af34784a2f489fbb7de9d4d3c8fa04623e9cea7364bf3f33d0d6a62",
    ],
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bare {
    prefix: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Oracle {
    prefix_sha256: String,
    query: u32,
    fact: u32,
}
#[derive(Deserialize)]
struct ContentState {
    prototypes: Prototypes,
}
#[derive(Deserialize)]
struct State<B> {
    bank: B,
}
#[derive(Serialize)]
struct Trace {
    prefix_sha256: String,
    frozen: SpanChoice,
    word_start: Option<u32>,
    value: Decisions,
    continuation: Continuation,
    #[serde(skip_serializing_if = "Option::is_none")]
    site_scores: Option<Vec<[u32; 2]>>,
}
fn pack<const N: usize>(ids: &[u32], fields: &[u32]) -> [u32; N] {
    let mut row = [0; N];
    row[..ids.len()].copy_from_slice(ids);
    row[WIDTH..].copy_from_slice(fields);
    row
}
fn ranked(
    bare: &[Bare],
    roots: &[u32],
    proto: &Prototypes,
    classes: &[u32],
    repair: bool,
) -> Result<(Vec<Choice>, Vec<Vec<[u32; 2]>>), Fail> {
    let data: Vec<_> = bare
        .iter()
        .flat_map(|q| pack::<{ content::STRIDE }>(&q.prefix, &[q.prefix.len() as u32, 0, 0, 0, 0]))
        .collect();
    let (gpu, scores) = content::Gpu::traced(&data, roots, proto, classes, repair)?;
    let mut costs = Vec::with_capacity(bare.len());
    for (i, q) in bare.iter().enumerate() {
        let (choice, sites) =
            content_selector::rank(&q.prefix, roots, proto, false, repair.then_some(classes))?;
        let start = i * WIDTH * 2;
        if gpu[i] != choice || sites.as_flattened() != &scores[start..start + q.prefix.len() * 2] {
            return Err(format!("entity pair/site cost CUDA mismatch at row {i}").into());
        }
        costs.push(sites);
    }
    Ok((gpu, costs))
}
fn run(arm: &str, paths: &[String], oracle: Option<&String>, out: &Path) -> Result<(), Fail> {
    let [model, content_path, span_path, exact_path, bare_path] = paths else {
        return Err("expected five inputs".into());
    };
    let model = IntegerDiscreteModel::load(Path::new(model))?;
    let cb = fs::read(content_path)?;
    let sb = fs::read(span_path)?;
    let eb = fs::read(exact_path)?;
    let ch = sha256(&cb);
    let sh = sha256(&sb);
    let eh = sha256(&eb);
    if !PINS.contains(&[model.artifact_sha256(), &ch, &sh, &eh]) {
        return Err("unregistered entity chain state".into());
    }
    let roots = model.packed_codebook_roots();
    let proto = serde_json::from_slice::<ContentState>(&cb)?.prototypes;
    let bank = serde_json::from_slice::<State<Bank>>(&sb)?.bank;
    let exact = serde_json::from_slice::<State<ExactBank>>(&eb)?.bank;
    proto.validate()?;
    bank.validate()?;
    exact.validate()?;
    let [legacy] = proto.relation.as_slice() else {
        return Err("unexpected relation count".into());
    };
    if legacy[0] != 1 {
        return Err("unexpected legacy root count".into());
    }
    let bytes = fs::read(bare_path)?;
    let bare: Vec<Bare> = serde_json::from_slice(&bytes)?;
    if !(1..=span_selector::CAP).contains(&bare.len()) {
        return Err("invalid panel length".into());
    }
    let mut seen = BTreeSet::new();
    let mut hashes = Vec::new();
    for q in &bare {
        if q.prefix.first() != Some(&0)
            || q.prefix.len() > WIDTH
            || q.prefix[1..].iter().any(|id| !(2..4096).contains(id))
        {
            return Err("invalid bare tape".into());
        }
        let h = sha256(
            &q.prefix
                .iter()
                .flat_map(|id| id.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        if !seen.insert(h.clone()) {
            return Err("duplicate bare prefix".into());
        }
        hashes.push(h);
    }
    let mut oracle_sha = None;
    let (anchors, scores) = if let Some(file) = oracle {
        let payload = fs::read(file)?;
        oracle_sha = Some(sha256(&payload));
        let projected: Vec<Oracle> = serde_json::from_slice(&payload)?;
        if projected.len() != bare.len() {
            return Err("oracle coverage mismatch".into());
        }
        let mut selected = Vec::new();
        for ((site, q), hash) in projected.into_iter().zip(&bare).zip(&hashes) {
            let (qa, fa) = (site.query as usize, site.fact as usize);
            if site.prefix_sha256 != *hash
                || qa >= q.prefix.len()
                || qa.saturating_sub(fa) < 2
                || q.prefix[qa] != q.prefix[fa]
            {
                return Err("invalid anchor intervention".into());
            }
            selected.push(Choice {
                query: Some(site.query),
                fact: Some(site.fact),
                source: None,
            });
        }
        (selected, None)
    } else {
        let (choices, costs) = ranked(&bare, &roots, &proto, &bank.classes, arm == "repair")?;
        (choices, Some(costs))
    };
    let span_data: Vec<_> = bare
        .iter()
        .zip(&anchors)
        .flat_map(|(q, a)| {
            pack::<{ span::STRIDE }>(
                &q.prefix,
                &[
                    q.prefix.len() as u32,
                    a.query.unwrap_or(BOUNDARY),
                    a.fact.unwrap_or(BOUNDARY),
                ],
            )
        })
        .collect();
    let raw_spans = span::Gpu::new()?.predict(&span_data, &roots, &bank, legacy[1])?;
    let mut spans = Vec::new();
    let mut value_data = Vec::new();
    for ((q, a), raw) in bare.iter().zip(anchors).zip(raw_spans) {
        let s = span_selector::resolve(&q.prefix, &roots, &bank, legacy[1], a)?;
        if ![s.span, s.endpoint_only, s.connector_only]
            .into_iter()
            .flat_map(|x| x.words())
            .eq(raw)
        {
            return Err("span replay mismatch".into());
        }
        let fields = if s.span.source.is_some() {
            [s.span.fact_end, s.span.query_start]
        } else {
            [None, None]
        };
        value_data.extend(pack::<{ value::STRIDE }>(
            &q.prefix,
            &[
                q.prefix.len() as u32,
                fields[0].unwrap_or(BOUNDARY),
                fields[1].unwrap_or(BOUNDARY),
            ],
        ));
        spans.push(s.span);
    }
    let raw_values = value::Gpu::new()?.predict(&value_data, &roots, &bank, &exact)?;
    let mut values = Vec::new();
    let mut cont_data = Vec::new();
    for ((q, &s), raw) in bare.iter().zip(&spans).zip(raw_values) {
        let v = value_selector::decide(&q.prefix, &roots, &bank, &exact, s)?;
        if v.words() != raw {
            return Err("value replay mismatch".into());
        }
        cont_data.push(pack::<{ continuation::STRIDE }>(
            &q.prefix,
            &[
                s.fact_end.unwrap_or(BOUNDARY),
                s.query_start.unwrap_or(BOUNDARY),
                v.word_start.unwrap_or(BOUNDARY),
                v.word_start_mask,
                v.ordered_mask,
            ],
        ));
        values.push(v);
    }
    let raw_cont = continuation::Gpu::new()?.predict(&cont_data, &roots, &bank)?;
    let mut rows = Vec::new();
    for (i, q) in bare.iter().enumerate() {
        let frozen = spans[i];
        let v = values[i];
        let c = continuation_selector::decide(&q.prefix, &roots, &bank, frozen, v)?;
        if c.words() != raw_cont[i] {
            return Err("continuation replay mismatch".into());
        }
        rows.push(Trace {
            prefix_sha256: hashes[i].clone(),
            frozen,
            word_start: v.word_start,
            value: v,
            continuation: c,
            site_scores: scores.as_ref().map(|s| s[i].clone()),
        });
    }
    let result = serde_json::json!({"schema":"uor-r4.entity-pair-chain/1","arm":arm,"artifact_sha256":model.artifact_sha256(),"bare_sha256":sha256(&bytes),"content_state_sha256":ch,"span_state_sha256":sh,"exact_state_sha256":eh,"oracle_projection_sha256":oracle_sha,"rows":rows});
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)?;
    serde_json::to_writer(&mut file, &result)?;
    file.write_all(b"\n")?;
    Ok(())
}
fn fixture() -> Result<(), Fail> {
    let roots = vec![0; 4096];
    let mut classes = vec![0; 4096];
    classes[9] = 1;
    classes[8] = 2;
    classes[7] = 2;
    let mut rel = [BOUNDARY; 16];
    rel[..2].copy_from_slice(&[1, 0]);
    let p = Prototypes {
        requested: vec![[0; 4]],
        excluded: vec![[65535; 4]],
        fact: vec![[0; 4]],
        relation: vec![rel],
    };
    let bare = [
        Bare {
            prefix: vec![0, 61, 9, 8, 19, 9, 7, 29, 9, 8, 32, 42],
        },
        Bare {
            prefix: vec![0, 61, 9, 8, 19, 9, 7, 29],
        },
        Bare {
            prefix: vec![0, 61, 9, 8, 19, 9, 8, 29, 32],
        },
        Bare {
            prefix: vec![0, 61, 9, 8, 8, 8, 29, 32],
        },
        Bare {
            prefix: vec![0, 61, 29, 42, 29, 63, 71],
        },
    ];
    let (old, os) = ranked(&bare, &roots, &p, &classes, false)?;
    let (new, ns) = ranked(&bare, &roots, &p, &classes, true)?;
    if os != ns
        || (old[0].query, old[0].fact) != (Some(5), Some(2))
        || (new[0].query, new[0].fact) != (Some(8), Some(2))
        || new[1] != Choice::default()
        || old[2] != new[2]
        || old[3].query.is_none()
        || new[3] != Choice::default()
        || old[4].query.is_none()
        || old[4] != new[4]
    {
        return Err(format!("invalid finite entity fixtures {old:?} vs {new:?}").into());
    }
    println!("PASS identical pair energies and stable ties; unequal BPE ending rejected, later exact span restored, no viable match abstains, compatible winner retained, overlapping interior offsets rejected, singleton punctuation preserved. AOT and native site and choice words match.");
    Ok(())
}
fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice(){
        [cmd] if cmd=="fixture"=>fixture(),
        [cmd,inputs @ ..,out] if matches!(cmd.as_str(),"baseline"|"repair")=>run(cmd,inputs,None,Path::new(out)),
        [cmd,inputs @ ..,oracle,out] if cmd=="oracle"=>run(cmd,inputs,Some(oracle),Path::new(out)),
        _=>Err("entity-pair-chain fixture | baseline|repair MODEL CONTENT_STATE SPAN_STATE EXACT_STATE BARE OUT | oracle MODEL CONTENT_STATE SPAN_STATE EXACT_STATE BARE ANCHORS OUT".into())
    }
}
