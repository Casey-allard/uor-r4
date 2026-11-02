use std::path::PathBuf;

use uor_r4_cpu_train::{
    audit_served_forward, read_uort_token_slice, ServedForwardAuditConfig, ServedForwardAuditReport,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-served-forward-audit: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let (config, artifact_path, output_path) = parse_args()?;
    let artifact = std::fs::read(&artifact_path)
        .map_err(|error| format!("{}: {error}", artifact_path.display()))?;
    let report = audit_served_forward(&artifact, &config).map_err(|error| error.to_string())?;
    let json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    if let Some(output_path) = output_path {
        std::fs::write(&output_path, &json)
            .map_err(|error| format!("{}: {error}", output_path.display()))?;
    } else {
        println!(
            "{}",
            String::from_utf8(json).map_err(|error| error.to_string())?
        );
    }
    if report.parity.instrument_fault {
        return Err(parity_failure(&report));
    }
    Ok(())
}

fn parity_failure(report: &ServedForwardAuditReport) -> String {
    format!(
        "instrument fault: CPU/serving mismatches query={} token={} kept-cell={}",
        report.parity.query_coordinate_mismatches,
        report.parity.token_mismatches,
        report.parity.kept_cell_mismatches
    )
}

fn parse_args() -> Result<(ServedForwardAuditConfig, PathBuf, Option<PathBuf>), String> {
    let mut artifact = None;
    let mut tokens = None;
    let mut uort = None;
    let mut token_offset = 0usize;
    let mut token_count = None;
    let mut expected_artifact_sha256 = None;
    let mut reply_start = None;
    let mut measure_timing = true;
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--artifact" => artifact = Some(path(&mut args, "--artifact")?),
            "--tokens" => {
                let value = value(&mut args, "--tokens")?;
                tokens = Some(parse_tokens(&value)?);
            }
            "--tokens-file" => {
                if tokens.is_some() {
                    return Err("choose only one textual token input".to_owned());
                }
                let path = path(&mut args, "--tokens-file")?;
                let text = std::fs::read_to_string(&path)
                    .map_err(|error| format!("{}: {error}", path.display()))?;
                tokens = Some(parse_tokens(&text)?);
            }
            "--uort" => {
                if uort.is_some() {
                    return Err("--uort given twice".to_owned());
                }
                uort = Some(path(&mut args, "--uort")?);
            }
            "--token-offset" => {
                let value = value(&mut args, "--token-offset")?;
                token_offset = value
                    .parse::<usize>()
                    .map_err(|_| format!("--token-offset is not an integer: {value}"))?;
            }
            "--token-count" => {
                let value = value(&mut args, "--token-count")?;
                token_count = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("--token-count is not an integer: {value}"))?,
                );
            }
            "--expected-artifact-sha256" => {
                let value = value(&mut args, "--expected-artifact-sha256")?;
                expected_artifact_sha256 = Some(value.to_ascii_lowercase());
            }
            "--reply-start" => {
                let value = value(&mut args, "--reply-start")?;
                reply_start = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("--reply-start is not an integer: {value}"))?,
                );
            }
            "--no-timing" => measure_timing = false,
            "--output" => output = Some(path(&mut args, "--output")?),
            "--help" | "-h" => {
                return Err(usage());
            }
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        }
    }
    let artifact = artifact.ok_or_else(usage)?;
    let (tokens, token_source) = match (tokens, uort) {
        (Some(tokens), None) if token_count.is_none() && token_offset == 0 => (tokens, None),
        (None, Some(path)) => {
            let count = token_count.ok_or_else(|| "--uort requires --token-count".to_owned())?;
            let bytes =
                std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            let slice = read_uort_token_slice(&bytes, token_offset, count)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            (slice.tokens, Some(slice.source))
        }
        (Some(_), Some(_)) => {
            return Err("--tokens/--tokens-file and --uort are mutually exclusive".to_owned())
        }
        (Some(_), None) => return Err("--token-offset/--token-count require --uort".to_owned()),
        (None, None) => return Err(format!("a token sequence is required\n{}", usage())),
    };
    Ok((
        ServedForwardAuditConfig {
            tokens,
            token_source,
            expected_artifact_sha256,
            reply_start,
            measure_timing,
        },
        artifact,
        output,
    ))
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))
}

fn path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(value(args, flag)?))
}

fn parse_tokens(text: &str) -> Result<Vec<u32>, String> {
    let tokens = text
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<u32>()
                .map_err(|_| format!("token sequence contains a non-integer: {part}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if tokens.is_empty() {
        return Err("token sequence is empty".to_owned());
    }
    Ok(tokens)
}

fn usage() -> String {
    "usage: cpu-served-forward-audit --artifact PATH \
     (--tokens IDS | --tokens-file PATH | --uort PATH --token-count N [--token-offset N]) \
     [--expected-artifact-sha256 HEX] [--reply-start ROW] [--no-timing] [--output PATH]"
        .to_owned()
}
