use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use uor_r4_cpu_train::{
    export_saved_checkpoint, grade_checkpoint_artifact, validate_corpus_manifest,
    CheckpointGradeConfig,
};
use uor_r4_integer::fbs_snapshot::write_new_file;
use uor_r4_integer::stack::{FbsExportSnaps, IntegerStackModel};

const USAGE: &str = "usage: cpu-checkpoint-grade \\
       (--artifact FILE | --checkpoint DIR --run-config FILE --snapshot-out FILE --artifact-out FILE) \\
       --tokens FILE --mask FILE --manifest FILE \\
       [--baseline FILE] [--max-evaluation N] [--max-replies N] [--reply-tokens N] \\
       [--margin-q12 N] [--raw-gates] [--raw-transition] [--out FILE]";

#[derive(Debug)]
struct Config {
    artifact: Option<PathBuf>,
    checkpoint: Option<PathBuf>,
    run_config: Option<PathBuf>,
    snapshot_out: Option<PathBuf>,
    artifact_out: Option<PathBuf>,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    baseline: Option<PathBuf>,
    out: Option<PathBuf>,
    grade: CheckpointGradeConfig,
    snaps: FbsExportSnaps,
}

#[derive(Debug)]
struct ResolvedArtifact {
    bytes: Vec<u8>,
    export: Option<CheckpointExport>,
}

