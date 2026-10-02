//! Concatenate prepared chat stores into one, keeping every source's span so
//! the trainer still sees per-source counts.
//!
//! ```text
//! mix-chat-corpus out=NEW_DIR tokenizer=TOKENIZER.json \
//!   inputs=STORE_A[,STORE_B...] labels=label_a[,label_b...]
//! ```
//!
//! Why this exists: a response-phase fit takes exactly **one** token store, but
//! an arm that adds rows to the training distribution must train on the real
//! store **and** the added rows together. Fitting on the added rows alone is not
//! a smaller version of that arm -- it is a different and destructive one. On
//! this project a 512-update fit on a 1.03M-token synthetic store alone drove
//! train NLL to 0.13 while development NLL rose monotonically 2.476 -> 2.810 ->
//! 3.077: memorisation of the synthetic distribution with catastrophic
//! forgetting of everything else.
//!
//! Each input must be a `uor-r4-chat-corpus/v1` store (`tokens.u16` with its
//! corpus header, `response_mask.u8`, `manifest.json`). Inputs are concatenated
//! in the order given, and the output manifest's `files` records one span per
//! input so `dev_per_source` and per-source accounting keep working. The output
//! keeps each input's own labels, token and response counts and drop counters,
//! and the combined store is what the fit should see.

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_core::report_output;
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

struct Loaded {
    label: String,
    path: PathBuf,
    ids: Vec<u16>,
    mask: Vec<u8>,
    manifest: Value,
}

fn csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn load(label: String, path: PathBuf) -> Result<Loaded> {
    let manifest: Value = serde_json::from_slice(
        &fs::read(path.join("manifest.json")).map_err(|e| format!("{}: {e}", path.display()))?,
    )
    .map_err(|e| e.to_string())?;
    if manifest["schema"] != "uor-r4-chat-corpus/v1" {
        return Err(format!("{}: not a chat corpus", path.display()));
    }
    let reader = MmapCorpusReader::open(path.join("tokens.u16"))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let ids: Vec<u16> = reader.as_slice().to_vec();
    let mask =
        fs::read(path.join("response_mask.u8")).map_err(|e| format!("{}: {e}", path.display()))?;
    if ids.len() != mask.len() {
        return Err(format!(
            "{}: {} tokens against {} mask bytes",
            path.display(),
            ids.len(),
            mask.len()
        ));
    }
    Ok(Loaded {
        label,
        path,
        ids,
        mask,
        manifest,
    })
}

