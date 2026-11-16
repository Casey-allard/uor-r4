//! Pair rows of a served stack (claude-384): the address rule, the dictionary, the turn scan, the
//! per-turn picks, the rows file and the served bag read, in plain data.
//!
//! The float trainer (`uor-r4-float-train`, `pair_rows.rs`, claude-370) learns one row per pair
//! of different tokens of a user turn. This module is the one place the pair rule lives: the
//! trainer and the served integer stack both pick rows through [`open_picks`] (the served stack
//! through [`PairTracker`], which gives the same picks token by token), so a served reply reads
//! the rows its trained twin read.
//!
//! 1. **Address.** Token `t` stands for the prime `primes[t]`, the first `vocab` primes in order.
//!    The unordered pair of different tokens `a`, `b` has the address `primes[a] * primes[b]`; by
//!    unique factorisation no two pairs share one. Addresses stay under 2^31.
//! 2. **Dictionary.** The addresses that have rows, ascending: row `r` is `addresses[r]`.
//! 3. **Picks.** The rows of the user turn whose reply holds the last position (see
//!    [`open_picks`]).
//! 4. **Read.** The served bag adds the sum of the picked rows to the residual ([`PairRowsImage::
//!    add_bag`]): a gather and an add, no product of a state with a table.
//!
//! The module is shared by both builds of this crate.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Exponent of a stored row value: rows are Q12, steps of 2^-12.
pub const PAIR_ROW_EXP: i32 = -12;
/// Left shift that takes a Q12 row value to the residual's 2^-16 grid: `-12 - (-16)`.
pub const PAIR_ROW_SHIFT: u32 = 4;
/// Widest trunk a rows file serves.
pub const PAIR_ROWS_MAX_WIDTH: usize = 4_096;
/// The key of a snapshot's pair-rows note.
pub const FBS_PAIR_ROWS_KEY: &str = "pair_rows";

/// Bound every pair address stays under.
const LIMIT: u64 = 1 << 31;
/// The first bytes of a dictionary file (the float trainer's checkpoints carry this format).
const DICT_MAGIC: &[u8; 8] = b"UORPAIR1";
/// The first bytes of a rows file.
const ROWS_MAGIC: &[u8; 8] = b"UORPRW01";

/// The first `count` primes in order, or fewer: generation stops once the last two multiply to
/// 2^31 or more, since no pair address may reach it.
pub fn first_primes(count: usize) -> Vec<u32> {
    let mut primes: Vec<u32> = Vec::new();
    let mut candidate = 2u32;
    while primes.len() < count {
        if let [.., a, b] = primes[..] {
            if u64::from(a) * u64::from(b) >= LIMIT {
                break;
            }
        }
        let prime = primes
            .iter()
            .take_while(|&&p| u64::from(p) * u64::from(p) <= u64::from(candidate))
            .all(|&p| candidate % p != 0);
        if prime {
            primes.push(candidate);
        }
        candidate += 1;
    }
    primes
}

/// The first `vocab` primes in order, one per token. Refuses fewer than two tokens and a
/// vocabulary whose two largest primes multiply to 2^31 or more.
pub fn token_primes(vocab: usize) -> Result<Vec<u32>, String> {
    if vocab < 2 {
        return Err(format!(
            "pair rows want a vocabulary of at least 2 tokens, not {vocab}"
        ));
    }
    let primes = first_primes(vocab);
    let last = primes.len();
    let top = u64::from(primes[last - 1]) * u64::from(primes[last - 2]);
    if last < vocab {
        return Err(format!(
            "pair rows hold a pair address in 31 bits; a vocabulary of {vocab} tokens has an \
             address of at least {top}"
        ));
    }
    if top >= LIMIT {
        return Err(format!(
            "pair rows hold a pair address in 31 bits; a vocabulary of {vocab} tokens has the \
             address {top}"
        ));
    }
    Ok(primes)
}

/// The token sequences that open a user turn and an assistant reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairMarks {
    /// The user marker: a turn's tokens start after its last token.
    pub user: Vec<u32>,
    /// The assistant header: a turn's tokens stop before its first token, and the reply starts
    /// at its last token.
    pub assistant: Vec<u32>,
}

