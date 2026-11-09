//! claude-342: how far back the question sits from each scored row of a pre-tokenized TRAIN.
//!
//! usage: train-question-reach DIR, where DIR holds `manifest.json`, `tokens.u16` and
//! `response_mask.u8`. It prints one JSON report. Laptop counting only: no model, no GPU.
//!
//! A document starts at a `bos_id` token. Within one, a run of unscored rows (mask 0) is a
//! question, and a scored row (mask 1) counts against the run just before it. So for a scored row
//! `r` after a run `[start, end]`, `r - start` is the reach that holds the whole question and
//! `r - end` the reach that holds its last row. Windows are ignored, so this is an upper bound on
//! what a training window can see.
//!
//! An unscored run is the question only where every reply is scored. `reply-cap` keeps a capped
//! reply as unscored context, so on a capped root one run can hold several whole turns. The
//! `asked` counts are therefore read from the tokens: the question a reply answers runs from the
//! word after the last `User:` before the reply to the word before the `\nAssistant:` header.

use std::fs;
use std::path::Path;

const REACHES: [usize; 6] = [4, 8, 16, 32, 64, 128];
/// Lengths of a question at or past this are counted in the last bin of the median's histogram.
const LENGTH_BINS: usize = 8192;
/// `tokens.u16` starts with this many bytes of header.
const TOKENS_HEADER_BYTES: usize = 64;
/// The tokens of `User:`, which opens every user turn of the chat roots.
const USER_MARKER: [u16; 3] = [55, 2728, 28];
/// The tokens of `\nAssistant:`, the six rows just before a reply.
const ASSISTANT_HEADER: [u16; 6] = [201, 35, 560, 652, 714, 28];

#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct Within {
    rows: u64,
    start: [u64; REACHES.len()],
    end: [u64; REACHES.len()],
}

