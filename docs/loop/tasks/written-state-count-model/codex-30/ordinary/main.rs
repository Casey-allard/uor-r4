use serde::Deserialize;
use serde_json::{json, Value};
use std::{env, error::Error, fs, io::Write, path::Path};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Targets {
    requested: Option<[usize; 2]>,
    excluded: Option<[usize; 2]>,
    unrequested: Option<[usize; 2]>,
}
#[derive(Deserialize)]
struct Row {
    row: u32,
    family: String,
    stratum: String,
    role_factors: Value,
    original_ids: Vec<u32>,
    expected_tape: Vec<u32>,
    role_targets: Targets,
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [model, key, clients, out] = &args[..] else {
        return Err("expected CARRIER ROLE_KEY CLIENT_DIR NEW_OUT".into());
    };
    let carrier = IntegerDiscreteModel::from_bytes(&fs::read(model)?)?;
    let replay = carrier.frozen_role_replay()?;
    let framing = replay.requested_framing.ok_or("missing framing")?;
    let policy = replay.requested_context.ok_or("missing role policy")?;
    let rows = serde_json::from_slice::<Vec<Row>>(&fs::read(key)?)?;
    let mut readings = Vec::new();
    for row in rows {
        let ids = &row.original_ids;
        let choice = carrier.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
        let trace = replay.roles.trace_requested_occurrence(
            ids,
            framing.content(ids)?,
            0..ids.len(),
            replay.delimiters,
            replay.assertion_classes,
            policy,
        )?;
        let boundary = trace.query_boundary;
        let candidates = trace.candidates;
        let client: Value = serde_json::from_slice(&fs::read(
            Path::new(clients).join(format!("{}.json", row.row)),
        )?)?;
        if client["prompt_ids"] != json!(ids)
            || client["paired"]["role_words"] != json!(choice.words())
            || client["artifact_sha256"] != carrier.artifact_sha256()
            || client["read_enabled"] != true
        {
            return Err("ordinary role receipt mismatch".into());
        }
        let names = [
            ("requested", row.role_targets.requested),
            ("excluded", row.role_targets.excluded),
            ("incidental", row.role_targets.unrequested),
        ];
        let mut inspected = serde_json::Map::new();
        for (name, span) in names {
            let hit = span.and_then(|span| candidates.iter().find(|c| c.span == span));
            let observation = if span.is_some() {
                json!({"span":span,"trace":hit})
            } else {
                Value::Null
            };
            inspected.insert(name.into(), observation);
        }
        let req = choice.requested.map(|r| [r.start, r.end]);
        let exc = choice.excluded.map(|r| [r.start, r.end]);
        let generated: Vec<u32> = serde_json::from_value(client["generated_ids"].clone())?;
        let first = (0..row.expected_tape.len().max(generated.len()))
            .find(|&i| row.expected_tape.get(i) != generated.get(i))
            .map(|i| json!([i, row.expected_tape.get(i), generated.get(i)]));
        readings.push(json!({"row":row.row,"family":row.family,"stratum":row.stratum,"factors":row.role_factors,"candidate_boundary":boundary,"content":[framing.content(ids)?.start,framing.content(ids)?.end],"targets":inspected,"role_words":choice.words(),"request_ok":req==row.role_targets.requested,"exclusion_ok":exc==row.role_targets.excluded,"joint_ok":req==row.role_targets.requested && exc==row.role_targets.excluded,"incidental_chosen":row.role_targets.unrequested.is_some_and(|s|req==Some(s)||exc==Some(s)),"all_candidates":candidates,"source":client["selected"]["source"],"first":first,"exact":row.expected_tape==generated,"stop":client["stop"]}));
    }
    let mut f = fs::File::create_new(out)?;
    serde_json::to_writer_pretty(&mut f, &readings)?;
    writeln!(f)?;
    Ok(())
}
