//! `whole-word-recall` (claude-352, ORDER decision 10): the place-read sheet's I-w score.
//!
//! Exact match calls a recall reply wrong when it says the stated value and then goes on. The
//! sheet's retake reads each reply as text instead: a reply is right when the stated value's
//! text stands in it as a whole word, case included, and no other value stated in the same
//! conversation stands in it the same way. So "470" is not right for "47".
//!
//! Where each text comes from:
//! - **The stated value** is the reference reply's value run: the corpus ids at the kit's value
//!   start, for its length, decoded and trimmed. The kit's writer finds that run from the twin's
//!   `Memory: {value}.` line as whole words, so it is the twin's text.
//! - **The other values** of a conversation are the value texts of the kit's other entries in the
//!   same document (a document starts at the BOS mark), leaving out any equal to the stated one.
//!   A value stated and never asked is in no line of the twin and is not in the kit.
//! - **A reply** is decoded from its ids with the BOS and end marks left out, wherever they
//!   stand. A reader that stops at the end mark writes none after it.
//!
//! It reads a kit (`panel-places.json`), the corpus file the kit was written from, the tokenizer,
//! the panel's rows (`position`, `reference`) and one reader's replies (`i`, `ids`). Before a row
//! is written it refuses a corpus that is not the kit's, a row that is not that corpus's, a row
//! without its reply, a recall reply given twice, and a set of rows that does not hold every
//! recall reply of the kit. It also scores the references: the corpus's own replies must all be
//! right, or the rule is refusing what the corpus says, and the program ends non-zero after
//! writing its rows.
//!
//! usage: whole-word-recall --tokenizer FILE --kit FILE --kit-tokens FILE --samples FILE
//!        --replies FILE --out FILE

use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

const USAGE: &str = "usage: whole-word-recall --tokenizer FILE --kit FILE --kit-tokens FILE \
--samples FILE --replies FILE --out FILE";

/// The names of the usage. Each is given once, with its value.
const NAMES: [&str; 6] = ["--tokenizer", "--kit", "--kit-tokens", "--samples", "--replies", "--out"];

/// A `UORT` corpus file: this many header bytes, then the ids, two little-endian bytes each.
const HEADER: usize = 64;

/// Whether `text` holds `value` as a whole word: an occurrence with no letter or digit directly
/// before it and none directly after it. The census's rule (`turn_census::holds`), read at every
/// start, so an occurrence that overlaps an earlier one is not passed over.
fn holds(text: &str, value: &str) -> bool {
    !value.is_empty()
        && text.char_indices().any(|(i, _)| {
            text[i..].starts_with(value) && {
                let before = text[..i].chars().next_back();
                let after = text[i + value.len()..].chars().next();
                !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
            }
        })
}

/// What one recall reply must say and must not say.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stated {
    value: String,
    others: Vec<String>,
}

/// One text read by the rule.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Judged {
    text: String,
    holds_value: bool,
    holds_other: bool,
}

impl Judged {
    fn right(&self) -> bool {
        self.holds_value && !self.holds_other
    }
}

/// Decodes `ids` with the marks left out and reads the text by the rule.
fn judge(ids: &[u32], stated: &Stated, marks: [u32; 2], decode: &dyn Fn(&[u32]) -> String) -> Judged {
    let kept: Vec<u32> = ids.iter().copied().filter(|id| !marks.contains(id)).collect();
    let text = decode(&kept);
    let holds_value = holds(&text, &stated.value);
    let holds_other = stated.others.iter().any(|other| holds(&text, other));
    Judged { text, holds_value, holds_other }
}

/// The ids of a `UORT` corpus file.
fn corpus_ids(bytes: &[u8], path: &str) -> Result<Vec<u32>> {
    if bytes.len() < HEADER || !bytes.starts_with(b"UORT") || (bytes.len() - HEADER) % 2 != 0 {
        return Err(format!("{path}: not a UORT corpus file"));
    }
    Ok(bytes[HEADER..]
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect())
}

