//! Open-domain knowledge corpus preparation for Step 5 of the 5 October plan
//! (#820): article cleaning and decontamination against the open chat panel.
//!
//! The decontamination rule is the one the M-world probe already uses
//! ([`crate::milestone_world_v2_probe::EXCLUSION_NGRAM`] words after
//! [`crate::milestone_world::normalized`]-style word splitting: lowercased,
//! punctuation removed, apostrophes kept inside words). A document is dropped
//! whole when any of its 8-word windows equals an 8-word window of any panel
//! user turn. Panel turns shorter than eight words cannot produce an 8-gram;
//! for those with at least [`MIN_SHORT_REQUEST_WORDS`] words the whole turn's
//! word sequence is excluded instead, so short requests such as "Why do
//! leaves change color in the fall" are not silently uncovered. Turns with
//! fewer words are counted as uncovered rather than matched, because a
//! two- or three-word sequence ("hi dr smith") is not evidence of contamination.
//!
//! Matching is exact: window keys are hashed to `u64` for speed and every
//! candidate hit is confirmed against the stored word sequence, so a hash
//! collision can never drop a document.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};

use serde_json::Value;

use crate::milestone_world::words;
use crate::milestone_world_v2_probe::EXCLUSION_NGRAM;

/// Word count of the panel exclusion window (shared with the M-world probe).
pub const PANEL_NGRAM: usize = EXCLUSION_NGRAM;

/// Panel turns with at least this many (but fewer than [`PANEL_NGRAM`]) words
/// are excluded by whole-sequence containment.
pub const MIN_SHORT_REQUEST_WORDS: usize = 4;

/// Errors at the knowledge-corpus library boundary.
#[derive(Debug)]
pub enum KnowledgeError {
    /// A panel file is not a JSON list of `{id, user_turns: [string]}` rows.
    Panel(String),
    /// A panel file is not JSON.
    Json(serde_json::Error),
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panel(message) => write!(f, "panel: {message}"),
            Self::Json(error) => write!(f, "panel JSON: {error}"),
        }
    }
}

impl std::error::Error for KnowledgeError {}

impl From<serde_json::Error> for KnowledgeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// One panel request: its id and user turns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelRequest {
    pub id: String,
    pub user_turns: Vec<String>,
}

/// The requests of one chat-grade panel file (`[{id, category, user_turns}]`).
pub fn parse_panel(bytes: &[u8]) -> Result<Vec<PanelRequest>, KnowledgeError> {
    let value: Value = serde_json::from_slice(bytes)?;
    let rows = value
        .as_array()
        .ok_or_else(|| KnowledgeError::Panel("not a JSON list".into()))?;
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            let id = row["id"]
                .as_str()
                .ok_or_else(|| KnowledgeError::Panel(format!("row {index} has no string id")))?;
            let turns = row["user_turns"]
                .as_array()
                .ok_or_else(|| KnowledgeError::Panel(format!("row {id} has no user_turns list")))?;
            let user_turns = turns
                .iter()
                .map(|t| {
                    t.as_str().map(str::to_owned).ok_or_else(|| {
                        KnowledgeError::Panel(format!("row {id} has a non-string user turn"))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            if user_turns.is_empty() {
                return Err(KnowledgeError::Panel(format!("row {id} has no user turn")));
            }
            Ok(PanelRequest {
                id: id.to_owned(),
                user_turns,
            })
        })
        .collect()
}

fn window_key(window: &[String]) -> u64 {
    // `DefaultHasher::new()` uses fixed keys, so keys are stable within a run;
    // they are never persisted, and every hit is confirmed exactly.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    window.hash(&mut hasher);
    hasher.finish()
}

/// Which exclusion rule matched a document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitKind {
    /// An 8-word window shared with a panel turn.
    Ngram,
    /// The whole word sequence of a short (4-7 word) panel turn.
    ShortRequest,
}

impl HitKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ngram => "panel_8gram",
            Self::ShortRequest => "panel_short_request",
        }
    }
}

/// The first exclusion match in a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub kind: HitKind,
    /// The matched normalized words, joined by single spaces.
    pub words: String,
    /// The panel request id whose turn supplied the matched sequence.
    pub request_id: String,
}

