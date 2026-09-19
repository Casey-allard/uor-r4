//! A minimal chat/instruction pipeline for the low-bit core.
//!
//! # Why byte-level
//!
//! The core's embedding is a selected column read over a vocabulary, so the vocabulary has to be
//! small enough to learn from a small corpus. A byte-level vocabulary (256 symbols) plus three chat
//! control tokens is lossless, needs no tokenizer training, and cannot invent an out-of-vocabulary
//! token. It is a deliberate starting point, not a claim that byte-level is the right final
//! tokenizer.
//!
//! # Sequence shape
//!
//! One example is one supervised sequence:
//!
//! ```text
//!   <|user|> instruction <|assistant|> response <|end|>
//! ```
//!
//! Next-token prediction over that whole sequence teaches the model to continue an instruction with
//! a response, and to stop at `<|end|>`. Generation feeds
//! `<|user|> instruction <|assistant|>` and samples until `<|end|>`.
//!
//! # What this is not
//!
//! This is a pipeline, not a capability. It moves the core from "cannot learn" to "can be trained on
//! instruction data"; whether the trained model says anything useful is a separate, empirical
//! question that must be answered by reading its generations.

#![forbid(unsafe_code)]

/// Marks the start of a user turn.
pub const TOK_USER: u32 = 256;
/// Marks the start of an assistant turn.
pub const TOK_ASSISTANT: u32 = 257;
/// Marks the end of an example.
pub const TOK_END: u32 = 258;
/// Byte-level vocabulary plus the three chat control tokens.
pub const CHAT_VOCAB: usize = 259;

/// Raw bytes as token ids.
pub fn encode_bytes(text: &str) -> Vec<u32> {
    text.bytes().map(|b| b as u32).collect()
}

/// Render tokens as text, showing control tokens as visible markers.
pub fn decode(tokens: &[u32]) -> String {
    let mut out = String::new();
    let mut buf: Vec<u8> = Vec::new();
    for &t in tokens {
        if t < 256 {
            buf.push(t as u8);
        } else {
            out.push_str(&String::from_utf8_lossy(&buf));
            buf.clear();
            match t {
                TOK_USER => out.push_str("<|user|>"),
                TOK_ASSISTANT => out.push_str("<|assistant|>"),
                TOK_END => out.push_str("<|end|>"),
                _ => out.push_str("<|?>"),
            }
        }
    }
    out.push_str(&String::from_utf8_lossy(&buf));
    out
}

/// A full supervised example: user turn, assistant turn, end marker.
pub fn encode_turn(instruction: &str, response: &str) -> Vec<u32> {
    let mut v = Vec::with_capacity(instruction.len() + response.len() + 3);
    v.push(TOK_USER);
    v.extend(encode_bytes(instruction));
    v.push(TOK_ASSISTANT);
    v.extend(encode_bytes(response));
    v.push(TOK_END);
    v
}

/// A prompt ending at the assistant marker, for generation.
pub fn encode_prompt(instruction: &str) -> Vec<u32> {
    let mut v = Vec::with_capacity(instruction.len() + 2);
    v.push(TOK_USER);
    v.extend(encode_bytes(instruction));
    v.push(TOK_ASSISTANT);
    v
}

/// Collapse all whitespace runs to single spaces and trim. Applied when writing a corpus so one
/// example is one line and the tab separator is unambiguous.
pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse the tab-separated instruction corpus: one `instruction<TAB>response` per line.
pub fn parse_tsv(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_end_matches(['\r', '\n']);
            if line.trim().is_empty() {
                return None;
            }
            let (a, b) = line.split_once('\t')?;
            Some((a.trim().to_string(), b.trim().to_string()))
        })
        .collect()
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

const WORDS: [&str; 16] = [
    "straw", "apple", "river", "cloud", "piano", "tiger", "meadow", "silver", "garden", "planet",
    "candle", "forest", "bridge", "winter", "marble", "rocket",
];

