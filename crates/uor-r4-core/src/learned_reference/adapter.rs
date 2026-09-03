//! Raw-byte adapter for the frozen `R4TextToClausesV1` policy.
//!
//! The parent wrapper verifies startup artifacts and the request schema before
//! calling [`parse`]. This module reads no files and has no model handles. The
//! grammar recognizer returns only acceptance; it exposes no semantic roles.

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};

pub(super) const REQUEST_SCHEMA: &str = "uor-r4.text-to-clauses/1";
pub(super) const RESULT_SCHEMA: &str = "uor-r4.text-to-clauses-result/1";

const POLICY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/policy.json"
));
const VOCABULARY_FILE_CID: &str =
    "blake3:571d5fbc282b17c8726eebd7b23c3ae55212a3de81b35d27722a0fa5979b8c5b";
const MAX_BYTES: usize = 4096;
const MAX_TOKENS: usize = 13;
const PAD_ID: i64 = 57;

// Exact reader prefix in the bound policy. This is lexical metadata, not the
// core output vocabulary. IDs 0, 7, 11 and 57 are not admitted input tokens.
const READER_PREFIX: [&str; 58] = [
    "<bos>", "put", "the", "in", ".", "where", "is", "'s", "?", "answer", ":", "unknown", "mara",
    "lena", "omar", "noah", "iris", "liam", "nora", "otto", "ada", "erin", "hugo", "iona", "jude",
    "kira", "leon", "mila", "key", "book", "coin", "cup", "ring", "ball", "pen", "map", "hat",
    "toy", "jar", "box", "comb", "fork", "shoe", "doll", "drawer", "cabinet", "basket", "closet",
    "pouch", "locker", "crate", "trunk", "not", "but", ",", "owned", "by", "<pad>",
];

const FACT_FORMS: [&[&str]; 4] = [
    &[
        "O", ",", "not", "D", ",", "put", "the", "X", "in", "the", "L", ".",
    ],
    &[
        "in", "the", "L", ",", "not", "D", "but", "O", "put", "the", "X", ".",
    ],
    &[
        "not", "D", "but", "O", "put", "the", "X", "in", "the", "L", ".",
    ],
    &[
        "in", "the", "L", ",", "O", ",", "not", "D", ",", "put", "the", "X", ".",
    ],
];
const QUERY_FORM: &[&str] = &[
    "where", "is", "the", "X", "owned", "by", "O", ",", "not", "D", "?", "answer", ":",
];

/// The original refusal record, with no token, tensor or partial parse fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Refusal {
    pub schema: &'static str,
    pub status: &'static str,
    pub byte_offset: Option<usize>,
}

impl Refusal {
    fn at(status: &'static str, byte_offset: usize) -> Self {
        Self {
            schema: RESULT_SCHEMA,
            status,
            byte_offset: Some(byte_offset),
        }
    }

    /// Refuse a malformed request before examining its raw buffer.
    pub fn unsupported_schema() -> Self {
        Self {
            schema: RESULT_SCHEMA,
            status: "UNSUPPORTED_SCHEMA",
            byte_offset: None,
        }
    }

    /// Refuse unavailable startup without examining a request.
    pub fn unavailable_artifact() -> Self {
        Self {
            schema: RESULT_SCHEMA,
            status: "UNAVAILABLE_ARTIFACT",
            byte_offset: None,
        }
    }
}

/// Internal unbatched storage; serialization preserves the original batch axis.
#[derive(Clone, Debug)]
pub(super) struct Parsed {
    pub(super) inputs: [[i64; 13]; 5],
    pub(super) lengths: [usize; 5],
    pub(super) token_spans: Vec<Vec<[usize; 2]>>,
    pub(super) clause_spans: Vec<[usize; 2]>,
    pub(super) raw_text_sha256: String,
    pub(super) derived_input_sha256: String,
}

