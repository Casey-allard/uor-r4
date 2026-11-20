//! Stage blind prompt tapes and independently adjudicated role interventions.
#![forbid(unsafe_code)]

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::RoleSpan,
    relation_retrieval::{write_device_preamble, Arm, DEVICE_OUTPUT_ROW_BYTES},
};

mod transfer;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [usize; 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Blind {
    row: u32,
    ids: Vec<u32>,
}

#[derive(Deserialize)]
struct Person {
    row: u32,
    requested: Span,
}

#[derive(Deserialize)]
struct Relation {
    row: u32,
    relation: Span,
}

fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<(String, Vec<T>)> {
    let bytes = fs::read(path)?;
    let sha = format!("{:x}", Sha256::digest(&bytes));
    let rows = std::str::from_utf8(&bytes)?
        .lines()
        .map(serde_json::from_str)
        .collect::<serde_json::Result<_>>()?;
    Ok((sha, rows))
}

fn span([start, end]: Span) -> RoleSpan {
    RoleSpan { start, end }
}

fn stage(model_path: &Path, blind: &Path, out: &Path, supplied: &[PathBuf]) -> Result<()> {
    let model = IntegerDiscreteModel::load(model_path)?;
    let (blind_sha, rows) = read_jsonl::<Blind>(blind)?;
    if rows.is_empty() || rows.windows(2).any(|pair| pair[0].row >= pair[1].row) {
        return Err("blind rows must be nonempty, strictly increasing unique IDs".into());
    }
    let annotations = match supplied {
        [] => None,
        [person, relation] => {
            let (person_sha, people) = read_jsonl::<Person>(person)?;
            let (relation_sha, relations) = read_jsonl::<Relation>(relation)?;
            if people.len() != rows.len()
                || relations.len() != rows.len()
                || rows
                    .iter()
                    .zip(&people)
                    .zip(&relations)
                    .any(|((row, person), relation)| {
                        row.row != person.row || row.row != relation.row
                    })
            {
                return Err("supplied row IDs must exactly match blind rows in order".into());
            }
            Some((person_sha, relation_sha, people, relations))
        }
        _ => return Err("provide either no annotations or both PERSON and RELATION files".into()),
    };
    fs::create_dir(out)?;
    let mut input = BufWriter::new(File::create_new(out.join("input.bin"))?);
    let mut oracle = BufWriter::new(File::create_new(out.join("oracle.bin"))?);
    let mut native = BufWriter::new(File::create_new(out.join("native.jsonl"))?);
    write_device_preamble(&mut input)?;

    let mut device_rows = 0;
    let mut skipped = 0;
    for (index, row) in rows.iter().enumerate() {
        let ids = &row.ids;
        let roles = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
        let mut lanes = vec![("ordinary", roles.requested, None)];
        if let Some((_, _, people, relations)) = &annotations {
            let person = Some(span(people[index].requested));
            lanes.push(("person", person, None));
            lanes.push((
                "person_relation",
                person,
                Some(span(relations[index].relation)),
            ));
        }
        for (lane, requested, supplied_relation) in lanes {
            let basic = json!({
                "row": row.row,
                "lane": lane,
                "ordinary_role_words": roles.words(),
                "requested": requested,
                "excluded": roles.excluded,
                "supplied_relation": supplied_relation,
            });
            let mut record = basic.as_object().ok_or("invalid native record")?.clone();
            let mut status = "missing_role";
            if let Some(requested) = requested {
                let mut plan = model
                    .assertion_retrieval_plan(ids, requested, roles.excluded)
                    .map_err(|error| format!("row {} lane {lane}: {error}", row.row))?;
                record.insert("query_fence".into(), json!(plan.query_fence()));
                record.insert("candidates".into(), json!(plan.candidates()));
                status = "relation_ineligible";
                let relation_available =
                    supplied_relation.is_none_or(|relation| plan.query_words().contains(&relation));
                if relation_available {
                    if let Some(relation) = supplied_relation {
                        plan.restrict_to_relation(relation)?;
                    }
                    record.insert("device_row".into(), json!(device_rows));
                    record.insert("query_words".into(), json!(plan.query_words()));
                    record.insert("evidence".into(), json!(plan.evidence()));
                    record.insert("exact".into(), json!(plan.select(Arm::Exact)));
                    record.insert("geometry".into(), json!(plan.select(Arm::Geometry)));
                    plan.write_device_row(&mut input)?;
                    plan.write_device_oracle(&mut oracle)?;
                    device_rows += 1;
                    status = "scored";
                }
            }
            if status != "scored" {
                record.insert("device_row".into(), Value::Null);
                skipped += 1;
            }
            record.insert("status".into(), json!(status));
            writeln!(native, "{}", Value::Object(record))?;
        }
    }
    input.flush()?;
    oracle.flush()?;
    native.flush()?;
    let binding = json!({
        "schema": "uor-r4.d27-relation-stage/1",
        "artifact_sha256": model.artifact_sha256(),
        "tokenizer_sha256": model.tokenizer_sha256(),
        "blind_sha256": blind_sha,
        "person_sha256": annotations.as_ref().map(|item| &item.0),
        "relation_sha256": annotations.as_ref().map(|item| &item.1),
        "blind_rows": rows.len(),
        "device_rows": device_rows,
        "skipped_lanes": skipped,
    });
    let mut output = BufWriter::new(File::create_new(out.join("binding.json"))?);
    serde_json::to_writer_pretty(&mut output, &binding)?;
    writeln!(output)?;
    output.flush()?;
    eprintln!("staged {device_rows} device rows and recorded {skipped} skipped lanes");
    Ok(())
}

#[derive(Deserialize)]
struct NativeIndex {
    row: u32,
    lane: String,
    device_row: Option<usize>,
}

fn verify(staged: &Path, gpu_path: &Path) -> Result<()> {
    let (_, native) = read_jsonl::<NativeIndex>(&staged.join("native.jsonl"))?;
    let mut oracle = BufReader::new(File::open(staged.join("oracle.bin"))?);
    let mut gpu = BufReader::new(File::open(gpu_path)?);
    let mut host_row = [0; DEVICE_OUTPUT_ROW_BYTES];
    let mut device_row = [0; DEVICE_OUTPUT_ROW_BYTES];
    let mut count = 0;
    for record in native {
        let Some(index) = record.device_row else {
            continue;
        };
        if index != count {
            return Err(format!("nonconsecutive device replay row {index}").into());
        }
        let identify = || {
            format!(
                "row {} lane {} (device row {index})",
                record.row, record.lane
            )
        };
        oracle
            .read_exact(&mut host_row)
            .map_err(|e| format!("oracle read failed at {}: {e}", identify()))?;
        gpu.read_exact(&mut device_row)
            .map_err(|e| format!("CUDA read failed at {}: {e}", identify()))?;
        if let Some((word, (left, right))) = host_row
            .chunks_exact(4)
            .zip(device_row.chunks_exact(4))
            .enumerate()
            .find(|(_, (left, right))| left != right)
        {
            let left = u32::from_le_bytes(left.try_into()?);
            let right = u32::from_le_bytes(right.try_into()?);
            return Err(format!(
                "differential mismatch at {}, word {word}: Rust {left}, CUDA {right}",
                identify()
            )
            .into());
        }
        count += 1;
    }
    if count == 0 || oracle.read(&mut [0])? != 0 || gpu.read(&mut [0])? != 0 {
        return Err("CUDA differential stream is empty or has trailing data".into());
    }
    println!(
        "{}",
        json!({"verified_rows": count, "bytes": count * DEVICE_OUTPUT_ROW_BYTES})
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).map(PathBuf::from).collect();
    match args.as_slice() {
        [command, staged, gold, relation, gpu] if command == Path::new("audit-transfer") => transfer::audit(staged, gold, relation, gpu),
        [command, model, blind, output, supplied @ ..] if command == Path::new("stage") =>
            stage(model, blind, output, supplied),
        [command, staged, gpu] if command == Path::new("verify") => verify(staged, gpu),
        _ => Err("usage: d27-relation-stage stage MODEL BLIND NEW_DIR [PERSON RELATION]\n       d27-relation-stage verify STAGE_DIR CUDA_OUTPUT\n       d27-relation-stage audit-transfer STAGE_DIR HELD_GOLD HELD_RELATION CUDA_OUTPUT".into()),
    }
}