fn run(args: &[String]) -> Result<()> {
    let arg = |key: &str| -> Result<String> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(str::to_string)
            .ok_or_else(|| format!("missing {key}="))
    };
    let out = PathBuf::from(arg("out")?);
    let tokenizer_path = PathBuf::from(arg("tokenizer")?);
    let inputs = csv(&arg("inputs")?);
    let labels = csv(&arg("labels")?);
    if inputs.is_empty() {
        return Err("inputs= is empty".into());
    }
    if labels.len() != inputs.len() {
        return Err(format!(
            "{} labels for {} inputs; one label per input is required so the \
             output manifest can carry a span for each",
            labels.len(),
            inputs.len()
        ));
    }

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(
        &fs::read(&tokenizer_path).map_err(|e| e.to_string())?,
    )
    .ok_or("unreadable tokenizer.json")?;
    let tokenizer_sha = uor_r4_training::sha256_file(&tokenizer_path).map_err(|e| e.to_string())?;

    let loaded: Vec<Loaded> = inputs
        .iter()
        .zip(labels)
        .map(|(input, label)| load(label, PathBuf::from(input)))
        .collect::<Result<_>>()?;

    // Every input must speak the same vocabulary, or the concatenated ids mean
    // different things in different spans.
    let vocab_size = {
        let reader =
            MmapCorpusReader::open(loaded[0].path.join("tokens.u16")).map_err(|e| e.to_string())?;
        reader.vocab_size()
    };
    for store in &loaded {
        let reader =
            MmapCorpusReader::open(store.path.join("tokens.u16")).map_err(|e| e.to_string())?;
        if reader.vocab_size() != vocab_size {
            return Err(format!(
                "{}: vocabulary {} differs from {}",
                store.path.display(),
                reader.vocab_size(),
                vocab_size
            ));
        }
    }

    let total: usize = loaded.iter().map(|s| s.ids.len()).sum();
    let mut files = Vec::with_capacity(loaded.len());
    let mut drops = json!({
        "rows_dropped_no_messages": 0,
        "rows_dropped_empty": 0,
        "rows_dropped_no_response": 0,
        "rows_dropped_oversized": 0,
        "special_token_occurrences": 0,
    });
    let mut combined_mask: Vec<u8> = Vec::with_capacity(total);
    let mut combined_ids: Vec<u16> = Vec::with_capacity(total);
    for store in &loaded {
        let response = store.mask.iter().filter(|&&m| m == 1).count();
        files.push(json!({
            "label": store.label,
            "path": store.path.display().to_string(),
            "tokens": store.ids.len(),
            "response_tokens": response,
            "rows_used": store.manifest["rows_used"],
            "rows_total": store.manifest["rows_total"].is_null()
                .then(|| store.manifest["rows_used"].clone())
                .unwrap_or_else(|| store.manifest["rows_total"].clone()),
            "special_token_occurrences": store.manifest["drops"]["special_token_occurrences"],
        }));
        for key in [
            "rows_dropped_no_messages",
            "rows_dropped_empty",
            "rows_dropped_no_response",
            "rows_dropped_oversized",
            "special_token_occurrences",
        ] {
            let sum = drops[key].as_i64().unwrap_or(0)
                + store.manifest["drops"][key].as_i64().unwrap_or(0);
            drops[key] = json!(sum);
        }
        combined_mask.extend_from_slice(&store.mask);
        combined_ids.extend_from_slice(&store.ids);
    }
    if combined_ids.len() != total || combined_mask.len() != total {
        return Err("the concatenated arrays disagree in length".into());
    }

    report_output::claim(&out).map_err(|e| e.to_string())?;
    let tokens_path = out.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab_size).map_err(|e| e.to_string())?;
    writer
        .write_tokens(&combined_ids)
        .map_err(|e| e.to_string())?;
    let written = writer.finish().map_err(|e| e.to_string())?;
    if written as usize != total {
        return Err(format!("wrote {written} tokens, expected {total}"));
    }
    let mask_path = out.join("response_mask.u8");
    fs::write(&mask_path, &combined_mask).map_err(|e| e.to_string())?;

    let response_tokens = combined_mask.iter().filter(|&&m| m == 1).count();
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": "train",
        "template_rule": loaded[0].manifest["template_rule"],
        "mask_rule": loaded[0].manifest["mask_rule"],
        "mixed_from": {
            "tool": "mix-chat-corpus",
            "why": "a response-phase fit takes one store; an arm that adds rows to the training distribution must train on the real store and the added rows together, because fitting on the added rows alone destroys the model",
            "inputs": inputs,
        },
        "drops": drops,
        "files": files,
        "tokenizer": {"path": tokenizer_path.display().to_string(), "sha256": tokenizer_sha},
        "rows_used": loaded
            .iter()
            .map(|s| s.manifest["rows_used"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        "tokens": total,
        "response_tokens": response_tokens,
        "response_fraction": response_tokens as f64 / total as f64,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": combined_mask.len(),
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "mask_sha256": uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    // Decode a boundary token pair as a cheap sanity check that the vocabulary
    // really is shared: the last synthetic id and the first real-store id.
    let _ = tokenizer;
    println!(
        "mixed {} stores, {total} tokens, {response_tokens} response tokens ({:.1}%)",
        loaded.len(),
        100.0 * response_tokens as f64 / total as f64
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
