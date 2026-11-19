use super::{
    checked, data_parts, key, key_hex, sha, write, write_json, Fail, COPIES, TEXTS, TOKENIZER,
    TOTAL, VOCABULARY,
};
use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use uor_r4_integer::discrete_d11::{
    s2_route_key, IntegerDiscreteModel, DISCRETE_S2_EXPERT_SCHEMA, DISCRETE_S2_SCHEMA,
};

fn recorded(out: &Path, name: &str, process: &mut Command) -> Result<Output, Fail> {
    let command = format!("{process:?}");
    let result = process.output()?;
    write(&out.join(format!("{name}.stdout")), &result.stdout)?;
    write(&out.join(format!("{name}.stderr")), &result.stderr)?;
    write_json(
        &out.join(format!("{name}.command.json")),
        &json!({"command": command, "status": result.status.code()}),
    )?;
    Ok(result)
}

fn require_status(result: &Output, success: bool, name: &str) -> Result<(), Fail> {
    checked(
        result.status.success() == success,
        &format!(
            "{name} had unexpected exit: {}",
            String::from_utf8_lossy(&result.stderr)
        ),
    )
}

fn activate(
    out: &Path,
    activator: &Path,
    passive: &Path,
    digest: &str,
    active: &Path,
) -> Result<(), Fail> {
    let result = recorded(
        out,
        "activate",
        Command::new(activator).arg(passive).arg(digest).arg(active),
    )?;
    require_status(&result, true, "activate")
}

enum Prompt<'a> {
    Text(&'a Path, &'a str),
    Ids(&'a str),
}

fn chat(client: &Path, artifact: &Path, prompt: Prompt<'_>, steps: usize) -> Command {
    let mut command = Command::new(client);
    command.arg("--discrete").arg(artifact);
    match prompt {
        Prompt::Text(tokenizer, text) => {
            command
                .arg("--tokenizer")
                .arg(tokenizer)
                .arg("--say")
                .arg(text);
        }
        Prompt::Ids(ids) => {
            command.arg("--prompt-ids").arg(ids);
        }
    }
    command.arg("--max-tokens").arg(steps.to_string());
    command
}

fn chat_json(out: &Path, name: &str, command: &mut Command) -> Result<Value, Fail> {
    let result = recorded(out, name, command)?;
    require_status(&result, true, name)?;
    Ok(serde_json::from_slice(&result.stdout)?)
}

fn same_payload(passive_bytes: &[u8], active_bytes: &[u8]) -> Result<(), Fail> {
    let (mut passive_header, passive_payload) = data_parts(passive_bytes)?;
    let (active_header, active_payload) = data_parts(active_bytes)?;
    checked(
        passive_header["schema"] == DISCRETE_S2_SCHEMA
            && active_header["schema"] == DISCRETE_S2_EXPERT_SCHEMA,
        "unexpected model schemas",
    )?;
    passive_header["schema"] = json!(DISCRETE_S2_EXPERT_SCHEMA);
    checked(
        passive_bytes.len() == active_bytes.len()
            && passive_header == active_header
            && passive_payload == active_payload,
        "activation altered deployed bytes, binds or table payload",
    )
}

fn prompt_ids(pins: &Value) -> Result<Vec<u32>, Fail> {
    pins.as_array()
        .ok_or("missing prefix")?
        .iter()
        .map(|token| {
            let token = token.as_u64().ok_or("noninteger token")?;
            Ok(u32::try_from(token)?)
        })
        .collect()
}

fn client_trace(expected: &Value) -> Value {
    json!({"order": expected["order"], "key_hex": expected["key"],
        "reads": expected["reads"], "top8": expected["top8"]})
}