/// Exact word-sequence exclusion built from panel user turns.
#[derive(Clone, Debug, Default)]
pub struct PanelExclusion {
    /// Sequence length -> key -> (sequence, request id); one entry per length
    /// in use: [`PANEL_NGRAM`] and every short-turn length.
    by_len: BTreeMap<usize, HashMap<u64, Vec<(Vec<String>, String)>>>,
    /// Panel requests read.
    pub requests: usize,
    /// User turns read.
    pub turns: usize,
    /// Turns with at least [`PANEL_NGRAM`] words (covered by 8-grams).
    pub ngram_turns: usize,
    /// Turns covered by whole-sequence containment.
    pub short_turns: usize,
    /// Turns with fewer than [`MIN_SHORT_REQUEST_WORDS`] words: not covered.
    pub uncovered_turns: usize,
    /// Distinct 8-word windows.
    pub ngrams: usize,
}

impl PanelExclusion {
    /// The exclusion of every user turn of `requests`.
    pub fn new(requests: &[PanelRequest]) -> Self {
        let mut exclusion = Self::default();
        for request in requests {
            exclusion.requests += 1;
            for turn in &request.user_turns {
                exclusion.turns += 1;
                let all = words(turn);
                if all.len() >= PANEL_NGRAM {
                    exclusion.ngram_turns += 1;
                    for window in all.windows(PANEL_NGRAM) {
                        if exclusion.insert(window, &request.id) {
                            exclusion.ngrams += 1;
                        }
                    }
                } else if all.len() >= MIN_SHORT_REQUEST_WORDS {
                    exclusion.short_turns += 1;
                    exclusion.insert(&all, &request.id);
                } else {
                    exclusion.uncovered_turns += 1;
                }
            }
        }
        exclusion
    }

    /// Insert one sequence; false when it was already present.
    fn insert(&mut self, sequence: &[String], request_id: &str) -> bool {
        let bucket = self
            .by_len
            .entry(sequence.len())
            .or_default()
            .entry(window_key(sequence))
            .or_default();
        if bucket.iter().any(|(seq, _)| seq.as_slice() == sequence) {
            return false;
        }
        bucket.push((sequence.to_vec(), request_id.to_owned()));
        true
    }

    /// The first excluded sequence `text` contains, scanning lengths in
    /// increasing order and positions left to right.
    pub fn first_hit(&self, text: &str) -> Option<Hit> {
        if self.by_len.is_empty() {
            return None;
        }
        let all = words(text);
        for (&len, table) in &self.by_len {
            if all.len() < len {
                continue;
            }
            for window in all.windows(len) {
                if let Some(bucket) = table.get(&window_key(window)) {
                    if let Some((seq, id)) = bucket.iter().find(|(seq, _)| seq.as_slice() == window)
                    {
                        return Some(Hit {
                            kind: if len == PANEL_NGRAM {
                                HitKind::Ngram
                            } else {
                                HitKind::ShortRequest
                            },
                            words: seq.join(" "),
                            request_id: id.clone(),
                        });
                    }
                }
            }
        }
        None
    }
}

/// An article's training text: optional title line, then the body with
/// `\r\n`/`\r` normalised, trailing spaces trimmed per line, runs of blank
/// lines collapsed to one, and the whole trimmed.
pub fn clean_article(title: Option<&str>, body: &str) -> String {
    let body = body.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::with_capacity(body.len() + 64);
    if let Some(title) = title.map(str::trim).filter(|t| !t.is_empty()) {
        out.push_str(title);
        out.push_str("\n\n");
    }
    let mut blank = false;
    let mut any = false;
    for line in body.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank = any;
            continue;
        }
        if blank {
            out.push('\n');
        }
        if any {
            out.push('\n');
        }
        out.push_str(line);
        any = true;
        blank = false;
    }
    out.trim().to_owned()
}