impl Serialize for Parsed {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut record = serializer.serialize_struct("Parsed", 9)?;
        record.serialize_field("schema", RESULT_SCHEMA)?;
        record.serialize_field("status", "SEGMENTED")?;
        record.serialize_field("policy_sha256", &policy_sha256())?;
        record.serialize_field("raw_text_sha256", &self.raw_text_sha256)?;
        record.serialize_field("derived_input_sha256", &self.derived_input_sha256)?;
        record.serialize_field("clause_spans", &self.clause_spans)?;
        record.serialize_field("token_spans", &self.token_spans)?;
        record.serialize_field("inputs", &[&self.inputs])?;
        record.serialize_field("lengths", &[&self.lengths])?;
        record.end()
    }
}

#[derive(Clone, Copy)]
struct Token<'a> {
    word: &'a [u8],
    id: i64,
    start: usize,
    end: usize,
}

pub(super) fn policy_sha256() -> String {
    hex::encode(Sha256::digest(POLICY.as_bytes()))
}

fn lex(raw: &[u8]) -> Result<Vec<Token<'_>>, Refusal> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < raw.len() {
        let byte = raw[cursor];
        match byte {
            b' ' | b'\t' | b'\n' => {
                cursor += 1;
                continue;
            }
            b'\r' => {
                if raw.get(cursor + 1) != Some(&b'\n') {
                    return Err(Refusal::at("INVALID_ENCODING", cursor));
                }
                cursor += 2;
                continue;
            }
            _ => {}
        }

        let start = cursor;
        let id = match byte {
            b'a'..=b'z' => {
                cursor += 1;
                while cursor < raw.len() && raw[cursor].is_ascii_lowercase() {
                    cursor += 1;
                }
                let word = &raw[start..cursor];
                let found = READER_PREFIX.iter().enumerate().find(|(index, value)| {
                    !matches!(*index, 0 | 7 | 11 | 57) && value.as_bytes() == word
                });
                match found {
                    Some((index, _)) => index as i64,
                    None => return Err(Refusal::at("UNKNOWN_LEXEME", start)),
                }
            }
            b'.' => {
                cursor += 1;
                4
            }
            b',' => {
                cursor += 1;
                54
            }
            b'?' => {
                cursor += 1;
                8
            }
            b':' => {
                cursor += 1;
                10
            }
            _ => return Err(Refusal::at("INVALID_ENCODING", cursor)),
        };
        tokens.push(Token {
            word: &raw[start..cursor],
            id,
            start,
            end: cursor,
        });
    }
    Ok(tokens)
}

fn boundaries<'a, 'raw>(
    tokens: &'a [Token<'raw>],
    end: usize,
) -> Result<Vec<&'a [Token<'raw>]>, Refusal> {
    let mut clauses = Vec::with_capacity(5);
    let mut start = 0;
    for (index, token) in tokens.iter().enumerate() {
        if token.word != b"." {
            continue;
        }
        if index == start || clauses.len() == 4 {
            return Err(Refusal::at("UNSUPPORTED_BOUNDARY", token.start));
        }
        clauses.push(&tokens[start..=index]);
        start = index + 1;
    }
    if clauses.len() != 4 || start == tokens.len() {
        return Err(Refusal::at("UNSUPPORTED_BOUNDARY", end));
    }

    let query = &tokens[start..];
    let Some(question_mark) = query.iter().position(|token| token.word == b"?") else {
        return Err(Refusal::at("UNSUPPORTED_BOUNDARY", end));
    };
    for (offset, expected) in [b"?".as_slice(), b"answer".as_slice(), b":".as_slice()]
        .into_iter()
        .enumerate()
    {
        let Some(token) = query.get(question_mark + offset) else {
            return Err(Refusal::at("UNSUPPORTED_BOUNDARY", end));
        };
        if token.word != expected {
            return Err(Refusal::at("UNSUPPORTED_BOUNDARY", token.start));
        }
    }
    if let Some(token) = query.get(question_mark + 3) {
        return Err(Refusal::at("UNSUPPORTED_BOUNDARY", token.start));
    }
    clauses.push(query);
    Ok(clauses)
}

