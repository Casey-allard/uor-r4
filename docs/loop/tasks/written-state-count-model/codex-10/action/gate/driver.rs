mod operator;
use operator::{Context, Geometry, Key, NONE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, IntegerDiscreteModel};
type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn save(p: &str, v: &impl Serialize) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(p)?;
    serde_json::to_writer(&mut f, v)?;
    f.write_all(b"\n")?;
    Ok(())
}
fn load<T: serde::de::DeserializeOwned>(p: &str) -> Result<(T, String)> {
    let b = fs::read(p)?;
    Ok((serde_json::from_slice(&b)?, hash(&b)))
}
fn prefix_hash(ids: &[u32]) -> String {
    hash(&ids.iter().flat_map(|i| i.to_le_bytes()).collect::<Vec<_>>())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bare {
    prefix: Vec<u32>,
}
#[derive(Deserialize)]
struct Roles {
    artifact: String,
    groups: [Vec<Context>; 5],
}
#[derive(Deserialize)]
struct Span {
    bank: Classes,
}
#[derive(Deserialize)]
struct Classes {
    classes: Vec<u32>,
}
#[derive(Deserialize)]
struct RoleTrace {
    arm: String,
    artifact_sha256: String,
    bare_sha256: String,
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    prefix_sha256: String,
    frozen: Anchor,
    continuation: Selection,
}
#[derive(Deserialize)]
struct Anchor {
    query: Option<usize>,
}
#[derive(Deserialize)]
struct Selection {
    source: Option<usize>,
}
#[derive(Deserialize)]
struct Grade {
    prefix: Vec<u32>,
    kind: String,
    asked_start: Option<usize>,
}
#[derive(Serialize, Deserialize)]
struct State {
    schema: String,
    artifact: String,
    role_sha256: String,
    span_sha256: String,
    train_bare_sha256: String,
    train_grade_sha256: String,
    train_handoff_sha256: String,
    positive: Vec<Key>,
    negative: Vec<Key>,
    eligible_training_rows: u32,
    conflicting_keys: u32,
    minimum_training_errors: u32,
}
struct Item<'a> {
    ids: &'a [u32],
    q: Option<usize>,
    source: Option<usize>,
    label: u32,
}
fn metric() -> [[u32; 16]; 16] {
    let o = spiralcore_orbit_roots();
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            o[i].iter()
                .zip(o[j])
                .map(|(x, y)| u32::from(x.abs_diff(y)))
                .sum()
        })
    })
}
fn execute(
    exe: &str,
    geo: &Geometry,
    roles: &[Vec<Context>; 5],
    pos: &[Key],
    neg: &[Key],
    items: &[Item],
) -> Result<Vec<[u32; 20]>> {
    let mut ends = [0; 5];
    let mut count = 0;
    for (i, g) in roles.iter().enumerate() {
        count += g.len() as u32;
        ends[i] = count;
    }
    if items.is_empty()
        || items.len() > 4096
        || geo.classes.len() != 4096
        || geo.roots.len() != 4096
        || roles[..2].iter().any(Vec::is_empty)
    {
        return Err("invalid data dimensions".into());
    }
    let mut words = vec![
        items.len() as u32,
        count,
        pos.len() as u32,
        (pos.len() + neg.len()) as u32,
    ];
    words.extend(geo.roots);
    words.extend(geo.classes);
    words.extend(geo.metric.as_flattened());
    words.extend(ends);
    for s in roles.iter().flatten() {
        words.extend(s);
    }
    for s in pos.iter().chain(neg) {
        words.extend(s);
    }
    for r in items {
        if r.ids.is_empty()
            || r.ids.len() > 192
            || r.ids[0] != 0
            || r.ids[1..].iter().any(|&id| !(2..4096).contains(&id))
            || r.q.into_iter().chain(r.source).any(|p| p >= r.ids.len())
        {
            return Err("invalid action row".into());
        }
        let mut v = [0; 584];
        v[..r.ids.len()].copy_from_slice(r.ids);
        v[576] = r.ids.len() as u32;
        v[577] = r.q.map_or(NONE, |p| p as u32);
        v[580] = r.source.map_or(NONE, |p| p as u32);
        v[581] = r.label;
        words.extend(v);
    }
    let buf: Vec<_> = words.into_iter().flat_map(|u| u.to_le_bytes()).collect();
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("missing input")?.write_all(&buf)?;
    let result = child.wait_with_output()?;
    if !result.status.success() || result.stdout.len() != items.len() * 20 * 4 {
        return Err("incomplete device response".into());
    }
    let mut out = Vec::new();
    for (i, raw) in result.stdout.chunks_exact(80).enumerate() {
        let device =
            std::array::from_fn(|j| u32::from_le_bytes(raw[4 * j..4 * j + 4].try_into().unwrap()));
        let r = &items[i];
        let mut reference = [NONE; 20];
        reference[19] = 0;
        if let Some((key, nom, bound)) = geo.features(r.ids, r.q, r.source) {
            reference[..14].copy_from_slice(&key);
            reference[14] = nom as u32;
            reference[15] = bound as u32;
            reference[16] = r.label;
            reference[17] = geo.nearest(&key, pos);
            reference[18] = geo.nearest(&key, neg);
            reference[19] = u32::from(reference[17] < reference[18]);
        }
        if reference != device {
            return Err(format!("action Rust/device mismatch row {i}").into());
        }
        out.push(device);
    }
    Ok(out)
}
fn run(cmd: &str, a: &[String]) -> Result<()> {
    let [exe, model, sp, role, tape, handoff, extra, out] = a else {
        return Err("expected 8 arguments".into());
    };
    let m = IntegerDiscreteModel::load(Path::new(model))?;
    let id = m.artifact_sha256();
    let (roles, rhash) = load::<Roles>(role)?;
    let (span, shash) = load::<Span>(sp)?;
    let (bare, bhash) = load::<Vec<Bare>>(tape)?;
    let (h, hh) = load::<RoleTrace>(handoff)?;
    let pins = [
        (
            "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
            "1d98243a79f9b68537add8445397316773a028260c39997fa9d9b1dc04dec48e",
            "3451b35c018687cd4ae37f80f306ffe2af3724eb4969d2f7271828cf1c67f870",
        ),
        (
            "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
            "32ac74888a382198f737c98d1c67d8352a93ead9d884a98e15f2768e6e86457b",
            "090c53c07a3c16e7d017c931b02e6749bca98affe71c5f8ca19e89f55d57436f",
        ),
    ];
    if !pins.contains(&(id, rhash.as_str(), shash.as_str()))
        || roles.artifact != id
        || h.arm != "role"
        || h.artifact_sha256 != id
        || h.bare_sha256 != bhash
        || bare.len() != h.rows.len()
    {
        return Err("action seed, bank or tape binding mismatch".into());
    }
    let mut state = None;
    let mut grade = Vec::<Grade>::new();
    let xh;
    if cmd == "fit" {
        let (g, h) = load::<Vec<Grade>>(extra)?;
        grade = g;
        xh = h;
        if grade.len() != bare.len() {
            return Err("TRAIN coverage".into());
        }
    } else {
        let (s, h) = load::<State>(extra)?;
        xh = h;
        if s.schema != "uor-r4.copy-eligibility/1"
            || s.role_sha256 != rhash
            || s.span_sha256 != shash
            || s.artifact != id
            || s.positive.is_empty()
            || s.negative.is_empty()
        {
            return Err("incompatible action bank".into());
        }
        state = Some(s);
    }
    let roots = m.packed_codebook_roots();
    let geo = Geometry {
        roots: &roots,
        classes: &span.bank.classes,
        metric: metric(),
        requested: &roles.groups[0],
        excluded: &roles.groups[1],
    };
    let mut items = Vec::new();
    for (i, (b, r)) in bare.iter().zip(&h.rows).enumerate() {
        if r.prefix_sha256 != prefix_hash(&b.prefix) {
            return Err("prefix/handoff drift".into());
        }
        let label = if cmd == "fit" {
            if grade[i].prefix != b.prefix
                || !matches!(
                    grade[i].kind.as_str(),
                    "positive" | "absent" | "ambiguous" | "prose"
                )
                || ((grade[i].kind == "positive") != grade[i].asked_start.is_some())
            {
                return Err("invalid TRAIN labels".into());
            }
            u32::from(grade[i].kind == "positive" && grade[i].asked_start == r.continuation.source)
        } else {
            NONE
        };
        items.push(Item {
            ids: &b.prefix,
            q: r.frozen.query,
            source: r.continuation.source,
            label,
        });
    }
    let (pos, neg) = state
        .as_ref()
        .map_or((&[][..], &[][..]), |s| (&s.positive[..], &s.negative[..]));
    let device = execute(exe, &geo, &roles.groups, pos, neg, &items)?;
    if cmd == "fit" {
        let (mut p, mut n) = (BTreeSet::new(), BTreeSet::new());
        let mut counts = BTreeMap::<Key, [u32; 2]>::new();
        for w in device.iter().filter(|r| r[14] != NONE) {
            let key: Key = w[..14].try_into()?;
            if w[16] == 1 {
                p.insert(key);
            } else {
                n.insert(key);
            }
            counts.entry(key).or_default()[w[16] as usize] += 1;
        }
        if p.is_empty() || n.is_empty() {
            return Err("vacuous fit".into());
        }
        return save(
            out,
            &State {
                schema: "uor-r4.copy-eligibility/1".into(),
                artifact: id.into(),
                role_sha256: rhash,
                span_sha256: shash,
                train_bare_sha256: bhash,
                train_grade_sha256: xh,
                train_handoff_sha256: hh,
                positive: p.into_iter().collect(),
                negative: n.into_iter().collect(),
                eligible_training_rows: counts.values().flatten().sum(),
                conflicting_keys: counts.values().filter(|v| v[0] * v[1] > 0).count() as u32,
                minimum_training_errors: counts.values().map(|v| v[0].min(v[1])).sum(),
            },
        );
    }
    let rows:Vec<_>=device.iter().zip(&h.rows).map(|(w,r)|{
   let allowed=w[19]!=0;json!({"prefix_sha256":r.prefix_sha256,"source":r.continuation.source.filter(|_|allowed),
      "allowed":allowed,"candidate":r.continuation.source,"chosen_query":r.frozen.query,"nominee":w[14],
      "earlier_matches":w[12],"same_query":w[13],"positive_distance":w[17],"negative_distance":w[18]})
 }).collect();
    save(
        out,
        &json!({"schema":"uor-r4.copy-eligibility-handoff/1","arm":"action","bare_sha256":bhash,"artifact_sha256":id,"role_sha256":hh,"gate_sha256":xh,"rows":rows}),
    )
}
fn fixture(exe: &str) -> Result<()> {
    let roots: Vec<u32> = (0..4096).map(|i| i * 9).collect();
    let cls = vec![1; 4096];
    let ids = [0, 7, 11, 3, 9, 7, 21, 31, 41];
    let ctx = [
        roots[11], roots[3], roots[9], roots[21], roots[31], roots[41],
    ];
    let groups = [vec![ctx], vec![[NONE; 6]], vec![], vec![], vec![]];
    let geo = Geometry {
        roots: &roots,
        classes: &cls,
        metric: metric(),
        requested: &groups[0],
        excluded: &groups[1],
    };
    let item = Item {
        ids: &ids,
        q: Some(5),
        source: Some(3),
        label: NONE,
    };
    let (positive, nom, _) = geo
        .features(&ids, item.q, item.source)
        .ok_or("missing finite predicate")?;
    if nom != 5 || positive[12..] != [1, 1] {
        return Err("wrong finite occurrence".into());
    }
    let mut negative = positive;
    negative[12] = 0;
    let missing = [0, 8, 11, 3, 9, 7, 21, 31, 41];
    let multiple = [0, 7, 7, 11, 3, 9, 7, 21, 31, 41];
    let train = [
        item,
        Item {
            ids: &missing,
            q: Some(5),
            source: Some(3),
            label: NONE,
        },
        Item {
            ids: &ids,
            q: None,
            source: None,
            label: NONE,
        },
    ];
    let w = execute(exe, &geo, &groups, &[positive], &[negative], &train)?;
    if w[0][19] != 1 || w[1][19] != 0 || w[2][19] != 0 {
        return Err("copy/generate/null release failed".into());
    }
    let ties = execute(exe, &geo, &groups, &[positive], &[positive], &train)?;
    if ties.iter().any(|v| v[19] != 0) {
        return Err("equal action distance did not generate".into());
    }
    let repeated = [Item {
        ids: &multiple,
        q: Some(6),
        source: Some(3),
        label: NONE,
    }];
    let mut duplicate = positive;
    duplicate[12] = 2;
    let ambiguous = execute(
        exe,
        &geo,
        &groups,
        &[positive],
        &[negative, duplicate],
        &repeated,
    )?;
    if ambiguous[0][12] != 2 || ambiguous[0][19] != 0 {
        return Err("multiple matches failed".into());
    }
    println!("PASS positive, absent, multiple-match, null and tie using bit-identical native Rust / compiled-host / GPU evidence words");
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.as_slice(){[c,exe]if c=="fixture"=>fixture(exe),[c,args @ ..]if matches!(c.as_str(),"fit"|"blind")=>run(c,args),
  _=>Err("copy-gate fixture DEVICE | fit|blind DEVICE MODEL SPAN ROLE_BANK BARE ROLE_HANDOFF TRAIN_GRADE|GATE_BANK NEW_OUT".into())}
}
