use std::path::PathBuf;

use uor_r4_cpu_train::{
    evaluate_copy_key_router, fit_copy_key_router, validate_corpus_manifest, CpuMaskedCorpus,
};

struct Split {
    label: String,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-copy-key-router: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let train = Split {
        label: "train".to_owned(),
        tokens: args.next().map(PathBuf::from).ok_or_else(usage)?,
        mask: args.next().map(PathBuf::from).ok_or_else(usage)?,
        manifest: args.next().map(PathBuf::from).ok_or_else(usage)?,
    };
    let test = Split {
        label: "test".to_owned(),
        tokens: args.next().map(PathBuf::from).ok_or_else(usage)?,
        mask: args.next().map(PathBuf::from).ok_or_else(usage)?,
        manifest: args.next().map(PathBuf::from).ok_or_else(usage)?,
    };
    let mut context = 192usize;
    let mut seed = 151u64;
    while let Some(flag) = args.next() {
        let value = args
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
    let (train_corpus, train_identity) = load(&train, context)?;
    let (test_corpus, test_identity) = load(&test, context)?;
    if train_identity.tokenizer_sha256 != test_identity.tokenizer_sha256 {
        return Err("copy-key router train and test tokenizer hashes differ".to_owned());
    }
    let router = fit_copy_key_router(
        train_corpus.tokens(),
        train_corpus.mask(),
        context,
        train_identity.bos,
        seed,
    )
    .map_err(|error| error.to_string())?;
    let train_reading = evaluate_copy_key_router(
        train_corpus.tokens(),
        train_corpus.mask(),
        context,
        train_identity.bos,
        &router,
        seed,
    )
    .map_err(|error| error.to_string())?;
    let test_reading = evaluate_copy_key_router(
        test_corpus.tokens(),
        test_corpus.mask(),
        context,
        test_identity.bos,
        &router,
        seed,
    )
    .map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "uor-r4.cpu-copy-key-router/1",
            "substrate": "cpu-development-research",
            "qualification": "development-only",
            "claim": "trained depth-and-age copy-key routing; no model or language claim",
            "inputs": {
                "train": input_json(&train, &train_identity),
                "test": input_json(&test, &test_identity),
            },
            "settings": {
                "context": context,
                "seed": seed,
                "depth": uor_r4_cpu_train::COPY_KEY_MAX_DEPTH,
                "age_bins": uor_r4_cpu_train::COPY_KEY_AGE_BINS,
            },
            "router": router,
            "training": train_reading,
            "test": test_reading,
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}

fn load(
    split: &Split,
    context: usize,
) -> Result<(CpuMaskedCorpus, uor_r4_cpu_train::CpuCorpusIdentity), String> {
    let token_bytes = std::fs::read(&split.tokens)
        .map_err(|error| format!("{}: {error}", split.tokens.display()))?;
    let mask =
        std::fs::read(&split.mask).map_err(|error| format!("{}: {error}", split.mask.display()))?;
    let manifest = std::fs::read(&split.manifest)
        .map_err(|error| format!("{}: {error}", split.manifest.display()))?;
    let identity = validate_corpus_manifest(&manifest, &token_bytes, &mask)
        .map_err(|error| error.to_string())?;
    if context > identity.max_tokens {
        return Err(format!(
            "{} context {context} exceeds corpus maximum {}",
            split.label, identity.max_tokens
        ));
    }
    let corpus =
        CpuMaskedCorpus::from_bytes(&token_bytes, &mask, identity.vocab, context, identity.bos)
            .map_err(|error| error.to_string())?;
    Ok((corpus, identity))
}

fn input_json(split: &Split, identity: &uor_r4_cpu_train::CpuCorpusIdentity) -> serde_json::Value {
    serde_json::json!({
        "tokens": split.tokens,
        "tokens_sha256": identity.tokens_sha256,
        "mask": split.mask,
        "mask_sha256": identity.mask_sha256,
        "manifest": split.manifest,
        "tokenizer_sha256": identity.tokenizer_sha256,
        "split": identity.split,
    })
}

fn usage() -> String {
    "usage: cpu-copy-key-router TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
     TEST_TOKENS TEST_MASK TEST_MANIFEST [--context N] [--seed N]"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_names_every_required_input() {
        let text = usage();
        for word in [
            "TRAIN_TOKENS",
            "TRAIN_MASK",
            "TRAIN_MANIFEST",
            "TEST_TOKENS",
            "TEST_MASK",
            "TEST_MANIFEST",
        ] {
            assert!(text.contains(word), "{text}");
        }
    }
}