/// Procedurally generated instruction/response pairs.
///
/// Deliberately synthetic and narrow: arithmetic, string reversal, casing, repetition and a few
/// fixed greetings. The point is instruction *following* with a checkable answer, not natural
/// conversation, and the narrowness is what lets a tiny core be measured rather than merely read.
pub fn procedural_examples(n: usize, seed: u64) -> Vec<(String, String)> {
    let mut st = seed | 1;
    let mut out = Vec::with_capacity(n + 6);
    for i in 0..n {
        let pick = (xorshift(&mut st) % WORDS.len() as u64) as usize;
        let w = WORDS[pick];
        match i % 6 {
            0 => {
                let a = xorshift(&mut st) % 100;
                let b = xorshift(&mut st) % 100;
                out.push((format!("What is {a} + {b}?"), format!("{}", a + b)));
            }
            1 => {
                let a = xorshift(&mut st) % 100;
                let b = xorshift(&mut st) % 100;
                let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
                out.push((format!("What is {hi} - {lo}?"), format!("{}", hi - lo)));
            }
            2 => {
                let rev: String = w.chars().rev().collect();
                out.push((format!("Reverse the word '{w}'."), rev));
            }
            3 => out.push((format!("Write '{w}' in uppercase."), w.to_uppercase())),
            4 => out.push((format!("Repeat '{w}' two times."), format!("{w} {w}"))),
            _ => {
                let first = w.chars().next().unwrap_or('?');
                out.push((
                    format!("What is the first letter of '{w}'?"),
                    first.to_string(),
                ));
            }
        }
    }
    // A handful of fixed conversational turns, so the corpus is not purely arithmetic.
    out.push((
        "Hello!".to_string(),
        "Hello! How can I help you today?".to_string(),
    ));
    out.push((
        "Hi there.".to_string(),
        "Hi! What would you like to do?".to_string(),
    ));
    out.push((
        "How are you?".to_string(),
        "I am a small language model, so I am ready to help.".to_string(),
    ));
    out.push((
        "What is your name?".to_string(),
        "I am the UOR-R4 geometric language model.".to_string(),
    ));
    out.push((
        "Goodbye.".to_string(),
        "Goodbye! Come back any time.".to_string(),
    ));
    out.push(("Thank you.".to_string(), "You are welcome.".to_string()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_round_trips_through_decode() {
        let seq = encode_turn("what is 2 + 2?", "4");
        let text = decode(&seq);
        assert_eq!(text, "<|user|>what is 2 + 2?<|assistant|>4<|end|>");
        // Byte tokens carry the exact bytes; control tokens are the only non-byte ids.
        assert!(seq.iter().all(|&t| t < CHAT_VOCAB as u32));
    }

    #[test]
    fn non_ascii_round_trips() {
        let seq = encode_turn("café ☕", "ok");
        assert_eq!(decode(&seq), "<|user|>café ☕<|assistant|>ok<|end|>");
    }

    #[test]
    fn prompt_ends_at_the_assistant_marker() {
        let p = encode_prompt("hi");
        assert_eq!(p, vec![TOK_USER, b'h' as u32, b'i' as u32, TOK_ASSISTANT]);
    }

    #[test]
    fn tsv_parsing_skips_blanks_and_splits_on_tab() {
        let rows = parse_tsv("a\tb\n\n  \nc\td\n");
        assert_eq!(
            rows,
            vec![("a".into(), "b".into()), ("c".into(), "d".into())]
        );
    }

    #[test]
    fn procedural_corpus_is_deterministic_and_well_formed() {
        let a = procedural_examples(60, 7);
        let b = procedural_examples(60, 7);
        assert_eq!(a, b);
        assert!(a.len() >= 60);
        for (i, r) in &a {
            assert!(!i.is_empty() && !r.is_empty());
            assert!(!i.contains('\t') && !r.contains('\t'));
        }
        // The arithmetic templates must actually be correct.
        let mut checked = 0;
        for (i, r) in &a {
            if let Some(rest) = i.strip_prefix("What is ") {
                if let Some(num) = rest.strip_suffix('?') {
                    if let Some((l, rr)) = num.split_once(" + ") {
                        let (x, y): (i64, i64) = (l.parse().unwrap(), rr.parse().unwrap());
                        assert_eq!(r.parse::<i64>().unwrap(), x + y);
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 0, "expected arithmetic examples");
    }
}
