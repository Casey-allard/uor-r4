use std::path::PathBuf;

use uor_r4_cpu_train::{evaluate_copy_key_depth, validate_corpus_manifest, CpuMaskedCorpus};

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-copy-key-depth: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut values = std::env::args().skip(1);
    let tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let mut context = 192usize;
    let mut seed = 149u64;
    while let Some(flag) = values.next() {
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))?;
        match flag.as_str() {
            "--context" => {
                context = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--seed" => {
                seed = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if context < 6 {
        return Err("copy-key context must be at least 6".to_owned());
    }
    let token_bytes =
        std::fs::read(&tokens).map_err(|error| format!("{}: {error}", tokens.display()))?;
    let mask_bytes =
        std::fs::read(&mask).map_err(|error| format!("{}: {error}", mask.display()))?;
    let manifest_bytes =
        std::fs::read(&manifest).map_err(|error| format!("{}: {error}", manifest.display()))?;
    let identity = validate_corpus_manifest(&manifest_bytes, &token_bytes, &mask_bytes)
        .map_err(|error| error.to_string())?;
    if context > identity.max_tokens {
        return Err(format!(
            "copy-key context {context} exceeds corpus maximum {}",
            identity.max_tokens
        ));
    }
    let corpus = CpuMaskedCorpus::from_bytes(
        &token_bytes,
        &mask_bytes,
        identity.vocab,
        context,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let reading =
        evaluate_copy_key_depth(corpus.tokens(), corpus.mask(), context, identity.bos, seed)
            .map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "uor-r4.cpu-copy-key-depth/1",
            "substrate": "cpu-development-research",
            "qualification": "development-only",
            "claim": "data-only exact suffix copy-key selection; no model or language claim",
            "inputs": {
                "tokens": tokens,
                "tokens_sha256": identity.tokens_sha256,
                "mask": mask,
                "mask_sha256": identity.mask_sha256,
                "manifest": manifest,
                "split": identity.split,
                "documents": corpus.document_count(),
            },
            "settings": {
                "context": context,
                "bos": identity.bos,
                "seed": seed,
                "maximum_depth": uor_r4_cpu_train::COPY_KEY_MAX_DEPTH,
            },
            "reading": reading,
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}

fn usage() -> String {
    "usage: cpu-copy-key-depth TOKENS MASK MANIFEST [--context N] [--seed N]".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_names_every_required_input() {
        let usage = usage();
        for word in ["TOKENS", "MASK", "MANIFEST", "--context", "--seed"] {
            assert!(usage.contains(word), "{usage}");
        }
    }
}