#[derive(Debug, Serialize)]
struct CheckpointExport {
    snapshot_sha256: String,
    artifact_sha256: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-checkpoint-grade: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args(std::env::args().skip(1))?;
    validate_output_paths(&config)?;
    let tokens = std::fs::read(&config.tokens)
        .map_err(|error| format!("{}: {error}", config.tokens.display()))?;
    let mask = std::fs::read(&config.mask)
        .map_err(|error| format!("{}: {error}", config.mask.display()))?;
    let manifest = std::fs::read(&config.manifest)
        .map_err(|error| format!("{}: {error}", config.manifest.display()))?;
    validate_corpus_manifest(&manifest, &tokens, &mask)
        .map_err(|error| format!("grading corpus: {error}"))?;
    let baseline = config
        .baseline
        .as_ref()
        .map(|path| std::fs::read(path).map_err(|error| format!("{}: {error}", path.display())))
        .transpose()?;
    if let Some(bytes) = baseline.as_deref() {
        IntegerStackModel::parse(bytes).map_err(|error| format!("baseline artifact: {error}"))?;
    }
    let artifact = resolve_artifact(&config)?;
    let report = grade_checkpoint_artifact(
        &artifact.bytes,
        baseline.as_deref(),
        &tokens,
        &mask,
        &manifest,
        config.grade,
    )
    .map_err(|error| error.to_string())?;
    let mut report = serde_json::to_value(report).map_err(|error| error.to_string())?;
    if let Some(export) = artifact.export {
        if report["artifact_sha256"].as_str() != Some(export.artifact_sha256.as_str()) {
            return Err("checkpoint export artifact identity differs from grading".to_owned());
        }
        report["export"] = serde_json::to_value(export).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    if let Some(path) = config.out {
        write_new_file(&path, &bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    } else {
        std::io::stdout()
            .write_all(&bytes)
            .and_then(|()| std::io::stdout().write_all(b"\n"))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn resolve_artifact(config: &Config) -> Result<ResolvedArtifact, String> {
    match (&config.artifact, &config.checkpoint) {
        (Some(path), None) => {
            if config.run_config.is_some()
                || config.snapshot_out.is_some()
                || config.artifact_out.is_some()
                || !config.snaps.gates
                || !config.snaps.transition
            {
                return Err(
                    "--artifact cannot be combined with checkpoint export arguments".to_owned(),
                );
            }
            Ok(ResolvedArtifact {
                bytes: std::fs::read(path)
                    .map_err(|error| format!("{}: {error}", path.display()))?,
                export: None,
            })
        }
        (None, Some(checkpoint)) => {
            let run_config = config
                .run_config
                .as_ref()
                .ok_or_else(|| "--checkpoint needs --run-config".to_owned())?;
            let snapshot_out = config
                .snapshot_out
                .as_ref()
                .ok_or_else(|| "--checkpoint needs --snapshot-out".to_owned())?;
            let artifact_out = config
                .artifact_out
                .as_ref()
                .ok_or_else(|| "--checkpoint needs --artifact-out".to_owned())?;
            let (snapshot_sha256, artifact_sha256) = export_saved_checkpoint(
                checkpoint,
                run_config,
                snapshot_out,
                artifact_out,
                config.snaps,
            )
            .map_err(|error| format!("checkpoint export: {error}"))?;
            Ok(ResolvedArtifact {
                bytes: std::fs::read(artifact_out)
                    .map_err(|error| format!("{}: {error}", artifact_out.display()))?,
                export: Some(CheckpointExport {
                    snapshot_sha256,
                    artifact_sha256,
                }),
            })
        }
        _ => Err("choose exactly one of --artifact or --checkpoint".to_owned()),
    }
}

fn validate_output_paths(config: &Config) -> Result<(), String> {
    let mut outputs = Vec::new();
    if config.checkpoint.is_some() {
        outputs.extend(
            config
                .snapshot_out
                .iter()
                .chain(&config.artifact_out)
                .cloned(),
        );
    }
    outputs.extend(config.out.iter().cloned());
    for (index, path) in outputs.iter().enumerate() {
        if path.exists() {
            return Err(format!("{}: output already exists", path.display()));
        }
        if outputs[..index].contains(path) {
            return Err(format!("{}: output paths must be distinct", path.display()));
        }
    }
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Config, String> {
    let mut artifact = None;
    let mut checkpoint = None;
    let mut run_config = None;
    let mut snapshot_out = None;
    let mut artifact_out = None;
    let mut tokens = None;
    let mut mask = None;
    let mut manifest = None;
    let mut baseline = None;
    let mut out = None;
    let mut grade = CheckpointGradeConfig::default();
    let mut snaps = FbsExportSnaps::SNAPPED;
    let mut args = args;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--artifact" => artifact = Some(path(&mut args, "--artifact")?),
            "--checkpoint" => checkpoint = Some(path(&mut args, "--checkpoint")?),
            "--run-config" => run_config = Some(path(&mut args, "--run-config")?),
            "--snapshot-out" => snapshot_out = Some(path(&mut args, "--snapshot-out")?),
            "--artifact-out" => artifact_out = Some(path(&mut args, "--artifact-out")?),
            "--tokens" => tokens = Some(path(&mut args, "--tokens")?),
            "--mask" => mask = Some(path(&mut args, "--mask")?),
            "--manifest" => manifest = Some(path(&mut args, "--manifest")?),
            "--baseline" => baseline = Some(path(&mut args, "--baseline")?),
            "--out" => out = Some(path(&mut args, "--out")?),
            "--max-evaluation" => {
                grade.max_evaluation_rows = number(&mut args, "--max-evaluation")?
            }
            "--max-replies" => grade.max_reply_examples = number(&mut args, "--max-replies")?,
            "--reply-tokens" => grade.reply_tokens = number(&mut args, "--reply-tokens")?,
            "--margin-q12" => {
                grade.margin_q12 = i32::try_from(number::<i64>(&mut args, "--margin-q12")?)
                    .map_err(|_| "--margin-q12 is outside i32".to_owned())?
            }
            "--raw-gates" if !snaps.gates => {
                return Err("--raw-gates given twice".to_owned());
            }
            "--raw-gates" => snaps.gates = false,
            "--raw-transition" if !snaps.transition => {
                return Err("--raw-transition given twice".to_owned());
            }
            "--raw-transition" => snaps.transition = false,
            "--help" | "-h" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument {argument}\n{USAGE}")),
        }
    }
    let tokens = tokens.ok_or_else(|| format!("--tokens is required\n{USAGE}"))?;
    let mask = mask.ok_or_else(|| format!("--mask is required\n{USAGE}"))?;
    let manifest = manifest.ok_or_else(|| format!("--manifest is required\n{USAGE}"))?;
    Ok(Config {
        artifact,
        checkpoint,
        run_config,
        snapshot_out,
        artifact_out,
        tokens,
        mask,
        manifest,
        baseline,
        out,
        grade,
        snaps,
    })
}

fn path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} needs a value"))
}

fn number<T>(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    args.next()
        .ok_or_else(|| format!("{flag} needs a value"))?
        .parse()
        .map_err(|_| format!("{flag} needs an integer"))
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
    fn argument_parser_requires_corpus_identity_and_rejects_duplicate_raw_flags() {
        let error = parse_args(args(&["--artifact", "model.uorlut"])).unwrap_err();
        assert!(error.contains("--tokens is required"));

        let error = parse_args(args(&[
            "--artifact",
            "model.uorlut",
            "--tokens",
            "tokens.u16",
            "--mask",
            "mask.u8",
            "--manifest",
            "manifest.json",
            "--raw-gates",
            "--raw-gates",
        ]))
        .unwrap_err();
        assert!(error.contains("--raw-gates given twice"));
    }

    #[test]
    fn argument_parser_reads_bounded_grade_settings() {
        let config = parse_args(args(&[
            "--artifact",
            "model.uorlut",
            "--tokens",
            "tokens.u16",
            "--mask",
            "mask.u8",
            "--manifest",
            "manifest.json",
            "--max-evaluation",
            "128",
            "--max-replies",
            "8",
            "--reply-tokens",
            "4",
        ]))
        .unwrap();
        assert_eq!(config.grade.max_evaluation_rows, 128);
        assert_eq!(config.grade.max_reply_examples, 8);
        assert_eq!(config.grade.reply_tokens, 4);
    }

    #[test]
    fn direct_artifacts_reject_checkpoint_export_flags() {
        let config = parse_args(args(&[
            "--artifact",
            "model.uorlut",
            "--tokens",
            "tokens.u16",
            "--mask",
            "mask.u8",
            "--manifest",
            "manifest.json",
            "--raw-gates",
        ]))
        .unwrap();
        let error = resolve_artifact(&config).unwrap_err();
        assert!(error.contains("checkpoint export arguments"));
    }
}
