//! Binding diagnostic for exact-recall memory rows (#820).
//!
//! The `binding-probe` binary answers one question about a geometric stack on
//! a panel's `exact` memory rows: when the model is about to give the value,
//! where do its read layers and pointer head put their mass, and which value
//! does its output prefer? This module holds the pure parts: word and token
//! span matching over a decoded window, the panel's `exact` row checks, and
//! the classification rule, which was fixed before any model was probed.
//!
//! **Pre-registered classification** (owner decision, #820 comment
//! 6003048397; do not change after seeing data). At the decision query (see
//! the binary), let `E` and `D` be the largest attention mass on the
//! expected-value span and on the distractor-value span over every read layer
//! and head and the pointer head, and let `o_E`, `o_D` be the decoded
//! distribution's largest probability of a first token of an expected and of a
//! distractor spelling. With threshold [`TAU`]:
//!
//! * the *mass pattern* is `Expected` when `E >= TAU` and `E >= D`,
//!   `Distractor` when `D >= TAU` and `D > E`, otherwise `Neither`;
//! * a miss is (c) READOUT when the pattern is `Expected` (the right value was
//!   reached but not produced); (a) BINDING-SWAP when the pattern is
//!   `Distractor`, or when it is `Neither` and `o_D > o_E` (the output prefers
//!   the other key's value without visible mass); otherwise (b) UNREACHED;
//! * a pass is reported by its mass pattern.
//!
//! The mapping to the next step is the owner's: a majority of (a) points to
//! read-key content (Step 7a), of (b) to an exact learned join (Step 7b), of
//! (c) to a copy-gate/readout fix. The full distribution is always reported,
//! with the thresholds [`SENSITIVITY`] alongside.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

/// The pre-registered mass threshold.
pub const TAU: f64 = 0.10;
/// Thresholds reported alongside [`TAU`] (not used for the verdict).
pub const SENSITIVITY: [f64; 2] = [0.05, 0.25];

/// A word of a text with its byte range, by `chat-grade`'s word rule:
/// lowercased, split at anything that is not a letter, digit or apostrophe (a
/// curly apostrophe counts as one), apostrophes trimmed from the ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub bytes: Range<usize>,
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\'' || c == '\u{2019}'
}

fn apostrophe(c: char) -> bool {
    c == '\'' || c == '\u{2019}'
}

/// The words of `text` with their byte ranges in `text`.
pub fn words_at(text: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let flush = |from: usize, to: usize, out: &mut Vec<Word>| {
        let raw = &text[from..to];
        let lead = raw.len() - raw.trim_start_matches(apostrophe).len();
        let trimmed = raw.trim_matches(apostrophe);
        if !trimmed.is_empty() {
            let begin = from + lead;
            out.push(Word {
                text: trimmed.replace('\u{2019}', "'").to_lowercase(),
                bytes: begin..begin + trimmed.len(),
            });
        }
    };
    for (i, c) in text.char_indices() {
        match (word_char(c), start) {
            (true, None) => start = Some(i),
            (false, Some(from)) => {
                flush(from, i, &mut out);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(from) = start {
        flush(from, text.len(), &mut out);
    }
    out
}

/// The words of `text` (`chat-grade`'s `words`).
pub fn words(text: &str) -> Vec<String> {
    words_at(text).into_iter().map(|w| w.text).collect()
}

/// Whether `phrase` occurs as consecutive words of `text`.
pub fn contains_phrase(text: &[String], phrase: &[String]) -> bool {
    !phrase.is_empty() && text.windows(phrase.len()).any(|w| w == phrase)
}

/// The byte range of each token in the concatenation of `pieces` (each
/// token's decoded bytes, in window order).
pub fn token_byte_ranges(pieces: &[Vec<u8>]) -> Vec<Range<usize>> {
    let mut at = 0;
    pieces
        .iter()
        .map(|piece| {
            let range = at..at + piece.len();
            at += piece.len();
            range
        })
        .collect()
}

/// The window positions whose tokens overlap an occurrence of `phrase` in
/// `text` (the decoded window), restricted to tokens in `region`. A phrase
/// occurrence counts when its first word starts inside `region`.
pub fn phrase_positions(
    text: &str,
    tokens: &[Range<usize>],
    region: Range<usize>,
    phrase: &[String],
) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    if phrase.is_empty() {
        return out;
    }
    let all = words_at(text);
    for window in all.windows(phrase.len()) {
        if !window.iter().map(|w| &w.text).eq(phrase.iter()) {
            continue;
        }
        let bytes = window[0].bytes.start..window[phrase.len() - 1].bytes.end;
        let covering: Vec<usize> = tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| t.start < bytes.end && bytes.start < t.end)
            .map(|(j, _)| j)
            .collect();
        if covering.first().is_some_and(|j| region.contains(j)) {
            out.extend(covering.into_iter().filter(|j| region.contains(j)));
        }
    }
    out
}

/// One `exact` row of a panel checks file (`id kind history terms forbid
/// [keys]`): the expected value's spellings, the forbidden (distractor)
/// values and the distractor's key words, each a list of word phrases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactCheck {
    pub expected: Vec<Vec<String>>,
    pub forbid: Vec<Vec<String>>,
    pub keys: Vec<Vec<String>>,
}

