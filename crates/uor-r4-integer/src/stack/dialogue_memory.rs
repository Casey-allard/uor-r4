//! Deterministic identity-channel memory for the chat CLI.
//!
//! This is an exact lexical sieve, not a learned reader. It searches only prior
//! user turns, ranks clauses by shared normalized content atoms and recency,
//! and extracts one bounded answer phrase for a question. Generated assistant
//! text is never part of the fact log.

use std::collections::BTreeSet;

/// Result of looking for a recall line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SieveRecall {
    /// The turn does not look like a question.
    NotRequested,
    /// A question was recognized, but no prior clause shared a content atom.
    Absent,
    /// The latest best-ranked matching clause supplied this value.
    Value(String),
}

const QUERY_STOP_WORDS: &[&str] = &[
    "a", "about", "am", "an", "and", "are", "be", "been", "being", "but", "by", "can", "could",
    "did", "do", "does", "for", "from", "had", "has", "have", "he", "her", "hers", "him", "his",
    "how", "i", "in", "is", "it", "its", "me", "my", "of", "on", "or", "our", "ours", "please",
    "say", "she", "should", "tell", "that", "the", "their", "theirs", "them", "they", "this", "to",
    "us", "was", "we", "were", "what", "when", "where", "which", "who", "whom", "whose", "why",
    "will", "with", "would", "you", "your", "yours",
];

const BOUNDARIES: &[&str] = &[
    "after", "and", "at", "behind", "before", "beside", "but", "by", "for", "from", "in", "inside",
    "near", "on", "outside", "to", "under", "while", "with",
];

const TEMPORAL_MARKERS: &[&str] = &["after", "at", "before", "by", "every", "in", "on"];
const LOCATIVE_MARKERS: &[&str] = &[
    "at", "behind", "beside", "in", "inside", "near", "on", "outside", "under",
];
const LEADING_ARTICLES: &[&str] = &["a", "an", "the"];
const PRONOUN_OBJECTS: &[&str] = &["him", "her", "it", "me", "them", "us"];
const NUMBER_WORDS: &[&str] = &[
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
];

#[derive(Clone, Copy, Eq, PartialEq)]
enum AnswerKind {
    Name,
    When,
    Where,
    Age,
    Value,
}

fn words(text: &str) -> Vec<String> {
    text.split(|ch: char| {
        !(ch.is_alphanumeric() || ch == '\'' || matches!(ch, '\u{2018}' | '\u{2019}'))
    })
    .map(|word| word.trim_matches(['\'', '\u{2018}', '\u{2019}']).to_owned())
    .filter(|word| !word.is_empty())
    .collect()
}

fn stem(text: &str) -> String {
    let mut word = text.to_lowercase();
    if let Some(base) = word.strip_suffix("'s") {
        word = base.to_owned();
    }
    match word.as_str() {
        "called" => return "call".to_owned(),
        "named" => return "name".to_owned(),
        "grows" => return "grow".to_owned(),
        _ => {}
    }
    if word.ends_with("ies") && word.len() > 4 {
        word.truncate(word.len() - 3);
        word.push('y');
    } else if word.ends_with("ing") && word.len() > 5 {
        word.truncate(word.len() - 3);
    } else if word.ends_with("ed") && word.len() > 4 {
        word.truncate(word.len() - 2);
    } else if word.ends_with('s') && word.len() > 3 && !word.ends_with("ss") {
        word.pop();
    }
    word
}

fn clauses(text: &str) -> Vec<Vec<String>> {
    text.split(|ch: char| matches!(ch, ',' | ';' | ':' | '.' | '!' | '?' | '(' | ')' | '\n'))
        .map(words)
        .filter(|clause| !clause.is_empty())
        .collect()
}

fn is_stop(stemmed: &str) -> bool {
    QUERY_STOP_WORDS.contains(&stemmed)
}

/// Whether this turn is a recall question and therefore should not be stored
/// as a fact for later turns.
pub fn is_query(query: &str) -> bool {
    query.trim_end().ends_with('?')
        || words(query).first().is_some_and(|word| {
            matches!(
                word.to_lowercase().as_str(),
                "what" | "when" | "where" | "which" | "who" | "whom" | "whose" | "why" | "how"
            )
        })
}