fn matches_clause(clause: &[Token<'_>], form: &[&str]) -> bool {
    if clause.len() != form.len() {
        return false;
    }
    for (token, predicate) in clause.iter().zip(form) {
        let accepted = match *predicate {
            "O" | "D" => (12..28).contains(&token.id),
            "X" => (28..44).contains(&token.id),
            "L" => (44..52).contains(&token.id),
            literal => token.word == literal.as_bytes(),
        };
        if !accepted {
            return false;
        }
    }
    // The two membership predicates must name distinct owner words. This is a
    // boolean syntax condition; no positions or assignments leave this function.
    let owner = form.iter().position(|predicate| *predicate == "O");
    let distractor = form.iter().position(|predicate| *predicate == "D");
    match (
        owner.and_then(|index| clause.get(index)),
        distractor.and_then(|index| clause.get(index)),
    ) {
        (Some(owner), Some(distractor)) => owner.word != distractor.word,
        _ => false,
    }
}

fn syntax(clauses: &[&[Token<'_>]]) -> Result<(), Refusal> {
    // `boundaries` is the only caller's producer and supplies five nonempty
    // clauses. Indexing here does not depend on a caller-provided clause array.
    let Some(form) = FACT_FORMS
        .iter()
        .find(|form| matches_clause(clauses[0], form))
    else {
        return Err(Refusal::at("UNSUPPORTED_SYNTAX", clauses[0][0].start));
    };
    for clause in &clauses[1..4] {
        if !matches_clause(clause, form) {
            return Err(Refusal::at("UNSUPPORTED_SYNTAX", clause[0].start));
        }
    }
    if !matches_clause(clauses[4], QUERY_FORM) {
        return Err(Refusal::at("UNSUPPORTED_SYNTAX", clauses[4][0].start));
    }
    Ok(())
}

fn derived_input_sha256(inputs: &[[i64; 13]; 5], lengths: &[usize; 5]) -> String {
    let mut digest = Sha256::new();
    let policy_sha256 = policy_sha256();
    for value in [
        "uor-r4.text-to-clauses-input/1",
        policy_sha256.as_str(),
        VOCABULARY_FILE_CID,
        "i64le",
    ] {
        digest.update((value.len() as u32).to_le_bytes());
        digest.update(value.as_bytes());
    }
    for dimension in [1u32, 5, 13, 1, 5] {
        digest.update(dimension.to_le_bytes());
    }
    for clause in inputs {
        for id in clause {
            digest.update(id.to_le_bytes());
        }
    }
    for length in lengths {
        digest.update((*length as i64).to_le_bytes());
    }
    hex::encode(digest.finalize())
}

/// Parse raw bytes after the wrapper has accepted startup and request schema.
pub(super) fn parse(raw: &[u8]) -> Result<Parsed, Refusal> {
    if raw.len() > MAX_BYTES {
        return Err(Refusal::at("INPUT_LIMIT", MAX_BYTES));
    }
    let tokens = lex(raw)?;
    let clauses = boundaries(&tokens, raw.len())?;
    for clause in &clauses {
        if let Some(token) = clause.get(MAX_TOKENS) {
            return Err(Refusal::at("INPUT_LIMIT", token.start));
        }
    }
    syntax(&clauses)?;

    let mut inputs = [[PAD_ID; 13]; 5];
    let mut lengths = [0; 5];
    let mut token_spans = Vec::with_capacity(5);
    let mut clause_spans = Vec::with_capacity(5);
    for (index, clause) in clauses.iter().enumerate() {
        lengths[index] = clause.len();
        let mut spans = Vec::with_capacity(clause.len());
        for (position, token) in clause.iter().enumerate() {
            inputs[index][position] = token.id;
            spans.push([token.start, token.end]);
        }
        token_spans.push(spans);
        clause_spans.push([clause[0].start, clause[clause.len() - 1].end]);
    }
    Ok(Parsed {
        raw_text_sha256: hex::encode(Sha256::digest(raw)),
        derived_input_sha256: derived_input_sha256(&inputs, &lengths),
        inputs,
        lengths,
        token_spans,
        clause_spans,
    })
}
