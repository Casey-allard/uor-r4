//! Post-seal census. Group by exactly the 14 words the blind gate observed.
#[path = "../operator.rs"]
mod operator;
use operator::{Context, Geometry, Key, NONE};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, IntegerDiscreteModel};
#[derive(Deserialize)]
struct Span {
    bank: Classes,
}
#[derive(Deserialize)]
struct Classes {
    classes: Vec<u32>,
}
#[derive(Deserialize)]
struct RoleBank {
    groups: [Vec<Context>; 5],
}
#[derive(Deserialize)]
struct Gate {
    positive: Vec<Key>,
    negative: Vec<Key>,
}
#[derive(Deserialize)]
struct Grade {
    prefix: Vec<u32>,
    kind: String,
    asked_start: Option<usize>,
}
#[derive(Deserialize)]
struct Out {
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    candidate: Option<usize>,
    chosen_query: Option<usize>,
    positive_distance: u32,
    negative_distance: u32,
    nominee: u32,
    same_query: u32,
    earlier_matches: u32,
    allowed: bool,
}
fn load<T: serde::de::DeserializeOwned>(p: &str) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    let [model, span, role, gate, grade, picks] = a.as_slice() else {
        return Err("MODEL SPAN ROLE GATE GRADE ACTION".into());
    };
    let roots = IntegerDiscreteModel::load(Path::new(model))?.packed_codebook_roots();
    let c = load::<Span>(span)?.bank.classes;
    let groups = load::<RoleBank>(role)?.groups;
    let bank: Gate = load(gate)?;
    let gold: Vec<Grade> = load(grade)?;
    let out: Out = load(picks)?;
    if out.rows.len() != gold.len() {
        return Err("incomplete audit".into());
    }
    let o = spiralcore_orbit_roots();
    let g = Geometry {
        metric: std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                o[i].iter()
                    .zip(o[j])
                    .map(|(x, y)| u32::from(x.abs_diff(y)))
                    .sum()
            })
        }),
        roots: &roots,
        classes: &c,
        requested: &groups[0],
        excluded: &groups[1],
    };
    let mut keys = BTreeMap::<Option<Key>, [u32; 2]>::new();
    let mut witnesses = BTreeMap::<(Key, u32, u32, u32), u32>::new();
    let mut held = BTreeSet::new();
    let mut preferred = 0;
    let mut exact_train = 0;
    for (w, r) in gold.iter().zip(&out.rows) {
        let eg = w.kind == "positive" && w.asked_start == r.candidate && r.candidate.is_some();
        let k = g.features(&w.prefix, r.chosen_query, r.candidate);
        if let Some((x, n, _)) = k {
            let p = g.nearest(&x, &bank.positive);
            let neg = g.nearest(&x, &bank.negative);
            if r.nominee != n as u32
                || r.earlier_matches != x[12]
                || r.same_query != x[13]
                || p != r.positive_distance
                || neg != r.negative_distance
                || r.allowed != (p < neg)
            {
                return Err("changed blind representation or action".into());
            }
            if eg && !r.allowed {
                *witnesses.entry((x, p, neg, w.prefix[n])).or_default() += 1;
                held.insert(x);
            }
            if bank.positive.contains(&x) || bank.negative.contains(&x) {
                exact_train += 1;
            }
        } else if r.nominee != NONE || r.allowed {
            return Err("null witness inconsistent".into());
        }
        preferred += u32::from(eg);
        keys.entry(k.map(|(x, _, _)| x)).or_default()[usize::from(eg)] += 1;
    }
    let records:Vec<Value>=witnesses.into_iter().map(|((k,p,n,id),count)|json!({
     "feature":k,"nearest_positive":p,"nearest_negative":n,"nominee_token":id,"count":count,
     "held_copy_targets":keys[&Some(k)][1],"held_generate_targets":keys[&Some(k)][0],
     "in_train_positive_bank":bank.positive.contains(&k),"in_train_negative_bank":bank.negative.contains(&k)})).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"rows":gold.len(),"rows_with_an_exact_train_descriptor":exact_train,
  "oracle_copy_targets":preferred,"held_distinct_keys_including_null":keys.len(),"conflicting_held_keys":keys.values().filter(|v|v[0]>0&&v[1]>0).count(),
  "minimum_held_errors_for_key_only":keys.values().map(|v|v[0].min(v[1])).sum::<u32>(),
  "wrongly_withheld_keys":held.len(),"wrongly_withheld":records})
        )?
    );
    Ok(())
}
