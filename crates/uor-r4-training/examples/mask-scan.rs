//! Count the literal role markers of the version-2 dialogue protocol in a u16 corpus stream.
//!
//! ```text
//! mask-scan <tokens.u16> <tokenizer.json> [--skip-header 64]
//! ```
//!
//! The chat corpus renders its roles as literal text (`User:`, `Assistant:`; the protocol's own
//! test asserts the marker at crates/uor-r4-tokenizer/src/dialogue.rs:475), so the response region
//! of every conversation is recoverable by scanning the token stream for those marker sequences.
//! This counts them and reports the token gaps, which is the verification that the markers are
//! present and the basis for emitting a response mask.

use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

fn occurrences(stream: &[u16], needle: &[u16]) -> Vec<usize> {
    if needle.is_empty() || stream.len() < needle.len() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    let mut i = 0;
    while i + needle.len() <= stream.len() {
        if stream[i..i + needle.len()] == *needle {
            hits.push(i);
            i += needle.len();
        } else {
            i += 1;
        }
    }
    hits
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.len() < 2 {
        eprintln!("usage: mask-scan <tokens.u16> <tokenizer.json> [--skip-header 64]");
        std::process::exit(1);
    }
    let skip = argv
        .iter()
        .position(|a| a == "--skip-header")
        .and_then(|i| argv.get(i + 1))
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(64);
    let raw = std::fs::read(&argv[0]).expect("read corpus");
    let tok_bytes = std::fs::read(&argv[1]).expect("read tokenizer");
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes).expect("parse tokenizer");
    let payload = if raw.len() > skip { &raw[skip..] } else { &raw[..] };
    let stream: Vec<u16> = payload
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();

    for label in ["Assistant:", "User:"] {
        let ids: Vec<u16> = tokenizer.encode(label).into_iter().map(|i| i as u16).collect();
        let hits = occurrences(&stream, &ids);
        let mut gaps: Vec<usize> = hits.windows(2).map(|w| w[1] - w[0]).collect();
        gaps.sort_unstable();
        let median = gaps.get(gaps.len() / 2).copied().unwrap_or(0);
        println!(
            "{{\"marker\":\"{}\",\"ids\":{:?},\"count\":{},\"first\":{:?},\"median_gap\":{},\"max_gap\":{:?}}}",
            label,
            ids,
            hits.len(),
            hits.first(),
            median,
            gaps.last()
        );
    }
    // Emit the response mask when asked: 1 for tokens inside an assistant response (after an
    // `Assistant:` marker, until the next role marker or the end of the conversation region).
    if let Some(idx) = argv.iter().position(|a| a == "--write-mask") {
        let out = argv.get(idx + 1).expect("--write-mask needs a path");
        let assistant: Vec<u16> = tokenizer.encode("Assistant:").into_iter().map(|i| i as u16).collect();
        let user: Vec<u16> = tokenizer.encode("User:").into_iter().map(|i| i as u16).collect();
        let mut marks = vec![(0usize, 0u8); 0];
        for hit in occurrences(&stream, &assistant) {
            marks.push((hit + assistant.len(), 1));
        }
        for hit in occurrences(&stream, &user) {
            marks.push((hit, 0));
        }
        marks.sort_unstable();
        let mut mask = vec![0u8; stream.len()];
        let mut state = 0u8;
        let mut next = 0usize;
        for i in 0..stream.len() {
            while next < marks.len() && marks[next].0 == i {
                state = marks[next].1;
                next += 1;
            }
            mask[i] = state;
        }
        let ones = mask.iter().filter(|m| **m == 1).count();
        std::fs::write(out, &mask).expect("write mask");
        println!(
            "{{\"mask\":\"{}\",\"bytes\":{},\"response_tokens\":{},\"response_fraction\":{:.6}}}",
            out,
            mask.len(),
            ones,
            ones as f64 / mask.len().max(1) as f64
        );
    }
    println!("{{\"tokens\":{},\"role_positions\":true}}", stream.len());
}
