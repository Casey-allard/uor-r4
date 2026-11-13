//! Pair experts (claude-367): exact counts of which replies followed which pairs of turn tokens.
//!
//! Each token `k` of the vocabulary has a prime: the (k+1)-th prime, so token 0 has 2, token 1
//! has 3, token 2 has 5. An expert is an unordered pair of two different tokens of a user turn,
//! and its address is the product of their two primes. Counting a training conversation adds
//! one to an expert's total, and one to its count for the reply, for every pair of the turn.
//!
//! At serving time the experts of the turn vote. An expert seen at most `cap` times gives each
//! reply it holds `(count << 20) / total`. The model's reply stands when its vote keeps a set
//! share of the top vote; otherwise the top reply is given instead, unless the experts' top vote
//! is under a floor: then they do not know the turn, and the reply stands. With hold-out on, a turn's
//! own counts are taken out first, so a turn is checked as if it had never been counted.
//!
//! A table file carries the SHA-256 of its body, and every structural rule is checked again
//! when a file is read.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use sha2::{Digest, Sha256};

use crate::fbs_snapshot::{PlaceMarks, PlaceRole, PlaceTurns};
use crate::place_asks::{Asked, AsksIndex};

/// The first bytes of a table file.
const MAGIC: &[u8; 8] = b"UORPEX01";
/// The file form this module writes.
const VERSION: u32 = 1;
/// Bytes before the body: the magic, nine 32-bit words, the tokens tag and the body's SHA-256.
const HEADER: usize = 8 + 9 * 4 + 32 + 32;
/// Where the body's SHA-256 sits in the header.
const BODY_SUM_AT: usize = HEADER - 32;
/// The shift of a vote's count before it is divided by the expert's total.
const VOTE_SHIFT: u32 = 20;

/// One time a user turn was answered by a reply in training.
#[derive(Clone, Copy, Debug)]
pub struct Occurrence<'a> {
    /// The user turn's tokens.
    pub turn: &'a [u32],
    /// The reply's tokens.
    pub reply: &'a [u32],
}

/// The share of the top vote a model's reply must keep to stand: `num / den`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    /// The share's top part.
    pub num: u32,
    /// The share's bottom part.
    pub den: u32,
}

/// One reply's score in a vote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplyScore {
    /// The reply's index.
    pub reply: u32,
    /// The sum of the experts' shares for it.
    pub score: u64,
}

/// What the experts of one turn gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Votes {
    /// Pairs of the turn that were experts.
    pub experts: usize,
    /// Every reply with a positive score, by reply index.
    pub scores: Vec<ReplyScore>,
}

impl Votes {
    /// The reply with the largest score; a tie goes to the smaller reply index.
    pub fn top(&self) -> Option<ReplyScore> {
        let mut best: Option<ReplyScore> = None;
        for entry in &self.scores {
            if best.map_or(true, |b| entry.score > b.score) {
                best = Some(*entry);
            }
        }
        best
    }

    /// The score of one reply; 0 when it has none.
    pub fn score(&self, reply: u32) -> u64 {
        self.scores
            .binary_search_by_key(&reply, |s| s.reply)
            .map_or(0, |at| self.scores[at].score)
    }
}

/// The verdict on a model's reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// The turn has no expert, or no reply scored: the model's reply stands.
    NoExpert,
    /// The model's reply kept its share of the top vote.
    Stands {
        /// The model's reply's vote.
        vote: u64,
        /// The top vote.
        top: u64,
    },
    /// The experts' top vote is under the floor: they do not know this turn, and the model's
    /// reply stands although it did not keep its share.
    Unsure {
        /// The model's reply's vote.
        vote: u64,
        /// The top vote.
        top: u64,
    },
    /// The model's reply is replaced by the experts' top reply.
    Replaced {
        /// The top reply's index.
        reply: u32,
        /// The model's reply's vote.
        vote: u64,
        /// The top vote.
        top: u64,
    },
}

/// How often one reply was seen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Count {
    reply: u32,
    count: u32,
}

/// One exact training turn and the replies it got.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Turn {
    tokens: Vec<u32>,
    counts: Vec<Count>,
}

/// One pair of tokens: its address, how often it was seen and with which replies.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Expert {
    address: u32,
    total: u32,
    counts: Vec<Count>,
}

/// The size of each section, in the units the header names.
struct Lengths {
    replies: u64,
    reply_words: u64,
    turns: u64,
    turn_words: u64,
    turn_entries: u64,
    experts: u64,
    expert_entries: u64,
}

impl Lengths {
    /// Bytes of the body these lengths name.
    fn body_bytes(&self) -> u64 {
        4 * (self.replies + self.reply_words)
            + 4 * (2 * self.turns + self.turn_words + 2 * self.turn_entries)
            + 4 * (3 * self.experts + 2 * self.expert_entries)
    }
}

/// A table of pair experts, counted from training conversations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairExperts {
    vocab: u32,
    tokens_sha256: [u8; 32],
    primes: Vec<u32>,
    /// Every reply, sorted and without repeats; a reply's index is its place here.
    replies: Vec<Vec<u32>>,
    /// Every exact turn, sorted and without repeats.
    turns: Vec<Turn>,
    /// Every expert, by address.
    experts: Vec<Expert>,
}

/// The first `vocab` primes, refused when the two largest multiply to 2^31 or more.
fn primes(vocab: u32) -> Result<Vec<u32>, String> {
    if vocab < 2 {
        return Err(format!("a vocabulary of {vocab} tokens holds no pair"));
    }
    let mut primes: Vec<u32> = Vec::new();
    let mut n: u32 = 2;
    while primes.len() < vocab as usize {
        let prime = primes
            .iter()
            .take_while(|p| u64::from(**p) * u64::from(**p) <= u64::from(n))
            .all(|p| n % p != 0);
        if prime {
            if let Some(last) = primes.last() {
                if u64::from(*last) * u64::from(n) >= 1 << 31 {
                    return Err(format!(
                        "a vocabulary of {vocab} tokens: the address of its two last tokens is past 31 bits"
                    ));
                }
            }
            primes.push(n);
        }
        n += 1;
    }
    Ok(primes)
}