/// For each recall reply of the kit, by its start: the value it states and the other values
/// stated in its conversation. `places` are the kit's `[reply start, place, value start, value
/// length]` rows. A kit that names one reply twice is refused.
fn stated(
    corpus: &[u32],
    places: &[[usize; 4]],
    bos: u32,
    decode: &dyn Fn(&[u32]) -> String,
) -> Result<HashMap<usize, Stated>> {
    let mut document = 0usize;
    let document_of: Vec<usize> = corpus
        .iter()
        .map(|&id| {
            document += usize::from(id == bos);
            document
        })
        .collect();
    let mut by_document: HashMap<usize, Vec<(usize, String)>> = HashMap::new();
    for &[reply, _, start, len] in places {
        let run = start.checked_add(len).and_then(|end| corpus.get(start..end));
        let (Some(&doc), Some(run)) = (document_of.get(reply), run) else {
            return Err(format!("the kit's reply at {reply} is past the corpus"));
        };
        let value = decode(run).trim().to_owned();
        if value.is_empty() {
            return Err(format!("the kit's reply at {reply} states no text"));
        }
        by_document.entry(doc).or_default().push((reply, value));
    }
    let mut out = HashMap::new();
    for entries in by_document.values() {
        for (reply, value) in entries {
            let mut others: Vec<String> =
                entries.iter().map(|(_, v)| v.clone()).filter(|v| v != value).collect();
            others.sort();
            others.dedup();
            if out.insert(*reply, Stated { value: value.clone(), others }).is_some() {
                return Err(format!("the kit names the reply at {reply} twice"));
            }
        }
    }
    Ok(out)
}

fn ids_of(value: &Value, what: &str) -> Result<Vec<u32>> {
    value
        .as_array()
        .ok_or_else(|| format!("{what}: not a list of ids"))?
        .iter()
        .map(|id| id.as_u64().and_then(|id| u32::try_from(id).ok()))
        .collect::<Option<Vec<u32>>>()
        .ok_or_else(|| format!("{what}: not a list of ids"))
}

fn index_of(value: &Value, what: &str) -> Result<usize> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("{what}: not a whole number"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn read(path: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{path}: {e}"))
}

/// Every argument is one of the usage's names followed by its value, each name once: six pairs
/// that hold all six names hold no other name. A name followed by another name has no value
/// and is refused.
fn checked_args(args: &[String]) -> Result<()> {
    let ok = args.len() == 2 * NAMES.len()
        && args
            .chunks(2)
            .all(|pair| !pair[1].starts_with("--"))
        && NAMES.iter().all(|name| args.chunks(2).any(|pair| pair[0] == *name));
    if ok {
        Ok(())
    } else {
        Err(format!("every argument is a `--name value` pair, each name of the usage once\n{USAGE}"))
    }
}

fn flag(args: &[String], name: &str) -> Result<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
        .ok_or_else(|| format!("{name} is needed\n{USAGE}"))
}

/// One reader's replies read by the rule: the rows to write and the counts.
#[derive(Debug, Default)]
struct Scored {
    rows: Vec<Value>,
    recall: usize,
    right: usize,
    value_held: usize,
    other_held: usize,
    exact: usize,
    exact_not_right: usize,
    references_right: usize,
}

