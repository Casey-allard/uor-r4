//! Observe the frozen downstream reader after a supplied correct assertion set.
use serde::Deserialize;
use serde_json::json;
use std::{env, error::Error, fs, path::Path};
use uor_r4_integer::{discrete_d11::IntegerDiscreteModel, paired_role::RoleSpan};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn read<T: for<'a> Deserialize<'a>>(p: &Path) -> Result<Vec<T>> {
    fs::read_to_string(p)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}
#[derive(Deserialize)]
struct Blind {
    row: u32,
    ids: Vec<u32>,
}
#[derive(Deserialize)]
struct Person {
    row: u32,
    requested: [usize; 2],
}
#[derive(Deserialize)]
struct Fact {
    person: [usize; 2],
    relation: [usize; 2],
    value: [usize; 2],
}
#[derive(Deserialize)]
struct Key {
    row: u32,
    stratum: String,
    assertions: Vec<Fact>,
    selected: Vec<usize>,
    expected_value: Option<String>,
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [artifact, panel] = args.as_slice() else {
        return Err("usage: d27-value-witness PARENT PANEL".into());
    };
    let model = IntegerDiscreteModel::load(Path::new(artifact))?;
    if ![
        "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3",
        "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4",
    ]
    .contains(&model.artifact_sha256())
    {
        return Err("unexpected parent".into());
    }
    let panel = Path::new(panel);
    let inputs = read::<Blind>(&panel.join("held.blind.jsonl"))?;
    let keys = read::<Key>(&panel.join("held.gold.jsonl"))?;
    let people = read::<Person>(&panel.join("held.person.jsonl"))?;
    if !(inputs.len() == 92 && keys.len() == inputs.len() && people.len() == inputs.len()) {
        return Err("incomplete panel".into());
    }
    let mut rows = Vec::new();
    for ((b, k), p) in inputs.iter().zip(keys).zip(people) {
        if b.row != k.row || b.row != p.row {
            return Err("row mismatch".into());
        }
        let mut roles = model.paired_roles_diagnostic(&b.ids, 0..b.ids.len(), false)?;
        let [start, end] = p.requested;
        roles.requested = Some(RoleSpan { start, end });
        let selected = k
            .selected
            .iter()
            .map(|&i| {
                let [start, end] = k.assertions[i].person;
                RoleSpan { start, end }
            })
            .collect::<Vec<_>>();
        let (decision, w) = model.paired_copy_selected_witness(&b.ids, roles, &selected)?;
        let mut observations = Vec::new();
        for (a, &i) in w.into_iter().zip(&k.selected) {
            let h = &k.assertions[i];
            if a.fact_span != h.person {
                return Err("crossed witness".into());
            }
            observations.push(json!({"expected_value_span":h.value,"predicate_span":h.relation,"gold_offset_from_predicate_end":h.value[0] as i64-h.relation[1] as i64,"witness":a}));
        }
        rows.push(json!({"row":b.row,"stratum":k.stratum,"expected_value":k.expected_value,"decision":decision,"assertions":observations}));
    }
    serde_json::to_writer(std::io::stdout(), &rows)?;
    Ok(())
}