impl ExactCheck {
    /// `chat-grade`'s exact rule: the reply names an expected spelling and no
    /// forbidden value and no distractor key.
    pub fn passes(&self, reply: &str) -> bool {
        let reply = words(reply);
        !self.forbid.iter().any(|t| contains_phrase(&reply, t))
            && !self.keys.iter().any(|t| contains_phrase(&reply, t))
            && self.expected.iter().any(|t| contains_phrase(&reply, t))
    }
}

fn term_list(field: &str) -> Vec<Vec<String>> {
    if field == "-" {
        return Vec::new();
    }
    field
        .split('|')
        .map(words)
        .filter(|w| !w.is_empty())
        .collect()
}

/// The `exact` rows of a checks file, by id. Other kinds are skipped.
pub fn parse_exact_checks(text: &str) -> Result<BTreeMap<String, ExactCheck>, String> {
    let mut out = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 4 {
            return Err(format!("checks line {}: too few fields", n + 1));
        }
        if fields[1] != "exact" {
            continue;
        }
        let check = ExactCheck {
            expected: term_list(fields[3]),
            forbid: term_list(fields.get(4).copied().unwrap_or("-")),
            keys: term_list(fields.get(5).copied().unwrap_or("-")),
        };
        if check.expected.is_empty() || check.forbid.is_empty() {
            return Err(format!(
                "checks line {}: an exact row needs expected and forbidden terms",
                n + 1
            ));
        }
        if out.insert(fields[0].to_owned(), check).is_some() {
            return Err(format!("checks line {}: repeated id {}", n + 1, fields[0]));
        }
    }
    Ok(out)
}

/// Function words never taken as the asked key.
const STOPWORDS: &[&str] = &[
    "a", "about", "after", "all", "am", "an", "and", "any", "are", "as", "at", "be", "been",
    "before", "but", "by", "can", "could", "did", "do", "does", "for", "from", "had", "has",
    "have", "he", "her", "hers", "him", "his", "how", "i", "i'm", "if", "in", "into", "is", "it",
    "it's", "its", "just", "kind", "me", "my", "name", "named", "no", "not", "now", "of", "on",
    "one", "or", "our", "remember", "said", "she", "so", "tell", "that", "the", "their", "them",
    "then", "there", "they", "this", "to", "told", "too", "up", "us", "was", "we", "were", "what",
    "what's", "when", "where", "which", "who", "whose", "why", "will", "with", "would", "you",
    "your",
];

/// The asked key's words: words of the last user turn that also occur in an
/// earlier user turn, other than function words and the row's value and
/// distractor-key words.
pub fn asked_key_words(last: &str, earlier: &[&str], check: &ExactCheck) -> Vec<String> {
    let earlier: BTreeSet<String> = earlier.iter().flat_map(|t| words(t)).collect();
    let excluded: BTreeSet<&String> = check
        .expected
        .iter()
        .chain(&check.forbid)
        .chain(&check.keys)
        .flatten()
        .collect();
    let mut seen = BTreeSet::new();
    words(last)
        .into_iter()
        .filter(|w| {
            earlier.contains(w)
                && !STOPWORDS.contains(&w.as_str())
                && !excluded.contains(w)
                && seen.insert(w.clone())
        })
        .collect()
}

/// The disjoint source sets of a probed window, in priority order: a position
/// matching several kinds belongs to the first.
pub const SPAN_KINDS: [&str; 5] = [
    "expected",
    "distractor",
    "distractor_key",
    "asked_key",
    "question",
];