/// The number of normalized words in `text` (the `min_words` filter's count).
pub fn word_count(text: &str) -> usize {
    words(text).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str, turns: &[&str]) -> PanelRequest {
        PanelRequest {
            id: id.into(),
            user_turns: turns.iter().map(|t| (*t).to_owned()).collect(),
        }
    }

    #[test]
    fn a_shared_8_gram_is_found_regardless_of_case_and_punctuation() {
        let exclusion = PanelExclusion::new(&[request(
            "heldout-020",
            &["Can you explain the process of pollination and seed dispersal in flowering plants?"],
        )]);
        assert_eq!(exclusion.ngram_turns, 1);
        assert_eq!(exclusion.ngrams, 13 - PANEL_NGRAM + 1);
        let hit = exclusion
            .first_hit(
                "Biology. THE PROCESS of Pollination -- and seed dispersal, in flowering plants!",
            )
            .expect("an 8-gram hit");
        assert_eq!(hit.kind, HitKind::Ngram);
        assert_eq!(hit.request_id, "heldout-020");
        assert_eq!(
            hit.words,
            "the process of pollination and seed dispersal in"
        );
    }

    #[test]
    fn a_seven_word_overlap_with_a_long_turn_is_kept() {
        let exclusion = PanelExclusion::new(&[request(
            "x",
            &["one two three four five six seven eight nine"],
        )]);
        assert!(exclusion
            .first_hit("zero one two three four five six seven zero")
            .is_none());
        assert!(exclusion
            .first_hit("zero one two three four five six seven eight")
            .is_some());
    }

    #[test]
    fn short_turns_are_matched_whole_and_tiny_turns_are_counted_uncovered() {
        let exclusion = PanelExclusion::new(&[
            request("ask-02", &["What do bees make?"]),
            request("heldout-006", &["Hi Dr. Smith,"]),
            request(
                "follow-04",
                &[
                    "I am planning a party for my sister.",
                    "She loves flowers. Any ideas?",
                ],
            ),
        ]);
        assert_eq!(exclusion.turns, 4);
        assert_eq!(exclusion.ngram_turns, 1);
        assert_eq!(exclusion.short_turns, 2);
        assert_eq!(exclusion.uncovered_turns, 1);
        let hit = exclusion
            .first_hit("Children often ask: what do bees make? Honey.")
            .expect("short request hit");
        assert_eq!(hit.kind, HitKind::ShortRequest);
        assert_eq!(hit.request_id, "ask-02");
        // The three-word salutation is not used as an exclusion sequence.
        assert!(exclusion.first_hit("hi dr smith wrote a paper").is_none());
        // A partial short request is not a hit.
        assert!(exclusion.first_hit("what do bees eat").is_none());
    }

    #[test]
    fn duplicate_windows_are_counted_once() {
        let turn = "a b c d e f g h";
        let exclusion = PanelExclusion::new(&[request("p", &[turn]), request("q", &[turn])]);
        assert_eq!(exclusion.ngrams, 1);
        assert_eq!(
            exclusion.first_hit(turn).map(|h| h.request_id),
            Some("p".into())
        );
    }

    #[test]
    fn panel_files_parse_and_malformed_ones_fail() {
        let rows = parse_panel(
            br#"[{"id":"talk-01","category":"smalltalk","user_turns":["Hi! How are you today?"]}]"#,
        )
        .unwrap();
        assert_eq!(rows, vec![request("talk-01", &["Hi! How are you today?"])]);
        assert!(parse_panel(br#"{"id":"x"}"#).is_err());
        assert!(parse_panel(br#"[{"id":"x","user_turns":[]}]"#).is_err());
        assert!(parse_panel(br#"[{"id":"x","user_turns":[3]}]"#).is_err());
        assert!(parse_panel(br#"[{"user_turns":["a"]}]"#).is_err());
        assert!(parse_panel(b"not json").is_err());
    }

    #[test]
    fn articles_are_cleaned_with_a_title_line() {
        assert_eq!(
            clean_article(
                Some(" Bee "),
                "Bees make honey.  \r\n\r\n\r\n\nBees live in hives.\rThey fly.\n\n"
            ),
            "Bee\n\nBees make honey.\n\nBees live in hives.\nThey fly."
        );
        assert_eq!(clean_article(None, "\n\n  Body \n"), "Body");
        assert_eq!(clean_article(Some(""), "x"), "x");
        assert_eq!(word_count("It's a bee's knees."), 4);
    }
}