fn verify_routes(
    active: &IntegerDiscreteModel,
    pins: &[Value],
    oracle: &[Value],
) -> Result<(), Fail> {
    let table = active.active_s2().ok_or("active state missing")?;
    checked(pins.len() == oracle.len(), "fixture query count drift")?;
    for (pin, expected) in pins.iter().zip(oracle) {
        let prefix = prompt_ids(&pin["prefix"])?;
        let actual = table.predict_expert(&prefix)?;
        let trace = json!({"order": actual.order, "key_hex": actual.matched_key.map(key_hex),
            "reads": actual.reads().iter().map(|r| (r.token, r.count)).collect::<Vec<_>>(),
            "top8": actual.top8.iter().map(|r| (r.token, r.mass)).collect::<Vec<_>>()});
        let mass: Vec<u64> = (0..u32::from(VOCABULARY))
            .map(|token| actual.mass(token))
            .collect();
        let reference: Vec<u64> = serde_json::from_value(expected["mass"].clone())?;
        checked(
            trace == client_trace(expected)
                && mass == reference
                && mass.iter().sum::<u64>() == TOTAL
                && mass[0] == 0
                && actual.mass(u32::from(VOCABULARY)) == 0
                && mass[1..]
                    .iter()
                    .all(|&value| value >= (TOTAL / 32) / u64::from(VOCABULARY - 1)),
            &format!(
                "route, counts, rank, floor, or total differ for {}",
                pin["case"]
            ),
        )?;
    }
    let prefix: Vec<u16> = prompt_ids(&pins[0]["prefix"])?
        .into_iter()
        .map(u16::try_from)
        .collect::<Result<_, _>>()?;
    for order in 0..=10 {
        let context = &prefix[prefix.len() - order..];
        checked(
            key(context)? == s2_route_key(context)?,
            "PrimeRouteKey parity drift",
        )?;
    }
    checked(
        table.predict_expert(&[4096]).is_err(),
        "out-of-vocabulary input admitted",
    )
}