/// Make `sets` (one per [`SPAN_KINDS`] entry) disjoint by priority.
pub fn disjoint(sets: [BTreeSet<usize>; 5]) -> [Vec<usize>; 5] {
    let mut taken = BTreeSet::new();
    sets.map(|set| {
        let kept: Vec<usize> = set.into_iter().filter(|j| !taken.contains(j)).collect();
        taken.extend(kept.iter().copied());
        kept
    })
}

/// What the probe saw at the decision query, reduced for classification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Evidence {
    /// Largest read mass on the expected / distractor span over layers and heads.
    pub read_expected: f64,
    pub read_distractor: f64,
    /// The pointer's attention mass on the spans (`None` without a pointer).
    pub pointer_expected: Option<f64>,
    pub pointer_distractor: Option<f64>,
    /// Largest decoded probability of a first token of an expected / distractor spelling.
    pub out_expected: f64,
    pub out_distractor: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MassPattern {
    Expected,
    Distractor,
    Neither,
}

impl MassPattern {
    pub fn name(self) -> &'static str {
        match self {
            MassPattern::Expected => "expected",
            MassPattern::Distractor => "distractor",
            MassPattern::Neither => "neither",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MissClass {
    /// (a) mass and/or output prefer the distractor's value.
    BindingSwap,
    /// (b) mass on neither value.
    Unreached,
    /// (c) mass on the right value, output does not produce it.
    Readout,
}

impl MissClass {
    pub fn name(self) -> &'static str {
        match self {
            MissClass::BindingSwap => "a_binding_swap",
            MissClass::Unreached => "b_unreached",
            MissClass::Readout => "c_readout",
        }
    }
}

impl Evidence {
    /// The larger of the read and pointer masses on each value.
    pub fn masses(&self) -> (f64, f64) {
        (
            self.read_expected.max(self.pointer_expected.unwrap_or(0.0)),
            self.read_distractor
                .max(self.pointer_distractor.unwrap_or(0.0)),
        )
    }

    pub fn pattern(&self, tau: f64) -> MassPattern {
        let (e, d) = self.masses();
        if e >= tau && e >= d {
            MassPattern::Expected
        } else if d >= tau && d > e {
            MassPattern::Distractor
        } else {
            MassPattern::Neither
        }
    }