/// Reads one reader's replies by the rule. Every binding is checked here, before a row is
/// written: the kit is this corpus's, each row is this corpus's and has its reply, no recall
/// reply is scored twice, and every recall reply of the kit is scored.
fn score(
    kit: &Value,
    tokens_bytes: &[u8],
    samples: &[Value],
    replies_text: &str,
    decode: &dyn Fn(&[u32]) -> String,
) -> Result<Scored> {
    if kit["tokens_sha256"].as_str() != Some(sha256_hex(tokens_bytes).as_str()) {
        return Err("the kit was written from another corpus".to_owned());
    }
    let corpus = corpus_ids(tokens_bytes, "the corpus")?;
    let bos = u32::try_from(index_of(&kit["marks"]["bos"], "the kit's BOS mark")?)
        .map_err(|_| "the kit's BOS mark is past 32 bits".to_owned())?;
    let eos = u32::try_from(index_of(&kit["marks"]["eos"], "the kit's end mark")?)
        .map_err(|_| "the kit's end mark is past 32 bits".to_owned())?;
    let mut places = Vec::new();
    for row in kit["places"].as_array().ok_or("the kit has no places")? {
        let cells = ids_of(row, "a place of the kit")?;
        let [reply, place, start, len] = cells[..] else {
            return Err("a place of the kit is not four numbers".to_owned());
        };
        places.push([reply as usize, place as usize, start as usize, len as usize]);
    }
    let by_reply = stated(&corpus, &places, bos, decode)?;

    let mut replies: HashMap<usize, Vec<u32>> = HashMap::new();
    for line in replies_text.lines().filter(|l| !l.trim().is_empty()) {
        let row: Value = serde_json::from_str(line).map_err(|e| format!("a reply row: {e}"))?;
        let i = index_of(&row["i"], "a reply's row number")?;
        if replies.insert(i, ids_of(&row["ids"], "a reply's ids")?).is_some() {
            return Err(format!("reply row {i} is there twice"));
        }
    }
    if replies.len() != samples.len() || !(0..samples.len()).all(|i| replies.contains_key(&i)) {
        return Err(format!(
            "{} replies for {} rows: each row has one reply, by its number",
            replies.len(),
            samples.len()
        ));
    }

    let mut scored = Scored::default();
    let mut seen = HashSet::new();
    for (i, sample) in samples.iter().enumerate() {
        let position = index_of(&sample["position"], "a row's position")?;
        let reference = ids_of(&sample["reference"], "a row's reference")?;
        let run = position.checked_add(reference.len()).and_then(|end| corpus.get(position..end));
        if reference.is_empty() || run != Some(&reference[..]) {
            return Err(format!("row {i} is not this corpus's"));
        }
        let Some(want) = by_reply.get(&position) else {
            continue;
        };
        if !seen.insert(position) {
            return Err(format!("row {i} repeats the recall reply at {position}"));
        }
        let ids = &replies[&i];
        let reply = judge(ids, want, [bos, eos], decode);
        let corpus_says = judge(&reference, want, [bos, eos], decode);
        let same = *ids == reference;
        scored.recall += 1;
        scored.right += usize::from(reply.right());
        scored.value_held += usize::from(reply.holds_value);
        scored.other_held += usize::from(reply.holds_value && reply.holds_other);
        scored.exact += usize::from(same);
        scored.exact_not_right += usize::from(same && !reply.right());
        scored.references_right += usize::from(corpus_says.right());
        scored.rows.push(json!({
            "i": i,
            "position": position,
            "value": want.value,
            "others": want.others,
            "reply": reply.text,
            "holds_value": reply.holds_value,
            "holds_other": reply.holds_other,
            "right": reply.right(),
            "exact": same,
            "reference": corpus_says.text,
            "reference_right": corpus_says.right(),
        }));
    }
    if seen.len() != by_reply.len() {
        return Err(format!(
            "{} of the kit's {} recall replies are among the rows",
            seen.len(),
            by_reply.len()
        ));
    }
    Ok(scored)
}

/// The corpus's own replies must all be right by the rule.
fn references_hold(scored: &Scored) -> Result<()> {
    if scored.references_right == scored.recall {
        return Ok(());
    }
    Err(format!(
        "{} of {} references are right by the rule: the rule refuses what the corpus says",
        scored.references_right, scored.recall
    ))
}

/// Writes the rows, one a line, to a file that must not exist yet.
fn write_rows(path: &str, rows: &[Value]) -> Result<()> {
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{path}: {e}"))?;
    for row in rows {
        writeln!(out, "{row}").map_err(|e| format!("{path}: {e}"))?;
    }
    Ok(())
}