impl PairMarks {
    /// Refuses an empty mark, two marks one of which begins the other, and an assistant header
    /// that holds the user marker, or a beginning of it running to its end, after its first token.
    pub fn check(&self) -> Result<(), String> {
        if self.user.is_empty() || self.assistant.is_empty() {
            return Err(
                "pair rows want a user marker and an assistant header of at least one token"
                    .to_owned(),
            );
        }
        // A turn is found by the user marker first, so a user marker that begins the header
        // would hide every header. The other order is refused too: the marks stay unambiguous.
        if self.user.starts_with(&self.assistant) || self.assistant.starts_with(&self.user) {
            return Err(
                "pair rows want a user marker and an assistant header neither of which \
                        begins the other"
                    .to_owned(),
            );
        }
        // A served reply holds back while the stream ends with a beginning of the user marker
        // ([`open_picks`]), and a whole user marker opens a turn. A header holding the user
        // marker, or a beginning of it that runs to the header's end, after its first token would
        // do either inside a header the scan takes whole, where the trainer's reply runs on
        // (claude-384's review).
        let overlap = (1..self.assistant.len()).any(|i| {
            let tail = &self.assistant[i..];
            let n = tail.len().min(self.user.len());
            tail[..n] == self.user[..n]
        });
        if overlap {
            return Err(
                "pair rows want an assistant header that holds no beginning of the user marker \
                 after its first token"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

/// One user turn found in a token run, by position in that run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairTurn {
    /// The first position after the user marker.
    pub start: usize,
    /// The assistant header's first position: the turn's tokens are `start..header`.
    pub header: usize,
    /// The assistant header's last position: the first reply position.
    pub reply: usize,
}

/// The user turns of `tokens` whose user marker and assistant header both lie inside it, and the
/// start of every user marker, ascending. A second user marker before the header starts the turn
/// again. The scan is greedy from the left: a mark found at `i` is skipped over whole. Checked
/// marks are wanted ([`PairMarks::check`]); with an empty mark the scan finds nothing.
pub fn scan(tokens: &[u32], marks: &PairMarks) -> (Vec<PairTurn>, Vec<usize>) {
    let (mut turns, mut users) = (Vec::new(), Vec::new());
    if marks.user.is_empty() || marks.assistant.is_empty() {
        return (turns, users);
    }
    let at = |i: usize, mark: &[u32]| tokens[i..].starts_with(mark);
    let mut open: Option<usize> = None;
    let mut i = 0;
    while i < tokens.len() {
        if at(i, &marks.user) {
            users.push(i);
            i += marks.user.len();
            open = Some(i);
        } else if at(i, &marks.assistant) {
            if let Some(start) = open.take() {
                turns.push(PairTurn {
                    start,
                    header: i,
                    reply: i + marks.assistant.len() - 1,
                });
            }
            i += marks.assistant.len();
        } else {
            i += 1;
        }
    }
    (turns, users)
}

/// The turn's different tokens, sorted.
fn distinct(turn: &[u32]) -> Vec<u32> {
    turn.iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The pairs that have rows, and the row each address names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairDictionary {
    primes: Vec<u32>,
    /// The kept addresses, ascending: row `r` is `addresses[r]`.
    addresses: Vec<u32>,
}

impl PairDictionary {
    /// The dictionary of `addresses` over a vocabulary of `vocab` tokens. Refuses a vocabulary
    /// [`token_primes`] refuses and addresses that are not strictly ascending.
    pub fn from_addresses(vocab: usize, addresses: Vec<u32>) -> Result<Self, String> {
        if addresses.windows(2).any(|w| w[0] >= w[1]) {
            return Err("a pair dictionary's addresses are not ascending".to_owned());
        }
        Ok(Self {
            primes: token_primes(vocab)?,
            addresses,
        })
    }

    /// Count the pairs of the user turns in `tokens` and keep those seen in at least `min` turns.
    /// A pair counts once per turn. Refuses a turn token outside the vocabulary.
    pub fn count(
        vocab: usize,
        tokens: &[u32],
        marks: &PairMarks,
        min: u32,
    ) -> Result<Self, String> {
        marks.check()?;
        let primes = token_primes(vocab)?;
        let (turns, _) = scan(tokens, marks);
        let mut seen: HashMap<u32, u32> = HashMap::new();
        for turn in &turns {
            let bag = distinct(&tokens[turn.start..turn.header]);
            if let Some(&token) = bag.last().filter(|&&t| t as usize >= vocab) {
                return Err(format!(
                    "a turn token {token} lies outside the vocabulary of {vocab}"
                ));
            }
            for (n, &a) in bag.iter().enumerate() {
                for &b in &bag[n + 1..] {
                    *seen
                        .entry(primes[a as usize] * primes[b as usize])
                        .or_default() += 1;
                }
            }
        }
        let mut addresses: Vec<u32> = seen
            .into_iter()
            .filter(|&(_, turns)| turns >= min)
            .map(|(address, _)| address)
            .collect();
        addresses.sort_unstable();
        Ok(Self { primes, addresses })
    }

    /// The vocabulary the addresses are drawn from.
    pub fn vocab(&self) -> usize {
        self.primes.len()
    }

    /// The number of rows.
    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    /// Whether no pair has a row.
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }

    /// The kept addresses, ascending.
    pub fn addresses(&self) -> &[u32] {
        &self.addresses
    }

    /// The address of the unordered pair `a`, `b`; none for equal tokens or a token outside the
    /// vocabulary.
    pub fn address(&self, a: u32, b: u32) -> Option<u32> {
        if a == b {
            return None;
        }
        let (pa, pb) = (self.primes.get(a as usize)?, self.primes.get(b as usize)?);
        Some(pa * pb)
    }

    /// The row of the pair `a`, `b`, if it has one.
    pub fn row(&self, a: u32, b: u32) -> Option<u32> {
        let address = self.address(a, b)?;
        self.addresses
            .binary_search(&address)
            .ok()
            .map(|r| r as u32)
    }

    /// The rows of every pair of the turn's tokens that has one, sorted and unique.
    pub fn turn_rows(&self, turn: &[u32]) -> Vec<u32> {
        let bag = distinct(turn);
        let mut rows = Vec::new();
        for (n, &a) in bag.iter().enumerate() {
            for &b in &bag[n + 1..] {
                rows.extend(self.row(a, b));
            }
        }
        rows.sort_unstable();
        rows.dedup();
        rows
    }

    /// The dictionary as bytes: the magic `UORPAIR1`, the vocabulary, the row count and the
    /// addresses, each a little-endian u32.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16 + 4 * self.addresses.len());
        bytes.extend_from_slice(DICT_MAGIC);
        bytes.extend_from_slice(&(self.primes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.addresses.len() as u32).to_le_bytes());
        for address in &self.addresses {
            bytes.extend_from_slice(&address.to_le_bytes());
        }
        bytes
    }

    /// The dictionary [`Self::to_bytes`] wrote. Refuses another magic, a length that is not the
    /// one the row count names, and addresses out of order.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let word =
            |n: usize| u32::from_le_bytes([bytes[n], bytes[n + 1], bytes[n + 2], bytes[n + 3]]);
        if bytes.len() < 16 || &bytes[..8] != DICT_MAGIC {
            return Err("not a pair dictionary: no magic".to_owned());
        }
        let (vocab, count) = (word(8) as usize, word(12) as usize);
        let want = count.checked_mul(4).and_then(|n| n.checked_add(16));
        if want != Some(bytes.len()) {
            return Err(format!(
                "a pair dictionary of {count} rows wants 16 + 4 * {count} bytes, not {}",
                bytes.len()
            ));
        }
        let addresses: Vec<u32> = (0..count).map(|r| word(16 + 4 * r)).collect();
        Self::from_addresses(vocab, addresses)
    }
}

/// The turn whose reply reaches the last position of `tokens` by the marks seen so far, as its
/// reply position and its rows (non-empty), or none: the float trainer's `plan` rule for one
/// window of `tokens.len()` tokens, read at the window's last position. A turn counts only when
/// its user marker and its assistant header both lie inside `tokens`; its reply runs from the
/// header's last token through the position before the first user marker that starts after that
/// token, or through the last position when no marker follows. None when no turn's reply reaches
/// the last position, when that turn has no rows, and when the marks fail [`PairMarks::check`].
///
/// Only the last turn of the scan can be open: any later turn starts with a user marker after an
/// earlier turn's reply, which closes it.
fn open_turn(
    dict: &PairDictionary,
    marks: &PairMarks,
    tokens: &[u32],
) -> Option<(usize, Vec<u32>)> {
    marks.check().ok()?;
    let (turns, users) = scan(tokens, marks);
    let turn = turns.last()?;
    if users.last().is_some_and(|&u| u > turn.reply) {
        return None;
    }
    let rows = dict.turn_rows(&tokens[turn.start..turn.header]);
    (!rows.is_empty()).then_some((turn.reply, rows))
}

/// Whether `tokens` ends with a non-empty proper beginning of the user marker that starts after
/// the reply position `reply`: a user marker that may be under way.
fn user_marker_under_way(marks: &PairMarks, tokens: &[u32], reply: usize) -> bool {
    (1..marks.user.len()).any(|k| {
        k <= tokens.len() && tokens.len() - k > reply && tokens.ends_with(&marks.user[..k])
    })
}

/// The rows the reply at the last position of `tokens` reads, or none.
///
/// The float trainer's `plan` ends a reply at the first position of the next user marker, a rule
/// that reads ahead: inside a window the marker's first tokens add nothing because the tokens
/// after them finish the marker. A reader fed token by token sees a marker only once it is
/// finished. So the reply also adds nothing while the stream ends with a non-empty proper
/// beginning of the user marker that starts after the reply (claude-384's review). This equals
/// the trainer's rule, read at the same position of a longer window, at every position where a
/// user marker that has begun is finished in that window: every position of a prompt whose user
/// markers are whole. It differs only where a marker's beginning is not followed by the rest of
/// the marker; there the trainer adds the rows and this rule does not.
///
/// The picks are the open turn's [`PairDictionary::turn_rows`]: every pair of different tokens of
/// the turn that the dictionary holds, sorted and unique. None when no turn's reply reaches the
/// last position, when that turn has no rows, when a user marker may be under way, and when the
/// marks fail [`PairMarks::check`].
pub fn open_picks(dict: &PairDictionary, marks: &PairMarks, tokens: &[u32]) -> Option<Vec<u32>> {
    let (reply, rows) = open_turn(dict, marks, tokens)?;
    (!user_marker_under_way(marks, tokens, reply)).then_some(rows)
}

/// The picks of a served session, kept as tokens arrive.
///
/// [`Self::push`] gives [`open_picks`] of the stream so far. It keeps the open turn
/// ([`open_turn`]) and recomputes it only when the stream ends with the user marker or the
/// assistant header (or did not grow by exactly one token since the last push); the user marker
/// under way is tested at every push.
///
/// Why only a mark completed at the end can change the open turn: let `P` be the stream before
/// the push and `P t` after. A mark that matches at a position `i` of `P` also matches there in
/// `P t`. A mark that matches at `i` in `P t` but not in `P` must run into the new last token, so
/// `P t` ends with it. So when `P t` ends with neither mark, the match test is the same at every
/// position of both streams (and no match starts at the new position), and the greedy scan finds
/// the same turns and the same user markers in both. The last turn is then the same, with the
/// same reply position, tokens and rows; it is open in `P t` exactly when no user marker follows
/// its reply, as in `P`, and an open reply now reaches the new last position.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PairTracker {
    /// Tokens seen at the last push.
    len: usize,
    /// The open turn at the last push: its reply position and rows.
    open: Option<(usize, Vec<u32>)>,
    /// Whether a user marker was under way at the last push.
    held: bool,
}