fn answer_kind(query: &[String]) -> AnswerKind {
    let lowered: Vec<String> = query.iter().map(|word| word.to_lowercase()).collect();
    if lowered
        .iter()
        .any(|word| matches!(word.as_str(), "name" | "call" | "called"))
    {
        AnswerKind::Name
    } else if lowered
        .iter()
        .any(|word| matches!(word.as_str(), "when" | "day" | "date" | "time"))
    {
        AnswerKind::When
    } else if lowered.iter().any(|word| word == "where") {
        AnswerKind::Where
    } else if lowered
        .windows(2)
        .any(|pair| pair[0] == "how" && pair[1] == "old")
    {
        AnswerKind::Age
    } else {
        AnswerKind::Value
    }
}

fn trim_answer(words: &[String], kind: AnswerKind) -> Option<String> {
    let mut start = 0;
    if words
        .first()
        .is_some_and(|word| LEADING_ARTICLES.contains(&word.to_lowercase().as_str()))
    {
        start = 1;
    }
    if kind == AnswerKind::Name
        && words
            .get(start)
            .is_some_and(|word| matches!(stem(word).as_str(), "name" | "call"))
    {
        start += 1;
    }
    let mut end = words.len();
    for (index, word) in words.iter().enumerate().skip(start) {
        if BOUNDARIES.contains(&word.to_lowercase().as_str()) {
            end = index;
            break;
        }
    }
    (start < end).then(|| words[start..end].join(" "))
}

fn extract_value(clause: &[String], shared_at: usize, kind: AnswerKind) -> Option<String> {
    match kind {
        AnswerKind::Name => {
            for (index, word) in clause.iter().enumerate() {
                if matches!(stem(word).as_str(), "name" | "call") {
                    let mut start = index + 1;
                    if clause.get(start).is_some_and(|word| {
                        let word = word.to_lowercase();
                        PRONOUN_OBJECTS.contains(&word.as_str())
                    }) {
                        start += 1;
                    }
                    if clause.get(start).is_some_and(|word| {
                        matches!(
                            word.to_lowercase().as_str(),
                            "am" | "are" | "is" | "was" | "were"
                        )
                    }) {
                        start += 1;
                    }
                    if start < clause.len() {
                        return trim_answer(&clause[start..], AnswerKind::Name);
                    }
                }
                if matches!(
                    word.to_lowercase().as_str(),
                    "is" | "was" | "are" | "were" | "am"
                ) && index > shared_at
                {
                    return trim_answer(&clause[index + 1..], AnswerKind::Name);
                }
            }
            None
        }
        AnswerKind::When => {
            for (index, word) in clause.iter().enumerate().skip(shared_at.saturating_add(1)) {
                if TEMPORAL_MARKERS.contains(&word.to_lowercase().as_str()) {
                    return trim_answer(&clause[index + 1..], AnswerKind::When);
                }
            }
            None
        }
        AnswerKind::Where => {
            for (index, word) in clause.iter().enumerate().skip(shared_at.saturating_add(1)) {
                if LOCATIVE_MARKERS.contains(&word.to_lowercase().as_str()) {
                    return trim_answer(&clause[index + 1..], AnswerKind::Where);
                }
            }
            None
        }
        AnswerKind::Age => clause
            .iter()
            .skip(shared_at.saturating_add(1))
            .find(|word| {
                let lower = word.to_lowercase();
                NUMBER_WORDS.contains(&lower.as_str()) || lower.parse::<u64>().is_ok()
            })
            .cloned(),
        AnswerKind::Value => {
            let mut start = shared_at + 1;
            if clause.get(start).is_some_and(|word| {
                matches!(
                    word.to_lowercase().as_str(),
                    "is" | "was" | "are" | "were" | "am"
                )
            }) {
                start += 1;
            }
            trim_answer(&clause[start..], AnswerKind::Value)
        }
    }
}

