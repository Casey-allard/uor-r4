use std::path::PathBuf;

use uor_r4_cpu_train::{grade_scenarios, ScenarioGradeConfig};
use uor_r4_integer::fbs_snapshot::write_new_file;

const USAGE: &str = "usage: cpu-scenario-grade ARTIFACT CORPUS_ROOT TOKENIZER \
--split train|development|fresh [--max-cases N] [--reply-tokens N] [--out FILE]";

#[derive(Debug)]
struct Config {
    artifact: PathBuf,
    corpus_root: PathBuf,
    tokenizer: PathBuf,
    out: Option<PathBuf>,
    grade: ScenarioGradeConfig,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-scenario-grade: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args(std::env::args().skip(1))?;
    if config.out.as_ref().is_some_and(|path| path.exists()) {
        return Err(format!(
            "{}: output already exists",
            config.out.as_ref().unwrap().display()
        ));
    }
    let artifact = std::fs::read(&config.artifact)
        .map_err(|error| format!("{}: {error}", config.artifact.display()))?;
    let report = grade_scenarios(
        &artifact,
        &config.corpus_root,
        &config.tokenizer,
        config.grade,
    )
    .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    if let Some(path) = config.out {
        write_new_file(&path, &bytes).map_err(|error| format!("{}: {error}", path.display()))
    } else {
        println!("{}", String::from_utf8_lossy(&bytes));
        Ok(())
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
    let mut positional = Vec::new();
    let mut split = None;
    let mut max_cases = 256;
    let mut reply_tokens = 64;
    let mut out = None;
    let mut args = args;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--split" => split = Some(value(&mut args, "--split")?),
            "--max-cases" => max_cases = number(&mut args, "--max-cases")?,
            "--reply-tokens" => reply_tokens = number(&mut args, "--reply-tokens")?,
            "--out" => out = Some(PathBuf::from(value(&mut args, "--out")?)),
            "--help" | "-h" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            value if value.starts_with("--") => {
                return Err(format!("unknown argument {value}\n{USAGE}"));
            }
            value => positional.push(value.to_owned()),
        }
    }
    if positional.len() != 3 {
        return Err(USAGE.to_owned());
    }
    Ok(Config {
        artifact: PathBuf::from(&positional[0]),
        corpus_root: PathBuf::from(&positional[1]),
        tokenizer: PathBuf::from(&positional[2]),
        out,
        grade: ScenarioGradeConfig {
            split: split.ok_or_else(|| format!("--split is required\n{USAGE}"))?,
            max_cases,
            reply_tokens,
        },
    })
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} needs a value"))
}

fn number(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<usize, String> {
    value(args, flag)?
        .parse()
        .map_err(|_| format!("{flag} needs a positive integer"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> std::vec::IntoIter<String> {
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn parser_requires_three_inputs_and_a_split() {
        assert!(parse_args(args(&["model", "corpus", "tokenizer"])).is_err());
        let parsed = parse_args(args(&[
            "model",
            "corpus",
            "tokenizer",
            "--split",
            "development",
            "--max-cases",
            "16",
            "--reply-tokens",
            "8",
        ]))
        .unwrap();
        assert_eq!(parsed.grade.max_cases, 16);
        assert_eq!(parsed.grade.reply_tokens, 8);
    }

    #[test]
    fn parser_refuses_unknown_flags() {
        let error = parse_args(args(&[
            "model",
            "corpus",
            "tokenizer",
            "--split",
            "fresh",
            "--guess",
            "1",
        ]))
        .unwrap_err();
        assert!(error.contains("unknown argument --guess"));
    }
}
