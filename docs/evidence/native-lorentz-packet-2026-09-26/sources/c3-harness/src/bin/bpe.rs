//! Scratch byte-level BPE: 256 byte symbols + 3,840 merges = 4,096 tokens, the
//! vocabulary size the D8 JointConfig requires. Deterministic: ties choose the
//! smallest pair. Pre-tokenization keeps identifier runs, punctuation runs (at
//! most 8 bytes) and whitespace runs (at most 32 bytes) apart; one space before
//! a word or punctuation run attaches to it.
//!
//! usage: bpe TRAIN_TEXT VALID_TEXT OUT_DIR
//! writes OUT_DIR/{train.u16, valid.u16, lens.u16, merges.txt, stats.json}

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;

const VOCAB: usize = 4096;
const MAX_PUNCT: usize = 8;
const MAX_SPACE: usize = 32;

fn class(b: u8) -> u8 {
    match b {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | 0x80..=0xff => 0,
        b' ' | b'\t' | b'\n' | b'\r' => 1,
        _ => 2,
    }
}

/// End of a run of class `c` starting at `from`, with the class's length cap.
fn run_end(text: &[u8], from: usize, c: u8) -> usize {
    let cap = match c {
        0 => usize::MAX,
        1 => MAX_SPACE,
        _ => MAX_PUNCT,
    };
    let mut end = from;
    while end < text.len() && class(text[end]) == c && end - from < cap {
        end += 1;
    }
    end
}

fn chunks(text: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let c = class(text[i]);
        if c == 1 {
            // A single space directly before a word or punctuation joins that chunk.
            if text[i] == b' ' && i + 1 < text.len() && class(text[i + 1]) != 1 {
                let end = run_end(text, i + 1, class(text[i + 1]));
                out.push(&text[i..end]);
                i = end;
                continue;
            }
            let mut end = run_end(text, i, 1);
            // Leave a final space for the following word.
            if end < text.len() && end - i > 1 && text[end - 1] == b' ' && class(text[end]) != 1 {
                end -= 1;
            }
            out.push(&text[i..end]);
            i = end;
            continue;
        }
        let end = run_end(text, i, c);
        out.push(&text[i..end]);
        i = end;
    }
    out
}

fn encode_chunk(chunk: &[u8], rank: &HashMap<(u32, u32), u32>) -> Vec<u32> {
    let mut symbols: Vec<u32> = chunk.iter().map(|&b| u32::from(b)).collect();
    loop {
        let mut best: Option<(u32, (u32, u32))> = None;
        for pair in symbols.windows(2) {
            if let Some(&r) = rank.get(&(pair[0], pair[1])) {
                if best.map_or(true, |(b, _)| r < b) {
                    best = Some((r, (pair[0], pair[1])));
                }
            }
        }
        let Some((r, (a, b))) = best else { break };
        let merged = 256 + r;
        let mut next = Vec::with_capacity(symbols.len());
        let mut i = 0;
        while i < symbols.len() {
            if i + 1 < symbols.len() && symbols[i] == a && symbols[i + 1] == b {
                next.push(merged);
                i += 2;
            } else {
                next.push(symbols[i]);
                i += 1;
            }
        }
        symbols = next;
    }
    symbols
}

fn encode(text: &[u8], rank: &HashMap<(u32, u32), u32>) -> Vec<u16> {
    let mut cache: HashMap<&[u8], Vec<u32>> = HashMap::new();
    let mut out = Vec::with_capacity(text.len() / 3);
    for chunk in chunks(text) {
        let ids = cache.entry(chunk).or_insert_with(|| encode_chunk(chunk, rank));
        out.extend(ids.iter().map(|&id| id as u16));
    }
    out
}