/// The address of every unordered pair of different tokens in the turn, each pair once. A token
/// outside the vocabulary names no pair.
fn pairs(primes: &[u32], turn: &[u32]) -> Vec<u32> {
    let mut tokens: Vec<u32> = turn
        .iter()
        .copied()
        .filter(|t| (*t as usize) < primes.len())
        .collect();
    tokens.sort_unstable();
    tokens.dedup();
    let mut out = Vec::with_capacity(tokens.len() * tokens.len().saturating_sub(1) / 2);
    for (i, a) in tokens.iter().enumerate() {
        for b in &tokens[i + 1..] {
            out.push(primes[*a as usize] * primes[*b as usize]);
        }
    }
    out
}

/// Adds one to a count, refused past 32 bits.
fn bump(count: &mut u32, by: u32) -> Result<(), String> {
    *count = count
        .checked_add(by)
        .ok_or_else(|| "a pair expert count is past 32 bits".to_owned())?;
    Ok(())
}

impl PairExperts {
    /// Counts the table from training occurrences. An occurrence with an empty turn or reply is
    /// skipped; a token outside the vocabulary is refused. The order of the occurrences does not
    /// change the table.
    pub fn count(
        vocab: u32,
        tokens_sha256: [u8; 32],
        occurrences: &[Occurrence<'_>],
    ) -> Result<Self, String> {
        let primes = primes(vocab)?;
        let kept: Vec<&Occurrence<'_>> = occurrences
            .iter()
            .filter(|o| !o.turn.is_empty() && !o.reply.is_empty())
            .collect();
        let mut replies: BTreeSet<&[u32]> = BTreeSet::new();
        for o in &kept {
            if let Some(token) = o.turn.iter().chain(o.reply).find(|t| **t >= vocab) {
                return Err(format!(
                    "token {token} is outside the vocabulary of {vocab}"
                ));
            }
            replies.insert(o.reply);
        }
        let replies: Vec<Vec<u32>> = replies.into_iter().map(<[u32]>::to_vec).collect();

        let mut turns: BTreeMap<&[u32], BTreeMap<u32, u32>> = BTreeMap::new();
        for o in &kept {
            let reply = replies
                .binary_search_by(|r| r.as_slice().cmp(o.reply))
                .map_err(|_| "a reply was not interned".to_owned())? as u32;
            bump(
                turns.entry(o.turn).or_default().entry(reply).or_insert(0),
                1,
            )?;
        }

        let mut experts: BTreeMap<u32, BTreeMap<u32, u32>> = BTreeMap::new();
        for (turn, counts) in &turns {
            for address in pairs(&primes, turn) {
                let held = experts.entry(address).or_default();
                for (reply, count) in counts {
                    bump(held.entry(*reply).or_insert(0), *count)?;
                }
            }
        }

        let as_counts = |counts: &BTreeMap<u32, u32>| -> Vec<Count> {
            counts
                .iter()
                .map(|(reply, count)| Count {
                    reply: *reply,
                    count: *count,
                })
                .collect()
        };
        let turns: Vec<Turn> = turns
            .iter()
            .map(|(tokens, counts)| Turn {
                tokens: tokens.to_vec(),
                counts: as_counts(counts),
            })
            .collect();
        let mut table_experts = Vec::with_capacity(experts.len());
        for (address, counts) in &experts {
            let mut total = 0u32;
            for count in counts.values() {
                bump(&mut total, *count)?;
            }
            table_experts.push(Expert {
                address: *address,
                total,
                counts: as_counts(counts),
            });
        }
        let table = Self {
            vocab,
            tokens_sha256,
            primes,
            replies,
            turns,
            experts: table_experts,
        };
        let lengths = table.lengths();
        let words = [
            lengths.replies,
            lengths.reply_words,
            lengths.turns,
            lengths.turn_words,
            lengths.turn_entries,
            lengths.experts,
            lengths.expert_entries,
        ];
        if words.iter().any(|w| *w > u64::from(u32::MAX)) {
            return Err("a pair expert section is past 32 bits".to_owned());
        }
        Ok(table)
    }

    /// The address of the pair of tokens `a` and `b`: the product of their primes. None when the
    /// tokens are the same or one is outside the vocabulary.
    pub fn address(&self, a: u32, b: u32) -> Option<u32> {
        if a == b || a >= self.vocab || b >= self.vocab {
            return None;
        }
        Some(self.primes[a as usize] * self.primes[b as usize])
    }

    /// The experts' votes for a turn. With `hold_out`, the exact turn's own counts are taken
    /// out of every total and count first. A pair is an expert when what is left of its total
    /// is at least 1 and at most `cap`: the cap is read after the hold-out. A count is never
    /// read as more than what is left of its expert's total, so one expert gives any one reply
    /// at most one whole vote whatever a file holds.
    pub fn votes(&self, turn: &[u32], hold_out: bool, cap: u32) -> Votes {
        let own: &[Count] = if hold_out {
            self.turns
                .binary_search_by(|t| t.tokens.as_slice().cmp(turn))
                .map_or(&[], |at| self.turns[at].counts.as_slice())
        } else {
            &[]
        };
        let own_total: u64 = own.iter().map(|c| u64::from(c.count)).sum();
        let mut experts = 0;
        let mut scores: BTreeMap<u32, u64> = BTreeMap::new();
        for address in pairs(&self.primes, turn) {
            let Ok(at) = self.experts.binary_search_by_key(&address, |e| e.address) else {
                continue;
            };
            let expert = &self.experts[at];
            let t = u64::from(expert.total).saturating_sub(own_total);
            if t < 1 || t > u64::from(cap) {
                continue;
            }
            experts += 1;
            for held in &expert.counts {
                let mine = own
                    .binary_search_by_key(&held.reply, |c| c.reply)
                    .map_or(0, |i| own[i].count);
                let c = u64::from(held.count.saturating_sub(mine)).min(t);
                if c > 0 {
                    let score = scores.entry(held.reply).or_insert(0);
                    *score = score.saturating_add((c << VOTE_SHIFT) / t);
                }
            }
        }
        Votes {
            experts,
            scores: scores
                .into_iter()
                .filter(|(_, score)| *score > 0)
                .map(|(reply, score)| ReplyScore { reply, score })
                .collect(),
        }
    }

    /// Checks the model's reply against the experts of the turn. It stands when its vote is
    /// positive and at least `keep` of the top vote; otherwise the top reply replaces it. A reply
    /// the table never saw has vote 0.
    pub fn check(
        &self,
        turn: &[u32],
        reply: &[u32],
        hold_out: bool,
        cap: u32,
        keep: Ratio,
    ) -> Check {
        let votes = self.votes(turn, hold_out, cap);
        let Some(top) = votes.top() else {
            return Check::NoExpert;
        };
        let vote = self.reply_index(reply).map_or(0, |r| votes.score(r));
        if vote > 0
            && u128::from(vote) * u128::from(keep.den)
                >= u128::from(top.score) * u128::from(keep.num)
        {
            Check::Stands {
                vote,
                top: top.score,
            }
        } else {
            Check::Replaced {
                reply: top.reply,
                vote,
                top: top.score,
            }
        }
    }

    /// The tokens of reply `index`.
    pub fn reply(&self, index: u32) -> Option<&[u32]> {
        self.replies.get(index as usize).map(Vec::as_slice)
    }

    /// The index of a reply, when the table holds it.
    pub fn reply_index(&self, reply: &[u32]) -> Option<u32> {
        self.replies
            .binary_search_by(|r| r.as_slice().cmp(reply))
            .ok()
            .map(|at| at as u32)
    }

    /// Whether the exact turn, as a token sequence, was counted with this reply.
    pub fn turn_holds(&self, turn: &[u32], reply: &[u32]) -> bool {
        let Some(reply) = self.reply_index(reply) else {
            return false;
        };
        self.turns
            .binary_search_by(|t| t.tokens.as_slice().cmp(turn))
            .map_or(false, |at| {
                self.turns[at]
                    .counts
                    .binary_search_by_key(&reply, |c| c.reply)
                    .is_ok()
            })
    }

    /// The vocabulary the table was counted on.
    pub fn vocab(&self) -> u32 {
        self.vocab
    }

    /// The tag of the tokens the table was counted on.
    pub fn tokens_sha256(&self) -> [u8; 32] {
        self.tokens_sha256
    }

    /// Experts in the table.
    pub fn experts(&self) -> usize {
        self.experts.len()
    }

    /// Different replies in the table.
    pub fn replies(&self) -> usize {
        self.replies.len()
    }

    /// Different exact turns in the table.
    pub fn turns(&self) -> usize {
        self.turns.len()
    }

    /// Reply counts held by all experts together.
    pub fn entries(&self) -> usize {
        self.experts.iter().map(|e| e.counts.len()).sum()
    }

    /// Refuses a table counted on another vocabulary or other tokens.
    pub fn expect(&self, vocab: u32, tokens_sha256: &[u8; 32]) -> Result<(), String> {
        if self.vocab != vocab {
            return Err(format!(
                "the pair experts were counted on a vocabulary of {}, not {vocab}",
                self.vocab
            ));
        }
        if &self.tokens_sha256 != tokens_sha256 {
            return Err(format!(
                "the pair experts were counted on tokens {}, not {}",
                hex::encode(self.tokens_sha256),
                hex::encode(tokens_sha256)
            ));
        }
        Ok(())
    }

    fn lengths(&self) -> Lengths {
        let len = |n: usize| n as u64;
        Lengths {
            replies: len(self.replies.len()),
            reply_words: len(self.replies.iter().map(Vec::len).sum()),
            turns: len(self.turns.len()),
            turn_words: len(self.turns.iter().map(|t| t.tokens.len()).sum()),
            turn_entries: len(self.turns.iter().map(|t| t.counts.len()).sum()),
            experts: len(self.experts.len()),
            expert_entries: len(self.entries()),
        }
    }

    /// The table file: the magic, then as 32-bit little-endian words the version, the vocabulary
    /// and the seven section lengths, then the tokens tag and the body's SHA-256, then the body:
    /// the replies, the turns with their counts, and the experts with their totals and counts.
    pub fn to_bytes(&self) -> Vec<u8> {
        let lengths = self.lengths();
        let mut body = Vec::with_capacity(lengths.body_bytes() as usize);
        let mut put = |word: u32| body.extend_from_slice(&word.to_le_bytes());
        for reply in &self.replies {
            put(reply.len() as u32);
            reply.iter().for_each(|t| put(*t));
        }
        for turn in &self.turns {
            put(turn.tokens.len() as u32);
            turn.tokens.iter().for_each(|t| put(*t));
            put(turn.counts.len() as u32);
            for c in &turn.counts {
                put(c.reply);
                put(c.count);
            }
        }
        for expert in &self.experts {
            put(expert.address);
            put(expert.total);
            put(expert.counts.len() as u32);
            for c in &expert.counts {
                put(c.reply);
                put(c.count);
            }
        }
        let mut out = Vec::with_capacity(HEADER + body.len());
        out.extend_from_slice(MAGIC);
        let words = [
            VERSION,
            self.vocab,
            lengths.replies as u32,
            lengths.reply_words as u32,
            lengths.turns as u32,
            lengths.turn_words as u32,
            lengths.turn_entries as u32,
            lengths.experts as u32,
            lengths.expert_entries as u32,
        ];
        for word in words {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&self.tokens_sha256);
        out.extend_from_slice(&Sha256::digest(&body));
        out.extend_from_slice(&body);
        out
    }

    /// The table a file holds, refused unless its header, its lengths, the order of every list
    /// and the body's sum hold. Whether the counts agree with each other (a turn's counts inside
    /// its pairs' counts) is not checked here; [`Self::votes`] bounds what such a file can do.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER || &bytes[..8] != MAGIC {
            return Err("not a pair experts file".to_owned());
        }
        let word = |i: usize| {
            let at = 8 + 4 * i;
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        if word(0) != VERSION {
            return Err(format!(
                "a pair experts file of version {}, not {VERSION}",
                word(0)
            ));
        }
        let vocab = word(1);
        let lengths = Lengths {
            replies: u64::from(word(2)),
            reply_words: u64::from(word(3)),
            turns: u64::from(word(4)),
            turn_words: u64::from(word(5)),
            turn_entries: u64::from(word(6)),
            experts: u64::from(word(7)),
            expert_entries: u64::from(word(8)),
        };
        let body = &bytes[HEADER..];
        if body.len() as u64 != lengths.body_bytes() {
            return Err(format!(
                "a pair experts body of {} bytes, not the {} its header names",
                body.len(),
                lengths.body_bytes()
            ));
        }
        if Sha256::digest(body).as_slice() != &bytes[BODY_SUM_AT..HEADER] {
            return Err("the pair experts body does not match its SHA-256".to_owned());
        }
        let primes = primes(vocab)?;
        let mut tokens_sha256 = [0u8; 32];
        tokens_sha256.copy_from_slice(&bytes[BODY_SUM_AT - 32..BODY_SUM_AT]);
        let mut cursor = Cursor { bytes: body, at: 0 };
        let mut seen = Lengths {
            replies: lengths.replies,
            reply_words: 0,
            turns: lengths.turns,
            turn_words: 0,
            turn_entries: 0,
            experts: lengths.experts,
            expert_entries: 0,
        };

        let mut replies: Vec<Vec<u32>> = Vec::new();
        for _ in 0..lengths.replies {
            let reply = cursor.tokens(vocab, &mut seen.reply_words)?;
            if replies.last().is_some_and(|last| *last >= reply) {
                return Err("the pair experts replies are not strictly increasing".to_owned());
            }
            replies.push(reply);
        }
        let reply_count = replies.len() as u32;
        let mut turns: Vec<Turn> = Vec::new();
        for _ in 0..lengths.turns {
            let tokens = cursor.tokens(vocab, &mut seen.turn_words)?;
            if turns.last().is_some_and(|last| last.tokens >= tokens) {
                return Err("the pair experts turns are not strictly increasing".to_owned());
            }
            let counts = cursor.counts(reply_count, &mut seen.turn_entries)?;
            turns.push(Turn { tokens, counts });
        }
        let mut experts: Vec<Expert> = Vec::new();
        for _ in 0..lengths.experts {
            let address = cursor.word()?;
            let total = cursor.word()?;
            if experts.last().is_some_and(|last| last.address >= address) {
                return Err("the pair experts addresses are not strictly increasing".to_owned());
            }
            let counts = cursor.counts(reply_count, &mut seen.expert_entries)?;
            let sum: u64 = counts.iter().map(|c| u64::from(c.count)).sum();
            if sum != u64::from(total) {
                return Err(format!(
                    "pair expert {address} has total {total}, but its counts sum to {sum}"
                ));
            }
            experts.push(Expert {
                address,
                total,
                counts,
            });
        }
        if seen.reply_words != lengths.reply_words
            || seen.turn_words != lengths.turn_words
            || seen.turn_entries != lengths.turn_entries
            || seen.expert_entries != lengths.expert_entries
            || cursor.at != body.len()
        {
            return Err(
                "the pair experts sections are not the lengths the header names".to_owned(),
            );
        }
        Ok(Self {
            vocab,
            tokens_sha256,
            primes,
            replies,
            turns,
            experts,
        })
    }

