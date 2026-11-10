//! Matched recall pairs over two scored panels of one model.
//!
//! Reads the id samples that two `train-in-space --score-rows` passes wrote for
//! one checkpoint, one on the panel with the memory line and one on the panel
//! without it, and counts exact replies over the replies both panels hold.
//!
//! ```text
//! recall-pairs tokenizer=TOKENIZER.json line=SAMPLES.json noline=SAMPLES.json \
//!   [control=SAMPLES.json] [eos=1] [none=" none.|OTHER"]
//! ```
//!
//! A recall reply is a line-panel sample whose prompt ends in the assistant
//! marker with a memory line directly before it, where the line names a value
//! and not `none` (`none=` gives the texts of lines that name nothing, each as it
//! follows `Memory:`, separated by `|`; a panel whose lines are replies passes its abstaining
//! replies). Its partner is the no-line sample with the same reference
//! and the same prompt once every memory line is taken out. A recall reply
//! whose partner sits in another window is counted and left out.
//!
//! `control` is another model's pass on the no-line panel (the twin trained
//! without the line). It is matched to the partner by prompt and reference.
//!
//! A reply is exact when its ids up to and including the end marker equal the
//! reference's, and the reference has the end marker. Prints one JSON object:
//! the counts, `E_line`, `E_noline`, `D = E_line - E_noline`, `E_control` and
//! `G = E_line - E_control`, `E_line_recall` (exact over every recall reply of
//! the line panel, paired or not), and the marker ids read from the tokenizer.

use std::collections::HashMap;
use std::fs;
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use uor_r4_tokenizer::ByteBpeTokenizer;

#[derive(Debug, Clone, Deserialize)]
struct Sample {
    prompt: Vec<u32>,
    reply: Vec<u32>,
    reference: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
struct Markers {
    /// `\nSystem: Memory:` as ids; the first id is the turn separator.
    line: Vec<u32>,
    /// `\nAssistant:` as ids; starts with the same turn separator.
    assistant: Vec<u32>,
    /// The value ids of each line that names nothing (` none.` unless `none=` gives others).
    none: Vec<Vec<u32>>,
    eos: u32,
}

impl Markers {
    fn separator(&self) -> u32 {
        self.line[0]
    }