fn write_u16(path: &Path, values: &[u16]) -> Result<(), Box<dyn Error>> {
    let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    fs::write(path, bytes)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: bpe TRAIN_TEXT VALID_TEXT OUT_DIR".into());
    }
    let train = fs::read(&args[1])?;
    let valid = fs::read(&args[2])?;
    let out = Path::new(&args[3]);
    fs::create_dir_all(out)?;

    let mut counts: HashMap<&[u8], i64> = HashMap::new();
    for chunk in chunks(&train) {
        *counts.entry(chunk).or_default() += 1;
    }
    let mut unique: Vec<(&[u8], i64)> = counts.into_iter().collect();
    unique.sort();
    let mut words: Vec<Vec<u32>> = unique
        .iter()
        .map(|(w, _)| w.iter().map(|&b| u32::from(b)).collect())
        .collect();
    let freq: Vec<i64> = unique.iter().map(|(_, f)| *f).collect();
    let mut pair_count: HashMap<(u32, u32), i64> = HashMap::new();
    let mut holders: HashMap<(u32, u32), HashSet<u32>> = HashMap::new();
    for (wi, w) in words.iter().enumerate() {
        for p in w.windows(2) {
            *pair_count.entry((p[0], p[1])).or_default() += freq[wi];
            holders.entry((p[0], p[1])).or_default().insert(wi as u32);
        }
    }
    let mut token_bytes: Vec<Vec<u8>> = (0..=255u8).map(|b| vec![b]).collect();
    let mut merges: Vec<(u32, u32)> = Vec::new();
    while token_bytes.len() < VOCAB {
        let Some((&best, &count)) = pair_count
            .iter()
            .filter(|(_, &c)| c > 0)
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        else {
            break;
        };
        if count < 2 {
            break;
        }
        let new_id = token_bytes.len() as u32;
        merges.push(best);
        let joined = [
            token_bytes[best.0 as usize].clone(),
            token_bytes[best.1 as usize].clone(),
        ]
        .concat();
        token_bytes.push(joined);
        let affected = holders.remove(&best).unwrap_or_default();
        for wi in affected {
            let wi_usize = wi as usize;
            let f = freq[wi_usize];
            let old = std::mem::take(&mut words[wi_usize]);
            for p in old.windows(2) {
                if let Some(c) = pair_count.get_mut(&(p[0], p[1])) {
                    *c -= f;
                }
            }
            let mut merged = Vec::with_capacity(old.len());
            let mut i = 0;
            while i < old.len() {
                if i + 1 < old.len() && old[i] == best.0 && old[i + 1] == best.1 {
                    merged.push(new_id);
                    i += 2;
                } else {
                    merged.push(old[i]);
                    i += 1;
                }
            }
            for p in merged.windows(2) {
                *pair_count.entry((p[0], p[1])).or_default() += f;
                holders.entry((p[0], p[1])).or_default().insert(wi);
            }
            words[wi_usize] = merged;
        }
        pair_count.remove(&best);
        if merges.len() % 500 == 0 {
            pair_count.retain(|_, c| *c > 0);
            eprintln!("merges {} pairs {}", merges.len(), pair_count.len());
        }
    }
    let rank: HashMap<(u32, u32), u32> = merges
        .iter()
        .enumerate()
        .map(|(r, &p)| (p, r as u32))
        .collect();
    let train_ids = encode(&train, &rank);
    let valid_ids = encode(&valid, &rank);
    // Round trip: decoding must reproduce both texts exactly.
    for (ids, text) in [(&train_ids, &train), (&valid_ids, &valid)] {
        let decoded: Vec<u8> = ids
            .iter()
            .flat_map(|&id| token_bytes[id as usize].iter().copied())
            .collect();
        if &decoded != text {
            return Err("BPE round trip failed".into());
        }
    }
    let lens: Vec<u16> = token_bytes.iter().map(|t| t.len() as u16).collect();
    write_u16(&out.join("train.u16"), &train_ids)?;
    write_u16(&out.join("valid.u16"), &valid_ids)?;
    write_u16(&out.join("lens.u16"), &lens)?;
    let merge_text: String = merges
        .iter()
        .map(|(a, b)| format!("{a} {b}\n"))
        .collect();
    fs::write(out.join("merges.txt"), merge_text)?;
    let stats = serde_json::json!({
        "vocab": token_bytes.len(),
        "merges": merges.len(),
        "train_bytes": train.len(),
        "valid_bytes": valid.len(),
        "train_tokens": train_ids.len(),
        "valid_tokens": valid_ids.len(),
        "train_bytes_per_token": train.len() as f64 / train_ids.len() as f64,
        "valid_bytes_per_token": valid.len() as f64 / valid_ids.len() as f64,
        "longest_token_bytes": lens.iter().max(),
        "round_trip": "exact",
    });
    fs::write(out.join("stats.json"), serde_json::to_string_pretty(&stats)?)?;
    println!("{stats}");
    Ok(())
}
