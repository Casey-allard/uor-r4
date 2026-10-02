//! Decode a prepared chat store with its own tokenizer and report its
//! structural invariants.
//!
//! ```text
//! inspect-chat-corpus store=SPLIT_DIR tokenizer=TOKENIZER.json [docs=3]
//! ```
//!
//! `SPLIT_DIR` holds `tokens.u16`, `response_mask.u8` and `manifest.json`
//! (schema `uor-r4-chat-corpus/v1`). Documents are split on the manifest's BOS
//! id, and for each one the tool prints the decoded text with the masked
//! (scored) region marked, so a corpus can be eyeballed before anything is
//! fitted on it. It also counts documents that fail the invariants the trainer
//! depends on: a document starts at BOS, at least one token is masked, and the
//! masked region is non-empty text.
//!
//! This exists because token ids are not characters: reading `tokens.u16` as
//! if it were text reports nonsense, and a corpus whose mask misses the answer
//! is silently useless rather than visibly broken.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

fn run(args: &[String]) -> Result<()> {
    let arg = |key: &str| -> Result<PathBuf> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing {key}="))
    };
    let number = |key: &str, default: u64| -> Result<u64> {
        match args.iter().find_map(|a| a.strip_prefix(&format!("{key}="))) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("{key}= must be a number")),
        }
    };
    let store = arg("store")?;
    let tokenizer_path = arg("tokenizer")?;
    let show = number("docs", 3)? as usize;

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(
        &fs::read(&tokenizer_path).map_err(|e| e.to_string())?,
    )
    .ok_or("unreadable tokenizer.json")?;
    let manifest: Value =
        serde_json::from_slice(&fs::read(store.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let bos = manifest["bos_id"]
        .as_u64()
        .ok_or("manifest has no bos_id")? as u32;
    let eos = manifest["eos_id"]
        .as_u64()
        .ok_or("manifest has no eos_id")? as u32;
    // The store is a corpus file with a header, not a bare token array: read it
    // through the same reader the trainer uses. Reading the file from byte 0
    // picks up the 64-byte `CORPUS_HEADER_SIZE` prefix and shifts every document boundary, which is
    // exactly the false alarm this tool raised before it was corrected.
    let reader = MmapCorpusReader::open(store.join("tokens.u16")).map_err(|e| e.to_string())?;
    let ids: Vec<u32> = reader.as_slice().iter().map(|&t| u32::from(t)).collect();
    let mask = fs::read(store.join("response_mask.u8")).map_err(|e| e.to_string())?;
    println!(
        "{} rows declared, {} tokens, mask {} bytes, bos={bos} eos={eos}",
        manifest["rows_used"].as_u64().unwrap_or(0),
        ids.len(),
        mask.len()
    );
    if mask.len() != ids.len() {
        // The trainer's reader treats the two as parallel; a mismatch is a
        // real defect, not a cosmetic one.
        println!(
            "MISMATCH: {} tokens against {} mask bytes (difference {})",
            ids.len(),
            mask.len(),
            ids.len() as i64 - mask.len() as i64
        );
    }

    let mut starts: Vec<usize> = ids
        .iter()
        .enumerate()
        .filter(|(_, &t)| t == bos)
        .map(|(i, _)| i)
        .collect();
    if starts.first() != Some(&0) {
        starts.insert(0, 0);
    }
    let mut no_mask = 0usize;
    let mut empty_scored = 0usize;
    let mut not_bos = 0usize;
    for (n, &start) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(ids.len());
        let doc = &ids[start..end];
        if doc.first() != Some(&bos) {
            not_bos += 1;
        }
        let scored: Vec<u32> = (start..end)
            .filter(|&i| mask.get(i) == Some(&1))
            .map(|i| ids[i])
            .collect();
        if scored.is_empty() {
            no_mask += 1;
        } else if tokenizer.decode(&scored).trim().is_empty() {
            empty_scored += 1;
        }
        if n < show {
            let full = tokenizer.decode(doc);
            let answer = tokenizer.decode(&scored);
            println!(
                "\n--- document {n} ({} tokens, {} scored)",
                doc.len(),
                scored.len()
            );
            println!("    full  : {:?}", full.replace('\n', "\\n"));
            println!("    scored: {:?}", answer);
        }
    }
    println!(
        "\n{} documents; {not_bos} not starting at BOS, {no_mask} with no masked token, \
         {empty_scored} whose masked region decodes to nothing",
        starts.len()
    );
    if not_bos > 0 || no_mask > 0 || empty_scored > 0 {
        return Err("the store fails a structural invariant the trainer relies on".into());
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