    fn check(&self) -> Result<(), String> {
        if self.line.len() < 2
            || self.assistant.len() < 2
            || self.none.is_empty()
            || self.none.iter().any(Vec::is_empty)
        {
            return Err("a marker is empty".to_string());
        }
        if self.line[0] != self.assistant[0] {
            return Err("the line and assistant markers do not share a turn separator".to_string());
        }
        if self
            .none
            .iter()
            .any(|none| none.contains(&self.separator()))
        {
            return Err("the none value holds the turn separator".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
struct Counts {
    line_samples: usize,
    noline_samples: usize,
    control_samples: usize,
    line_exact_all: usize,
    noline_exact_all: usize,
    line_complete_all: usize,
    noline_complete_all: usize,
    closing_lines: usize,
    none_lines: usize,
    recall_replies: usize,
    recall_line_exact: usize,
    matched_pairs: usize,
    unmatched: usize,
    unmatched_with_other_window: usize,
    pair_line_exact: usize,
    pair_noline_exact: usize,
    pair_line_complete: usize,
    pair_noline_complete: usize,
    control_pairs: usize,
    pair_control_exact: usize,
    pair_reference_names_value: usize,
    pair_line_names_value: usize,
    pair_noline_names_value: usize,
    pair_control_names_value: usize,
    pair_reference_names_value_closed: usize,
    pair_line_names_value_closed: usize,
    pair_noline_names_value_closed: usize,
    pair_control_names_value_closed: usize,
    /// Samples of the no-line pass whose prompt and reference repeat an
    /// earlier sample's; only the first of each can be a partner.
    noline_repeated_keys: usize,
    /// Matched pairs whose partner an earlier pair already took.
    pairs_on_a_taken_partner: usize,
}

/// The value's ids without the id that closes the line, when it has more than one.
fn value_core(value: &[u32]) -> &[u32] {
    if value.len() > 1 {
        &value[..value.len() - 1]
    } else {
        value
    }
}

/// Whether `ids`, up to its end marker, holds the line's value as one run.
/// The value's last id (the full stop that closes the line) is left out when
/// the value has more than one id, so "is Maya." and "Maya, right?" both count.
/// This is a loose count: the run may be the start of a longer word, and a
/// reply that lists several values counts for each. [`names_value_closed`] is
/// the strict one.
fn names_value(ids: &[u32], value: &[u32], eos: u32) -> bool {
    let core = value_core(value);
    !core.is_empty() && cut(ids, eos).windows(core.len()).any(|run| run == core)
}

/// Whether `ids`, up to its end marker, holds the line's value as one run
/// that then ends: the next id is the id that closes the line, or the end
/// marker. So "is Maya." counts, and neither "Maya, right?" nor a longer word
/// that starts with the value's ids does. A one-id value has no closing id
/// and must be followed by the end marker.
fn names_value_closed(ids: &[u32], value: &[u32], eos: u32) -> bool {
    let core = value_core(value);
    let close = (value.len() > 1).then(|| value[value.len() - 1]);
    let ids = cut(ids, eos);
    !core.is_empty()
        && (0..ids.len().saturating_sub(core.len())).any(|at| {
            let next = ids[at + core.len()];
            ids[at..at + core.len()] == *core && (next == eos || Some(next) == close)
        })
}

/// The ids up to and including the first end marker, or all of them.
fn cut(ids: &[u32], eos: u32) -> &[u32] {
    match ids.iter().position(|&id| id == eos) {
        Some(at) => &ids[..=at],
        None => ids,
    }
}

fn exact(sample: &Sample, eos: u32) -> bool {
    let reference = cut(&sample.reference, eos);
    reference.last() == Some(&eos) && cut(&sample.reply, eos) == reference
}

fn complete(sample: &Sample, eos: u32) -> bool {
    sample.reply.contains(&eos)
}

fn find_last(haystack: &[u32], needle: &[u32]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len())
        .rev()
        .find(|&at| haystack[at..at + needle.len()] == *needle)
}

/// The value ids of the memory line directly before the prompt's closing
/// assistant marker, if the prompt ends that way.
fn closing_line_value<'a>(prompt: &'a [u32], markers: &Markers) -> Option<&'a [u32]> {
    let body = prompt.strip_suffix(markers.assistant.as_slice())?;
    let at = find_last(body, &markers.line)?;
    let value = &body[at + markers.line.len()..];
    (!value.is_empty() && !value.contains(&markers.separator())).then_some(value)
}

/// The prompt with every memory line removed: each marker and the ids after
/// it, up to the next turn separator.
fn strip_lines(prompt: &[u32], markers: &Markers) -> Vec<u32> {
    let mut out = Vec::with_capacity(prompt.len());
    let mut at = 0;
    while at < prompt.len() {
        if prompt[at..].starts_with(&markers.line) {
            at += markers.line.len();
            while at < prompt.len() && prompt[at] != markers.separator() {
                at += 1;
            }
        } else {
            out.push(prompt[at]);
            at += 1;
        }
    }
    out
}

type Key = (Vec<u32>, Vec<u32>);

fn key(prompt: &[u32], reference: &[u32], eos: u32) -> Key {
    (prompt.to_vec(), cut(reference, eos).to_vec())
}

fn index(samples: &[Sample], eos: u32) -> HashMap<Key, usize> {
    let mut map = HashMap::new();
    for (at, sample) in samples.iter().enumerate() {
        map.entry(key(&sample.prompt, &sample.reference, eos))
            .or_insert(at);
    }
    map
}

fn count(
    line: &[Sample],
    noline: &[Sample],
    control: Option<&[Sample]>,
    markers: &Markers,
) -> Counts {
    let eos = markers.eos;
    let by_key = index(noline, eos);
    let control_by_key = control.map(|samples| index(samples, eos));
    let mut by_reference: HashMap<Vec<u32>, Vec<usize>> = HashMap::new();
    for (at, sample) in noline.iter().enumerate() {
        by_reference
            .entry(cut(&sample.reference, eos).to_vec())
            .or_default()
            .push(at);
    }

    let mut counts = Counts {
        line_samples: line.len(),
        noline_samples: noline.len(),
        control_samples: control.map_or(0, <[Sample]>::len),
        line_exact_all: line.iter().filter(|s| exact(s, eos)).count(),
        noline_exact_all: noline.iter().filter(|s| exact(s, eos)).count(),
        line_complete_all: line.iter().filter(|s| complete(s, eos)).count(),
        noline_complete_all: noline.iter().filter(|s| complete(s, eos)).count(),
        noline_repeated_keys: noline.len() - by_key.len(),
        ..Counts::default()
    };
    let mut taken = std::collections::HashSet::new();

    for sample in line {
        let Some(value) = closing_line_value(&sample.prompt, markers) else {
            continue;
        };
        counts.closing_lines += 1;
        if markers.none.iter().any(|none| value == none.as_slice()) {
            counts.none_lines += 1;
            continue;
        }
        counts.recall_replies += 1;
        counts.recall_line_exact += usize::from(exact(sample, eos));
        let stripped = strip_lines(&sample.prompt, markers);
        let wanted = key(&stripped, &sample.reference, eos);
        let Some(&partner) = by_key.get(&wanted) else {
            counts.unmatched += 1;
            let other_window = by_reference.get(&wanted.1).is_some_and(|list| {
                list.iter().any(|&at| {
                    let prompt = &noline[at].prompt;
                    stripped.ends_with(prompt) || prompt.ends_with(&stripped)
                })
            });
            counts.unmatched_with_other_window += usize::from(other_window);
            continue;
        };
        counts.pairs_on_a_taken_partner += usize::from(!taken.insert(partner));
        let partner = &noline[partner];
        counts.matched_pairs += 1;
        counts.pair_line_exact += usize::from(exact(sample, eos));
        counts.pair_noline_exact += usize::from(exact(partner, eos));
        counts.pair_line_complete += usize::from(complete(sample, eos));
        counts.pair_noline_complete += usize::from(complete(partner, eos));
        counts.pair_reference_names_value +=
            usize::from(names_value(&sample.reference, value, eos));
        counts.pair_line_names_value += usize::from(names_value(&sample.reply, value, eos));
        counts.pair_noline_names_value += usize::from(names_value(&partner.reply, value, eos));
        counts.pair_reference_names_value_closed +=
            usize::from(names_value_closed(&sample.reference, value, eos));
        counts.pair_line_names_value_closed +=
            usize::from(names_value_closed(&sample.reply, value, eos));
        counts.pair_noline_names_value_closed +=
            usize::from(names_value_closed(&partner.reply, value, eos));
        if let (Some(samples), Some(map)) = (control, control_by_key.as_ref()) {
            if let Some(&at) = map.get(&key(&partner.prompt, &partner.reference, eos)) {
                counts.control_pairs += 1;
                counts.pair_control_exact += usize::from(exact(&samples[at], eos));
                counts.pair_control_names_value +=
                    usize::from(names_value(&samples[at].reply, value, eos));
                counts.pair_control_names_value_closed +=
                    usize::from(names_value_closed(&samples[at].reply, value, eos));
            }
        }
    }
    counts
}

/// The arm's share minus the control's, only when the control pass holds a
/// partner for every matched pair. Without that the two shares are taken over
/// different replies and their difference is not the reading.
fn against_control(covered: bool, arm: Option<f64>, control: Option<f64>) -> Option<f64> {
    if covered {
        Some(arm? - control?)
    } else {
        None
    }
}

/// `part / whole`, or nothing when `whole` is zero.
fn share(part: usize, whole: usize) -> Option<f64> {
    let part = f64::from(u32::try_from(part).ok()?);
    let whole = f64::from(u32::try_from(whole).ok()?);
    (whole > 0.0).then(|| part / whole)
}

fn read_samples(path: &str) -> Result<Vec<Sample>, String> {
    let bytes = fs::read(path).map_err(|error| format!("{path}: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{path}: {error}"))
}

fn markers_from(
    tokenizer: &ByteBpeTokenizer,
    eos: u32,
    none_texts: &[&str],
) -> Result<Markers, String> {
    let head = tokenizer.encode(" Memory:");
    let mut none = Vec::new();
    for text in none_texts {
        let with_none = tokenizer.encode(&format!(" Memory:{text}"));
        let ids = with_none
            .strip_prefix(head.as_slice())
            .ok_or("a none line does not start with the memory marker's ids")?;
        none.push(ids.to_vec());
    }
    let markers = Markers {
        line: [tokenizer.encode("\nSystem:"), head].concat(),
        assistant: tokenizer.encode("\nAssistant:"),
        none,
        eos,
    };
    markers.check()?;
    Ok(markers)
}

fn run(arguments: &[String]) -> Result<String, String> {
    let mut named: HashMap<&str, &str> = HashMap::new();
    for argument in arguments {
        let (name, value) = argument
            .split_once('=')
            .ok_or_else(|| format!("argument is not name=value: {argument}"))?;
        if !["tokenizer", "line", "noline", "control", "eos", "none"].contains(&name) {
            return Err(format!("unknown argument: {name}"));
        }
        if named.insert(name, value).is_some() {
            return Err(format!("argument given twice: {name}"));
        }
    }
    let need = |name: &str| {
        named
            .get(name)
            .copied()
            .ok_or_else(|| format!("missing argument: {name}="))
    };
    let eos = match named.get("eos") {
        Some(text) => text
            .parse::<u32>()
            .map_err(|error| format!("eos: {error}"))?,
        None => 1,
    };
    let tokenizer_path = need("tokenizer")?;
    let bytes = fs::read(tokenizer_path).map_err(|error| format!("{tokenizer_path}: {error}"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| format!("{tokenizer_path}: not a readable tokenizer.json"))?;
    let none_texts: Vec<&str> = named
        .get("none")
        .map_or_else(|| vec![" none."], |texts| texts.split('|').collect());
    let markers = markers_from(&tokenizer, eos, &none_texts)?;

    let line = read_samples(need("line")?)?;
    let noline = read_samples(need("noline")?)?;
    let control = named
        .get("control")
        .map(|path| read_samples(path))
        .transpose()?;
    if !noline.iter().any(|sample| sample.reference.contains(&eos)) {
        return Err(format!(
            "no reference of the no-line pass holds the end marker {eos}: is eos= right?"
        ));
    }
    let counts = count(&line, &noline, control.as_deref(), &markers);
    let covered = control.is_some() && counts.control_pairs == counts.matched_pairs;
    let closed_line = share(counts.pair_line_names_value_closed, counts.matched_pairs);
    let closed_noline = share(counts.pair_noline_names_value_closed, counts.matched_pairs);
    let closed_control = share(counts.pair_control_names_value_closed, counts.control_pairs);

    let e_line = share(counts.pair_line_exact, counts.matched_pairs);
    let e_noline = share(counts.pair_noline_exact, counts.matched_pairs);
    let e_control = share(counts.pair_control_exact, counts.control_pairs);
    let v_line = share(counts.pair_line_names_value, counts.matched_pairs);
    let v_noline = share(counts.pair_noline_names_value, counts.matched_pairs);
    let v_control = share(counts.pair_control_names_value, counts.control_pairs);
    let minus = |left: Option<f64>, right: Option<f64>| Some(left? - right?);
    let report = serde_json::json!({
        "counts": counts,
        "E_line": e_line,
        "E_noline": e_noline,
        "D": minus(e_line, e_noline),
        "E_line_recall": share(counts.recall_line_exact, counts.recall_replies),
        "E_control": e_control,
        "G": against_control(covered, e_line, e_control),
        "V_reference": share(counts.pair_reference_names_value, counts.matched_pairs),
        "V_line": v_line,
        "V_noline": v_noline,
        "V_D": minus(v_line, v_noline),
        "V_control": v_control,
        "V_G": against_control(covered, v_line, v_control),
        "V_closed_reference": share(counts.pair_reference_names_value_closed, counts.matched_pairs),
        "V_closed_line": closed_line,
        "V_closed_noline": closed_noline,
        "V_closed_D": minus(closed_line, closed_noline),
        "V_closed_control": closed_control,
        "V_closed_G": against_control(covered, closed_line, closed_control),
        "control_covers_every_pair": covered,
        "markers": markers,
    });
    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("recall-pairs: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn markers() -> Markers {
        Markers {
            line: vec![9, 5, 6],
            assistant: vec![9, 7],
            none: vec![vec![8, 3]],
            eos: 1,
        }
    }

    fn sample(prompt: &[u32], reply: &[u32], reference: &[u32]) -> Sample {
        Sample {
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
            reference: reference.to_vec(),
        }
    }

    #[test]
    fn exact_needs_the_end_marker_and_ignores_ids_after_it() {
        assert!(exact(&sample(&[], &[30, 1, 44], &[30, 1]), 1));
        assert!(!exact(&sample(&[], &[30], &[30]), 1));
        assert!(!exact(&sample(&[], &[30, 31, 1], &[30, 1]), 1));
        assert!(!complete(&sample(&[], &[30], &[30, 1]), 1));
    }

    #[test]
    fn a_line_counts_only_directly_before_the_closing_assistant_marker() {
        let m = markers();
        assert_eq!(
            closing_line_value(&[0, 20, 9, 5, 6, 30, 3, 9, 7], &m),
            Some(&[30, 3][..])
        );
        // An earlier line, then a later user turn with no line of its own.
        assert_eq!(
            closing_line_value(&[0, 28, 9, 5, 6, 35, 3, 9, 7, 62, 1, 9, 29, 9, 7], &m),
            None
        );
        // A prompt that does not end in the assistant marker.
        assert_eq!(closing_line_value(&[0, 20, 9, 5, 6, 30, 3], &m), None);
    }

    #[test]
    fn strip_lines_removes_every_line_and_nothing_else() {
        let m = markers();
        let prompt = [
            0, 26, 9, 5, 6, 33, 3, 9, 7, 61, 1, 9, 27, 9, 5, 6, 34, 3, 9, 7,
        ];
        assert_eq!(
            strip_lines(&prompt, &m),
            vec![0, 26, 9, 7, 61, 1, 9, 27, 9, 7]
        );
        assert_eq!(strip_lines(&[0, 25, 9, 7], &m), vec![0, 25, 9, 7]);
    }

    #[test]
    fn every_none_text_marks_a_line_that_names_nothing() {
        let mut m = markers();
        m.none.push(vec![41, 42, 3]);
        m.check().expect("two none texts");
        let line = [
            sample(&[0, 22, 9, 5, 6, 8, 3, 9, 7], &[40, 1], &[40, 1]),
            sample(
                &[0, 23, 9, 5, 6, 41, 42, 3, 9, 7],
                &[41, 42, 3, 1],
                &[41, 42, 3, 1],
            ),
            sample(&[0, 24, 9, 5, 6, 30, 3, 9, 7], &[30, 1], &[30, 1]),
        ];
        let counts = count(&line, &[], None, &m);
        assert_eq!(
            (
                counts.closing_lines,
                counts.none_lines,
                counts.recall_replies
            ),
            (3, 2, 1)
        );
        assert_eq!(counts.recall_line_exact, 1);
        m.none.push(Vec::new());
        assert!(m.check().is_err(), "an empty none text is refused");
    }

    #[test]
    fn counts_a_hand_made_panel() {
        let m = markers();
        let line = [
            // A: recall, partner present, right with the line.
            sample(&[0, 20, 21, 9, 5, 6, 30, 3, 9, 7], &[30, 1, 44], &[30, 1]),
            // B: the line names nothing.
            sample(&[0, 22, 9, 5, 6, 8, 3, 9, 7], &[40, 1], &[40, 1]),
            // C: recall, partner only in another window.
            sample(&[0, 23, 24, 9, 5, 6, 32, 3, 9, 7], &[99, 1], &[32, 1]),
            // D: no line.
            sample(&[0, 25, 9, 7], &[50, 1], &[50, 1]),
            // E: recall with an earlier line in the same prompt.
            sample(
                &[
                    0, 26, 9, 5, 6, 33, 3, 9, 7, 61, 1, 9, 27, 9, 5, 6, 34, 3, 9, 7,
                ],
                &[34, 1],
                &[34, 1],
            ),
            // F: a line that is not directly before the closing marker; no end marker.
            sample(
                &[0, 28, 9, 5, 6, 35, 3, 9, 7, 62, 1, 9, 29, 9, 7],
                &[63],
                &[63, 1],
            ),
        ];
        let noline = [
            sample(&[0, 20, 21, 9, 7], &[31, 1], &[30, 1]),
            sample(&[24, 9, 7], &[32, 1], &[32, 1]),
            sample(&[0, 26, 9, 7, 61, 1, 9, 27, 9, 7], &[34, 1], &[34, 1]),
        ];
        let control = [sample(&[0, 20, 21, 9, 7], &[30, 1], &[30, 1])];

        let counts = count(&line, &noline, Some(&control), &m);
        assert_eq!(
            counts,
            Counts {
                line_samples: 6,
                noline_samples: 3,
                control_samples: 1,
                line_exact_all: 4,
                noline_exact_all: 2,
                line_complete_all: 5,
                noline_complete_all: 3,
                closing_lines: 4,
                none_lines: 1,
                recall_replies: 3,
                recall_line_exact: 2,
                matched_pairs: 2,
                unmatched: 1,
                unmatched_with_other_window: 1,
                pair_line_exact: 2,
                pair_noline_exact: 1,
                pair_line_complete: 2,
                pair_noline_complete: 2,
                control_pairs: 1,
                pair_control_exact: 1,
                pair_reference_names_value: 2,
                pair_line_names_value: 2,
                pair_noline_names_value: 1,
                pair_control_names_value: 1,
                pair_reference_names_value_closed: 2,
                pair_line_names_value_closed: 2,
                pair_noline_names_value_closed: 1,
                pair_control_names_value_closed: 1,
                noline_repeated_keys: 0,
                pairs_on_a_taken_partner: 0,
            }
        );
        assert_eq!(
            share(counts.pair_line_exact, counts.matched_pairs),
            Some(1.0)
        );
        assert_eq!(
            share(counts.pair_noline_exact, counts.matched_pairs),
            Some(0.5)
        );
        assert_eq!(share(0, 0), None);
    }

    #[test]
    fn a_reply_names_the_value_in_any_wording() {
        // The line's value is [30, 31, 3]; its last id closes the line.
        let value = [30, 31, 3];
        assert!(names_value(&[70, 30, 31, 3, 1], &value, 1));
        assert!(names_value(&[71, 72, 30, 31, 73, 1], &value, 1));
        assert!(!names_value(&[30, 72, 31, 1], &value, 1));
        // Ids after the end marker do not count.
        assert!(!names_value(&[70, 1, 30, 31], &value, 1));
        // A one-id value is looked for whole.
        assert!(names_value(&[70, 40, 1], &[40], 1));
        assert!(!names_value(&[70, 1], &[], 1));
    }

    #[test]
    fn a_closed_value_is_not_the_start_of_a_longer_word() {
        // The line's value is [30, 3]: one id and the id that closes the line.
        let value = [30, 3];
        // The value, then another id: a longer word. Loose yes, closed no.
        assert!(names_value(&[70, 30, 45, 1], &value, 1));
        assert!(!names_value_closed(&[70, 30, 45, 1], &value, 1));
        // The value, then the closing id or the end marker.
        assert!(names_value_closed(&[70, 30, 3, 1], &value, 1));
        assert!(names_value_closed(&[70, 30, 1], &value, 1));
        // A decode cut right after the value has not closed it.
        assert!(!names_value_closed(&[70, 30], &value, 1));
        // Ids after the end marker do not count.
        assert!(!names_value_closed(&[70, 1, 30, 3], &value, 1));
        // A one-id value has no closing id: only the end marker closes it.
        assert!(names_value_closed(&[70, 40, 1], &[40], 1));
        assert!(!names_value_closed(&[70, 40, 41, 1], &[40], 1));
        assert!(!names_value_closed(&[70, 1], &[], 1));
    }

    #[test]
    fn the_gap_to_the_control_is_withheld_without_full_cover() {
        assert_eq!(against_control(true, Some(0.5), Some(0.25)), Some(0.25));
        assert_eq!(against_control(false, Some(0.5), Some(0.25)), None);
        assert_eq!(against_control(true, Some(0.5), None), None);
    }

    #[test]
    fn a_repeated_window_is_counted() {
        let m = markers();
        let recall = sample(&[0, 20, 21, 9, 5, 6, 30, 3, 9, 7], &[30, 1], &[30, 1]);
        let partner = sample(&[0, 20, 21, 9, 7], &[31, 1], &[30, 1]);
        let counts = count(
            &[recall.clone(), recall],
            &[partner.clone(), partner],
            None,
            &m,
        );
        assert_eq!(
            (
                counts.matched_pairs,
                counts.noline_repeated_keys,
                counts.pairs_on_a_taken_partner
            ),
            (2, 1, 1)
        );
    }

    #[test]
    fn markers_must_share_the_turn_separator() {
        let mut m = markers();
        assert!(m.check().is_ok());
        m.assistant = vec![4, 7];
        assert!(m.check().is_err());
    }

    #[test]
    fn arguments_are_strict() {
        assert!(run(&["bogus=1".to_string()]).is_err());
        assert!(run(&["line".to_string()]).is_err());
        assert!(run(&["line=a".to_string(), "line=b".to_string()]).is_err());
        assert!(run(&["line=a".to_string(), "noline=b".to_string()]).is_err());
    }
}