    /// SHA-256 of the table file, in hex.
    pub fn sha256(&self) -> String {
        hex::encode(Sha256::digest(self.to_bytes()))
    }
}

/// Reads a table body word by word, refusing to run past its end.
struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Cursor<'_> {
    fn word(&mut self) -> Result<u32, String> {
        let end = self.at + 4;
        if end > self.bytes.len() {
            return Err("the pair experts body ends early".to_owned());
        }
        let b = &self.bytes[self.at..end];
        self.at = end;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// A length, then that many tokens, each inside the vocabulary; an empty list is refused.
    fn tokens(&mut self, vocab: u32, seen: &mut u64) -> Result<Vec<u32>, String> {
        let len = self.word()?;
        if len == 0 {
            return Err("the pair experts file holds an empty reply or turn".to_owned());
        }
        if (self.bytes.len() - self.at) / 4 < len as usize {
            return Err("the pair experts body ends early".to_owned());
        }
        *seen += u64::from(len);
        let mut out = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let token = self.word()?;
            if token >= vocab {
                return Err(format!(
                    "the pair experts file holds token {token}, outside the vocabulary of {vocab}"
                ));
            }
            out.push(token);
        }
        Ok(out)
    }

    /// A length, then that many reply counts: reply indices strictly increasing and in range,
    /// every count positive. An empty list is refused.
    fn counts(&mut self, replies: u32, seen: &mut u64) -> Result<Vec<Count>, String> {
        let len = self.word()?;
        if len == 0 {
            return Err("the pair experts file holds a turn or an expert with no reply".to_owned());
        }
        if (self.bytes.len() - self.at) / 8 < len as usize {
            return Err("the pair experts body ends early".to_owned());
        }
        *seen += u64::from(len);
        let mut out: Vec<Count> = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let reply = self.word()?;
            let count = self.word()?;
            if reply >= replies {
                return Err(format!(
                    "the pair experts file names reply {reply} of {replies}"
                ));
            }
            if out.last().is_some_and(|last| last.reply >= reply) {
                return Err("the pair experts reply indices are not strictly increasing".to_owned());
            }
            if count == 0 {
                return Err("the pair experts file holds a count of 0".to_owned());
            }
            out.push(Count { reply, count });
        }
        Ok(out)
    }
}

