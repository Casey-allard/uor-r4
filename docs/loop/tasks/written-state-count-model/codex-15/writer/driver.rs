mod pack;

mod grade;

mod replay;
mod selection;

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, fs, io, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;

#[derive(Deserialize)]
struct Prompt {
    text: String,
    prefix: Vec<u32>,
}

fn load(path: &Path) -> Result<Value, Error> {
    Ok(serde_json::from_reader(io::BufReader::new(
        fs::File::open(path)?,
    ))?)
}

fn save(path: &Path, value: &Value) -> Result<(), Error> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.sync_all()?;
    Ok(())
}

fn hash(path: &Path) -> Result<String, Error> {
    let mut data = fs::File::open(path)?;
    let mut hash = Sha256::new();
    io::copy(&mut data, &mut hash)?;
    Ok(format!("{:x}", hash.finalize()))
}

fn public(args: &[String]) -> Result<(), Error> {
    let [binary, model, tokenizer, blind, arm, out] = args else {
        return Err("public CHAT ARTIFACT TOKENIZER BLIND public|no-read|trace OUTDIR".into());
    };
    if !matches!(arm.as_str(), "public" | "no-read" | "trace") {
        return Err("unknown public intervention".into());
    }
    let prompts: Vec<Prompt> = serde_json::from_value(load(Path::new(blind))?)?;
    if prompts.is_empty() {
        return Err("empty public panel".into());
    }
    let target = Path::new(out);
    fs::create_dir(target)?;
    let mode = if arm == "no-read" {
        "no_read"
    } else {
        "enabled"
    };
    let artifact_sha = hash(Path::new(model))?;
    let tokenizer_sha = hash(Path::new(tokenizer))?;
    let mut receipts = Vec::with_capacity(prompts.len());
    for (index, prompt) in prompts.iter().enumerate() {
        let mut cmd = Command::new(binary);
        cmd.args([
            "--discrete",
            model,
            "--tokenizer",
            tokenizer,
            "--protocol",
            "2",
            "--max-tokens",
            "16",
            "--read-mode",
            mode,
            "--temperature",
            "0",
            "--say",
            &prompt.text,
        ]);
        if arm == "trace" {
            cmd.arg("--ordered-role-trace");
        }
        let answer = cmd.output()?;
        if !answer.status.success() {
            return Err(format!(
                "row {index} public client failed: {}",
                String::from_utf8_lossy(&answer.stderr)
            )
            .into());
        }
        let record: Value = serde_json::from_slice(&answer.stdout)?;
        let ids: Vec<u32> = serde_json::from_value(record["generated_ids"].clone())?;
        let steps = record["copy_step_words"]
            .as_array()
            .ok_or("missing writer trace")?;
        if record["schema"] != "uor-r4.uor-chat-discrete-text/1"
            || record["protocol"] != "uor-r4.literal-role-dialogue/2"
            || record["artifact_sha256"] != artifact_sha
            || record["tokenizer_sha256"] != tokenizer_sha
            || record["read_enabled"] != (mode == "enabled")
            || record["prompt_ids"] != json!(prompt.prefix)
            || steps.len() != ids.len()
            || ids.is_empty()
            || ids.len() > 16
            || ids[..ids.len() - 1].contains(&1)
            || ((ids.last() == Some(&1)) != (record["stop"] == "eos"))
            || (ids.last() != Some(&1) && ids.len() != 16)
        {
            return Err(format!(
                "row {index}: public framing, artifact, cursor or terminal mismatch"
            )
            .into());
        }
        let file = target.join(format!("row-{index:04}.json"));
        save(&file, &record)?;
        receipts.push(hash(&file)?);
    }
    save(
        &target.join("seal.json"),
        &json!({"schema":"uor-r4.ordered-role.public/1",
        "client":hash(Path::new(binary))?,"model":artifact_sha,"tokenizer":tokenizer_sha,
        "blind":hash(Path::new(blind))?,"arm":arm,"row_sha256":receipts}),
    )
}

fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.split_first() {
        Some((cmd, rest)) if cmd == "public" => public(rest),
        Some((cmd, rest)) if cmd == "select" => selection::select(rest),
        Some((cmd, rest)) if cmd == "replay" => replay::replay(rest),
        Some((cmd, rest)) if cmd == "chain" => replay::chain(rest),
        Some((cmd, rest)) if cmd == "grade" => grade::grade(rest),
        Some((cmd, rest)) if cmd == "pack" => pack::pack(rest),
        _ => Err("expected public, select, or replay".into()),
    }
}
