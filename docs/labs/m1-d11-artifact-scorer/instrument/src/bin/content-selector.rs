//! Native E8 context prototype training and label-free occurrence selection.
#[path = "../content_gpu.rs"]
mod gpu;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Write, path::Path};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    content_selector::{select, Choice, Prototypes, BOUNDARY, WIDTH},
    discrete_d11::IntegerDiscreteModel,
};
type Fail = Box<dyn std::error::Error>;
const TRAIN_HASH: &str = "72d877a2ee5b9469ca0445618f6f1222c3ec5fb14063fcc0d4abbf7bc7a15c17";
const SCHEMA: &str = "uor-r4.content-entity-selector/1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    prefix: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Training {
    prefix: Vec<u32>,
    query: u32,
    query_other: Option<u32>,
    fact: u32,
    other_fact: u32,
    source: u32,
    other_source: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema: String,
    artifact_sha256: String,
    train_sha256: String,
    arm: String,
    prototypes: Prototypes,
}
#[derive(Serialize)]
struct Row {
    prefix_sha256: String,
    #[serde(flatten)]
    choice: Choice,
}
#[derive(Serialize)]
struct Predictions {
    artifact_sha256: String,
    state_sha256: String,
    arm: String,
    rows: Vec<Row>,
}

fn write(out: &Path, value: &impl Serialize) -> Result<(), Fail> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)?;
    f.write_all(&serde_json::to_vec(value)?)?;
    f.write_all(b"\n")?;
    Ok(())
}
fn model(path: &Path) -> Result<IntegerDiscreteModel, Fail> {
    let m = IntegerDiscreteModel::load(path)?;
    if !matches!(
        m.artifact_sha256(),
        "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d"
            | "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de"
    ) {
        return Err("unregistered D11 artifact".into());
    }
    Ok(m)
}
fn pack(prefix: &[u32], sites: [u32; 4]) -> Result<[u32; gpu::STRIDE], Fail> {
    if prefix.first() != Some(&0)
        || prefix.len() > WIDTH
        || prefix[1..].iter().any(|id| !(2..4096).contains(id))
    {
        return Err("invalid token in entity selector".into());
    }
    let mut row = [0; gpu::STRIDE];
    row[..prefix.len()].copy_from_slice(prefix);
    row[WIDTH] = prefix.len() as u32;
    row[WIDTH + 1..].copy_from_slice(&sites);
    Ok(row)
}
fn fit(path: &Path, train: &Path, out: &Path) -> Result<(), Fail> {
    let bytes = fs::read(train)?;
    if sha256(&bytes) != TRAIN_HASH {
        return Err("TRAIN digest mismatch".into());
    }
    let rows: Vec<Training> = serde_json::from_slice(&bytes)?;
    if rows.len() != 288 {
        return Err("training row inventory mismatch".into());
    }
    let mut bank = Vec::with_capacity(288 * gpu::STRIDE);
    for row in rows {
        if row.other_source == row.source
            || row.other_fact == row.fact
            || row.other_source as usize >= row.prefix.len()
            || row.other_fact >= row.other_source
            || row.query_other.is_some_and(|i| {
                i == row.query
                    || i as usize >= row.prefix.len()
                    || row.prefix[i as usize] != row.prefix[row.other_fact as usize]
            })
        {
            return Err("inconsistent disjoint TRAIN anchors".into());
        }
        bank.extend(pack(
            &row.prefix,
            [
                row.query,
                row.query_other.unwrap_or(BOUNDARY),
                row.fact,
                row.source,
            ],
        )?);
    }
    let m = model(path)?;
    let prototypes = gpu::Gpu::new()?.fit(&bank, &m.packed_codebook_roots())?;
    fs::create_dir(out)?;
    let mut state = State {
        schema: SCHEMA.into(),
        artifact_sha256: m.artifact_sha256().into(),
        train_sha256: TRAIN_HASH.into(),
        arm: "trained".into(),
        prototypes,
    };
    write(&out.join("trained.json"), &state)?;
    state.arm = "query_ablated".into();
    write(&out.join("query_ablated.json"), &state)?;
    Ok(())
}
fn predict(path: &Path, state: &Path, input: &Path, out: &Path) -> Result<(), Fail> {
    let m = model(path)?;
    let bytes = fs::read(state)?;
    let s: State = serde_json::from_slice(&bytes)?;
    let ablated = match s.arm.as_str() {
        "query_ablated" => true,
        "trained" => false,
        _ => return Err("unregistered arm".into()),
    };
    if s.schema != SCHEMA
        || s.artifact_sha256 != m.artifact_sha256()
        || s.train_sha256 != TRAIN_HASH
    {
        return Err("model/state/TRAIN mismatch".into());
    }
    let queries: Vec<Query> = serde_json::from_slice(&fs::read(input)?)?;
    if !(1..=gpu::MAX_PREDICTIONS).contains(&queries.len()) {
        return Err("prediction panel outside device row bounds".into());
    }
    let mut bank = Vec::with_capacity(queries.len() * gpu::STRIDE);
    for q in &queries {
        bank.extend(pack(&q.prefix, [BOUNDARY; 4])?);
    }
    let roots = m.packed_codebook_roots();
    let predictions = gpu::Gpu::new()?.predict(&bank, &roots, &s.prototypes, ablated)?;
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for (q, choice) in queries.iter().zip(predictions) {
        if select(&q.prefix, &roots, &s.prototypes, ablated)? != choice {
            return Err("native CUDA/Rust triplet mismatch".into());
        }
        let prefix_sha256 = sha256(
            &q.prefix
                .iter()
                .flat_map(|id| id.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        if !seen.insert(prefix_sha256.clone()) {
            return Err("duplicate blind prefix".into());
        }
        rows.push(Row {
            prefix_sha256,
            choice,
        });
    }
    write(
        out,
        &Predictions {
            artifact_sha256: s.artifact_sha256,
            state_sha256: sha256(&bytes),
            arm: s.arm,
            rows,
        },
    )
}
fn fixture() -> Result<(), Fail> {
    let roots: Vec<u32> = (0..4096).map(|i| i * 37 & 65535).collect();
    let gpu = gpu::Gpu::new()?;
    let train = pack(
        &[0, 3, 17, 20, 40, 5, 18, 21, 41, 10, 3, 11, 12, 5, 13],
        [10, 13, 1, 3],
    )?;
    let m = gpu.fit(&train, &roots)?;
    // One duplicated example must leave each geometric bank byte-identical.
    let mut expected_relation = [BOUNDARY; 16];
    expected_relation[..2].copy_from_slice(&[1, roots[17]]);
    if m != gpu.fit(&[train, train].concat(), &roots)? || m.relation != [expected_relation] {
        return Err("device prototype update/deduplication failure".into());
    }
    let positive = [
        0, 3, 17, 20, 40, 5, 18, 21, 41, 12, 3, 13, 41, 10, 3, 11, 12, 5, 13,
    ];
    let empty = [0, 2, 3, 4];
    let padded = [
        pack(&positive, [BOUNDARY; 4])?,
        pack(&empty, [BOUNDARY; 4])?,
    ]
    .concat();
    for (ablated, q) in [(false, 14), (true, 10)] {
        let output = gpu.predict(&padded, &roots, &m, ablated)?;
        let correct = Choice {
            query: Some(q),
            fact: Some(1),
            source: Some(3),
        };
        if output != [correct, Choice::default()]
            || select(&positive, &roots, &m, ablated)? != correct
            || select(&empty, &roots, &m, ablated)? != Choice::default()
        {
            return Err("finite trained/ablated geometry, tie or empty-domain failure".into());
        }
    }
    // Relabel only the same TRAIN fact's value one token later. The learned
    // bridge must move the returned source and preserve the selected mention.
    let mut displaced_train = train;
    displaced_train[WIDTH + 4] += 1;
    let delayed = gpu.fit(&displaced_train, &roots)?;
    let shifted = gpu.predict(&padded, &roots, &delayed, false)?[0];
    if shifted
        != (Choice {
            query: Some(14),
            fact: Some(1),
            source: Some(4),
        })
        || shifted != select(&positive, &roots, &delayed, false)?
    {
        return Err("relation-label displacement ignored".into());
    }
    // Identical rooted codes at distinct raw token IDs never authorize a join.
    let mut alias_roots = roots.clone();
    alias_roots[222] = alias_roots[3];
    let collision = [0, 3, 17, 20, 222];
    if gpu.predict(&pack(&collision, [BOUNDARY; 4])?, &alias_roots, &m, false)?[0]
        != Choice::default()
        || select(&collision, &alias_roots, &m, false)? != Choice::default()
    {
        return Err("root collision bypassed exact entity ID".into());
    }
    // All interior contexts tie. q then f pick the first interior pair.
    let zroots = vec![0; 4096];
    let mut bridge = [BOUNDARY; 16];
    bridge[..2].copy_from_slice(&[1, 0]);
    let tied = Prototypes {
        requested: vec![[0; 4]],
        excluded: vec![[0; 4]],
        fact: vec![[0; 4]],
        relation: vec![bridge],
    };
    let tie = [0, 3, 3, 3, 3, 3, 3, 3, 3, 3];
    let expected = Choice {
        query: Some(4),
        fact: Some(2),
        source: Some(3),
    };
    if gpu.predict(&pack(&tie, [BOUNDARY; 4])?, &zroots, &tied, false)?[0] != expected
        || select(&tie, &zroots, &tied, false)? != expected
    {
        return Err("earliest tied geometric entity pair changed".into());
    }
    println!("CUDA/Rust finite triples matched: trained (14,1,3), ablated (10,1,3), displaced relation (14,1,4), tied pair (4,2,3), no eligible pair for empty domain and distinct IDs with equal packed roots. Duplicate TRAIN prototypes unchanged. Cubin {}",sha256(gpu::CUBIN));
    Ok(())
}
fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let Some((command, paths)) = args.split_first() else {
        return Err("missing entity selector command".into());
    };
    let paths: Vec<_> = paths.iter().map(Path::new).collect();
    match (command.to_str(), paths.as_slice()) {
        (Some("fixture"), []) => fixture(),
        (Some("fit"), [m, t, o]) => fit(m, t, o),
        (Some("predict"), [m, s, q, o]) => predict(m, s, q, o),
        _ => Err(concat!(
            "content-selector fixture | fit MODEL TRAIN NEW_DIR | ",
            "predict MODEL STATE BARE_PREFIXES NEW_JSON"
        )
        .into()),
    }
}
