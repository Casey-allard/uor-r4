//! Generate, check or summarize the aligned scenario corpus.
//!
//! ```text
//! scenario-corpus generate out=DIR tokenizer=TOKENIZER.json [seed=N]
//!   [train=N] [development=N] [fresh=N] [context=N]
//! scenario-corpus check out=DIR tokenizer=TOKENIZER.json
//! scenario-corpus census out=DIR
//! ```
//!
//! Generated tokens and masks use `uor-r4-chat-corpus/v1`; the scenario labels
//! live beside them in `scenarios.jsonl`. Large corpora are build outputs and
//! should not be committed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::json;
use uor_r4_training::scenario_corpus::{
    check_corpus, manifest_path, write_corpus, ScenarioCorpusConfig, ScenarioCorpusReceipt,
};

type Result<T> = std::result::Result<T, String>;

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("scenario-corpus: {error}");
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = "usage: scenario-corpus generate out=DIR tokenizer=FILE [seed=N] [train=N] [development=N] [fresh=N] [context=N]\n       scenario-corpus check out=DIR tokenizer=FILE\n       scenario-corpus census out=DIR";

fn run(arguments: Vec<String>) -> Result<()> {
    let (command, arguments) = arguments
        .split_first()
        .ok_or_else(|| "missing command".to_owned())?;
    let options = Options::parse(arguments)?;
    let out = options.out.ok_or_else(|| "missing out=DIR".to_owned())?;
    match command.as_str() {
        "generate" => {
            let tokenizer = options
                .tokenizer
                .ok_or_else(|| "missing tokenizer=FILE".to_owned())?;
            let config = options.config;
            let receipt = write_corpus(&config, &tokenizer, &out)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "out": out,
                    "schema": receipt.schema,
                    "tokenizer": receipt.tokenizer,
                    "splits": receipt.splits,
                }))
                .map_err(|e| e.to_string())?
            );
        }
        "check" => {
            let tokenizer = options
                .tokenizer
                .ok_or_else(|| "missing tokenizer=FILE".to_owned())?;
            let manifest = read_manifest(&out)?;
            check_corpus(&manifest.config, &tokenizer, &out)?;
            println!("PASS scenario corpus {}", out.display());
        }
        "census" => {
            let manifest = read_manifest(&out)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schema": manifest.schema,
                    "generator": manifest.generator,
                    "config": manifest.config,
                    "holdouts": manifest.holdouts,
                    "splits": manifest.splits,
                }))
                .map_err(|e| e.to_string())?
            );
        }
        other => return Err(format!("unknown command {other}")),
    }
    Ok(())
}

#[derive(Default)]
struct Options {
    out: Option<PathBuf>,
    tokenizer: Option<PathBuf>,
    config: ScenarioCorpusConfig,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, found {argument:?}"))?;
            if !matches!(
                key,
                "out" | "tokenizer" | "seed" | "train" | "development" | "fresh" | "context"
            ) {
                return Err(format!("unknown argument {key}="));
            }
            if pairs.insert(key, value).is_some() {
                return Err(format!("duplicate argument {key}="));
            }
        }
        let path = |key: &str| pairs.get(key).map(PathBuf::from);
        let number = |key: &str, default: u64| -> Result<u64> {
            pairs.get(key).map_or(Ok(default), |value| {
                value
                    .parse()
                    .map_err(|_| format!("{key}= must be a nonnegative integer"))
            })
        };
        let config = ScenarioCorpusConfig {
            seed: number("seed", 1)?,
            train: usize::try_from(number("train", 20_000)?)
                .map_err(|_| "train= does not fit this machine".to_owned())?,
            development: usize::try_from(number("development", 1_024)?)
                .map_err(|_| "development= does not fit this machine".to_owned())?,
            fresh: usize::try_from(number("fresh", 1_024)?)
                .map_err(|_| "fresh= does not fit this machine".to_owned())?,
            context: usize::try_from(number("context", 256)?)
                .map_err(|_| "context= does not fit this machine".to_owned())?,
        };
        Ok(Self {
            out: path("out"),
            tokenizer: path("tokenizer"),
            config,
        })
    }
}

fn read_manifest(root: &Path) -> Result<ScenarioCorpusReceipt> {
    let path = manifest_path(root);
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}