impl PairTracker {
    /// A tracker for an empty stream: no picks.
    pub fn new() -> Self {
        Self::default()
    }

    /// The picks at the last position of `tokens`, the stream so far including the new token.
    pub fn push(
        &mut self,
        dict: &PairDictionary,
        marks: &PairMarks,
        tokens: &[u32],
    ) -> Option<&[u32]> {
        let grew = tokens.len() == self.len + 1;
        if !grew || tokens.ends_with(&marks.user) || tokens.ends_with(&marks.assistant) {
            self.open = open_turn(dict, marks, tokens);
        }
        self.held = self
            .open
            .as_ref()
            .is_some_and(|(reply, _)| user_marker_under_way(marks, tokens, *reply));
        self.len = tokens.len();
        self.picks()
    }

    /// The picks at the last push.
    pub fn picks(&self) -> Option<&[u32]> {
        match &self.open {
            Some((_, rows)) if !self.held => Some(rows),
            _ => None,
        }
    }
}

/// Round `value` to the Q12 grid (steps of 2^-12), half away from zero. Refuses a value that is
/// not finite or whose steps lie outside i16.
pub fn quantize_q12(value: f32) -> Result<i16, String> {
    if !value.is_finite() {
        return Err(format!("a pair row value {value} is not finite"));
    }
    let steps = (f64::from(value) * 4096.0).round();
    if steps < f64::from(i16::MIN) || steps > f64::from(i16::MAX) {
        return Err(format!(
            "a pair row value {value} is {steps} steps of 2^-12, outside i16"
        ));
    }
    Ok(steps as i16)
}