/// How a served reply is checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setting {
    /// An expert seen more than this many times does not vote.
    pub cap: u32,
    /// The share of the top vote a reply keeps to stand.
    pub keep: Ratio,
    /// Whether the turn's own counts are taken out before the vote.
    pub hold_out: bool,
    /// The top vote the experts need before they answer in the model's place, in thousandths
    /// of one expert's whole vote. Under it they do not know the turn and the reply stands.
    pub floor: u32,
}

/// What the experts decide about the reply to a prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The prompt does not end where a model turn answers a user turn: the reply stands.
    NoTurn,
    /// The counted table finds a told value, an ask for one never told, or a repeat ask: the
    /// experts are not asked and the reply stands.
    LeftAlone,
    /// The experts' check of the reply against the turn.
    Checked(Check),
    /// The experts hold another reply and it does not fit in the tokens the caller asked for:
    /// the reply stands. Only a caller that gives replies sets this; [`PairExperts::check_prompt`]
    /// never does.
    NoRoom,
}

/// The user turn a prompt asks the model to answer: its content and the finished user turns
/// before it, as the place register cuts them. `Some` only when the prompt's last token opens
/// the first model turn after a user turn, which is where [`crate::place_asks::ask_cases`]
/// counts a reply.
pub fn asked_turn(prompt: &[u32], marks: &PlaceMarks) -> Option<(Range<usize>, Vec<Range<usize>>)> {
    let mut turns = PlaceTurns::default();
    let mut answered = false;
    let mut opens = false;
    for (t, &token) in prompt.iter().enumerate() {
        if token == marks.bos {
            answered = false;
        }
        opens = false;
        match turns.push(&prompt[..=t], marks) {
            Some(PlaceRole::User) => answered = false,
            Some(PlaceRole::Assistant) => {
                opens = !answered;
                answered = true;
            }
            Some(PlaceRole::System) | None => {}
        }
    }
    if !opens {
        return None;
    }
    let (asked, prior) = turns.users().split_last()?;
    Some((asked.clone(), prior.to_vec()))
}

