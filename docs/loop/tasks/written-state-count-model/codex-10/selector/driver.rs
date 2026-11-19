//! Inputs to the Wick operator. Device-produced words must match its host replay.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, IntegerDiscreteModel};
type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
const W: usize = 192;
const NIL: u32 = u32::MAX;
const TRAIN: &str = "2930a72c927a102d6d8e404bd7970bc88479b45e2ef05acfbba83f443520516c";
const MODELS: [&str; 2] = [
    "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
    "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
];
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn json_file<T: serde::de::DeserializeOwned>(p: &str) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn save(p: &Path, v: &impl Serialize) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(p)?;
    serde_json::to_writer(&mut f, v)?;
    f.write_all(b"\n")?;
    Ok(())
}
#[derive(Deserialize)]
struct SpanState {
    bank: ClassBank,
}
#[derive(Deserialize)]
struct ClassBank {
    classes: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bare {
    prefix: Vec<u32>,
}
#[derive(Deserialize)]
struct Grade {
    prefix: Vec<u32>,
    asked_start: u32,
    other_start: u32,
    token_positions: Positions,
}
#[derive(Deserialize)]
struct Positions {
    query_start: u32,
    fact_start: u32,
    query_other_start: Option<u32>,
}
#[derive(Deserialize)]
struct Prior {
    arm: String,
    artifact_sha256: String,
    bare_sha256: String,
    rows: Vec<Previous>,
}
#[derive(Deserialize)]
struct Previous {
    prefix_sha256: String,
    site_scores: Vec<[u32; 2]>,
    frozen: Anchors,
}
#[derive(Deserialize)]
struct Anchors {
    query: Option<u32>,
    fact: Option<u32>,
}
#[derive(Serialize, Deserialize)]
struct Bank {
    artifact: String,
    train: String,
    groups: [Vec<[u32; 6]>; 5],
}
fn new_groups() -> [Vec<[u32; 6]>; 5] {
    Default::default()
}
fn token_hash(p: &[u32]) -> String {
    hash(&p.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>())
}
fn row(ids: &[u32], sites: &[[u32; 2]], labels: [u32; 5]) -> Result<Vec<u32>> {
    if ids.is_empty()
        || ids.len() > W
        || ids[0] != 0
        || ids[1..].iter().any(|&x| !(2..4096).contains(&x))
        || (!sites.is_empty() && sites.len() != ids.len())
        || labels.iter().any(|&x| x != NIL && x as usize >= ids.len())
    {
        return Err("bad input row".into());
    }
    let mut v = vec![0; W * 3 + 8];
    v[..ids.len()].copy_from_slice(ids);
    v[W..W + 2 * sites.len()].copy_from_slice(sites.as_flattened());
    v[W * 3] = ids.len() as u32;
    v[W * 3 + 1..W * 3 + 6].copy_from_slice(&labels);
    Ok(v)
}
fn launch(
    exe: &str,
    roots: &[u32],
    classes: &[u32],
    bank: &Bank,
    rows: &[Vec<u32>],
    fit: bool,
) -> Result<Vec<u32>> {
    if roots.len() != 4096
        || classes.len() != 4096
        || roots.iter().any(|&x| x > 65535)
        || classes.iter().any(|&x| x > 2)
        || rows.is_empty()
        || rows.len() > 4096
    {
        return Err("bad selector dimensions".into());
    }
    let mut ends = [0; 5];
    let mut total = 0;
    for (i, group) in bank.groups.iter().enumerate() {
        total += group.len() as u32;
        ends[i] = total;
    }
    let mut words = vec![u32::from(fit), rows.len() as u32, total];
    words.extend(roots);
    words.extend(classes);
    let orb = spiralcore_orbit_roots();
    for a in orb {
        for b in orb {
            words.push(
                a.iter()
                    .zip(b)
                    .map(|(x, y)| u32::from(x.abs_diff(*y)))
                    .sum(),
            );
        }
    }
    words.extend(ends);
    for g in &bank.groups {
        for s in g {
            words.extend(s);
        }
    }
    for r in rows {
        words.extend(r);
    }
    let bytes: Vec<_> = words
        .into_iter()
        .flat_map(|v: u32| v.to_le_bytes())
        .collect();
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("missing device pipe")?;
    // Device executable reads complete input before emitting data.
    stdin.write_all(&bytes)?;
    drop(stdin);
    let mut out = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing device response")?
        .read_to_end(&mut out)?;
    if !child.wait()?.success() || out.len() != rows.len() * 4 * if fit { 42 } else { 20 } {
        return Err("incomplete device result".into());
    }
    Ok(out
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}
fn collect_bank(words: &[u32]) -> Result<[Vec<[u32; 6]>; 5]> {
    let mut groups: [BTreeSet<[u32; 6]>; 5] = Default::default();
    for s in words.chunks_exact(7) {
        if s[0] != NIL {
            groups
                .get_mut(s[0] as usize)
                .ok_or("invalid bank kind")?
                .insert(s[1..].try_into()?);
        }
    }
    if groups.iter().any(|g| g.is_empty()) {
        return Err("missing TRAIN class".into());
    }
    Ok(groups.map(|g| g.into_iter().collect()))
}
fn model(path: &str, span: &str) -> Result<(String, Vec<u32>, Vec<u32>)> {
    let model = IntegerDiscreteModel::load(Path::new(path))?;
    let id = model.artifact_sha256().to_owned();
    let seed = MODELS
        .iter()
        .position(|m| *m == id)
        .ok_or("unpinned model")?;
    let bytes = fs::read(span)?;
    if hash(&bytes)
        != [
            "3451b35c018687cd4ae37f80f306ffe2af3724eb4969d2f7271828cf1c67f870",
            "090c53c07a3c16e7d017c931b02e6749bca98affe71c5f8ca19e89f55d57436f",
        ][seed]
    {
        return Err("wrong class bank".into());
    }
    Ok((
        id,
        model.packed_codebook_roots(),
        serde_json::from_slice::<SpanState>(&bytes)?.bank.classes,
    ))
}
fn fit(exe: &str, path: &str, span: &str, train: &str, out: &str) -> Result<()> {
    let (artifact, roots, cls) = model(path, span)?;
    let data = fs::read(train)?;
    if hash(&data) != TRAIN {
        return Err("wrong training split".into());
    }
    let grade: Vec<Grade> = serde_json::from_slice(&data)?;
    let mut bank = Bank {
        artifact,
        train: TRAIN.into(),
        groups: new_groups(),
    };
    let rows = grade
        .iter()
        .map(|g| row(&g.prefix, &[], labels(g)))
        .collect::<Result<Vec<_>>>()?;
    bank.groups = collect_bank(&launch(exe, &roots, &cls, &bank, &rows, true)?)?;
    save(Path::new(out), &bank)
}
fn labels(g: &Grade) -> [u32; 5] {
    [
        g.token_positions.query_start,
        g.token_positions.query_other_start.unwrap_or(NIL),
        g.token_positions.fact_start,
        g.asked_start,
        g.other_start,
    ]
}
fn opt(x: u32) -> Option<u32> {
    (x != NIL).then_some(x)
}
fn apply(cmd: &str, args: &[String]) -> Result<()> {
    let [exe, path, span, bank_path, bare_path, old_path, out, projection @ ..] = args else {
        return Err("invalid selector arguments".into());
    };
    let (artifact, roots, cls) = model(path, span)?;
    let b = fs::read(bank_path)?;
    let bank: Bank = serde_json::from_slice(&b)?;
    if bank.artifact != artifact || bank.train != TRAIN || bank.groups.iter().any(|g| g.is_empty())
    {
        return Err("wrong frozen fit".into());
    }
    let bytes = fs::read(bare_path)?;
    let bare_hash = hash(&bytes);
    let bare: Vec<Bare> = serde_json::from_slice(&bytes)?;
    let prior: Prior = json_file(old_path)?;
    if prior.arm != "repair"
        || prior.artifact_sha256 != artifact
        || prior.bare_sha256 != bare_hash
        || prior.rows.len() != bare.len()
    {
        return Err("old blind scores do not match model and tape".into());
    }
    let grades: Option<Vec<Grade>> = match (cmd, projection) {
        ("blind", []) => None,
        ("oracle", [g]) => Some(json_file(g)?),
        _ => return Err("grades permitted only in a separate oracle process".into()),
    };
    if grades.as_ref().is_some_and(|g| g.len() != bare.len()) {
        return Err("grade length mismatch".into());
    }
    let mut input = Vec::new();
    for (i, (q, old)) in bare.iter().zip(&prior.rows).enumerate() {
        if token_hash(&q.prefix) != old.prefix_sha256 {
            return Err("old row hash mismatch".into());
        }
        let l = if let Some(g) = &grades {
            if g[i].prefix != q.prefix {
                return Err("oracle row mismatch".into());
            }
            labels(&g[i])
        } else {
            [NIL; 5]
        };
        input.push(row(&q.prefix, &old.site_scores, l)?);
    }
    let words = launch(exe, &roots, &cls, &bank, &input, false)?;
    let blind = ["baseline", "role", "boundary", "both", "no-read"];
    let oracle = ["query", "fact", "anchors", "value", "wrong"];
    let arms = if grades.is_some() { oracle } else { blind };
    for arm in arms {
        let mut rows = Vec::<Value>::new();
        for (i, r) in words.chunks_exact(20).enumerate() {
            let base = &prior.rows[i];
            if (opt(r[0]), opt(r[1])) != (base.frozen.query, base.frozen.fact) {
                return Err(format!("baseline rank drift at {i}").into());
            }
            let k = match arm {
                "role" | "both" => 1,
                "query" => 2,
                "fact" => 3,
                "anchors" => 4,
                _ => 0,
            };
            let forced = grades.as_ref().and_then(|g| match arm {
                "value" => Some(g[i].asked_start),
                "wrong" => Some(g[i].other_start),
                _ => None,
            });
            rows.push(json!({"prefix_sha256":base.prefix_sha256,"q":opt(r[4*k]),"f":opt(r[4*k+1]),
                "boundary":if matches!(arm,"both"|"boundary"){opt(r[4*k+2])}else{None},"forced":forced}));
        }
        save(
            &Path::new(out).join(format!("{arm}.json")),
            &json!({"arm":arm,"bare":bare_hash,"artifact":artifact,"state":hash(&b),"rows":rows}),
        )?;
    }
    Ok(())
}
fn fixture(exe: &str) -> Result<()> {
    let cls = vec![1; 4096];
    let roots = (0..4096).collect::<Vec<_>>();
    let mut bank = Bank {
        artifact: String::new(),
        train: String::new(),
        groups: new_groups(),
    };
    let ids = [
        0, 20, 100, 50, 51, 60, 101, 53, 54, 102, 70, 20, 76, 101, 77,
    ];
    let cells = row(&ids, &[], [11, 13, 1, 4, 8])?;
    bank.groups = collect_bank(&launch(exe, &roots, &cls, &bank, &[cells.clone()], true)?)?;
    let w = launch(exe, &roots, &cls, &bank, &[cells], false)?;
    if w[0..2] != [11, 1] || w[4..7] != [11, 1, 4] || w[8..10] != [11, 1] || w[16..18] != [11, 1] {
        return Err(format!("finite word or value trace failed {w:?}").into());
    }
    let miss = row(&[0, 2, 3, 4], &[], [NIL; 5])?;
    let blank = launch(exe, &roots, &cls, &bank, &[miss], false)?;
    if blank.iter().any(|&v| v != NIL) {
        return Err("empty pair failed".into());
    }
    println!("PASS TRAIN context classes, requested-only and full oracle, source 4, baseline tie, abstention, all device and host words");
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice(){
        [cmd,exe] if cmd=="fixture"=>fixture(exe),
        [cmd,exe,model,span,train,out] if cmd=="fit"=>fit(exe,model,span,train,out),
        [cmd,tail @ ..] if matches!(cmd.as_str(),"blind"|"oracle")=>apply(cmd,tail),
        _=>Err("fixture DEVICE | fit DEVICE MODEL SPAN TRAIN OUT | blind|oracle DEVICE MODEL SPAN BANK BARE BASELINE OUTDIR [GRADE]".into())
    }
}
