//! Reconstruct the legacy assertion-fence input from a separately replayed role decision.
use serde_json::{from_reader, json, to_value, to_writer, Value};
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::{PairedRoleDecision, RoleSpan},
};
use uor_r4_tokenizer::ByteBpeTokenizer;

fn rows(value: &Value) -> Result<&Vec<Value>, Box<dyn Error>> {
    value["rows"]
        .as_array()
        .ok_or_else(|| "missing rows".into())
}

fn span(words: &[u32], len: usize) -> Result<Option<RoleSpan>, Box<dyn Error>> {
    if words == [u32::MAX, u32::MAX] {
        return Ok(None);
    }
    let start = words[0] as usize;
    let end = words[1] as usize;
    if start >= end || end > len {
        return Err("role span outside original tape".into());
    }
    Ok(Some(RoleSpan { start, end }))
}

fn project(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (artifact, treatment_sha, blind, public, roles, out, policy) = match args {
        [a, b, c, d, e, f] => (a, b, c, d, e, f, "assertion_agreement"),
        [a, b, c, d, e, f, policy] if matches!(policy.as_str(), "user_text_only" | "clause_to_entity") => {
            (a, b, c, d, e, f, policy.as_str())
        }
        _ => return Err(
            "project-role LEGACY_PAIRED TREATMENT_SHA BLIND PUBLIC_SELECTION DEVICE_ROLES NEW_SELECTION [user_text_only|clause_to_entity]".into(),
        ),
    };
    let model = IntegerDiscreteModel::load(Path::new(artifact))?;
    let blind_bytes = fs::read(blind)?;
    let blind_sha = format!("{:x}", Sha256::digest(&blind_bytes));
    let prompts: Vec<Value> = serde_json::from_slice(&blind_bytes)?;
    let mut public: Value = from_reader(File::open(public)?)?;
    let device: Value = from_reader(File::open(roles)?)?;
    let selections = rows(&public)?;
    let decisions = rows(&device)?;
    if device["schema"] != "uor-r4.requested-role-device/1"
        || device["policy"] != policy
        || device["artifact"] != treatment_sha.as_str()
        || device["blind"] != blind_sha
        || public["schema"] != "uor-r4.ordered-role.selections/1"
        || public["arm"] != "public"
        || public["blind"] != blind_sha
        || public["artifact"] != model.artifact_sha256()
        || prompts.len() != selections.len()
        || prompts.len() != decisions.len()
    {
        return Err("inconsistent role projection inputs".into());
    }
    let mut projected = Vec::with_capacity(prompts.len());
    for (index, ((prompt, prior), decision)) in
        prompts.iter().zip(selections).zip(decisions).enumerate()
    {
        let ids: Vec<u32> = serde_json::from_value(prompt["prefix"].clone())?;
        let words: [u32; 8] = serde_json::from_value(decision["role_words"].clone())?;
        let mut prefix = Sha256::new();
        for id in &ids {
            prefix.update(id.to_le_bytes());
        }
        let prefix_sha = format!("{:x}", prefix.finalize());
        if prior["index"] != index
            || prior["region"] != json!([0, ids.len()])
            || prior["prefix_sha256"] != prefix_sha
            || decision["index"] != index
            || decision["prefix_sha256"] != prefix_sha
        {
            return Err(format!("misordered projection at row {index}").into());
        }
        let roles = PairedRoleDecision {
            requested: span(&words[..2], ids.len())?,
            excluded: span(&words[2..4], ids.len())?,
            requested_score: words[4] as i32,
            excluded_score: words[5] as i32,
            requested_runner_up: words[6] as i32,
            excluded_runner_up: words[7] as i32,
        };
        let (picked, assertions) = model.paired_copy_witness(&ids, roles, None)?;
        if picked.role_words != Some(words) {
            return Err(format!("legacy role projection changed words at row {index}").into());
        }
        let mut row = prior.clone();
        row["original_role_words"] = json!(words);
        row["paired"] = to_value(picked)?;
        row["assertions"] = to_value(assertions)?;
        projected.push(row);
    }
    public["rows"] = json!(projected);
    to_writer(File::create_new(out)?, &public)?;
    Ok(())
}

fn rendered_span(
    words: &[u32],
    ids: &[u32],
    tokenizer: &ByteBpeTokenizer,
) -> Result<Value, Box<dyn Error>> {
    let Some(selected) = span(words, ids.len())? else {
        return Ok(Value::Null);
    };
    let original = &ids[selected.start..selected.end];
    Ok(json!({
        "span": [selected.start, selected.end],
        "original_ids": original,
        "text": tokenizer.decode(original)
    }))
}

fn render(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (args, treatment_policy) = match args.split_last() {
        Some((policy, inputs)) if policy == "clause_to_entity" => (inputs, policy.as_str()),
        _ => (args, "user_text_only"),
    };
    let [tokenizer, blind, original, treatment, out] = args else {
        return Err(
            "project-role --render TOKENIZER BLIND ASSERTION_ROLES USER_TEXT_ROLES OUT".into(),
        );
    };
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(tokenizer)?)
        .ok_or("invalid tokenizer")?;
    let blind_bytes = fs::read(blind)?;
    let blind_sha = format!("{:x}", Sha256::digest(&blind_bytes));
    let prompts: Vec<Value> = serde_json::from_slice(&blind_bytes)?;
    let arms = [
        ("assertion_agreement", original),
        (treatment_policy, treatment),
    ];
    let mut rendered = serde_json::Map::new();
    for (policy, device_path) in arms {
        let device: Value = from_reader(File::open(device_path)?)?;
        let decisions = rows(&device)?;
        if device["schema"] != "uor-r4.requested-role-device/1"
            || device["policy"] != policy
            || device["blind"] != blind_sha
            || decisions.len() != prompts.len()
        {
            return Err("misbound role rendering inputs".into());
        }
        let mut output = Vec::with_capacity(prompts.len());
        for (index, (prompt, decision)) in prompts.iter().zip(decisions).enumerate() {
            let ids: Vec<u32> = serde_json::from_value(prompt["prefix"].clone())?;
            let words: [u32; 8] = serde_json::from_value(decision["role_words"].clone())?;
            let mut prefix = Sha256::new();
            for id in &ids {
                prefix.update(id.to_le_bytes());
            }
            if decision["index"] != index
                || decision["prefix_sha256"] != format!("{:x}", prefix.finalize())
            {
                return Err(format!("role rendering out of order at row {index}").into());
            }
            output.push(json!({
                "index": index,
                "requested": rendered_span(&words[..2], &ids, &tokenizer)?,
                "excluded": rendered_span(&words[2..4], &ids, &tokenizer)?
            }));
        }
        rendered.insert(policy.to_owned(), json!(output));
    }
    to_writer(File::create_new(out)?, &json!(rendered))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.split_first() {
        Some((flag, args)) if flag == "--render" => render(args),
        _ => project(&args),
    }
}