impl Within {
    fn add(&mut self, to_start: usize, to_end: usize) {
        self.rows += 1;
        for (i, &reach) in REACHES.iter().enumerate() {
            self.start[i] += u64::from(to_start <= reach);
            self.end[i] += u64::from(to_end <= reach);
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Reach {
    documents: u64,
    scored: u64,
    no_question: u64,
    all: Within,
    first: Within,
    /// How many replies have a question of each length, for the median.
    lengths: Vec<u64>,
    /// Scored rows and reply first rows against the question read from the tokens.
    asked_all: Within,
    asked_first: Within,
    asked_lengths: Vec<u64>,
    /// Replies with no `User:` turn and header before them, and their scored rows.
    replies_not_placed: u64,
    scored_not_placed: u64,
}

fn count(tokens: &[u16], mask: &[u8], bos: u16) -> Result<Reach, String> {
    if tokens.len() != mask.len() {
        return Err(format!(
            "{} tokens against {} mask rows",
            tokens.len(),
            mask.len()
        ));
    }
    let mut reach = Reach {
        documents: 0,
        scored: 0,
        no_question: 0,
        all: Within::default(),
        first: Within::default(),
        lengths: vec![0; LENGTH_BINS],
        asked_all: Within::default(),
        asked_first: Within::default(),
        asked_lengths: vec![0; LENGTH_BINS],
        replies_not_placed: 0,
        scored_not_placed: 0,
    };
    let mut question: Option<(usize, usize)> = None;
    let mut last_scored = false;
    // The word after this document's last `User:`, and the question of the current reply.
    let mut after_marker: Option<usize> = None;
    let mut asked: Option<(usize, usize)> = None;
    for (row, (&token, &flag)) in tokens.iter().zip(mask).enumerate() {
        if token == bos {
            if flag > 1 {
                return Err(format!("mask value {flag} at row {row}"));
            }
            reach.documents += 1;
            question = None;
            last_scored = false;
            after_marker = None;
            asked = None;
            continue;
        }
        if row >= 2 && tokens[row - 2..=row] == USER_MARKER {
            after_marker = Some(row + 1);
        }
        match flag {
            0 => {
                question = match question {
                    Some((start, end)) if end + 1 == row => Some((start, row)),
                    _ => Some((row, row)),
                };
                last_scored = false;
            }
            1 => {
                reach.scored += 1;
                if !last_scored {
                    let header_starts = row.checked_sub(ASSISTANT_HEADER.len());
                    asked = match (after_marker, header_starts) {
                        (Some(first), Some(header))
                            if tokens[header..row] == ASSISTANT_HEADER && first < header =>
                        {
                            reach.asked_first.add(row - first, row - (header - 1));
                            reach.asked_lengths[(header - first).min(LENGTH_BINS - 1)] += 1;
                            Some((first, header - 1))
                        }
                        _ => {
                            reach.replies_not_placed += 1;
                            None
                        }
                    };
                }
                match asked {
                    Some((first, last)) => reach.asked_all.add(row - first, row - last),
                    None => reach.scored_not_placed += 1,
                }
                match question {
                    Some((start, end)) => {
                        reach.all.add(row - start, row - end);
                        if !last_scored {
                            reach.first.add(row - start, row - end);
                            reach.lengths[(end + 1 - start).min(LENGTH_BINS - 1)] += 1;
                        }
                    }
                    None => reach.no_question += 1,
                }
                last_scored = true;
            }
            other => return Err(format!("mask value {other} at row {row}")),
        }
    }
    Ok(reach)
}

fn median(lengths: &[u64]) -> usize {
    let total: u64 = lengths.iter().sum();
    let mut seen = 0;
    for (length, &n) in lengths.iter().enumerate() {
        seen += n;
        if total > 0 && 2 * seen >= total {
            return length;
        }
    }
    0
}

fn within_json(within: &Within) -> serde_json::Value {
    let per_mille = |n: u64| {
        if within.rows == 0 {
            0
        } else {
            n * 1000 / within.rows
        }
    };
    let rows = |counts: &[u64; REACHES.len()]| {
        REACHES
            .iter()
            .zip(counts)
            .map(|(reach, &n)| serde_json::json!({"reach": reach, "rows": n, "per_mille": per_mille(n)}))
            .collect::<Vec<_>>()
    };
    serde_json::json!({
        "rows": within.rows,
        "to_question_start": rows(&within.start),
        "to_question_end": rows(&within.end),
    })
}

fn run(dir: &Path) -> Result<serde_json::Value, String> {
    let read = |name: &str| fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"));
    let manifest: serde_json::Value = serde_json::from_slice(&read("manifest.json")?)
        .map_err(|e| format!("manifest.json: {e}"))?;
    let total = manifest["tokens"]
        .as_u64()
        .ok_or("manifest has no tokens")? as usize;
    let bos = u16::try_from(
        manifest["bos_id"]
            .as_u64()
            .ok_or("manifest has no bos_id")?,
    )
    .map_err(|e| format!("bos_id: {e}"))?;
    let bytes = read("tokens.u16")?;
    if bytes.len() != TOKENS_HEADER_BYTES + 2 * total {
        return Err(format!(
            "tokens.u16 has {} bytes for {total} tokens",
            bytes.len()
        ));
    }
    let tokens: Vec<u16> = bytes[TOKENS_HEADER_BYTES..]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let mask = read("response_mask.u8")?;
    if mask.len() != total {
        return Err(format!(
            "response_mask.u8 has {} rows for {total} tokens",
            mask.len()
        ));
    }
    let mut at = 0;
    let mut roots = Vec::new();
    for root in manifest["roots"]
        .as_array()
        .ok_or("manifest has no roots")?
    {
        let name = root["name"].as_str().ok_or("a root has no name")?;
        let taken = root["tokens_taken"]
            .as_u64()
            .ok_or("a root has no tokens_taken")? as usize;
        let end = at + taken;
        if end > total {
            return Err(format!("root {name} runs past the tokens"));
        }
        let reach = count(&tokens[at..end], &mask[at..end], bos)?;
        let expected = root["scored_taken"]
            .as_u64()
            .ok_or("a root has no scored_taken")?;
        if reach.scored != expected {
            return Err(format!(
                "root {name}: {} scored rows, manifest says {expected}",
                reach.scored
            ));
        }
        roots.push(serde_json::json!({
            "name": name,
            "documents": reach.documents,
            "tokens": taken,
            "scored": reach.scored,
            "scored_without_a_question": reach.no_question,
            "median_question_rows": median(&reach.lengths),
            "all_scored_rows": within_json(&reach.all),
            "first_row_of_each_reply": within_json(&reach.first),
            "replies_not_placed": reach.replies_not_placed,
            "scored_not_placed": reach.scored_not_placed,
            "median_asked_question_rows": median(&reach.asked_lengths),
            "asked_all_scored_rows": within_json(&reach.asked_all),
            "asked_first_row_of_each_reply": within_json(&reach.asked_first),
        }));
        at = end;
    }
    if at != total {
        return Err(format!("the roots cover {at} of {total} tokens"));
    }
    Ok(serde_json::json!({
        "schema": "uor-r4-train-question-reach/v1",
        "notes": {
            "all_scored_rows, first_row_of_each_reply": "against the unscored run before the row; to_question_end is the run's last row, the header's colon on a chat root",
            "asked_all_scored_rows, asked_first_row_of_each_reply": "against the question read from the tokens; to_question_end is the question's last word",
            "rows 0": "no data: its per_mille and medians are 0 for that reason",
            "lengths": "a question of 8191 rows or more counts as 8191 in its median",
        },
        "tokens": total,
        "roots": roots,
    }))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir] = args.as_slice() else {
        eprintln!("usage: train-question-reach DIR");
        std::process::exit(2);
    };
    match run(Path::new(dir)) {
        Ok(report) => println!("{report:#}"),
        Err(error) => {
            eprintln!("train-question-reach: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_each_scored_row_against_the_question_just_before_it() {
        // Document 1: a 3-row question, a 2-row reply, a 1-row question, a 1-row reply.
        // Document 2: a scored row with no question before it, then BOS resets nothing else.
        let tokens = [0, 5, 6, 7, 8, 9, 5, 8, 0, 9];
        let mask = [0, 0, 0, 0, 1, 1, 0, 1, 0, 1];
        let reach = count(&tokens, &mask, 0).unwrap();
        assert_eq!(
            (reach.documents, reach.scored, reach.no_question),
            (2, 4, 1)
        );
        assert_eq!(reach.all.rows, 3);
        // Rows 4, 5 against [1, 3]: to the start 3 and 4, to the end 1 and 2. Row 7 against
        // [6, 6]: 1 and 1. Reach 4 holds all three starts, and no reach below 4 is listed.
        assert_eq!((reach.all.start[0], reach.all.end[0]), (3, 3));
        assert_eq!(reach.first.rows, 2);
        assert_eq!(reach.first.start[0], 2);
        assert_eq!(median(&reach.lengths), 1);
        assert_eq!((reach.lengths[3], reach.lengths[1]), (1, 1));
    }

    /// `User:`, the words, the header: the rows of one prompt.
    fn turn(words: &[u16]) -> Vec<u16> {
        let mut rows = USER_MARKER.to_vec();
        rows.extend(words);
        rows.extend(ASSISTANT_HEADER);
        rows
    }

    #[test]
    fn the_asked_question_is_the_last_turn_and_not_the_whole_unscored_run() {
        // A capped turn stays unscored, so the run before the scored reply holds two whole turns.
        let mut tokens = vec![0];
        tokens.extend(turn(&[7, 7]));
        tokens.extend([9; 20]); // the capped reply, kept as context
        let asked_turn = tokens.len();
        tokens.extend(turn(&[8, 8]));
        let scored = tokens.len();
        tokens.extend([9, 9, 9]);
        let mut mask = vec![0; scored];
        mask.extend([1, 1, 1]);
        let reach = count(&tokens, &mask, 0).unwrap();
        // The run starts at row 1; the question's first word is three rows into the second turn.
        let first_word = asked_turn + USER_MARKER.len();
        let last_word = scored - ASSISTANT_HEADER.len() - 1;
        assert_eq!(scored - first_word, 8);
        assert_eq!(scored - last_word, 7);
        assert_eq!((reach.replies_not_placed, reach.scored_not_placed), (0, 0));
        assert_eq!((reach.asked_all.rows, reach.asked_first.rows), (3, 1));
        // The three rows are 8, 9, 10 back from the first word and 7, 8, 9 from the last, so
        // reach 8 sits on both boundaries: one first word and two last words. 16 holds them all.
        assert_eq!((reach.asked_all.start[1], reach.asked_all.end[1]), (1, 2));
        assert_eq!((reach.asked_all.start[2], reach.asked_all.end[2]), (3, 3));
        assert_eq!(
            (reach.asked_first.start[0], reach.asked_first.end[0]),
            (0, 0)
        );
        assert_eq!(
            (reach.asked_first.start[1], reach.asked_first.end[1]),
            (1, 1)
        );
        assert_eq!(median(&reach.asked_lengths), 2);
        // The run-based count measures from the first turn, 42 to 44 rows back.
        assert_eq!(reach.all.start[3], 0);
        assert_eq!(reach.all.start[4], 3);
    }

    #[test]
    fn the_run_based_distances_sit_on_their_boundary() {
        // A 3-row run and a 5-row reply: 3 to 7 back from its start, 1 to 5 from its end.
        let tokens = [0, 5, 5, 5, 9, 9, 9, 9, 9];
        let mask = [0, 0, 0, 0, 1, 1, 1, 1, 1];
        let reach = count(&tokens, &mask, 0).unwrap();
        assert_eq!((reach.all.start[0], reach.all.end[0]), (2, 4));
        assert_eq!((reach.all.start[1], reach.all.end[1]), (5, 5));
        assert_eq!((reach.first.start[0], reach.first.end[0]), (1, 1));
    }

    #[test]
    fn a_reply_first_row_sits_on_both_sides_of_a_reach() {
        // Questions of 2, 3 and 1 words, one scored row each: their first words are 8, 9, 7 back.
        let mut tokens = Vec::new();
        for words in [&[8u16, 8][..], &[8, 8, 8], &[8]] {
            tokens.push(0);
            tokens.extend(turn(words));
            tokens.push(9);
        }
        let mask: Vec<u8> = tokens
            .iter()
            .enumerate()
            .map(|(row, _)| u8::from(tokens.get(row + 1).is_none_or(|&next| next == 0)))
            .collect();
        let reach = count(&tokens, &mask, 0).unwrap();
        assert_eq!(reach.replies_not_placed, 0);
        assert_eq!(reach.asked_first.rows, 3);
        // Reach 8 holds the rows 8 and 7 back and not the one 9 back.
        assert_eq!(reach.asked_first.start[1], 2);
        assert_eq!(
            (
                reach.asked_lengths[1],
                reach.asked_lengths[2],
                reach.asked_lengths[3]
            ),
            (1, 1, 1)
        );
    }

    #[test]
    fn a_header_straight_after_user_has_no_question_to_place() {
        let mut tokens = vec![0];
        tokens.extend(turn(&[]));
        let mut mask = vec![0; tokens.len()];
        tokens.push(9);
        mask.push(1);
        let reach = count(&tokens, &mask, 0).unwrap();
        assert_eq!((reach.replies_not_placed, reach.scored_not_placed), (1, 1));
        assert_eq!(reach.asked_first.rows, 0);
    }

    #[test]
    fn a_reply_with_no_turn_before_it_is_counted_as_not_placed() {
        // No header before the reply; then no `User:` before a header.
        let mut tokens = vec![0];
        tokens.extend(USER_MARKER);
        tokens.extend([7, 7, 9]);
        let mut mask = vec![0; tokens.len() - 1];
        mask.push(1);
        tokens.push(0);
        mask.push(0);
        tokens.extend(ASSISTANT_HEADER);
        mask.extend([0; 6]);
        tokens.push(9);
        mask.push(1);
        let reach = count(&tokens, &mask, 0).unwrap();
        assert_eq!((reach.replies_not_placed, reach.scored_not_placed), (2, 2));
        assert_eq!(reach.asked_all.rows, 0);
    }

    #[test]
    fn a_far_question_falls_outside_a_short_reach_and_a_bad_mask_is_refused() {
        // A 10-row question and a 1-row reply: the start is 10 back, the end 1 back.
        let mut tokens = vec![0];
        tokens.extend([5; 11]);
        let mut mask = vec![0; 11];
        mask.push(1);
        let reach = count(&tokens, &mask, 0).unwrap();
        assert_eq!(reach.all.start[..3], [0, 0, 1]);
        assert_eq!(reach.all.end[..3], [1, 1, 1]);
        assert!(count(&[5], &[2], 0).unwrap_err().contains("mask value 2"));
        assert!(count(&[5], &[], 0).unwrap_err().contains("mask rows"));
        assert!(count(&[0], &[2], 0).unwrap_err().contains("mask value 2"));
    }
}
