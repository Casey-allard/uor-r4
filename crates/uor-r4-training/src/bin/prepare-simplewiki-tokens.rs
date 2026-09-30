//! Prepare 32 windows x 1,024 tokens from simple-wiki-20231101 using SmolLM2 tokenizer.

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

const TARGET_TOKENS: usize = 32 * 1024; // 32,768 tokens
const EOS_TOKEN: u32 = 2; // <|im_end|> for SmolLM2-Instruct

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let articles_path = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(
            "/Volumes/UOR-Workspace/uor-r4-models/corpora/simple-wiki-20231101/articles.jsonl",
        )
    });
    let tokenizer_path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(
            "/Volumes/UOR-Workspace/uor-r4-models/sources/smollm2-360m-instruct/tokenizer.json",
        )
    });
    let out_path = args.get(3).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("/Volumes/UOR-Workspace/uor-r4-models/corpora/simple-wiki-20231101/simplewiki_32x1024.u16")
    });

    println!("Loading tokenizer from {}", tokenizer_path.display());
    let tok_bytes = std::fs::read(&tokenizer_path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .ok_or("failed to parse tokenizer.json")?;

    println!("Reading articles from {}", articles_path.display());
    let file = File::open(&articles_path)?;
    let reader = BufReader::new(file);

    let mut tokens: Vec<u32> = Vec::with_capacity(TARGET_TOKENS + 1024);
    let mut article_count = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(&line)?;
        if let Some(text) = val.get("text").and_then(|v| v.as_str()) {
            let enc = tokenizer.encode(text);
            tokens.extend_from_slice(&enc);
            tokens.push(EOS_TOKEN);
            article_count += 1;
            if tokens.len() >= TARGET_TOKENS {
                break;
            }
        }
    }

    if tokens.len() < TARGET_TOKENS {
        return Err(format!(
            "insufficient tokens in corpus: got {} < target {}",
            tokens.len(),
            TARGET_TOKENS
        )
        .into());
    }

    tokens.truncate(TARGET_TOKENS);
    println!(
        "Collected {} tokens across {} articles",
        tokens.len(),
        article_count
    );

    let mut u16_bytes = Vec::with_capacity(tokens.len() * 2);
    for &t in &tokens {
        if t > u16::MAX as u32 {
            return Err(format!("token {t} exceeds u16::MAX").into());
        }
        u16_bytes.extend_from_slice(&(t as u16).to_le_bytes());
    }

    let mut hasher = Sha256::new();
    hasher.update(&u16_bytes);
    let hash = hex::encode(hasher.finalize());

    std::fs::write(&out_path, &u16_bytes)?;
    println!("Wrote {} bytes to {}", u16_bytes.len(), out_path.display());
    println!("SHA-256: {}", hash);

    Ok(())
}
