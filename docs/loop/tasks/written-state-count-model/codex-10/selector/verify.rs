//! Independent integer serving replay and observable-key collision census.
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, IntegerDiscreteModel};
type Err = Box<dyn std::error::Error>;
type Key = [u32; 6];
type Group = [Vec<Key>; 5];
type Hist = BTreeMap<Key, [usize; 2]>;
const NIL: u32 = u32::MAX;
#[derive(Deserialize)]
struct Bank {
    groups: Group,
}
#[derive(Deserialize)]
struct SpanState {
    bank: Classes,
}
#[derive(Deserialize)]
struct Classes {
    classes: Vec<u32>,
}
#[derive(Deserialize)]
struct Grade {
    prefix: Vec<u32>,
    asked_start: usize,
    token_positions: Positions,
}
#[derive(Deserialize)]
struct Positions {
    query_start: usize,
    fact_start: usize,
    query_other_start: Option<usize>,
}
#[derive(Deserialize)]
struct Sealed {
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    q: Option<usize>,
    f: Option<usize>,
    boundary: Option<usize>,
}
fn load<T: serde::de::DeserializeOwned>(p: impl AsRef<Path>) -> Result<T, Err> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
struct Tokens<'a> {
    ids: &'a [u32],
    roots: &'a [u32],
    classes: &'a [u32],
}
impl Tokens<'_> {
    fn bounds(&self, p: usize) -> (usize, usize) {
        let c = |i| self.classes[self.ids[i] as usize];
        let (mut l, mut h) = (p, p);
        if c(p) != 0 {
            while l > 0 && c(l) == 2 && c(l - 1) != 0 {
                l -= 1;
            }
            while h + 1 < self.ids.len() && c(h + 1) == 2 {
                h += 1;
            }
        }
        (l, h)
    }
    fn key(&self, start: usize, end: usize) -> Key {
        std::array::from_fn(|j| {
            let i = if j < 3 {
                start.checked_sub(3 - j)
            } else {
                Some(end + 1 + j - 3)
            };
            i.and_then(|i| self.ids.get(i))
                .map(|&id| self.roots[id as usize])
                .unwrap_or(NIL)
        })
    }
    fn onset(&self, end: usize, s: usize) -> Key {
        std::array::from_fn(|j| {
            let i = if j < 3 {
                s.checked_sub(3 - j).filter(|&i| i > end)
            } else {
                Some(s + j - 3)
            };
            i.and_then(|i| self.ids.get(i))
                .map(|&id| self.roots[id as usize])
                .unwrap_or(NIL)
        })
    }
    fn can_start(&self, j: usize) -> bool {
        self.classes[self.ids[j] as usize] == 1
    }
}
fn near(s: &Key, g: &[Key], lut: &[[u32; 16]; 16]) -> u32 {
    let d = |a: u32, b: u32| {
        if a == b {
            0
        } else if a == NIL || b == NIL {
            64
        } else {
            (0..4)
                .map(|k| lut[((a >> (4 * k)) & 15) as usize][((b >> (4 * k)) & 15) as usize])
                .sum()
        }
    };
    g.iter()
        .map(|p| p.iter().zip(s).map(|(&a, &b)| d(a, b)).sum())
        .min()
        .unwrap_or(u32::MAX)
}
fn bounds_hist(h: &Hist) -> serde_json::Value {
    let n: usize = h.values().map(|v| v.iter().sum::<usize>()).sum();
    let misses: usize = h.values().map(|v| v[0].min(v[1])).sum();
    json!({"sites":n,"distinct_keys":h.len(),"conflicting_keys":h.values().filter(|v|v[0]*v[1]!=0).count(),
         "unavoidable_binary_errors_for_key_alone":misses,"best_correct_for_key_alone":n-misses})
}
fn main() -> Result<(), Err> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    let [model, bank, span, grade, overrides] = a.as_slice() else {
        return Err("MODEL BANK SPAN GRADE TRAIN|OVERRIDE_DIR".into());
    };
    let roots = IntegerDiscreteModel::load(Path::new(model))?.packed_codebook_roots();
    let groups = load::<Bank>(bank)?.groups;
    let classes = load::<SpanState>(span)?.bank.classes;
    let data: Vec<Grade> = load(grade)?;
    let training = overrides == "TRAIN";
    let role = if training {
        None
    } else {
        Some(load::<Sealed>(Path::new(overrides).join("both.json"))?.rows)
    };
    let mut fitted: [BTreeSet<Key>; 5] = Default::default();
    let mut rh = Hist::new();
    let mut bh = Hist::new();
    let mut ids = BTreeMap::new();
    let o = spiralcore_orbit_roots();
    let lut = std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            o[i].iter()
                .zip(o[j])
                .map(|(a, b)| u32::from(a.abs_diff(b)))
                .sum()
        })
    });
    let margin = |p, n| p + 48u32.saturating_sub(n);
    for (index, g) in data.iter().enumerate() {
        let t = Tokens {
            ids: &g.prefix,
            roots: &roots,
            classes: &classes,
        };
        let a = t.bounds(g.token_positions.query_start);
        let f = t.bounds(g.token_positions.fact_start);
        let occurrence = |s: (usize, usize)| t.key(s.0, s.1);
        let query = occurrence(a);
        let fact = occurrence(f);
        fitted[0].insert(query);
        rh.entry(query).or_default()[1] += 1;
        fitted[2].insert(fact);
        if let Some(i) = g.token_positions.query_other_start {
            let other = occurrence(t.bounds(i));
            fitted[1].insert(other);
            rh.entry(other).or_default()[0] += 1;
        }
        for j in f.1 + 2..=(f.1 + 4).min(a.0 - 1) {
            if t.can_start(j) {
                let positive = j == g.asked_start;
                let key = t.onset(f.1, j);
                fitted[if positive { 3 } else { 4 }].insert(key);
                bh.entry(key).or_default()[usize::from(positive)] += 1;
            }
        }
        ids.entry(&g.prefix)
            .or_insert_with(BTreeSet::new)
            .insert(g.asked_start);
        if let Some(rows) = &role {
            let mut sites = Vec::new();
            let mut win = None;
            for q in 0..g.prefix.len() {
                let bounds = t.bounds(q);
                let k = occurrence(bounds);
                let query = margin(near(&k, &groups[0], &lut), near(&k, &groups[1], &lut));
                let fact = near(&k, &groups[2], &lut);
                sites.push((bounds, fact));
                for (f, &(b, fe)) in sites.iter().take(q.saturating_sub(1)).enumerate() {
                    if b.1 < bounds.0
                        && f - b.0 == q - bounds.0
                        && g.prefix[b.0..=b.1] == g.prefix[bounds.0..=bounds.1]
                    {
                        let v = (query + fe, q, f);
                        if win.is_none_or(|w| v < w) {
                            win = Some(v);
                        }
                    }
                }
            }
            let (q, f) = win.map_or((None, None), |(_, q, f)| (Some(q), Some(f)));
            let mut bv = None;
            if let (Some(q), Some(f)) = (q, f) {
                let end = sites[f].0 .1;
                let lim = sites[q].0 .0;
                for j in end + 2..(end + 5).min(lim) {
                    if t.can_start(j) {
                        let k = t.onset(end, j);
                        let e = margin(near(&k, &groups[3], &lut), near(&k, &groups[4], &lut));
                        if bv.is_none_or(|p| (e, j) < p) {
                            bv = Some((e, j));
                        }
                    }
                }
            }
            let r = rows.get(index).ok_or("missing blind output row")?;
            if (r.q, r.f, r.boundary) != (q, f, bv.map(|(_, j)| j)) {
                return Err(format!("held replay mismatch {index}").into());
            }
        }
    }
    if training && fitted.map(|s| s.into_iter().collect::<Vec<_>>()) != groups {
        return Err("TRAIN bank words differ".into());
    }
    println!(
        "{}",
        json!({"rows":data.len(),"train":training,"group_sizes":groups.each_ref().map(|g|g.len()),"query_role":bounds_hist(&rh),
    "value_onset":bounds_hist(&bh),"identical_tapes_with_multiple_requested_addresses":ids.values().filter(|s|s.len()>1).count()})
    );
    Ok(())
}