fn fixture(
    dir: &Path,
    client: &Path,
    activator: &Path,
    base: &Path,
    tokenizer: &Path,
) -> Result<(), Fail> {
    let paths = dir.join("validation");
    fs::create_dir(&paths)?;
    let passive = dir.join("fixture-passive.d11");
    let active = paths.join("fixture-active.d11");
    let plan: Value = serde_json::from_slice(&fs::read(dir.join("plan.json"))?)?;
    let oracle: Vec<Value> = serde_json::from_slice(&fs::read(dir.join("oracle.json"))?)?;
    let pins = plan["queries"].as_array().ok_or("missing pinned queries")?;
    let passive_bytes = fs::read(&passive)?;
    let passive_sha = plan["passive_sha256"]
        .as_str()
        .ok_or("missing passive hash")?;
    checked(sha(&passive_bytes) == passive_sha, "stale fixture input")?;
    checked(
        sha(&fs::read(tokenizer)?) == TOKENIZER,
        "stale fixture tokenizer",
    )?;
    let refused = paths.join("stale-output.d11");
    let stale = recorded(
        &paths,
        "refused-stale-passive-hash",
        Command::new(activator)
            .arg(&passive)
            .arg("0".repeat(64))
            .arg(&refused),
    )?;
    require_status(&stale, false, "stale expected digest")?;
    checked(
        !refused.exists(),
        "stale activation published a destination",
    )?;
    activate(&paths, activator, &passive, passive_sha, &active)?;
    let mut active_bytes = fs::read(&active)?;
    same_payload(&passive_bytes, &active_bytes)?;
    let active_sha = sha(&active_bytes);
    let mut stale_tokenizer = fs::read(tokenizer)?;
    stale_tokenizer.push(b' ');
    let stale = paths.join("stale-tokenizer.json");
    write(&stale, &stale_tokenizer)?;
    let wrong_tokenizer = recorded(
        &paths,
        "refused-stale-tokenizer",
        &mut chat(client, &active, Prompt::Text(&stale, TEXTS[0]), 2),
    )?;
    require_status(&wrong_tokenizer, false, "stale tokenizer")?;
    checked(
        String::from_utf8_lossy(&wrong_tokenizer.stderr).contains("does not bind this tokenizer"),
        "unexpected stale tokenizer refusal",
    )?;

    let rank = active_bytes.len() - 2;
    active_bytes[rank] = 32;
    let corrupt = paths.join("corrupt-rank-active.d11");
    write(&corrupt, &active_bytes)?;
    drop(active_bytes);
    drop(passive_bytes);
    let corrupt_rank = recorded(
        &paths,
        "refused-corrupt-rank",
        &mut chat(client, &corrupt, Prompt::Text(tokenizer, TEXTS[0]), 2),
    )?;
    require_status(&corrupt_rank, false, "invalid count rank")?;
    checked(
        String::from_utf8_lossy(&corrupt_rank.stderr).contains("invalid S2 token, rank or padding"),
        "unexpected corrupt state refusal",
    )?;

    {
        let model = IntegerDiscreteModel::load(&passive)?;
        checked(
            model.frozen_s2().is_some() && model.active_s2().is_none(),
            "passive state activated",
        )?;
    }
    {
        let model = IntegerDiscreteModel::load(&active)?;
        verify_routes(&model, pins, &oracle)?;
    }

    let mut comparison = Vec::new();
    for (index, text) in TEXTS.iter().enumerate() {
        let pass = chat_json(
            &paths,
            &format!("passive-text-{index}"),
            &mut chat(client, &passive, Prompt::Text(tokenizer, text), 2),
        )?;
        let step = 2 * index;
        let raw = prompt_ids(&pins[step]["prefix"])?
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let original = chat_json(
            &paths,
            &format!("original-raw-{index}"),
            &mut chat(client, base, Prompt::Ids(&raw), 2),
        )?;
        checked(
            pass["model_schema"] == DISCRETE_S2_SCHEMA
                && pass["s2_steps"].is_null()
                && pass["prompt_ids"] == pins[step]["prefix"]
                && pass["generated_ids"] == original["generated_ids"],
            "passive generation differs from public seed1 with equivalent prompt IDs",
        )?;
        let first = chat_json(
            &paths,
            &format!("active-text-{index}-a"),
            &mut chat(client, &active, Prompt::Text(tokenizer, text), 2),
        )?;
        let second = chat_json(
            &paths,
            &format!("active-text-{index}-b"),
            &mut chat(client, &active, Prompt::Text(tokenizer, text), 2),
        )?;
        let raw_active = chat_json(
            &paths,
            &format!("active-raw-{index}"),
            &mut chat(client, &active, Prompt::Ids(&raw), 2),
        )?;
        checked(
            first == second
                && first["model_schema"] == DISCRETE_S2_EXPERT_SCHEMA
                && first["prompt_ids"] == pins[step]["prefix"]
                && first["generated_ids"] == json!([COPIES[index], 1])
                && first["generated_ids"] != pass["generated_ids"]
                && first["generated_ids"] == raw_active["generated_ids"]
                && first["s2_steps"] == raw_active["s2_steps"]
                && first["stop"] == "eos"
                && first["s2_steps"]
                    == json!([client_trace(&oracle[step]), client_trace(&oracle[step + 1])])
                && first["generated_mass_q32"]
                    == json!([oracle[step]["top8"][0][1], oracle[step + 1]["top8"][0][1]]),
            "ordinary client order-ten copy/EOS or reproducibility check failed",
        )?;
        comparison.push(json!({"text": text, "passive": pass, "active": first}));
    }
    // Compare the ordinary client with every independent query, including boundary and miss controls.
    for (index, (pin, expected)) in pins.iter().zip(&oracle).enumerate() {
        let raw = prompt_ids(&pin["prefix"])?
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let actual = chat_json(
            &paths,
            &format!("active-query-{index}"),
            &mut chat(client, &active, Prompt::Ids(&raw), 1),
        )?;
        checked(
            actual["s2_steps"] == json!([client_trace(expected)])
                && actual["generated_ids"] == json!([expected["top1"]])
                && actual["generated_mass_q32"] == json!([expected["top8"][0][1]]),
            &format!("ordinary client finite query mismatch for {}", pin["case"]),
        )?;
    }
    write_json(
        &paths.join("result.json"),
        &json!({
            "schema": "uor-r4.s2-ordinary-client-finite/1",
            "unit": "planted integer rows on public M1 seed1",
            "passive_sha256": passive_sha, "active_sha256": active_sha,
            "equal_deployed_bytes": fs::metadata(&active)?.len(),
            "uor_chat_sha256": sha(&fs::read(client)?), "activator_sha256": sha(&fs::read(activator)?),
            "fixture_plan": plan, "compared_q32_masses": pins.len() * usize::from(VOCABULARY),
            "prime_route_key_orders": [0, 10],
            "client_comparison": comparison,
            "refusals": ["stale passive SHA-256", "stale original tokenizer", "invalid serialized count rank"],
        }),
    )
}

pub fn run(args: &[String]) -> Result<(), Fail> {
    match args {
        [dir, client, activator, base, tokenizer] => fixture(
            Path::new(dir),
            Path::new(client),
            Path::new(activator),
            Path::new(base),
            Path::new(tokenizer),
        ),
        _ => Err("active-s2-finite validate DIRECTORY CLIENT ACTIVATOR BASE TOKENIZER".into()),
    }
}