/// Search prior user turns for the latest clause with the strongest shared
/// content atoms and return its bounded value span.
pub fn recall_value(history: &[&str], query: &str) -> Option<String> {
    let query_words = words(query);
    let atoms: BTreeSet<String> = query_words
        .iter()
        .map(|word| stem(word))
        .filter(|word| !is_stop(word))
        .collect();
    if atoms.is_empty() {
        return None;
    }

    let kind = answer_kind(&query_words);
    let mut best: Option<((usize, usize, usize), String)> = None;
    for (turn_index, turn) in history.iter().enumerate() {
        for (clause_index, clause) in clauses(turn).into_iter().enumerate() {
            let shared: Vec<usize> = clause
                .iter()
                .enumerate()
                .filter_map(|(index, word)| atoms.contains(&stem(word)).then_some(index))
                .collect();
            if shared.is_empty() {
                continue;
            }
            let rank = (shared.len(), turn_index, clause_index);
            if best.as_ref().is_some_and(|(top, _)| rank <= *top) {
                continue;
            }
            if let Some(value) = extract_value(&clause, *shared.last().unwrap_or(&0), kind) {
                best = Some((rank, value));
            }
        }
    }
    best.map(|(_, value)| value)
}

/// The serving recall decision. A question with no matching prior clause is an
/// explicit absence; ordinary non-question text requests no memory line.
pub fn recall(history: &[&str], query: &str) -> SieveRecall {
    if !is_query(query) {
        SieveRecall::NotRequested
    } else if let Some(value) = recall_value(history, query) {
        SieveRecall::Value(value)
    } else {
        SieveRecall::Absent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_name_and_action_values() {
        assert_eq!(
            recall_value(
                &["The new puppy at our house is named Pickle."],
                "What did we name the new puppy?"
            )
            .as_deref(),
            Some("Pickle")
        );
        assert_eq!(
            recall_value(
                &[
                    "Grandpa grows tomatoes in a box by the window.",
                    "He waters them every day."
                ],
                "What does grandpa grow?"
            )
            .as_deref(),
            Some("tomatoes")
        );
    }

    #[test]
    fn latest_correction_wins() {
        assert_eq!(
            recall_value(
                &[
                    "The password is pancake.",
                    "Correction: the password is waffle."
                ],
                "What is the password?"
            )
            .as_deref(),
            Some("waffle")
        );
    }

    #[test]
    fn unrelated_and_pronoun_only_queries_are_absent() {
        assert_eq!(
            recall(
                &["Grandpa grows tomatoes."],
                "What did I eat for breakfast?"
            ),
            SieveRecall::Absent
        );
        assert_eq!(
            recall_value(&["Grandpa grows tomatoes."], "What is it?"),
            None
        );
        assert_eq!(
            recall(&["Grandpa grows tomatoes."], "Hello!"),
            SieveRecall::NotRequested
        );
    }

    #[test]
    fn named_and_called_forms_extract_the_following_value() {
        assert_eq!(
            recall_value(&["My sister is called Rosa."], "What is my sister called?").as_deref(),
            Some("Rosa")
        );
        assert_eq!(
            recall_value(&["Our fish is named Goldie."], "What is the fish's name?").as_deref(),
            Some("Goldie")
        );
        assert_eq!(
            recall_value(&["My name is Pico."], "What is my name?").as_deref(),
            Some("Pico")
        );
    }

    #[test]
    fn bounded_answer_types_cover_other_public_panel_forms() {
        let cases = [
            (
                &[
                    "Our classroom fish is called Goldie.",
                    "Yesterday Goldie hid behind a rock.",
                ][..],
                "Where was Goldie hiding?",
                Some("rock"),
            ),
            (
                &["I am going to visit my aunt on Saturday."][..],
                "When am I visiting my aunt?",
                Some("Saturday"),
            ),
            (
                &["My sister is named Rosa and she is six."][..],
                "How old is Rosa?",
                Some("six"),
            ),
            (
                &["My cat is orange and very fluffy."][..],
                "Which color is the cat I told you about?",
                Some("orange"),
            ),
            (
                &["Please call me Captain Sam from now on."][..],
                "What should you call me?",
                Some("Captain Sam"),
            ),
        ];
        for (history, query, expected) in cases {
            assert_eq!(recall_value(history, query).as_deref(), expected, "{query}");
        }
    }
}
