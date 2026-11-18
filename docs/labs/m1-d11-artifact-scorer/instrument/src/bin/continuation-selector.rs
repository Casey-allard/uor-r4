//! Blind, forward-only connector continuation over frozen, replayed WordStart choices.
#[path = "../continuation_gpu.rs"]
mod gpu;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Write, path::Path};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    content_selector::{BOUNDARY, WIDTH},
    continuation_selector::{decide, Continuation},
    discrete_d11::IntegerDiscreteModel,
    span_selector::{Bank, SpanChoice, CAP, CONT, NONWORD, START},
    value_selector::{decide as baseline, Decisions, ExactBank},
};
type Fail = Box<dyn std::error::Error>;
const PINS: [[&str; 3]; 2] = [
    [
        "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
        "3451b35c018687cd4ae37f80f306ffe2af3724eb4969d2f7271828cf1c67f870",
        "c208bdc55a9df266e59ab19c4e930813c63522619a6734936d58040b73c3f726",
    ],
    [
        "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
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
struct State<B> {
    bank: B,
}
#[derive(Deserialize)]
struct Prior {
    artifact_sha256: String,
    state_sha256: String,
    span_state_sha256: String,
    rows: Vec<PriorRow>,
}
#[derive(Deserialize)]
struct PriorRow {
    prefix_sha256: String,
    frozen: SpanChoice,
    #[serde(flatten)]
    decisions: Decisions,
}
#[derive(Serialize)]
struct Row {
    prefix_sha256: String,
    frozen: SpanChoice,
    word_start: Option<u32>,
    continuation: Continuation,
}
#[derive(Serialize)]
struct Predictions {
    schema: &'static str,
    artifact_sha256: String,
    span_state_sha256: String,
    exact_state_sha256: String,
    value_picks_sha256: String,
    bare_sha256: String,
    rows: Vec<Row>,
}
fn pack(ids: &[u32], frozen: SpanChoice, base: Decisions) -> Result<[u32; gpu::STRIDE], Fail> {
    if ids.len() > WIDTH
        || ids.first() != Some(&0)
        || ids[1..].iter().any(|&id| !(2..4096).contains(&id))
    {
        return Err("invalid continuation prefix".into());
    }
    let mut row = [0; gpu::STRIDE];
    row[..ids.len()].copy_from_slice(ids);
    row[WIDTH..].copy_from_slice(&[
        frozen.fact_end.unwrap_or(BOUNDARY),
        frozen.query_start.unwrap_or(BOUNDARY),
        base.word_start.unwrap_or(BOUNDARY),
        base.word_start_mask,
        base.ordered_mask,
    ]);
    Ok(row)
}
fn predict(
    model: &Path,
    span: &Path,
    exact: &Path,
    bare: &Path,
    picks: &Path,
    out: &Path,
) -> Result<(), Fail> {
    let model = IntegerDiscreteModel::load(model)?;
    let span = fs::read(span)?;
    let exact = fs::read(exact)?;
    let bare = fs::read(bare)?;
    let picks = fs::read(picks)?;
    let prior: Prior = serde_json::from_slice(&picks)?;
    let artifact_sha256 = model.artifact_sha256().to_string();
    let span_state_sha256 = sha256(&span);
    let exact_state_sha256 = sha256(&exact);
    if !PINS.contains(&[
        artifact_sha256.as_str(),
        span_state_sha256.as_str(),
        exact_state_sha256.as_str(),
    ]) || prior.artifact_sha256 != artifact_sha256
        || prior.span_state_sha256 != span_state_sha256
        || prior.state_sha256 != exact_state_sha256
    {
        return Err("unregistered baseline state or incompatible prior predictions".into());
    }
    let bank = serde_json::from_slice::<State<Bank>>(&span)?.bank;
    let exact = serde_json::from_slice::<State<ExactBank>>(&exact)?.bank;
    let queries: Vec<Bare> = serde_json::from_slice(&bare)?;
    if queries.len() != prior.rows.len() || !(1..=CAP).contains(&queries.len()) {
        return Err("invalid continuation panel count".into());
    }
    let roots = model.packed_codebook_roots();
    let mut seen = BTreeSet::new();
    let mut packed = Vec::with_capacity(queries.len());
    for (q, old) in queries.iter().zip(&prior.rows) {
        let hash = sha256(
            &q.prefix
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        if hash != old.prefix_sha256
            || !seen.insert(hash)
            || baseline(&q.prefix, &roots, &bank, &exact, old.frozen)? != old.decisions
        {
            return Err("prefix or frozen value decision mismatch".into());
        }
        packed.push(pack(&q.prefix, old.frozen, old.decisions)?);
    }
    let device = gpu::Gpu::new()?.predict(&packed, &roots, &bank)?;
    let mut rows = Vec::with_capacity(queries.len());
    for ((q, old), words) in queries.into_iter().zip(prior.rows).zip(device) {
        let native = decide(&q.prefix, &roots, &bank, old.frozen, old.decisions)?;
        if native.words() != words {
            return Err("continuation CUDA/Rust decision mismatch".into());
        }
        rows.push(Row {
            prefix_sha256: old.prefix_sha256,
            frozen: old.frozen,
            word_start: old.decisions.word_start,
            continuation: native,
        });
    }
    let result = Predictions {
        schema: "uor-r4.continuation/1",
        artifact_sha256,
        span_state_sha256,
        exact_state_sha256,
        value_picks_sha256: sha256(&picks),
        bare_sha256: sha256(&bare),
        rows,
    };
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)?;
    serde_json::to_writer(&mut output, &result)?;
    output.write_all(b"\n")?;
    Ok(())
}

fn fixture() -> Result<(), Fail> {
    let gpu = gpu::Gpu::new()?;
    let mut roots = vec![0; 4096];
    roots[11] = 65535;
    roots[12] = 65535;
    let mut classes = vec![NONWORD; 4096];
    classes[10] = START;
    classes[11] = START;
    classes[12] = CONT;
    classes[13] = CONT;
    let rel = [1, 0, BOUNDARY, BOUNDARY, 65535];
    let bank = Bank {
        classes,
        relations: vec![rel],
    };
    let frozen = SpanChoice {
        fact_end: Some(1),
        query_start: Some(6),
        source: Some(3),
        ..Default::default()
    };
    let base = Decisions {
        word_start: Some(3),
        word_start_mask: 7,
        ..Default::default()
    };
    let check = |ids: &[u32],
                 span: SpanChoice,
                 masks: Decisions,
                 bank: &Bank,
                 want: [u32; 4]|
     -> Result<(), Fail> {
        let words = decide(ids, &roots, bank, span, masks)?.words();
        let device = gpu.predict(&[pack(ids, span, masks)?], &roots, bank)?;
        if words != want || device != [want] {
            return Err(format!("wanted {want:?}, Rust {words:?}, device {device:?}").into());
        }
        Ok(())
    };
    let ids = [0, 9, 10, 10, 11, 11, 9];
    check(&ids, frozen, base, &bank, [4, 2, 1, 6])?;
    check(
        &ids,
        frozen,
        Decisions {
            ordered_mask: 6,
            ..base
        },
        &bank,
        [5, 1, 1, 6],
    )?;
    check(
        &[0, 9, 10, 10, 13, 11, 9],
        frozen,
        Decisions {
            word_start_mask: 5,
            ..base
        },
        &bank,
        [5, 2, 3, 4],
    )?;
    check(
        &[0, 9, 10, 10, 12, 11, 9],
        frozen,
        Decisions {
            word_start_mask: 5,
            ..base
        },
        &bank,
        [3, 0, 1, 6],
    )?;
    check(
        &[0, 9, 10, 10, 9, 11, 9],
        frozen,
        Decisions {
            word_start_mask: 5,
            ordered_mask: 4,
            ..base
        },
        &bank,
        [3, 0, 1, 4],
    )?;
    let tied = Bank {
        classes: bank.classes.clone(),
        relations: vec![[1, 0, BOUNDARY, BOUNDARY, 0]],
    };
    check(&ids, frozen, base, &tied, [3, 0, 0, 7])?;
    check(
        &ids,
        SpanChoice {
            query_start: Some(4),
            ..frozen
        },
        Decisions {
            word_start_mask: 1,
            ..base
        },
        &bank,
        [3, 0, 1, 0],
    )?;
    check(
        &ids,
        SpanChoice::default(),
        Decisions::default(),
        &bank,
        [BOUNDARY, 0, 0, 0],
    )?;
    println!("PASS: geometric landing, longest exact override, inner BPE crossing/refusal, NONWORD barrier, conservative ties, query fence and empty source, device/Rust equality, cubin {}",sha256(gpu::CUBIN));
    Ok(())
}
fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let paths: Vec<_> = args.iter().skip(1).map(Path::new).collect();
    match (args.first().and_then(|s|s.to_str()),paths.as_slice()){
        (Some("fixture"),[])=>fixture(),
        (Some("predict"),[m,s,e,b,p,o])=>predict(m,s,e,b,p,o),
        _=>Err("continuation-selector fixture | predict MODEL SPAN_STATE EXACT_STATE BARE VALUE_PICKS NEW_OUTPUT".into())
    }
}
