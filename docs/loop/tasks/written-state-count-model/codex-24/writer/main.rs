//! Finite downstream intervention. All arms call the original binder and cursor.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{env, error::Error, fs, path::Path};
use uor_r4_integer::discrete_d11::{ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS};
use uor_r4_integer::paired_role::{PairedRoleDecision, RoleSpan};

type Fallible<T> = Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Case {
    prompt_id: String,
    prefix: Vec<u32>,
    ordinary_client: Value,
    identity: Vec<[usize; 2]>,
    positional: Vec<[usize; 2]>,
    gold_person: [usize; 2],
    gold_ids: Vec<u32>,
    gold_onset: usize,
}

fn write(model: &IntegerDiscreteModel, prompt: &[u32], source: Option<u32>) -> Fallible<Value> {
    let mut cursor = ExactCopyState::new(source, prompt.len() as u32);
    let mut context = prompt.to_vec();
    let mut ids = Vec::new();
    let mut steps = Vec::new();
    for _ in 0..16 {
        let ordinary = model.evaluate_prefix(&context)?.prediction;
        let step = cursor
            .write(prompt, ordinary, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        steps.push(step.words());
        ids.push(step.emitted);
        cursor = step.state;
        if step.emitted == 1 {
            break;
        }
        if context.len() == model.shape().context as usize {
            context.remove(0);
        }
        context.push(step.emitted);
    }
    Ok(json!({"source":source,"ids":ids,"steps":steps,"eos":ids.last()==Some(&1)}))
}

fn run(model: &IntegerDiscreteModel, case: Case) -> Fallible<Value> {
    let Case {
        prompt_id,
        prefix,
        ordinary_client,
        identity,
        positional,
        gold_person,
        gold_ids,
        gold_onset,
    } = case;
    let reference = model
        .paired_copy_decision(&prefix, true)?
        .ok_or("missing paired reader")?;
    let mut paths = json!({});
    paths["reference"] = write(model, &prefix, reference.choice.source)?;
    if ordinary_client["prompt_ids"] != json!(prefix)
        || ordinary_client["artifact_sha256"] != model.artifact_sha256()
        || ordinary_client["generated_ids"] != paths["reference"]["ids"]
        || ordinary_client["copy_step_words"] != paths["reference"]["steps"]
        || ordinary_client["paired"] != serde_json::to_value(&reference)?
    {
        return Err("ordinary client and native reference differ".into());
    }
    let empty = PairedRoleDecision {
        requested: None,
        excluded: None,
        requested_score: 0,
        excluded_score: 0,
        requested_runner_up: 0,
        excluded_runner_up: 0,
    };
    for (name, spans) in [
        ("identity", identity),
        ("positional", positional),
        ("gold_person", vec![gold_person]),
    ] {
        let chosen = match spans.as_slice() {
            [[start, end]] => Some(RoleSpan {
                start: *start,
                end: *end,
            }),
            _ => None,
        };
        let (decision, _) = model.paired_copy_witness(&prefix, empty, chosen)?;
        paths[name] = write(model, &prefix, decision.choice.source)?;
        paths[name]["requested_spans"] = json!(spans);
    }
    if gold_ids.len() < 2
        || gold_ids.last() != Some(&1)
        || prefix.get(gold_onset..gold_onset + gold_ids.len() - 1)
            != Some(&gold_ids[..gold_ids.len() - 1])
    {
        return Err("gold literal is not at the designated original address".into());
    }
    paths["gold_onset"] = write(model, &prefix, Some(u32::try_from(gold_onset)?))?;
    if paths["gold_onset"]["ids"] != json!(gold_ids) {
        return Err("forced gold was not writable".into());
    }
    paths["disabled"] = write(model, &prefix, None)?;
    for path in paths.as_object_mut().unwrap().values_mut() {
        let ids: Vec<u32> = serde_json::from_value(path["ids"].clone())?;
        let first = (0..ids.len().max(gold_ids.len())).find(|&i| ids.get(i) != gold_ids.get(i));
        path["first_error"] = json!(
            first.map(|i| json!({"offset":i,"expected":gold_ids.get(i),"emitted":ids.get(i)}))
        );
    }
    Ok(
        json!({"prompt_id":prompt_id,"gold_ids":gold_ids,"gold_person":gold_person,"gold_onset":gold_onset,"arms":paths}),
    )
}

fn main() -> Fallible<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [path, cases] = args.as_slice() else {
        return Err("usage: clause-role-writer MODEL CASES.json".into());
    };
    let model = IntegerDiscreteModel::load(Path::new(path))?;
    let cases: Vec<Case> = serde_json::from_slice(&fs::read(cases)?)?;
    let rows = cases
        .into_iter()
        .map(|case| run(&model, case))
        .collect::<Fallible<Vec<_>>>()?;
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(())
}