impl PairExperts {
    /// Checks the finished reply to a whole prompt: the one rule the served chat and the panel
    /// replay both apply. `reply` is the reply's ids without its end mark. A prompt with no
    /// turn to answer, and a turn the counted table `asks` finds something for, are left alone;
    /// every other reply is checked against the experts of the prompt's last user turn.
    pub fn check_prompt(
        &self,
        asks: Option<&AsksIndex>,
        marks: &PlaceMarks,
        prompt: &[u32],
        reply: &[u32],
        setting: Setting,
    ) -> Verdict {
        let Some((asked, prior)) = asked_turn(prompt, marks) else {
            return Verdict::NoTurn;
        };
        if asks.is_some_and(|asks| asks.find(prompt, &prior, &asked) != Asked::Nothing) {
            return Verdict::LeftAlone;
        }
        Verdict::Checked(self.check_setting(&prompt[asked], reply, setting))
    }

    /// [`Self::check`] under a whole setting: a reply the check would replace stands instead
    /// when the experts' top vote is under the setting's floor. One expert that saw only the top
    /// reply gives it a whole vote, `1 << 20`; the floor counts in thousandths of that.
    pub fn check_setting(&self, turn: &[u32], reply: &[u32], setting: Setting) -> Check {
        match self.check(turn, reply, setting.hold_out, setting.cap, setting.keep) {
            Check::Replaced { vote, top, .. }
                if u128::from(top) * 1000 < u128::from(setting.floor) << VOTE_SHIFT =>
            {
                Check::Unsure { vote, top }
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAG: [u8; 32] = [7; 32];

    /// Reply `[5]` is index 0 and reply `[6]` is index 1.
    const A: &[u32] = &[5];
    const B: &[u32] = &[6];

    fn occ<'a>(turn: &'a [u32], reply: &'a [u32]) -> Occurrence<'a> {
        Occurrence { turn, reply }
    }

    /// Experts: 6 = {0,1} total 3 (A 2, B 1); 10 = {0,2} total 2 (A 2); 15 = {1,2} total 3 (A 3);
    /// 21 = {1,3} total 1 (A 1); 35 = {2,3} total 1 (A 1).
    fn sample() -> Vec<Occurrence<'static>> {
        vec![
            occ(&[0, 1, 2], A),
            occ(&[0, 1, 2], A),
            occ(&[0, 1], B),
            occ(&[1, 2, 3], A),
        ]
    }

    fn table(occurrences: &[Occurrence<'_>]) -> PairExperts {
        PairExperts::count(8, TAG, occurrences).unwrap()
    }

    fn scores(pairs: &[[u64; 2]]) -> Vec<ReplyScore> {
        pairs
            .iter()
            .map(|p| ReplyScore {
                reply: p[0] as u32,
                score: p[1],
            })
            .collect()
    }

    fn word(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    }

    fn put(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// Writes the body's SHA-256 again, so only a structural rule can refuse the file.
    fn reseal(bytes: &mut [u8]) {
        let sum = Sha256::digest(&bytes[HEADER..]);
        bytes[BODY_SUM_AT..HEADER].copy_from_slice(&sum);
    }

    #[test]
    fn a_vote_is_the_integer_sum_of_the_experts_shares() {
        let experts = table(&sample());
        let votes = experts.votes(&[0, 1, 2], false, 100);
        assert_eq!(votes.experts, 3);
        // A: (2 << 20) / 3 + (2 << 20) / 2 + (3 << 20) / 3 = 699_050 + 1_048_576 + 1_048_576.
        // B: (1 << 20) / 3 = 349_525.
        assert_eq!(votes.scores, scores(&[[0, 2_796_202], [1, 349_525]]));
        assert_eq!(
            votes.top(),
            Some(ReplyScore {
                reply: 0,
                score: 2_796_202
            })
        );
    }

    #[test]
    fn a_turn_holds_only_the_replies_it_was_counted_with_in_its_own_order() {
        let experts = table(&sample());
        assert!(experts.turn_holds(&[0, 1, 2], A));
        assert!(experts.turn_holds(&[0, 1], B));
        // The same tokens in another order are another turn; another reply was not counted with
        // it; a reply the table never saw is held by no turn.
        assert!(!experts.turn_holds(&[2, 1, 0], A));
        assert!(!experts.turn_holds(&[0, 1, 2], B));
        assert!(!experts.turn_holds(&[0, 1], A));
        assert!(!experts.turn_holds(&[0, 1, 2], &[7]));
        // A table read back from its file holds the same.
        let read = PairExperts::from_bytes(&experts.to_bytes()).unwrap();
        assert!(read.turn_holds(&[1, 2, 3], A));
        assert!(!read.turn_holds(&[1, 2, 3], B));
    }

    #[test]
    fn hold_out_takes_out_exactly_the_turns_own_counts() {
        let experts = table(&sample());
        // The turn [0, 1, 2] got A twice. Pair 6 keeps total 1 (B 1); pair 10 keeps total 0 and
        // is no expert; pair 15 keeps total 1 (A 1).
        let votes = experts.votes(&[0, 1, 2], true, 100);
        assert_eq!(votes.experts, 2);
        assert_eq!(votes.scores, scores(&[[0, 1_048_576], [1, 1_048_576]]));
        // [0, 2, 1] was never counted as an exact turn, so nothing is taken out.
        assert_eq!(
            experts.votes(&[0, 2, 1], true, 100),
            experts.votes(&[0, 1, 2], false, 100)
        );
        // Every pair of [2, 3]'s only turn is its own: none is left.
        let alone = table(&[occ(&[2, 3], A)]);
        assert_eq!(alone.votes(&[2, 3], true, 100).experts, 0);
        assert_eq!(alone.votes(&[2, 3], false, 100).experts, 1);
    }

    #[test]
    fn a_pair_over_the_cap_never_votes() {
        let experts = table(&sample());
        // Only pair 10 (total 2) is within a cap of 2.
        let votes = experts.votes(&[0, 1, 2], false, 2);
        assert_eq!(votes.experts, 1);
        assert_eq!(votes.scores, scores(&[[0, 1_048_576]]));
        // At exactly the cap of 3, all three pairs vote.
        assert_eq!(experts.votes(&[0, 1, 2], false, 3).experts, 3);
        assert_eq!(experts.votes(&[0, 1, 2], false, 1).experts, 0);
    }

    #[test]
    fn a_turn_with_no_expert_lets_the_reply_stand() {
        let experts = table(&sample());
        let keep = Ratio { num: 1, den: 2 };
        assert_eq!(experts.check(&[4, 5], B, false, 100, keep), Check::NoExpert);
        assert_eq!(experts.check(&[0], B, false, 100, keep), Check::NoExpert);
        assert_eq!(
            experts.check(&[0, 1, 2], B, false, 1, keep),
            Check::NoExpert
        );
    }

    #[test]
    fn the_reply_stands_at_the_keep_share_and_is_replaced_below_it() {
        let experts = table(&sample());
        // B's vote is 349_525 and the top (A) is 2_796_202.
        let equal = Ratio {
            num: 349_525,
            den: 2_796_202,
        };
        assert_eq!(
            experts.check(&[0, 1, 2], B, false, 100, equal),
            Check::Stands {
                vote: 349_525,
                top: 2_796_202
            }
        );
        let above = Ratio {
            num: 349_526,
            den: 2_796_202,
        };
        assert_eq!(
            experts.check(&[0, 1, 2], B, false, 100, above),
            Check::Replaced {
                reply: 0,
                vote: 349_525,
                top: 2_796_202
            }
        );
        // A reply the table never saw has vote 0 and is replaced, even at a share of 0.
        let none = Ratio { num: 0, den: 1 };
        assert_eq!(
            experts.check(&[0, 1, 2], &[7, 7], false, 100, none),
            Check::Replaced {
                reply: 0,
                vote: 0,
                top: 2_796_202
            }
        );
    }

    #[test]
    fn a_tie_goes_to_the_smaller_reply_and_the_order_of_counting_does_not_matter() {
        let tied = table(&[occ(&[0, 1], B), occ(&[0, 1], A)]);
        let votes = tied.votes(&[0, 1], false, 100);
        assert_eq!(votes.scores, scores(&[[0, 524_288], [1, 524_288]]));
        assert_eq!(
            votes.top(),
            Some(ReplyScore {
                reply: 0,
                score: 524_288
            })
        );

        let forward = sample();
        let mut backward = forward.clone();
        backward.reverse();
        let mut turned = forward.clone();
        turned.rotate_left(1);
        let bytes = table(&forward).to_bytes();
        assert_eq!(table(&backward).to_bytes(), bytes);
        assert_eq!(table(&turned).to_bytes(), bytes);
    }

    #[test]
    fn pairs_are_unordered_distinct_and_counted_once_an_occurrence() {
        let experts = table(&[
            occ(&[1, 0, 1, 0], A),
            occ(&[0, 1], A),
            occ(&[3, 3], B),
            occ(&[], A),
            occ(&[2, 4], &[]),
        ]);
        assert_eq!(experts.experts(), 1);
        assert_eq!(experts.entries(), 1);
        assert_eq!(experts.turns(), 3);
        assert_eq!(experts.replies(), 2);
        assert_eq!(
            experts.experts[0],
            Expert {
                address: 6,
                total: 2,
                counts: vec![Count { reply: 0, count: 2 }],
            }
        );
    }

    #[test]
    fn the_address_of_tokens_0_and_1_is_6_and_a_vocabulary_past_32_bits_is_refused() {
        let experts = table(&sample());
        assert_eq!(experts.address(0, 1), Some(6));
        assert_eq!(experts.address(1, 0), Some(6));
        assert_eq!(experts.address(2, 7), Some(5 * 19));
        assert_eq!(experts.address(1, 1), None);
        assert_eq!(experts.address(0, 8), None);
        // The 4792nd prime is 46_337 and the 4793rd is 46_349: 46_327 * 46_337 is under 2^31 and
        // 46_337 * 46_349 is not.
        let widest = PairExperts::count(4_792, TAG, &[]).unwrap();
        assert_eq!(widest.address(4_790, 4_791), Some(46_327 * 46_337));
        assert!(46_327u64 * 46_337 < 1 << 31 && 46_337u64 * 46_349 >= 1 << 31);
        assert!(PairExperts::count(4_793, TAG, &[])
            .unwrap_err()
            .contains("past 31 bits"));
        assert!(PairExperts::count(8, TAG, &[occ(&[0, 8], A)])
            .unwrap_err()
            .contains("outside the vocabulary"));
    }

    #[test]
    fn a_table_file_reads_back_and_each_broken_one_is_refused() {
        let experts = table(&sample());
        let bytes = experts.to_bytes();
        let back = PairExperts::from_bytes(&bytes).unwrap();
        assert_eq!(back, experts);
        assert_eq!(back.reply(1), Some(B));
        assert_eq!(back.reply_index(A), Some(0));
        assert_eq!(back.sha256().len(), 64);
        assert!(back.expect(8, &TAG).is_ok());
        assert!(back.expect(9, &TAG).unwrap_err().contains("vocabulary"));
        assert!(back.expect(8, &[8; 32]).unwrap_err().contains("tokens"));

        let mut magic = bytes.clone();
        magic[0] ^= 1;
        assert!(PairExperts::from_bytes(&magic)
            .unwrap_err()
            .contains("not a pair experts file"));
        let mut version = bytes.clone();
        put(&mut version, 8, 2);
        assert!(PairExperts::from_bytes(&version)
            .unwrap_err()
            .contains("version 2"));
        assert!(PairExperts::from_bytes(&bytes[..bytes.len() - 4])
            .unwrap_err()
            .contains("its header names"));
        let mut longer = bytes.clone();
        longer.extend_from_slice(&[0; 4]);
        assert!(PairExperts::from_bytes(&longer)
            .unwrap_err()
            .contains("its header names"));
        let mut flipped = bytes.clone();
        flipped[HEADER + 4] ^= 1;
        assert!(PairExperts::from_bytes(&flipped)
            .unwrap_err()
            .contains("SHA-256"));

        // Body bytes: replies [5] [6] at 108; turns [0,1] [0,1,2] [1,2,3]; experts from here.
        let experts_at = HEADER
            + 4 * (word(&bytes, 16) + word(&bytes, 20)) as usize
            + 4 * (2 * word(&bytes, 24) + word(&bytes, 28) + 2 * word(&bytes, 32)) as usize;
        assert_eq!(word(&bytes, experts_at), 6);
        assert_eq!(word(&bytes, experts_at + 4), 3);

        // Each file below is resealed, so only its structural rule refuses it.
        let mut total = bytes.clone();
        put(&mut total, experts_at + 4, 4);
        reseal(&mut total);
        assert!(PairExperts::from_bytes(&total)
            .unwrap_err()
            .contains("counts sum to 3"));
        let mut reply = bytes.clone();
        put(&mut reply, experts_at + 12, 2);
        reseal(&mut reply);
        assert!(PairExperts::from_bytes(&reply)
            .unwrap_err()
            .contains("reply 2 of 2"));
        // The second expert (10) sits after the first's three words and two counts.
        let mut address = bytes.clone();
        put(&mut address, experts_at + 28, 6);
        reseal(&mut address);
        assert!(PairExperts::from_bytes(&address)
            .unwrap_err()
            .contains("addresses are not strictly increasing"));
        let mut replies = bytes.clone();
        put(&mut replies, HEADER + 12, 5);
        reseal(&mut replies);
        assert!(PairExperts::from_bytes(&replies)
            .unwrap_err()
            .contains("replies are not strictly increasing"));
        // The second turn [0, 1, 2] starts after the first's 1 + 2 + 1 + 2 words; make it
        // [0, 0, 0], which sorts before the first turn [0, 1].
        let mut turns = bytes.clone();
        put(&mut turns, HEADER + 16 + 24 + 4, 0);
        put(&mut turns, HEADER + 16 + 24 + 8, 0);
        put(&mut turns, HEADER + 16 + 24 + 12, 0);
        reseal(&mut turns);
        assert!(PairExperts::from_bytes(&turns)
            .unwrap_err()
            .contains("turns are not strictly increasing"));
        // The first reply [5] made [8]: a token outside the vocabulary of 8.
        let mut token = bytes.clone();
        put(&mut token, HEADER + 4, 8);
        reseal(&mut token);
        assert!(PairExperts::from_bytes(&token)
            .unwrap_err()
            .contains("token 8, outside the vocabulary of 8"));
        // The first reply's length made 0.
        let mut empty = bytes.clone();
        put(&mut empty, HEADER, 0);
        reseal(&mut empty);
        assert!(PairExperts::from_bytes(&empty)
            .unwrap_err()
            .contains("an empty reply or turn"));
        // The first expert's counts are (reply 0, 2) (reply 1, 1): a count of 0, then the second
        // reply index made 0 again.
        let mut zero = bytes.clone();
        put(&mut zero, experts_at + 16, 0);
        reseal(&mut zero);
        assert!(PairExperts::from_bytes(&zero)
            .unwrap_err()
            .contains("a count of 0"));
        let mut order = bytes.clone();
        put(&mut order, experts_at + 20, 0);
        reseal(&mut order);
        assert!(PairExperts::from_bytes(&order)
            .unwrap_err()
            .contains("reply indices are not strictly increasing"));
        // One word moved in the header from the turns' tokens to the replies': the body is as
        // long as the header says and no section is.
        let mut moved = bytes.clone();
        put(&mut moved, 20, word(&bytes, 20) + 1);
        put(&mut moved, 28, word(&bytes, 28) - 1);
        assert!(PairExperts::from_bytes(&moved)
            .unwrap_err()
            .contains("are not the lengths the header names"));
    }

    #[test]
    fn one_expert_never_gives_one_reply_more_than_one_whole_vote_whatever_a_file_holds() {
        // A table no count could give: the turn {0, 1} was seen with B almost 2^32 times, while
        // its one expert, 6, holds A almost 2^32 times and B once.
        let mut t = table(&[occ(&[0, 1], A), occ(&[0, 1], B)]);
        let big = u32::MAX - 1;
        t.turns[0].counts = vec![Count {
            reply: 1,
            count: big,
        }];
        t.experts[0].total = u32::MAX;
        t.experts[0].counts = vec![
            Count {
                reply: 0,
                count: big,
            },
            Count { reply: 1, count: 1 },
        ];
        // Held out, 1 is left of the expert's total: A's count is read as that 1.
        let votes = t.votes(&[0, 1], true, u32::MAX);
        assert_eq!(votes.experts, 1);
        assert_eq!(votes.scores, scores(&[[0, 1 << 20]]));
    }

    #[test]
    fn the_experts_do_not_answer_when_their_top_vote_is_under_the_floor() {
        let t = table(&sample());
        let turn = [0, 1, 2];
        // Reply A's vote on turn {0, 1, 2}: 2/3 of expert 6 and the whole of experts 10 and 15.
        let top = (2u64 << 20) / 3 + (1 << 20) + (1 << 20);
        let vote = (1u64 << 20) / 3;
        assert_eq!(top * 1000 / (1 << 20), 2666);
        let at = |floor: u32| Setting {
            cap: 100,
            keep: Ratio { num: 1, den: 2 },
            hold_out: false,
            floor,
        };
        // With no floor the setting's check is the check.
        assert_eq!(
            t.check_setting(&turn, B, at(0)),
            t.check(&turn, B, false, 100, at(0).keep)
        );
        // A top vote of 2.666 whole votes answers at a floor of 2.666 and not at 2.667.
        assert_eq!(
            t.check_setting(&turn, B, at(2666)),
            Check::Replaced {
                reply: 0,
                vote,
                top
            }
        );
        assert_eq!(
            t.check_setting(&turn, B, at(2667)),
            Check::Unsure { vote, top }
        );
        // The floor never touches a reply that stands, and a turn with no expert has none.
        assert_eq!(
            t.check_setting(&turn, A, at(u32::MAX)),
            Check::Stands { vote: top, top }
        );
        assert_eq!(t.check_setting(&[4, 5], B, at(u32::MAX)), Check::NoExpert);
        // A top vote equal to the floor answers: the turn {2, 3} has one expert, which saw only A.
        let whole = 1u64 << 20;
        assert_eq!(
            t.check_setting(&[2, 3], B, at(1000)),
            Check::Replaced {
                reply: 0,
                vote: 0,
                top: whole
            }
        );
        assert_eq!(
            t.check_setting(&[2, 3], B, at(1001)),
            Check::Unsure {
                vote: 0,
                top: whole
            }
        );
    }

    const BOS: u32 = 100;
    const EOS: u32 = 101;
    const SEP: u32 = 102;
    const SYSTEM: u32 = 103;
    const USER: u32 = 104;
    const MODEL: u32 = 105;

    fn marks() -> PlaceMarks {
        PlaceMarks {
            bos: BOS,
            eos: EOS,
            sep: vec![SEP],
            system: vec![SYSTEM],
            user: vec![USER],
            assistant: vec![MODEL],
        }
    }

    #[test]
    fn the_asked_turn_is_the_user_turn_the_prompts_last_model_turn_answers() {
        let m = marks();
        // One user turn, then the model turn's opening.
        let one = [BOS, USER, 0, 1, 2, SEP, MODEL];
        assert_eq!(asked_turn(&one, &m), Some((2..5, vec![])));
        // A second user turn: the first is a finished turn before it.
        let two = [
            BOS, USER, 0, 1, SEP, MODEL, 5, EOS, SEP, USER, 1, 2, 3, SEP, MODEL,
        ];
        assert_eq!(asked_turn(&two, &m), Some((10..13, vec![2..4])));
        // A system turn between the user turn and the model turn keeps the user turn.
        let memory = [BOS, USER, 0, 1, SEP, SYSTEM, 3, SEP, MODEL];
        assert_eq!(asked_turn(&memory, &m), Some((2..4, vec![])));
        // No model turn opened; a model turn opened inside the reply's own text; a second
        // model turn with no user turn between.
        assert_eq!(asked_turn(&[BOS, USER, 0, 1], &m), None);
        assert_eq!(asked_turn(&[BOS, USER, 0, 1, SEP, MODEL, 5], &m), None);
        let twice = [BOS, USER, 0, 1, SEP, MODEL, 5, EOS, SEP, MODEL];
        assert_eq!(asked_turn(&twice, &m), None);
        assert_eq!(asked_turn(&[], &m), None);
    }

    #[test]
    fn a_prompts_reply_is_checked_against_its_last_user_turn_only() {
        let t = table(&sample());
        let m = marks();
        let setting = Setting {
            cap: 100,
            keep: Ratio { num: 1, den: 2 },
            hold_out: false,
            floor: 0,
        };
        // The earlier turn {0, 1} holds reply B; the asked turn {0, 1, 2} holds A three ways.
        let prompt = [
            BOS, USER, 0, 1, SEP, MODEL, 6, EOS, SEP, USER, 0, 1, 2, SEP, MODEL,
        ];
        let turn = [0, 1, 2];
        assert_eq!(
            t.check_prompt(None, &m, &prompt, A, setting),
            Verdict::Checked(t.check(&turn, A, false, 100, setting.keep))
        );
        assert!(matches!(
            t.check_prompt(None, &m, &prompt, A, setting),
            Verdict::Checked(Check::Stands { .. })
        ));
        assert!(matches!(
            t.check_prompt(None, &m, &prompt, B, setting),
            Verdict::Checked(Check::Replaced { reply: 0, .. })
        ));
        // The hold-out reaches the check: turn {0, 1, 2} was counted twice, both with A.
        let held = Setting {
            hold_out: true,
            ..setting
        };
        assert_eq!(
            t.check_prompt(None, &m, &prompt, A, held),
            Verdict::Checked(t.check(&turn, A, true, 100, setting.keep))
        );
        assert_ne!(
            t.check_prompt(None, &m, &prompt, A, held),
            t.check_prompt(None, &m, &prompt, A, setting)
        );
        // A prompt that opens no model turn is not checked.
        assert_eq!(
            t.check_prompt(None, &m, &[BOS, USER, 0, 1, 2], A, setting),
            Verdict::NoTurn
        );
    }
}