/// What a snapshot says of the pair-rows file written beside it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FbsPairRowsNote {
    /// The rows' width.
    pub width: usize,
    /// The number of pairs with a row.
    pub pairs: usize,
    /// The vocabulary the addresses are drawn from.
    pub vocab: usize,
    /// SHA-256 of the rows file, in hex.
    pub sha256: String,
}

impl FbsPairRowsNote {
    /// The table the rows file `bytes` holds, refused unless it is the file this note names: the
    /// same bytes by SHA-256, and the same width, pair count and vocabulary.
    pub fn rows(&self, bytes: &[u8]) -> Result<PairRowsImage, String> {
        let sum = hex::encode(Sha256::digest(bytes));
        if sum != self.sha256 {
            return Err(format!(
                "the pair rows file's SHA-256 is {sum}; the snapshot names {}",
                self.sha256
            ));
        }
        let image = PairRowsImage::from_bytes(bytes)?;
        if image.width() != self.width
            || image.dict().len() != self.pairs
            || image.dict().vocab() != self.vocab
        {
            return Err("the pair rows file is not the table the snapshot describes".to_owned());
        }
        Ok(image)
    }
}

/// The pair rows of a served stack: the dictionary, the marks, and one Q12 row per pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairRowsImage {
    dict: PairDictionary,
    width: usize,
    marks: PairMarks,
    /// Row `r` is `rows[r * width..(r + 1) * width]`.
    rows: Vec<i16>,
}

impl PairRowsImage {
    /// The rows `rows` (pair-major, `addresses.len() * width` values) of the pairs `addresses`.
    /// Refuses checked marks' faults, a vocabulary [`token_primes`] refuses, a width that is zero,
    /// not a multiple of 8 or over [`PAIR_ROWS_MAX_WIDTH`], addresses that are not strictly
    /// ascending, and a row count that is not the one the addresses and width name. An address is
    /// held under 2^31 by its type's use but is not factored here: one that names no pair is never
    /// picked.
    pub fn checked(
        vocab: usize,
        width: usize,
        marks: PairMarks,
        addresses: Vec<u32>,
        rows: Vec<i16>,
    ) -> Result<Self, String> {
        marks.check()?;
        if width == 0 || width % 8 != 0 || width > PAIR_ROWS_MAX_WIDTH {
            return Err(format!(
                "pair rows want a width of 8 to {PAIR_ROWS_MAX_WIDTH} in steps of 8, not {width}"
            ));
        }
        if let Some(&address) = addresses.iter().find(|&&a| u64::from(a) >= LIMIT) {
            return Err(format!("a pair address {address} is not under 2^31"));
        }
        let want = addresses.len().checked_mul(width);
        if want != Some(rows.len()) {
            return Err(format!(
                "{} pairs of width {width} want {} values, not {}",
                addresses.len(),
                want.map_or_else(|| "too many".to_owned(), |n| n.to_string()),
                rows.len()
            ));
        }
        let dict = PairDictionary::from_addresses(vocab, addresses)?;
        Ok(Self {
            dict,
            width,
            marks,
            rows,
        })
    }

    /// The dictionary.
    pub fn dict(&self) -> &PairDictionary {
        &self.dict
    }

    /// The marks.
    pub fn marks(&self) -> &PairMarks {
        &self.marks
    }

    /// The rows' width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Row `r`, Q12.
    pub fn row(&self, r: usize) -> &[i16] {
        &self.rows[r * self.width..(r + 1) * self.width]
    }