/// Scores one reader's replies. The rows written and the counts returned are the reading; the
/// error after them, if any, says the references themselves are not all right by the rule.
fn run_with_args(args: &[String]) -> Result<Value> {
    checked_args(args)?;
    let paths = ["--tokenizer", "--kit", "--kit-tokens", "--samples", "--replies"]
        .map(|name| flag(args, name));
    let [tokenizer_path, kit_path, tokens_path, samples_path, replies_path] = paths;
    let (tokenizer_path, kit_path, tokens_path) = (tokenizer_path?, kit_path?, tokens_path?);
    let (samples_path, replies_path, out_path) = (samples_path?, replies_path?, flag(args, "--out")?);

    let tokenizer_bytes = read(&tokenizer_path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| format!("{tokenizer_path}: not a tokenizer this crate reads"))?;
    let decode = |ids: &[u32]| tokenizer.decode(ids);

    let kit: Value = serde_json::from_slice(&read(&kit_path)?).map_err(|e| format!("{kit_path}: {e}"))?;
    let tokens_bytes = read(&tokens_path)?;
    let samples_bytes = read(&samples_path)?;
    let samples: Vec<Value> =
        serde_json::from_slice(&samples_bytes).map_err(|e| format!("{samples_path}: {e}"))?;
    let replies_bytes = read(&replies_path)?;
    let scored = score(&kit, &tokens_bytes, &samples, &String::from_utf8_lossy(&replies_bytes), &decode)
        .map_err(|e| format!("{e} ({kit_path}, {tokens_path}, {samples_path}, {replies_path})"))?;

    write_rows(&out_path, &scored.rows)?;
    receipt(
        &scored,
        &[
            ("kit_tokens_sha256", tokens_bytes.as_slice()),
            ("tokenizer_sha256", tokenizer_bytes.as_slice()),
            ("samples_sha256", samples_bytes.as_slice()),
            ("replies_sha256", replies_bytes.as_slice()),
        ],
    )
}

