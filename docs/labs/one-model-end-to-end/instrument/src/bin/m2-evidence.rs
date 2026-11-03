use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use one_model_end_to_end_panel::evidence::{evaluate, CorpusFiles, ReferenceRole};

fn main() {
    if let Err(error) = run() {
        eprintln!("m2-evidence: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args(std::env::args().skip(1))?;
    if config.out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            config.out.display()
        ));
    }
    let report = evaluate(
        &config.artifact,
        &config.tokenizer,
        &config.panel,
        config.train.as_ref(),
        config.train_role,
        config.max_targets,
        config.generate_new_tokens,
    )?;
    let mut text = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    text.push(b'\n');
    write_new(&config.out, &text)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": report.schema,
            "measured_targets": report.measured_targets,
            "overall": report.overall,
            "writer_parity": report.writer_parity,
            "cpu_engine_parity": report.cpu_engine_parity,
            "generated_replies": report.generated_replies,
            "count_table_control": report.count_table_control,
            "out": config.out,
            "sha256": report_sha256(&text),
        })
    );
    Ok(())
}

fn report_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Config {
    artifact: PathBuf,
    tokenizer: PathBuf,
    panel: CorpusFiles,
    train: Option<CorpusFiles>,
    train_role: Option<ReferenceRole>,
    max_targets: Option<usize>,
    generate_new_tokens: Option<usize>,
    out: PathBuf,
}

fn parse_args(values: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut args = BTreeMap::new();
    for argument in values {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| format!("argument {argument:?} is not KEY=VALUE"))?;
        if key.is_empty() || value.is_empty() {
            return Err(format!("argument {argument:?} has an empty key or value"));
        }
        if args.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("argument {key:?} is named twice"));
        }
    }
    const ALLOWED: [&str; 12] = [
        "artifact",
        "tokenizer",
        "panel-tokens",
        "panel-mask",
        "panel-manifest",
        "train-tokens",
        "train-mask",
        "train-manifest",
        "train-role",
        "max-targets",
        "generate-new-tokens",
        "out",
    ];
    if let Some(key) = args.keys().find(|key| !ALLOWED.contains(&key.as_str())) {
        return Err(format!("unknown argument {key:?}; {}", usage()));
    }
    let required = |key: &str| {
        args.get(key)
            .cloned()
            .ok_or_else(|| format!("{key}= is required"))
    };
    let corpus = |prefix: &str, is_required: bool| -> Result<Option<CorpusFiles>, String> {
        let keys = [
            format!("{prefix}-tokens"),
            format!("{prefix}-mask"),
            format!("{prefix}-manifest"),
        ];
        let present = keys.iter().filter(|key| args.contains_key(*key)).count();
        if present == 0 && !is_required {
            return Ok(None);
        }
        if present != keys.len() {
            return Err(format!(
                "{prefix}-tokens, {prefix}-mask and {prefix}-manifest must be supplied together"
            ));
        }
        Ok(Some(CorpusFiles {
            tokens: PathBuf::from(required(&keys[0])?),
            mask: PathBuf::from(required(&keys[1])?),
            manifest: PathBuf::from(required(&keys[2])?),
        }))
    };
    let panel = corpus("panel", true)?.expect("required panel");
    let train = corpus("train", false)?;
    let train_role = args
        .get("train-role")
        .map(|role| ReferenceRole::parse(role))
        .transpose()?;
    if train.is_some() != train_role.is_some() {
        return Err(
            "train-role= is required exactly when the three train paths are supplied".into(),
        );
    }
    let max_targets = args
        .get("max-targets")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| "max-targets= wants a positive whole number".to_owned())?;
    if max_targets == Some(0) {
        return Err("max-targets= must be positive".to_owned());
    }
    let generate_new_tokens = args
        .get("generate-new-tokens")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| "generate-new-tokens= wants a positive whole number".to_owned())?;
    if generate_new_tokens == Some(0) {
        return Err("generate-new-tokens= must be positive".to_owned());
    }
    Ok(Config {
        artifact: PathBuf::from(required("artifact")?),
        tokenizer: PathBuf::from(required("tokenizer")?),
        panel,
        train,
        train_role,
        max_targets,
        generate_new_tokens,
        out: PathBuf::from(required("out")?),
    })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn usage() -> String {
    "usage: m2-evidence artifact=FILE tokenizer=FILE panel-tokens=FILE panel-mask=FILE \
     panel-manifest=FILE [train-tokens=FILE train-mask=FILE train-manifest=FILE \
     train-role=canonical_train|generated_control] [max-targets=N] \
     [generate-new-tokens=N] out=NEW_JSON"
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PANEL: [&str; 4] = [
        "panel-tokens=p.u16",
        "panel-mask=p.u8",
        "panel-manifest=p.json",
        "artifact=a.lut",
    ];

    fn args(extra: &[&str]) -> Vec<String> {
        PANEL
            .iter()
            .chain(extra.iter())
            .chain(std::iter::once(&"tokenizer=t.json"))
            .map(|value| (*value).to_owned())
            .collect()
    }

    #[test]
    fn unknown_duplicate_and_incomplete_arguments_fail() {
        let mut unknown = args(&["out=r.json", "typo=1"]);
        unknown.retain(|argument| argument != "typo=1");
        unknown.push("typo=1".into());
        assert!(parse_args(unknown)
            .unwrap_err()
            .contains("unknown argument"));

        let mut duplicate = args(&["out=r.json"]);
        duplicate.push("artifact=b.lut".into());
        assert!(parse_args(duplicate).unwrap_err().contains("twice"));

        let incomplete = vec![
            "artifact=a.lut".to_owned(),
            "tokenizer=t.json".to_owned(),
            "panel-tokens=p.u16".to_owned(),
            "out=r.json".to_owned(),
        ];
        assert!(parse_args(incomplete)
            .unwrap_err()
            .contains("supplied together"));
    }

    #[test]
    fn train_paths_require_an_explicit_role() {
        let without = args(&[
            "train-tokens=t.u16",
            "train-mask=t.u8",
            "train-manifest=t.json",
            "out=r.json",
        ]);
        assert!(parse_args(without).unwrap_err().contains("train-role"));
        let with = args(&[
            "train-tokens=t.u16",
            "train-mask=t.u8",
            "train-manifest=t.json",
            "train-role=generated_control",
            "out=r.json",
        ]);
        assert_eq!(
            parse_args(with).unwrap().train_role,
            Some(ReferenceRole::GeneratedControl)
        );
    }

    #[test]
    fn zero_max_targets_and_existing_outputs_are_refused() {
        let zero = args(&["out=r.json", "max-targets=0"]);
        assert!(parse_args(zero).unwrap_err().contains("positive"));
        let zero_generation = args(&["out=r.json", "generate-new-tokens=0"]);
        assert!(parse_args(zero_generation)
            .unwrap_err()
            .contains("positive"));

        let path =
            std::env::temp_dir().join(format!("uor-m2-evidence-create-new-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        write_new(&path, b"first").unwrap();
        assert!(write_new(&path, b"second").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"first");
        let _ = std::fs::remove_file(path);
    }
}