    /// The rows file: the magic `UORPRW01`; then little-endian 32-bit words: the vocabulary, the
    /// width, the row exponent (signed), the user marker's length, the assistant header's length,
    /// the user marker's ids, the header's ids and the pair count; then the addresses, u32 LE; then
    /// the rows, i16 LE, pair-major.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(ROWS_MAGIC);
        let mut word = |w: u32| bytes.extend_from_slice(&w.to_le_bytes());
        word(self.dict.vocab() as u32);
        word(self.width as u32);
        word(PAIR_ROW_EXP as u32);
        word(self.marks.user.len() as u32);
        word(self.marks.assistant.len() as u32);
        for &id in self.marks.user.iter().chain(&self.marks.assistant) {
            word(id);
        }
        word(self.dict.len() as u32);
        for &address in self.dict.addresses() {
            word(address);
        }
        for value in &self.rows {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    /// The table [`Self::to_bytes`] wrote. Refuses another magic or row exponent, a length that
    /// is not the one the header names, and every fault [`Self::checked`] refuses.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 || &bytes[..8] != ROWS_MAGIC {
            return Err("not a pair rows file: no magic".to_owned());
        }
        let short = || "a pair rows file ends inside its header".to_owned();
        let mut at = 8usize;
        let word = |at: &mut usize| -> Result<u32, String> {
            let n = *at;
            let end = n
                .checked_add(4)
                .filter(|&e| e <= bytes.len())
                .ok_or_else(short)?;
            *at = end;
            Ok(u32::from_le_bytes([
                bytes[n],
                bytes[n + 1],
                bytes[n + 2],
                bytes[n + 3],
            ]))
        };
        let vocab = word(&mut at)? as usize;
        let width = word(&mut at)? as usize;
        let exp = word(&mut at)? as i32;
        if exp != PAIR_ROW_EXP {
            return Err(format!(
                "a pair rows file of row exponent {exp}, not {PAIR_ROW_EXP}"
            ));
        }
        let (user_len, assistant_len) = (word(&mut at)? as usize, word(&mut at)? as usize);
        let marks_len = user_len.checked_add(assistant_len).ok_or_else(short)?;
        if marks_len > bytes.len() / 4 {
            return Err(short());
        }
        let mut ids = Vec::with_capacity(marks_len);
        for _ in 0..marks_len {
            ids.push(word(&mut at)?);
        }
        let assistant = ids.split_off(user_len);
        let marks = PairMarks {
            user: ids,
            assistant,
        };
        let count = word(&mut at)? as usize;
        let want = count
            .checked_mul(4)
            .and_then(|a| count.checked_mul(width)?.checked_mul(2)?.checked_add(a))
            .and_then(|n| n.checked_add(at));
        if want != Some(bytes.len()) {
            return Err(format!(
                "a pair rows file of {count} pairs of width {width} has the wrong length {}",
                bytes.len()
            ));
        }
        let mut addresses = Vec::with_capacity(count);
        for _ in 0..count {
            addresses.push(word(&mut at)?);
        }
        let rows = bytes[at..]
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();
        Self::checked(vocab, width, marks, addresses, rows)
    }

    /// SHA-256 of the rows file, in hex.
    pub fn sha256(&self) -> String {
        hex::encode(Sha256::digest(self.to_bytes()))
    }

    /// The note a snapshot carries for this table's rows file.
    pub fn note(&self) -> FbsPairRowsNote {
        FbsPairRowsNote {
            width: self.width,
            pairs: self.dict.len(),
            vocab: self.dict.vocab(),
            sha256: self.sha256(),
        }
    }

    /// The served bag read: `sum` becomes the sum of the picked rows at 2^-12, then each
    /// `x[i]` (the residual at 2^-16) adds `sum[i] << 4`.
    ///
    /// Bounds: a row value is at most 2^15 steps in size, so up to 2^16 picks sum inside i32 and
    /// up to 2^12 picks keep `sum << 4` inside i32. Past those the sum, the shift and the add
    /// saturate (the shift is a saturating multiply by 16, equal to `<< 4` wherever that does not
    /// overflow) rather than wrap. A turn's picks are its distinct pairs that have rows.
    ///
    /// Panics on a broken caller: a pick that is no row of the table, or `x` or `sum` not
    /// `width` long.
    pub fn add_bag(&self, picks: &[u32], x: &mut [i32], sum: &mut [i32]) {
        assert_eq!(
            x.len(),
            self.width,
            "the residual is not the pair rows' width"
        );
        assert_eq!(
            sum.len(),
            self.width,
            "the sum buffer is not the pair rows' width"
        );
        sum.fill(0);
        for &pick in picks {
            let r = pick as usize;
            assert!(
                r < self.dict.len(),
                "pick {pick} is no pair row of {}",
                self.dict.len()
            );
            for (s, &v) in sum.iter_mut().zip(self.row(r)) {
                *s = s.saturating_add(i32::from(v));
            }
        }
        for (xi, &s) in x.iter_mut().zip(sum.iter()) {
            *xi = xi.saturating_add(s.saturating_mul(1 << PAIR_ROW_SHIFT));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VOCAB: usize = 12;

    fn marks(user: &[u32], assistant: &[u32]) -> PairMarks {
        PairMarks {
            user: user.to_vec(),
            assistant: assistant.to_vec(),
        }
    }

    /// Every pair of the vocabulary, less `drop`.
    fn all_pairs(vocab: usize, drop: &[(u32, u32)]) -> PairDictionary {
        let primes = token_primes(vocab).unwrap();
        let dropped: Vec<u32> = drop
            .iter()
            .map(|&(a, b)| primes[a as usize] * primes[b as usize])
            .collect();
        let mut addresses = Vec::new();
        for a in 0..vocab {
            for b in a + 1..vocab {
                let address = primes[a] * primes[b];
                if !dropped.contains(&address) {
                    addresses.push(address);
                }
            }
        }
        addresses.sort_unstable();
        PairDictionary::from_addresses(vocab, addresses).unwrap()
    }

    /// The float trainer's `plan` rule for one window of `tokens.len()` tokens, read directly:
    /// every turn with rows gets a span from its reply to the next user marker after the reply or
    /// the window's end; the open turn is the one whose span reaches the end.
    fn float_rule(dict: &PairDictionary, marks: &PairMarks, tokens: &[u32]) -> Option<Vec<u32>> {
        let time = tokens.len();
        let (turns, users) = scan(tokens, marks);
        let mut open = None;
        for turn in turns {
            let rows = dict.turn_rows(&tokens[turn.start..turn.header]);
            if rows.is_empty() {
                continue;
            }
            let end = users
                .iter()
                .copied()
                .find(|&u| u > turn.reply)
                .unwrap_or(time);
            if end == time {
                open = Some(rows);
            }
        }
        open
    }

    fn rows_of(dict: &PairDictionary, pairs: &[(u32, u32)]) -> Vec<u32> {
        let mut rows: Vec<u32> = pairs
            .iter()
            .map(|&(a, b)| dict.row(a, b).unwrap())
            .collect();
        rows.sort_unstable();
        rows
    }

    #[test]
    fn open_picks_reads_the_float_rule() {
        let dict = all_pairs(VOCAB, &[]);
        let one = marks(&[1], &[2]);
        let cases: Vec<(&[u32], Option<Vec<u32>>)> = vec![
            // One turn, reply open to the end.
            (&[1, 5, 6, 2, 9, 9], Some(rows_of(&dict, &[(5, 6)]))),
            // Two turns: the second is open.
            (
                &[1, 5, 6, 2, 9, 1, 7, 8, 2, 9],
                Some(rows_of(&dict, &[(7, 8)])),
            ),
            // The marker lies before the window: no turn.
            (&[5, 6, 2, 9], None),
            // A second user marker before the header starts the turn again.
            (&[1, 5, 6, 1, 7, 8, 2, 9], Some(rows_of(&dict, &[(7, 8)]))),
            // Repeated tokens count once; the last position is the header itself.
            (&[1, 5, 5, 6, 6, 5, 2], Some(rows_of(&dict, &[(5, 6)]))),
            // A one-token turn has no pair.
            (&[1, 5, 2, 9], None),
            // The next user marker ends the reply.
            (&[1, 5, 6, 2, 9, 1, 7], None),
            (&[1, 5, 6, 2, 9, 1], None),
            // A later turn without rows still closes the earlier reply.
            (&[1, 5, 6, 2, 9, 1, 7, 2, 9], None),
            // A header with no open turn opens nothing; the earlier reply runs on.
            (&[1, 5, 6, 2, 9, 2, 9], Some(rows_of(&dict, &[(5, 6)]))),
            (&[], None),
        ];
        for (tokens, want) in &cases {
            assert_eq!(&open_picks(&dict, &one, tokens), want, "{tokens:?}");
            assert_eq!(&float_rule(&dict, &one, tokens), want, "{tokens:?}");
        }
        // A pair outside the dictionary has no row.
        let holed = all_pairs(VOCAB, &[(5, 6)]);
        assert_eq!(open_picks(&holed, &one, &[1, 5, 6, 2, 9]), None);
        assert_eq!(
            open_picks(&holed, &one, &[1, 5, 6, 7, 2]),
            Some(rows_of(&holed, &[(5, 7), (6, 7)]))
        );
        assert_eq!(
            float_rule(&holed, &one, &[1, 5, 6, 7, 2]),
            open_picks(&holed, &one, &[1, 5, 6, 7, 2])
        );
        // Multi-token marks.
        let two = marks(&[1, 3], &[2, 4]);
        let cases: Vec<(&[u32], Option<Vec<u32>>)> = vec![
            (&[1, 3, 5, 6, 2, 4, 9], Some(rows_of(&dict, &[(5, 6)]))),
            (&[1, 3, 5, 6, 2, 4], Some(rows_of(&dict, &[(5, 6)]))),
            (&[1, 3, 5, 6, 2], None),
            (&[1, 5, 6, 2, 4, 9], None),
            (&[1, 3, 5, 6, 2, 4, 9, 1, 3], None),
            (
                &[1, 3, 5, 6, 2, 4, 9, 1, 3, 7, 8, 2, 4],
                Some(rows_of(&dict, &[(7, 8)])),
            ),
        ];
        for (tokens, want) in &cases {
            assert_eq!(&open_picks(&dict, &two, tokens), want, "{tokens:?}");
            assert_eq!(&float_rule(&dict, &two, tokens), want, "{tokens:?}");
        }
        // A user marker under way closes the reply: in the trainer's window the rest of the marker
        // follows, so the trainer adds nothing at its first token either.
        let under_way: &[u32] = &[1, 3, 5, 6, 2, 4, 9, 1];
        assert_eq!(open_picks(&dict, &two, under_way), None);
        assert_eq!(
            float_rule(&dict, &two, &[1, 3, 5, 6, 2, 4, 9, 1, 3]),
            None,
            "the trainer's window with the marker finished"
        );
        // A beginning the next token breaks: the reply reads again (the trainer read it all along).
        assert_eq!(
            open_picks(&dict, &two, &[1, 3, 5, 6, 2, 4, 9, 1, 9]),
            Some(rows_of(&dict, &[(5, 6)]))
        );
        // A header holding a beginning of the user marker after its first token is refused: the hold
        // would fire inside a header the scan takes whole, where the trainer's reply runs on.
        for overlap in [
            marks(&[4, 7], &[2, 4]),
            marks(&[1, 2], &[2, 1]),
            marks(&[3, 5, 6], &[2, 3, 5]),
            marks(&[1, 9], &[2, 1, 9, 3]),
        ] {
            assert!(overlap.check().is_err(), "{overlap:?}");
            assert_eq!(open_picks(&dict, &overlap, &[4, 7, 5, 6, 2, 4]), None);
        }
        // A beginning inside the header that the header itself breaks is not refused.
        assert!(marks(&[1, 9], &[2, 1, 3, 4]).check().is_ok());
        // Marks that fail the check pick nothing.
        assert_eq!(open_picks(&dict, &marks(&[], &[2]), &[1, 5, 6, 2]), None);
    }

    /// A small linear congruential generator.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self, n: u32) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) % u64::from(n)) as u32
        }
    }

    #[test]
    fn tracker_equals_open_picks_of_every_prefix() {
        let vocab = 8;
        let primes = token_primes(vocab).unwrap();
        // Every third pair dropped, so some turns have no rows.
        let mut addresses = Vec::new();
        for a in 0..vocab {
            for b in a + 1..vocab {
                if (a + 2 * b) % 3 != 0 {
                    addresses.push(primes[a] * primes[b]);
                }
            }
        }
        addresses.sort_unstable();
        let dict = PairDictionary::from_addresses(vocab, addresses).unwrap();
        let mark_sets = [
            marks(&[1], &[2]),
            marks(&[1, 3], &[2, 4]),
            marks(&[1, 1], &[2]),
            marks(&[1, 2, 1], &[2, 3]),
            marks(&[1, 4, 5], &[2, 3]),
        ];
        let mut rng = Lcg(0x5eed);
        let mut checked = 0usize;
        for set in &mark_sets {
            set.check().unwrap();
            for _ in 0..200 {
                let len = 20 + rng.next(60) as usize;
                // Mark tokens come often so turns open, close and restart.
                let tokens: Vec<u32> = (0..len)
                    .map(|_| {
                        if rng.next(3) == 0 {
                            1 + rng.next(4)
                        } else {
                            rng.next(vocab as u32)
                        }
                    })
                    .collect();
                let mut tracker = PairTracker::new();
                for end in 1..=tokens.len() {
                    let prefix = &tokens[..end];
                    let got = tracker.push(&dict, set, prefix).map(<[u32]>::to_vec);
                    assert_eq!(got, open_picks(&dict, set, prefix), "{set:?} {prefix:?}");
                    checked += usize::from(got.is_some());
                }
            }
        }
        assert!(
            checked > 1_000,
            "the streams open too few replies: {checked}"
        );
    }

    fn image() -> PairRowsImage {
        let dict = all_pairs(VOCAB, &[(3, 4)]);
        let width = 16;
        let rows = (0..dict.len() * width)
            .map(|n| (n as i16).wrapping_mul(97).wrapping_sub(3000))
            .collect();
        PairRowsImage::checked(
            VOCAB,
            width,
            marks(&[1, 3], &[2]),
            dict.addresses().to_vec(),
            rows,
        )
        .unwrap()
    }

    #[test]
    fn dictionary_and_rows_round_trip_bit_for_bit() {
        let image = image();
        let bytes = image.dict().to_bytes();
        let dict = PairDictionary::from_bytes(&bytes).unwrap();
        assert_eq!(&dict, image.dict());
        assert_eq!(dict.to_bytes(), bytes);
        let counted =
            PairDictionary::count(VOCAB, &[1, 5, 6, 2, 9, 1, 5, 7, 2], &marks(&[1], &[2]), 1)
                .unwrap();
        assert_eq!(
            PairDictionary::from_bytes(&counted.to_bytes()).unwrap(),
            counted
        );

        let bytes = image.to_bytes();
        let back = PairRowsImage::from_bytes(&bytes).unwrap();
        assert_eq!(back, image);
        assert_eq!(back.to_bytes(), bytes);
        let note = image.note();
        assert_eq!(note.sha256, hex::encode(Sha256::digest(&bytes)));
        assert_eq!(note.rows(&bytes).unwrap(), image);
    }

    #[test]
    fn damaged_files_are_refused() {
        let image = image();
        let bytes = image.to_bytes();
        let mut magic = bytes.clone();
        magic[0] ^= 1;
        assert!(PairRowsImage::from_bytes(&magic)
            .unwrap_err()
            .contains("magic"));
        assert!(PairRowsImage::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(PairRowsImage::from_bytes(&bytes[..20]).is_err());
        let mut longer = bytes.clone();
        longer.extend_from_slice(&[0, 0]);
        assert!(PairRowsImage::from_bytes(&longer).is_err());
        // Swap the first two addresses: header is 8 + 5 words + 3 mark ids + the count.
        let first = 8 + 4 * (5 + 3 + 1);
        let mut unordered = bytes.clone();
        let (a, b) = (
            unordered[first..first + 4].to_vec(),
            unordered[first + 4..first + 8].to_vec(),
        );
        unordered[first..first + 4].copy_from_slice(&b);
        unordered[first + 4..first + 8].copy_from_slice(&a);
        assert!(PairRowsImage::from_bytes(&unordered)
            .unwrap_err()
            .contains("ascending"));
        // The note names another file, or another shape.
        let note = image.note();
        assert!(note.rows(&longer).unwrap_err().contains("SHA-256"));
        let wide = FbsPairRowsNote {
            width: 8,
            ..note.clone()
        };
        assert!(wide.rows(&bytes).is_err());
        let fewer = FbsPairRowsNote {
            pairs: note.pairs - 1,
            ..note
        };
        assert!(fewer.rows(&bytes).is_err());

        let dict = image.dict().to_bytes();
        let mut magic = dict.clone();
        magic[7] ^= 1;
        assert!(PairDictionary::from_bytes(&magic).is_err());
        assert!(PairDictionary::from_bytes(&dict[..dict.len() - 4]).is_err());
        let mut unordered = dict.clone();
        unordered.copy_within(16..20, 20);
        assert!(PairDictionary::from_bytes(&unordered).is_err());

        let rows = |w: usize, n: usize| vec![0i16; w * n];
        let m = marks(&[1], &[2]);
        assert!(PairRowsImage::checked(VOCAB, 12, m.clone(), vec![6, 10], rows(12, 2)).is_err());
        assert!(PairRowsImage::checked(VOCAB, 0, m.clone(), vec![6, 10], vec![]).is_err());
        assert!(PairRowsImage::checked(VOCAB, 8, m.clone(), vec![10, 6], rows(8, 2)).is_err());
        assert!(PairRowsImage::checked(VOCAB, 8, m.clone(), vec![6, 10], rows(8, 1)).is_err());
        assert!(
            PairRowsImage::checked(VOCAB, 8, marks(&[1], &[1, 2]), vec![6], rows(8, 1)).is_err()
        );
        assert!(PairRowsImage::checked(1, 8, m, vec![6], rows(8, 1)).is_err());
    }

    #[test]
    fn add_bag_adds_the_sum_shifted() {
        let zero = PairRowsImage {
            rows: vec![0; image().rows.len()],
            ..image()
        };
        let start: Vec<i32> = (0..16).map(|i| i * 1_000 - 7_000).collect();
        let (mut x, mut sum) = (start.clone(), vec![5; 16]);
        zero.add_bag(&[0, 3, 7], &mut x, &mut sum);
        assert_eq!(x, start);

        let image = image();
        let picks = [0u32, 3, 7];
        let (mut x, mut sum) = (start.clone(), vec![0; 16]);
        image.add_bag(&picks, &mut x, &mut sum);
        for i in 0..16 {
            let want: i32 = picks
                .iter()
                .map(|&r| i32::from(image.row(r as usize)[i]))
                .sum();
            assert_eq!(sum[i], want);
            assert_eq!(x[i], start[i] + (want << 4));
        }
        // Saturates rather than wraps.
        let (mut x, mut sum) = (vec![i32::MAX - 1; 16], vec![0; 16]);
        let full = PairRowsImage {
            rows: vec![i16::MAX; image.rows.len()],
            ..image
        };
        full.add_bag(&[1], &mut x, &mut sum);
        assert!(x.iter().all(|&v| v == i32::MAX));
    }

    #[test]
    fn quantize_q12_rounds_and_refuses() {
        assert_eq!(quantize_q12(0.0), Ok(0));
        assert_eq!(quantize_q12(0.5), Ok(2_048));
        assert_eq!(quantize_q12(-1.25), Ok(-5_120));
        // Half a step rounds away from zero.
        assert_eq!(quantize_q12(1.0 / 8_192.0), Ok(1));
        assert_eq!(quantize_q12(-1.0 / 8_192.0), Ok(-1));
        assert_eq!(quantize_q12(32_767.0 / 4_096.0), Ok(i16::MAX));
        assert_eq!(quantize_q12(-8.0), Ok(i16::MIN));
        assert!(quantize_q12(8.0).is_err());
        assert!(quantize_q12(-8.001).is_err());
        assert!(quantize_q12(f32::NAN).is_err());
        assert!(quantize_q12(f32::INFINITY).is_err());
        assert!(quantize_q12(f32::NEG_INFINITY).is_err());
    }

    #[test]
    fn token_primes_refuse_a_wide_vocabulary() {
        assert_eq!(token_primes(4).unwrap(), vec![2, 3, 5, 7]);
        assert!(token_primes(1).is_err());
        let primes = first_primes(6_000);
        let wide = (2..=primes.len())
            .find(|&v| u64::from(primes[v - 1]) * u64::from(primes[v - 2]) >= LIMIT)
            .unwrap_or(6_000);
        assert!(token_primes(wide - 1).is_ok());
        assert!(token_primes(wide).is_err());
    }
}