/// The reading's receipt: its counts and each input's hash under its name. When the corpus's
/// own replies are not all right by the rule, the receipt comes back inside the error.
fn receipt(scored: &Scored, inputs: &[(&str, &[u8])]) -> Result<Value> {
    let mut receipt = json!({
        "recall": scored.recall,
        "right": scored.right,
        "holds_the_value": scored.value_held,
        "holds_the_value_and_another": scored.other_held,
        "exact": scored.exact,
        "exact_and_not_right": scored.exact_not_right,
        "references_right": scored.references_right,
    });
    for (name, bytes) in inputs {
        receipt[*name] = json!(sha256_hex(bytes));
    }
    references_hold(scored).map_err(|e| format!("{receipt}\n{e}"))?;
    Ok(receipt)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run_with_args(&args) {
        Ok(receipt) => println!("{receipt}"),
        Err(message) => {
            eprintln!("whole-word-recall: {message}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 13] = [
        "<bos>", "<eos>", " It", " is", " 47", ".", " Lisbon", " 4", "0", " x", "47", " in", "y",
    ];

    fn decode(ids: &[u32]) -> String {
        ids.iter().map(|&id| WORDS[id as usize]).collect()
    }

    fn ids(words: &[&str]) -> Vec<u32> {
        words
            .iter()
            .map(|word| WORDS.iter().position(|w| w == word).expect("a word of the test") as u32)
            .collect()
    }

    /// Two conversations. The first states 47 and Lisbon and is asked for each; the second
    /// states 4 and is asked for it.
    fn corpus() -> (Vec<u32>, Vec<[usize; 4]>) {
        let words = [
            "<bos>", " 47", " Lisbon", " It", " is", " 47", ".", "<eos>", " It", " is", " Lisbon",
            ".", "<eos>", "<bos>", " 4", " It", " is", " 4", ".", "<eos>",
        ];
        (ids(&words), vec![[3, 1, 5, 1], [8, 2, 10, 1], [15, 14, 17, 1]])
    }

    /// The corpus as its file's bytes, and a kit of `places` written from it.
    fn file_and_kit(tokens: &[u32], places: &[[usize; 4]]) -> (Vec<u8>, Value) {
        let mut bytes = b"UORT".to_vec();
        bytes.resize(HEADER, 0);
        for &id in tokens {
            bytes.extend_from_slice(&(id as u16).to_le_bytes());
        }
        let kit = json!({
            "marks": {"bos": 0, "eos": 1},
            "places": places,
            "tokens_sha256": sha256_hex(&bytes),
        });
        (bytes, kit)
    }

    /// The panel's rows at `positions` (five ids of reference each) and one reply a row.
    fn rows_and_replies(tokens: &[u32], positions: &[usize], replies: &[&[&str]]) -> (Vec<Value>, String) {
        let rows = positions
            .iter()
            .map(|&p| json!({"position": p, "reference": tokens[p..p + 5]}))
            .collect();
        let text = replies
            .iter()
            .enumerate()
            .map(|(i, words)| json!({"i": i, "ids": ids(words)}).to_string() + "\n")
            .collect();
        (rows, text)
    }

    const REPLIES: [&[&str]; 3] = [
        &[" It", " is", " 47", ".", "<eos>"],
        &[" It", " is", " 47", " in", " Lisbon", "."],
        &[" It", " is", " 47"],
    ];

    #[test]
    fn the_values_are_the_kit_runs_and_the_others_are_the_conversations() {
        let (tokens, places) = corpus();
        let by_reply = stated(&tokens, &places, 0, &decode).expect("a kit of this corpus");
        let lisbon = vec!["Lisbon".to_owned()];
        assert_eq!(by_reply[&3], Stated { value: "47".to_owned(), others: lisbon });
        assert_eq!(by_reply[&8], Stated { value: "Lisbon".to_owned(), others: vec!["47".to_owned()] });
        // Another conversation's value is not one of this conversation's others.
        assert_eq!(by_reply[&15], Stated { value: "4".to_owned(), others: Vec::new() });
        assert!(stated(&tokens, &[[3, 1, 19, 4]], 0, &decode).is_err());
        // One reply named twice is refused, not read as the later entry.
        let twice = stated(&tokens, &[[3, 1, 5, 1], [3, 2, 2, 1]], 0, &decode);
        assert!(twice.is_err_and(|e| e.contains("twice")));
    }

    /// The sheet's four hand-built replies, each decoded from ids: one right, one that holds
    /// another value, one that holds half the value, one that holds the value inside a longer
    /// word or number.
    #[test]
    fn the_four_hand_built_replies() {
        let (tokens, places) = corpus();
        let by_reply = stated(&tokens, &places, 0, &decode).expect("a kit of this corpus");
        let want = &by_reply[&3];
        let read = |words: &[&str]| judge(&ids(words), want, [0, 1], &decode);

        let right = read(&[" It", " is", " 47", ".", "<eos>"]);
        assert_eq!(right.text, " It is 47.");
        assert!(right.right());
        // The value and then more text is still right: this is what exact match refuses.
        assert!(read(&[" It", " is", " 47", " in", " x", "<eos>"]).right());
        // A BOS mark in a reply is left out like the end mark.
        assert_eq!(read(&["<bos>", " It", " is", " 47", "."]).text, " It is 47.");

        let another = read(&[" It", " is", " 47", " in", " Lisbon", "."]);
        assert!(another.holds_value && another.holds_other && !another.right());

        let half = read(&[" It", " is", " 4", "."]);
        assert!(!half.holds_value && !half.right());

        let longer_number = read(&[" It", " is", " 47", "0", "."]);
        assert_eq!(longer_number.text, " It is 470.");
        assert!(!longer_number.right());
        assert!(!read(&[" It", " is", " x", "47", "."]).right());
        assert!(!read(&[" It", " is", " 47", "y"]).right());
    }

    #[test]
    fn a_word_is_matched_whole_and_by_case() {
        assert!(holds("You grew up in Lisbon", "Lisbon"));
        assert!(holds("Lisbon.", "Lisbon"));
        assert!(!holds("You grew up in Lisb", "Lisbon"));
        assert!(!holds("You grew up in lisbon", "Lisbon"));
        assert!(!holds("the Lisbonian", "Lisbon"));
        assert!(!holds("anything", ""));
        assert!(!holds(".", "") && !holds(" x", ""), "an empty value is no word, even between marks");
        // A whole-word occurrence that overlaps an earlier, not whole, one is still found.
        assert!(holds("ba a a", "a a"));
        // A letter of more than one byte beside the value is a letter; a dash is not.
        assert!(!holds("é47", "47"));
        assert!(!holds("47é", "47"));
        assert!(holds("—47—", "47"));
    }

    #[test]
    fn a_corpus_file_is_read_past_its_header() {
        let mut bytes = b"UORT".to_vec();
        bytes.resize(HEADER, 0);
        bytes.extend_from_slice(&[7, 0, 1, 1]);
        assert_eq!(corpus_ids(&bytes, "t").expect("a corpus file"), vec![7, 257]);
        assert!(corpus_ids(&bytes[..HEADER + 3], "t").is_err());
        assert!(corpus_ids(&bytes[4..], "t").is_err());
    }

    #[test]
    fn a_name_without_its_value_is_refused() {
        let args = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect::<Vec<String>>();
        let good = [
            "--tokenizer", "t", "--kit", "k", "--kit-tokens", "c", "--samples", "s", "--replies", "r",
            "--out", "o",
        ];
        assert!(checked_args(&args(&good)).is_ok());
        assert!(run_with_args(&args(&["--kit"])).is_err_and(|e| e.contains("pair")));
        // Two names in a row: `--kit` would take `--kit-tokens` as its value.
        let mut two = good;
        two[3] = "--kit-tokens";
        assert!(checked_args(&args(&two)).is_err());
        // A name the usage does not have, a name given twice, and a name left out.
        let mut unknown = good;
        unknown[0] = "--tokeniser";
        assert!(checked_args(&args(&unknown)).is_err());
        let mut repeated = good;
        repeated[10] = "--kit";
        assert!(checked_args(&args(&repeated)).is_err());
        assert!(checked_args(&args(&good[..10])).is_err());
        // A seventh pair beside the six.
        let mut seven = good.to_vec();
        seven.extend(["--extra", "x"]);
        assert!(checked_args(&args(&seven)).is_err());
    }

    /// The whole reading on a small panel, then each binding broken in turn. Nothing is
    /// written here: `score` returns the rows, and each refusal comes before any row.
    #[test]
    fn the_bindings_are_checked_before_a_row_is_written() {
        let (tokens, places) = corpus();
        let (bytes, kit) = file_and_kit(&tokens, &places);
        // Four rows: the three recall replies and one general reply (position 1, not in the kit).
        let general: &[&str] = &[" x"];
        let replies = [REPLIES[0], general, REPLIES[1], REPLIES[2]];
        let (rows, text) = rows_and_replies(&tokens, &[3, 1, 8, 15], &replies);
        let scored = score(&kit, &bytes, &rows, &text, &decode).expect("a bound reading");
        let counts = (scored.recall, scored.right, scored.value_held, scored.other_held);
        assert_eq!(counts, (3, 1, 2, 1));
        assert_eq!((scored.exact, scored.exact_not_right, scored.references_right), (1, 0, 3));
        assert_eq!(scored.rows.len(), 3);
        assert_eq!(scored.rows[1]["i"], 2);
        assert_eq!(scored.rows[1]["reply"], " It is 47 in Lisbon.");
        assert!(references_hold(&scored).is_ok());
        let written = receipt(&scored, &[("kit_tokens_sha256", bytes.as_slice())]).expect("a receipt");
        assert_eq!((&written["right"], &written["references_right"]), (&json!(1), &json!(3)));
        assert_eq!(written["kit_tokens_sha256"], json!(sha256_hex(&bytes)));

        let refused = |kit: &Value, bytes: &[u8], rows: &[Value], text: &str, word: &str| {
            let err = score(kit, bytes, rows, text, &decode).expect_err(word);
            assert!(err.contains(word), "{err}");
        };
        // A corpus the kit was not written from.
        let mut other = bytes.clone();
        *other.last_mut().expect("bytes") ^= 1;
        refused(&kit, &other, &rows, &text, "another corpus");
        // A row whose reference is not the corpus at its position, and a row without one.
        let mut moved = rows.clone();
        moved[1]["position"] = json!(2);
        refused(&kit, &bytes, &moved, &text, "row 1 is not this corpus's");
        let mut empty = rows.clone();
        empty[1]["reference"] = json!([]);
        refused(&kit, &bytes, &empty, &text, "row 1 is not this corpus's");
        let mut past = rows.clone();
        past[1]["position"] = json!(usize::MAX);
        refused(&kit, &bytes, &past, &text, "row 1 is not this corpus's");
        // A reply short, a reply twice, and a reply under a number no row has.
        let short: String = text.lines().take(3).map(|l| l.to_owned() + "\n").collect();
        refused(&kit, &bytes, &rows, &short, "3 replies for 4 rows");
        refused(&kit, &bytes, &rows, &(text.clone() + text.lines().next().expect("a row")), "twice");
        refused(&kit, &bytes, &rows, &text.replace("\"i\":3", "\"i\":7"), "by its number");
        // One reply more than the rows, under the next number.
        refused(&kit, &bytes, &rows, &(text.clone() + "{\"i\":4,\"ids\":[]}\n"), "5 replies for 4 rows");
        // One recall reply given twice in place of another: the count is right, the set is not.
        let (twice, twice_text) = rows_and_replies(&tokens, &[3, 3, 15], &REPLIES);
        refused(&kit, &bytes, &twice, &twice_text, "repeats the recall reply at 3");
        // A recall reply of the kit that is not among the rows.
        let (fewer, fewer_text) = rows_and_replies(&tokens, &[3, 1, 15], &[REPLIES[0], general, REPLIES[2]]);
        refused(&kit, &bytes, &fewer, &fewer_text, "2 of the kit's 3");
    }

    /// A kit whose value is not what the corpus's own reply says: the rows are still returned,
    /// to be written, and the reading is refused after them.
    #[test]
    fn the_references_must_all_be_right() {
        let (tokens, _) = corpus();
        let (bytes, kit) = file_and_kit(&tokens, &[[3, 1, 2, 1]]);
        let (rows, text) = rows_and_replies(&tokens, &[3], &REPLIES[..1]);
        let scored = score(&kit, &bytes, &rows, &text, &decode).expect("a bound reading");
        assert_eq!((scored.rows.len(), scored.recall, scored.references_right), (1, 1, 0));
        assert_eq!(scored.rows[0]["reference_right"], false);
        assert!(references_hold(&scored).is_err_and(|e| e.contains("0 of 1")));
        // The run ends on the receipt, which carries the refusal after the counts.
        let refused = receipt(&scored, &[]).expect_err("the references are not all right");
        assert!(refused.contains("\"references_right\":0") && refused.contains("0 of 1"), "{refused}");
    }

    #[test]
    fn the_rows_file_is_never_written_over() {
        let path = std::env::temp_dir().join(format!("whole-word-recall-test-{}.jsonl", std::process::id()));
        let path_text = path.to_str().expect("a path").to_owned();
        let _ = fs::remove_file(&path);
        write_rows(&path_text, &[json!({"i": 0})]).expect("a new file");
        assert!(write_rows(&path_text, &[json!({"i": 1})]).is_err());
        assert_eq!(fs::read_to_string(&path).expect("the rows"), "{\"i\":0}\n");
        fs::remove_file(&path).expect("the test's own file");
    }
}