    /// The pre-registered class of a missed row (see the module documentation).
    pub fn miss_class(&self, tau: f64) -> MissClass {
        match self.pattern(tau) {
            MassPattern::Expected => MissClass::Readout,
            MassPattern::Distractor => MissClass::BindingSwap,
            MassPattern::Neither if self.out_distractor > self.out_expected => {
                MissClass::BindingSwap
            }
            MassPattern::Neither => MissClass::Unreached,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_match_chat_grade_rule_with_offsets() {
        let text = "My cousin Ezra\u{2019}s ferret, and 'Thea' keeps gerbils!";
        let got = words_at(text);
        let texts: Vec<&str> = got.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(
            texts,
            ["my", "cousin", "ezra's", "ferret", "and", "thea", "keeps", "gerbils"]
        );
        for w in &got {
            assert_eq!(
                text[w.bytes.clone()]
                    .replace('\u{2019}', "'")
                    .to_lowercase(),
                w.text
            );
        }
    }

    #[test]
    fn spans_map_phrases_to_token_positions_on_a_toy_history() {
        // A toy tokenization of a two-turn history and a reply prefix.
        let pieces: Vec<&str> = vec![
            "<s>",
            "User",
            ":",
            " My",
            " cousin",
            " Ez",
            "ra",
            " keeps",
            " a",
            " fer",
            "ret",
            ",",
            " and",
            " Thea",
            " keeps",
            " a",
            " ger",
            "bil",
            ".",
            "\n",
            "Assistant",
            ":",
            " Nice",
            "!",
            "</s>",
            "User",
            ":",
            " What",
            " pet",
            " does",
            " Thea",
            " have",
            "?",
            "\n",
            "Assistant",
            ":",
        ];
        let bytes: Vec<Vec<u8>> = pieces.iter().map(|p| p.as_bytes().to_vec()).collect();
        let tokens = token_byte_ranges(&bytes);
        let text: String = pieces.concat();
        let window = 0..pieces.len();
        let at = |phrase: &str| -> Vec<usize> {
            phrase_positions(&text, &tokens, window.clone(), &words(phrase))
                .into_iter()
                .collect()
        };
        assert_eq!(at("gerbil"), vec![16, 17]);
        assert_eq!(at("ferret"), vec![9, 10]);
        assert_eq!(at("ezra"), vec![5, 6]);
        assert_eq!(at("thea"), vec![13, 30]);
        assert!(at("hamster").is_empty());
        // "ger bil" is not the word "gerbil" split by a space: no match.
        assert!(at("ger").is_empty());
        // Restricting the region to the first user turn drops the question's Thea.
        let first_turn: Vec<usize> = phrase_positions(&text, &tokens, 0..22, &words("thea"))
            .into_iter()
            .collect();
        assert_eq!(first_turn, vec![13]);
        // Multi-word phrases and disjoint priority.
        assert_eq!(at("a gerbil"), vec![15, 16, 17]);
        let check = ExactCheck {
            expected: vec![words("gerbil"), words("gerbils")],
            forbid: vec![words("ferret")],
            keys: vec![words("ezra")],
        };
        let asked = asked_key_words(
            "What pet does Thea have?",
            &["My cousin Ezra keeps a ferret, and Thea keeps a gerbil."],
            &check,
        );
        assert_eq!(asked, vec!["thea".to_owned()]);
        let sets = disjoint([
            at("gerbil").into_iter().collect(),
            at("ferret").into_iter().collect(),
            at("ezra").into_iter().collect(),
            phrase_positions(&text, &tokens, 0..25, &words("thea")),
            (25..36).collect(),
        ]);
        assert_eq!(sets[3], vec![13]);
        assert_eq!(sets[4], (25..36).collect::<Vec<_>>());
        assert!(
            sets.iter().flatten().collect::<BTreeSet<_>>().len()
                == sets.iter().map(Vec::len).sum::<usize>()
        );
    }

    #[test]
    fn exact_checks_parse_and_judge_like_chat_grade() {
        let tsv = "# header\nm1\texact\trecall\tgerbil|gerbils\tferret|ferrets\tezra|ezra's\n\
                   u1\tabstain_exact\tnone\t-\trose\nm2\texact\trecall\tduluth\tcanoe\t-\n";
        let checks = parse_exact_checks(tsv).expect("parse");
        assert_eq!(checks.len(), 2);
        let m1 = &checks["m1"];
        assert!(m1.passes("Thea has a gerbil."));
        assert!(!m1.passes("Thea has a ferret."));
        assert!(!m1.passes("A gerbil or a ferret."));
        assert!(!m1.passes("Ezra's gerbil."));
        assert!(!m1.passes("A hamster."));
        assert!(checks["m2"].keys.is_empty());
        assert!(parse_exact_checks("x\texact\trecall\tgerbil\t-\n").is_err());
    }

    #[test]
    fn classification_follows_the_preregistered_rule() {
        let e = |re, rd, pe: Option<f64>, pd: Option<f64>, oe, od| Evidence {
            read_expected: re,
            read_distractor: rd,
            pointer_expected: pe,
            pointer_distractor: pd,
            out_expected: oe,
            out_distractor: od,
        };
        // Mass on the right value but the wrong output: readout.
        assert_eq!(
            e(0.4, 0.1, None, None, 0.1, 0.6).miss_class(TAU),
            MissClass::Readout
        );
        // Mass on the distractor: swap.
        assert_eq!(
            e(0.05, 0.3, None, None, 0.0, 0.0).miss_class(TAU),
            MissClass::BindingSwap
        );
        // Pointer mass counts as mass.
        assert_eq!(
            e(0.02, 0.01, Some(0.1), Some(0.5), 0.0, 0.0).miss_class(TAU),
            MissClass::BindingSwap
        );
        // No mass, output prefers the distractor: swap.
        assert_eq!(
            e(0.02, 0.03, Some(0.0), Some(0.0), 0.01, 0.2).miss_class(TAU),
            MissClass::BindingSwap
        );
        // No mass, output prefers neither: unreached.
        assert_eq!(
            e(0.02, 0.03, None, None, 0.01, 0.001).miss_class(TAU),
            MissClass::Unreached
        );
        // A tie at the threshold goes to the expected value.
        assert_eq!(
            e(TAU, TAU, None, None, 0.0, 0.0).pattern(TAU),
            MassPattern::Expected
        );
    }
}
