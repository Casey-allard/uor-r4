use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, env, error::Error, fs, path::Path};
use uor_r4_integer::{
    discrete_d11::{
        pack_assertion_retrieval, ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS,
    },
    paired_role::RoleSpan,
    relation_retrieval::Arm,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const LIMIT: usize = 16;

fn load<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<Vec<T>> {
    fs::read_to_string(path)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}
fn span([start, end]: [usize; 2]) -> RoleSpan {
    RoleSpan { start, end }
}
fn coords(s: RoleSpan) -> [usize; 2] {
    [s.start, s.end]
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
struct Head {
    person: [usize; 2],
    value: [usize; 2],
}
#[derive(Deserialize)]
struct Key {
    row: u32,
    stratum: String,
    factors: Option<Vec<usize>>,
    instructed: bool,
    assertions: Vec<Head>,
    selected: Vec<usize>,
    selected_value_spans: Vec<[usize; 2]>,
    expected_tape: Option<Vec<u32>>,
    expected_value: Option<String>,
}
#[derive(Deserialize)]
struct Endpoint {
    start: usize,
    end: usize,
}
impl Endpoint {
    fn native(&self) -> RoleSpan {
        RoleSpan {
            start: self.start,
            end: self.end,
        }
    }
}
#[derive(Deserialize)]
struct Selection {
    selected_facts: Vec<Endpoint>,
}
#[derive(Deserialize)]
struct Observation {
    row: u32,
    lane: String,
    requested: Option<Endpoint>,
    ordinary_role_words: [u32; 8],
    exact: Option<Selection>,
    geometry: Option<Selection>,
}

fn emit(
    model: &IntegerDiscreteModel,
    ids: &[u32],
    source: Option<u32>,
) -> Result<(Vec<u32>, Vec<[u32; 8]>)> {
    let mut context = ids.to_vec();
    let mut cursor = ExactCopyState::new(source, ids.len().try_into()?);
    let (mut tape, mut steps) = (Vec::new(), Vec::new());
    for _ in 0..LIMIT {
        let trial = cursor
            .write(ids, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        let step = if trial.copied != 0 || trial.completed_span() {
            trial
        } else {
            cursor
                .write(
                    ids,
                    model.evaluate_prefix(&context)?.prediction,
                    ROLE_COPY_STOPS,
                )?
                .finish_with_eos(true, 1)
        };
        cursor = step.state;
        tape.push(step.emitted);
        steps.push(step.words());
        if step.emitted == 1 {
            break;
        }
        if context.len() == model.shape().context as usize {
            context.remove(0);
        }
        context.push(step.emitted);
    }
    Ok((tape, steps))
}
fn grade(model_file: &str, panel: &Path, native_file: &Path, clients: &Path) -> Result<()> {
    let model = IntegerDiscreteModel::load(Path::new(model_file))?;
    let parents = [
        "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3",
        "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4",
    ];
    if !parents.contains(&model.artifact_sha256()) {
        return Err("unexpected downstream carrier".into());
    }
    let blind = load::<Blind>(&panel.join("held.blind.jsonl"))?;
    let people = load::<Person>(&panel.join("held.person.jsonl"))?;
    let keys = load::<Key>(&panel.join("held.gold.jsonl"))?;
    let observations = load::<Observation>(native_file)?;
    if blind.len() != keys.len()
        || blind.len() != people.len()
        || observations.len() != 3 * blind.len()
    {
        return Err("incomplete panel".into());
    }
    let mut emitted = Vec::new();
    for (((input, person), key), views) in blind
        .iter()
        .zip(people)
        .zip(keys)
        .zip(observations.chunks_exact(3))
    {
        if input.row != person.row
            || input.row != key.row
            || views.iter().any(|v| v.row != input.row)
        {
            return Err("crossed panel row".into());
        }
        let ids = &input.ids;
        let ordinary = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
        let expected_heads: Vec<_> = key
            .selected
            .iter()
            .map(|&i| key.assertions[i].person)
            .collect();
        let mut jobs = Vec::new();
        for (v, kind) in views.iter().zip(["ordinary", "person", "person_relation"]) {
            let requested = v.requested.as_ref().map(Endpoint::native);
            if v.lane != kind
                || ordinary.words() != v.ordinary_role_words
                || (kind == "ordinary" && requested != ordinary.requested)
                || (kind != "ordinary" && requested != Some(span(person.requested)))
            {
                return Err("role/device index mismatch".into());
            }
            let mut roles = ordinary;
            roles.requested = requested;
            if kind != "person_relation" {
                let (decision, w) = model.paired_copy_witness(ids, roles, None)?;
                jobs.push((
                    kind,
                    "unfiltered",
                    roles.requested,
                    w.iter().map(|w| w.fact_span).collect::<Vec<_>>(),
                    decision.choice.source,
                    Some(json!(decision)),
                ));
            }
            for (arm, selection) in [
                ("exact", v.exact.as_ref()),
                ("geometry", v.geometry.as_ref()),
            ] {
                let heads: Vec<_> = selection
                    .into_iter()
                    .flat_map(|s| s.selected_facts.iter().map(Endpoint::native))
                    .collect();
                let (decision, w) = model.paired_copy_selected_witness(ids, roles, &heads)?;
                jobs.push((
                    kind,
                    arm,
                    roles.requested,
                    w.iter().map(|w| w.fact_span).collect(),
                    decision.choice.source,
                    Some(json!(decision)),
                ));
            }
        }
        let mut roles = ordinary;
        roles.requested = Some(span(person.requested));
        let head_spans = expected_heads.iter().copied().map(span).collect::<Vec<_>>();
        let (oracle, _) = model.paired_copy_selected_witness(ids, roles, &head_spans)?;
        jobs.push((
            "oracle",
            "assertions",
            roles.requested,
            expected_heads.clone(),
            oracle.choice.source,
            Some(json!(oracle)),
        ));
        jobs.push(("disabled", "disabled", None, vec![], None, None));
        if key.expected_value.is_some() {
            let correct = key
                .selected_value_spans
                .first()
                .ok_or("missing supported value")?[0];
            jobs.push((
                "forced",
                "value",
                roles.requested,
                expected_heads.clone(),
                Some(correct as u32),
                None,
            ));
            if let Some(wrong) = key.assertions.iter().find(|a| {
                ids[a.value[0]..a.value[1]] != ids[correct..key.selected_value_spans[0][1]]
            }) {
                jobs.push((
                    "forced",
                    "opposing",
                    roles.requested,
                    vec![],
                    Some(wrong.value[0] as u32),
                    None,
                ));
            }
        }
        let mut cache = BTreeMap::new();
        for (lane, arm, requested, heads, source, decision) in jobs {
            if !cache.contains_key(&source) {
                cache.insert(source, emit(&model, ids, source)?);
            }
            let (tape, steps) = &cache[&source];
            let exact = key.expected_tape.as_ref().map(|expected| tape == expected);
            let first = key.expected_tape.as_ref().and_then(|expected| {
                (0..expected.len().max(tape.len()))
                    .find(|&i| expected.get(i) != tape.get(i))
                    .map(|i| json!({"offset":i,"expected":expected.get(i),"actual":tape.get(i)}))
            });
            if lane == "forced" && ((arm == "value") != exact.unwrap_or(false)) {
                return Err(format!("writer control failed on {}", key.row).into());
            }
            if lane == "ordinary" {
                let c: Value = serde_json::from_slice(&fs::read(
                    clients.join(arm).join(format!("{}.json", key.row)),
                )?)?;
                if c["prompt_ids"] != json!(ids)
                    || c["generated_ids"] != json!(tape)
                    || c["copy_step_words"] != json!(steps)
                    || Some(&c["paired"]) != decision.as_ref()
                    || c["stop"]
                        != if tape.last() == Some(&1) {
                            "eos"
                        } else {
                            "max_tokens"
                        }
                    || c["read_enabled"] != true
                    || c["tokenizer_sha256"]
                        != model
                            .tokenizer_sha256()
                            .ok_or("missing tokenizer binding")?
                {
                    return Err(format!("client differs on row {} arm {arm}", key.row).into());
                }
                let pin = fs::read_to_string(clients.join(arm).join("artifact.sha256"))?;
                if c["artifact_sha256"] != pin.trim() {
                    return Err("client artifact pin mismatch".into());
                }
            }
            emitted.push(json!({"row":key.row,"stratum":key.stratum,"factors":key.factors,"instructed":key.instructed,"lane":lane,"arm":arm,
                "person":requested.map(coords),"person_hit":requested.map(coords)==Some(person.requested),"heads":heads,"expected_heads":expected_heads,"head_set_hit":heads==expected_heads,
                "source":source,"source_hit":source.is_some_and(|s|key.selected_value_spans.iter().any(|v|v[0]==s as usize)) && key.expected_value.is_some(),
                "unsupported":key.expected_value.is_none(),"false_write":key.expected_value.is_none()&&source.is_some(),"expected_tape":key.expected_tape,"tape":tape,"cursor":steps,"first_wrong":first,"exact":exact,"end_eos":tape.last()==Some(&1),"decision":decision}));
        }
    }
    serde_json::to_writer(std::io::stdout(), &emitted)?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, parent, arm, out] if command == "pack" => {
            let lane = match arm.as_str() {
                "exact" => Arm::Exact,
                "geometry" => Arm::Geometry,
                _ => return Err("unknown lane".into()),
            };
            use std::io::Write;
            fs::File::create_new(out)?
                .write_all(&pack_assertion_retrieval(&fs::read(parent)?, lane)?)?;
            Ok(())
        }
        [command, parent, panel, native, clients] if command == "grade" => grade(
            parent,
            Path::new(panel),
            Path::new(native),
            Path::new(clients),
        ),
        _ => Err(
            "use pack PARENT exact|geometry NEW or grade PARENT PANEL VERIFIED_NATIVE CLIENTS"
                .into(),
        ),
    }
}
