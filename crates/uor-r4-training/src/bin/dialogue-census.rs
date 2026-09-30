//! Episode census of a prepared dialogue split under each prefix policy
//! (D18 section 4). No model is loaded and nothing is written.
//!
//! ```text
//! dialogue-census tokens=TOKENS.u16 mask=MASK.u8 manifest=MANIFEST.json \
//!   tokenizer=TOKENIZER.json [policies=full_prefix,truncated_prefix]
//! ```
//!
//! For each policy it prints the eligible population that `dialogue-train`
//! records as `train_population`, and an episode digest: the SHA-256 of every
//! eligible response's corpus ordinal, document start, response start and
//! response end, as little-endian u64s in index order. Two runs with equal
//! digests sample the same episodes. The inputs are bound by SHA-256.

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::dialogue_episodes::{EpisodeIndex, PrefixPolicy};
use uor_r4_training::sha256_file;
use uor_r4_training::stack_dialogue::{episode_contract, DialogueSplit};

const USAGE: &str = "usage: dialogue-census tokens=TOKENS.u16 mask=MASK.u8 \
                     manifest=MANIFEST.json tokenizer=TOKENIZER.json [policies=P1,P2,...]";

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("dialogue-census: {error}");
            ExitCode::FAILURE
        }
    }
}

fn episode_digest(index: &EpisodeIndex<'_>) -> String {
    let mut digest = Sha256::new();
    for span in index.episodes() {
        for value in [
            span.corpus_response_index,
            span.document_start,
            span.response_start,
            span.response_end,
        ] {
            digest.update((value as u64).to_le_bytes());
        }
    }
    hex::encode(digest.finalize())
}

fn run() -> Result<String, String> {
    let (mut tokens, mut mask, mut manifest, mut tokenizer) = (None, None, None, None);
    let mut policies = "full_prefix,truncated_prefix".to_owned();
    for argument in std::env::args().skip(1) {
        let Some((key, value)) = argument.split_once('=') else {
            return Err(USAGE.to_owned());
        };
        let slot = match key {
            "tokens" => &mut tokens,
            "mask" => &mut mask,
            "manifest" => &mut manifest,
            "tokenizer" => &mut tokenizer,
            "policies" => {
                policies = value.to_owned();
                continue;
            }
            _ => return Err(USAGE.to_owned()),
        };
        *slot = Some(PathBuf::from(value));
    }
    let (Some(tokens), Some(mask), Some(manifest), Some(tokenizer)) =
        (tokens, mask, manifest, tokenizer)
    else {
        return Err(USAGE.to_owned());
    };
    let policies = policies
        .split(',')
        .map(|policy| PrefixPolicy::parse(Some(policy)).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let tokenizer_bytes = fs::read(&tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let parsed = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer: not a supported byte-level BPE tokenizer.json")?;
    let split = DialogueSplit::load(&tokens, &mask, &manifest).map_err(|e| e.to_string())?;
    let (_, contract) = episode_contract(&parsed, split.vocab_size()).map_err(|e| e.to_string())?;
    let mut rows = Vec::with_capacity(policies.len());
    for policy in policies {
        let index = split
            .index_for(contract.clone(), policy)
            .map_err(|e| e.to_string())?;
        rows.push(json!({
            "policy": policy,
            "episode_digest": episode_digest(&index),
            "population": index.population(),
        }));
    }
    let file = |path: &PathBuf| -> Result<serde_json::Value, String> {
        Ok(json!({"path": path, "sha256": sha256_file(path).map_err(|e| e.to_string())?}))
    };
    Ok(json!({
        "schema": "uor-r4.dialogue-census/1",
        "inputs": {
            "tokens": file(&tokens)?, "mask": file(&mask)?, "manifest": file(&manifest)?,
            "tokenizer": file(&tokenizer)?, "tokenizer_cid": parsed.address(),
        },
        "context": contract.context,
        "assistant_marker_ids": contract.assistant_marker_ids,
        "policies": rows,
    })
    .to_string())
}
