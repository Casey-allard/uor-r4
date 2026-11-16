//! pair-positions (claude-384): the per-position replay check of the served pair rows.
//!
//! Usage: `pair-positions PAIR_ROWS_FILE SAMPLES_JSON...`
//!
//! `PAIR_ROWS_FILE` is the rows file `fbs-snapshot` wrote (`snapshot.json.pair-rows`), which holds
//! the dictionary and the marks the artifact serves. Each `SAMPLES_JSON` is a scoring pass's
//! `samples-*.json` (an array of objects with `prompt`, the prompt's token ids). For every
//! position `p` of every prompt, the float crate's own `pair_rows::plan` of the whole prompt (one
//! window, as the scoring pass reads it) names the rows the trunk adds at `p`; the shared
//! `PairTracker`, fed the prompt token by token as a served session is, names its picks at `p`.
//! The two are compared as raw u32 row numbers.
//!
//! Printed, one JSON line per file: the prompts, the positions, the positions with rows, the
//! positions that agree, `marker_starts` (reply positions where a user marker that the prompt
//! finishes begins: the trainer adds nothing there, and a reader that waited for the whole marker
//! would add the turn's rows), `broken_beginnings` (positions where a beginning of the user marker
//! the prefix ends with is not finished by the text after it: the trainer adds the rows and the
//! served rule does not), and
//! `disagree` (any other difference). The exit code is 1 when `disagree` is not 0 anywhere.
//!
//! It reads files only: no GPU, no training, no evaluation.

use std::path::Path;

use serde::Deserialize;
use uor_r4_float_train::pair_rows::plan;
use uor_r4_float_train::{FloatTrainError, Result};
use uor_r4_integer::fbs_pair_rows::{scan, PairRowsImage, PairTracker};

#[derive(Deserialize)]
struct Sample {
    prompt: Vec<u32>,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [rows_file, files @ ..] = args.as_slice() else {
        eprintln!("usage: pair-positions PAIR_ROWS_FILE SAMPLES_JSON...");
        std::process::exit(2);
    };
    if files.is_empty() {
        eprintln!("usage: pair-positions PAIR_ROWS_FILE SAMPLES_JSON...");
        std::process::exit(2);
    }
    let image = PairRowsImage::from_bytes(&std::fs::read(rows_file)?)
        .map_err(|e| FloatTrainError::Invalid(format!("{rows_file}: {e}")))?;
    let (dict, marks) = (image.dict(), image.marks());
    let mut failed = false;
    for file in files {
        let samples: Vec<Sample> = serde_json::from_slice(&std::fs::read(Path::new(file))?)?;
        let (mut positions, mut with_rows, mut agree) = (0usize, 0usize, 0usize);
        let (mut marker_starts, mut broken, mut disagree) = (0usize, 0usize, 0usize);
        let mut first_disagreement = None;
        for (index, sample) in samples.iter().enumerate() {
            let ids = &sample.prompt;
            let plan = plan(dict, marks, ids, 1, ids.len())?;
            let (_, users) = scan(ids, marks);
            let mut tracker = PairTracker::new();
            let mut before: Option<Vec<u32>> = None;
            for p in 0..ids.len() {
                let turn = plan.turn_of[p];
                let want = (turn < plan.turns as u32).then(|| {
                    let mut rows: Vec<u32> = plan
                        .hit_turn
                        .iter()
                        .zip(&plan.hit_row)
                        .filter(|&(&t, _)| t == turn)
                        .map(|(_, &r)| r)
                        .collect();
                    rows.sort_unstable();
                    rows
                });
                let got = tracker.push(dict, marks, &ids[..=p]).map(<[u32]>::to_vec);
                positions += 1;
                with_rows += usize::from(want.is_some());
                // The proper beginnings of the user marker the prefix ends with, by length, and
                // whether the text after any of them finishes the whole marker.
                let under_way: Vec<usize> = (1..marks.user.len())
                    .filter(|&k| ids[..=p].ends_with(&marks.user[..k]))
                    .collect();
                let finishes = under_way
                    .iter()
                    .any(|&k| ids[p + 1 - k..].starts_with(&marks.user));
                let finished = users.contains(&p);
                if got == want {
                    agree += 1;
                    marker_starts += usize::from(finished && before.is_some());
                } else if !under_way.is_empty() && !finishes && want.is_some() && got.is_none() {
                    broken += 1;
                } else {
                    disagree += 1;
                    first_disagreement.get_or_insert((index, p));
                }
                before = got;
            }
        }
        failed |= disagree > 0;
        let line = serde_json::json!({
            "file": file,
            "prompts": samples.len(),
            "positions": positions,
            "with_rows": with_rows,
            "agree": agree,
            "marker_starts": marker_starts,
            "broken_beginnings": broken,
            "disagree": disagree,
            "first_disagreement": first_disagreement,
        });
        println!("{}", serde_json::to_string(&line)?);
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
