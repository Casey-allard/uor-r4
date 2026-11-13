//! The asks (claude-359): an exact find for the place register, fitted by counting.
//!
//! The place register ([`crate::fbs_snapshot::PlaceState`]) reads a told value from the position
//! it holds. This module answers where that position is, from tokens alone. A candidate place is
//! a position in a finished user turn whose token before it has stood before a true place in the
//! fitted split. A candidate's frame is the five tokens before it and the last three tokens of
//! its turn that stand after it ([`ASKS_FRAME`] in all): what a statement says its value is often
//! comes after the value ("I like black best of all the colors"). Its score is a sum of fitted
//! integer weights: for each frame token, one weight when the question holds that token too and
//! another when it does not; and for each pair of a question token and a frame token, how much
//! more often the question holds that token at a true place than at any candidate. A place is
//! found when a candidate's score reaches the fitted floor: one floor for a question that asks
//! for a stored fact, another for every other question.
//!
//! A second table scores whether a question asks for a stored fact at all. When it does and no
//! place is found, the answer is [`Asked::NotTold`], and the register reads the stored reply.
//!
//! Every weight is a difference of [`lg`] values of counts; the read is adds and compares.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::fbs_snapshot::{PlaceMarks, PlaceRole, PlaceTurns};

/// The tokens before a candidate place that are read into its frame.
const ASKS_BEFORE: usize = 5;
/// The last tokens of the candidate's turn, after the place, that are read into its frame.
const ASKS_TAIL: usize = 3;
/// A frame's offsets: 1 to [`ASKS_BEFORE`] count back from the place, the rest count back from
/// the turn's end.
pub const ASKS_FRAME: usize = ASKS_BEFORE + ASKS_TAIL;
// A cut keeps a frame's offsets as the bits of one byte.
const _: () = assert!(ASKS_FRAME <= 8);

/// Which frame offsets, and how much of a question, a fit and its reader use (claude-360): a
/// census's cut, to measure what the find needs. A table file and a session never carry one:
/// both read every offset and the whole question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut {
    /// Bit `j` keeps frame offset `j + 1`.
    pub slots: u8,
    /// How many of a question's last tokens are read; 0 reads them all.
    pub question_last: usize,
}

impl Cut {
    /// Every offset and the whole question: no cut.
    pub const ALL: Cut = Cut { slots: u8::MAX, question_last: 0 };

    /// The frame with each offset the cut drops filled as an offset with no token is, so it is
    /// the same at every candidate and tells none from another.
    fn frame(self, mut f: [u32; ASKS_FRAME], start: u32) -> [u32; ASKS_FRAME] {
        for (j, slot) in f.iter_mut().enumerate() {
            if self.slots & (1 << j) == 0 {
                *slot = start + 1;
            }
        }
        f
    }

    /// The part of a question the cut reads.
    fn asked(self, asked: &Range<usize>) -> Range<usize> {
        match self.question_last {
            0 => asked.clone(),
            n => asked.end.saturating_sub(n).max(asked.start)..asked.end,
        }
    }
}
/// A token may stand before a place when it did so at least this often in the fitted split.
const LINK_MIN: u32 = 3;
/// A frame token has its own weights when it was counted at least this often.
const SHARED_MIN: u32 = 8;
/// A pair has a weight when it stood at a true place at least this often.
const PAIR_MIN: u32 = 3;
/// A pair's weight is kept when it is at least this far from zero (half a bit).
const PAIR_KEEP: i32 = 8;
/// A question token has an asking weight when it was counted at least this often.
const ASKING_MIN: u32 = 8;
/// A reply is read as a form up to this many tokens.
const REPLY_CAP: usize = 64;
/// A stored-reply form is kept when it is at least one in this many of its kind.
const FORM_SHARE: u32 = 100;
/// Scores are binned inside this bound when the floors are fitted.
const LIM: i32 = 16_384;
/// A pair of reply tokens says whether its row opens when it was counted at least this often.
const OPEN_MIN: u32 = 3;
/// The margins the fit chooses among, in [`lg`] units (16 is one bit).
const MARGINS: [i32; 10] = [0, 8, 16, 24, 32, 48, 64, 96, 128, 192];
/// A lead-in pair is kept when at least this many replies are the rest of their turn after it.
const REPEAT_MIN: u32 = 16;
/// A kept lead-in pair is answered so in at least `REPEAT_KEEP.0` of every `REPEAT_KEEP.1` turns
/// that hold it.
const REPEAT_KEEP: (u64, u64) = (7, 8);
/// The fit's account names each pair counted at least this often, kept or not.
const REPEAT_SHOWN: u32 = 4;

/// A multiply-and-rotate hasher for the packed integer keys of the count tables.
#[derive(Clone, Copy, Default)]
pub struct Mix(u64);

impl Hasher for Mix {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(u64::from(b));
        }
    }
    fn write_u64(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
    fn write_u32(&mut self, v: u32) {
        self.write_u64(u64::from(v));
    }
    fn write_usize(&mut self, v: usize) {
        self.write_u64(v as u64);
    }
}

type Map<K, V> = HashMap<K, V, BuildHasherDefault<Mix>>;

/// Sixteen times the base-two logarithm of `n`, from its leading bit and the four bits after it.
pub fn lg(n: u64) -> i32 {
    let n = n.max(1);
    let k = 63 - n.leading_zeros();
    let next = if k >= 4 { (n >> (k - 4)) & 15 } else { (n << (4 - k)) & 15 };
    (16 * k + next as u32) as i32
}

/// How much likelier a count is on the true side than on the other, in [`lg`] units: `t` of
/// `true_total` against `o` of `other_total`, each count raised by one.
fn odds(t: u32, true_total: u64, o: u32, other_total: u64) -> i32 {
    lg((u64::from(t) + 1).saturating_mul(other_total.saturating_add(1))) - lg((u64::from(o) + 1).saturating_mul(true_total.saturating_add(1)))
}

/// One more in a count that stops at its type's top and never wraps.
fn bump(n: &mut u32) {
    *n = n.saturating_add(1);
}

/// Whether a vocabulary, with the two frame values past it, fits the 16 bits a key gives a token.
fn fits_the_keys(vocab: u32) -> bool {
    vocab < u32::from(u16::MAX)
}

fn key2(j: usize, c: u32) -> u64 {
    ((j as u64) << 16) | u64::from(c)
}

/// The key of two neighbouring tokens, each under a vocabulary that fits the keys.
fn pair_key(before: u32, token: u32) -> u64 {
    (u64::from(before) << 16) | u64::from(token)
}

/// The key of three neighbouring tokens, each under a vocabulary that fits the keys.
fn triple_key(first: u32, before: u32, token: u32) -> u64 {
    (u64::from(first) << 32) | pair_key(before, token)
}

fn key3(q: u32, j: usize, c: u32) -> u64 {
    (u64::from(q) << 24) | key2(j, c)
}

/// The fitted tables, as a caller hands them to a session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceAsks {
    /// The hash of the token file the tables were fitted on.
    pub tokens_sha256: String,
    pub marks: PlaceMarks,
    pub vocab: u32,
    /// The tokens that may stand before a place; `vocab` stands for a turn's start.
    pub link: Vec<u32>,
    /// `[offset, frame token, weight when the question holds it, weight when it does not]`.
    pub shared: Vec<[i32; 4]>,
    /// For each offset, the two weights of a frame token with no row in `shared`.
    pub unseen: Vec<[i32; 2]>,
    /// `[question token, offset, frame token, weight]`.
    pub pairs: Vec<[i32; 4]>,
    /// The least score a place needs when the question asks for a stored fact: below it the
    /// answer is the stored reply.
    pub floor: i32,
    /// The least score a place needs when the question does not ask for one. Such questions are
    /// most of a conversation and almost none has a place, so this floor is fitted apart.
    pub floor_other: i32,
    /// When the best score reaches the floor, the place is the latest candidate within this
    /// much of it: a fact stated again later wins when it scores nearly as well.
    pub margin: i32,
    /// `[token before, token, 1 or 0]`: whether a reply row with that token after that one is
    /// open, by where the fitted split's replies start their value. A pair with no row is left
    /// to the place read's own class.
    pub opens: Vec<[i32; 3]>,
    /// `[question token, weight]`: whether a question asks for a stored fact.
    pub asking: Vec<[i32; 2]>,
    pub asking_prior: i32,
    pub asking_floor: i32,
    /// The stored reply for a fact never told, its end mark included.
    pub not_told: Vec<u32>,
    /// Every form the fitted split gives that reply in, the stored one first.
    pub forms: Vec<Vec<u32>>,
    /// `[token before, token]`: a pair after which, in the turn being answered, the rest of the
    /// turn is the reply asked for (claude-364), ascending. A table fitted before that holds
    /// none, and a table that holds none is written without the list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat: Vec<[u32; 2]>,
    /// `[token two before, token before, token]`: three tokens that lead in a phrase to repeat
    /// where the pair they end on does not (claude-364's addendum), ascending. Absent, like
    /// `repeat`, from a table that holds none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat3: Vec<[u32; 3]>,
}

impl PlaceAsks {
    /// Set the pair of the model marker's last two tokens open (claude-362): a reply's first row
    /// is the marker's last token, so a place the asks find is offered, and spoken, from it. The
    /// row of that pair is replaced or put in its sorted place. Returns what the pair was before:
    /// counted open, counted not open, or not counted. A marker of fewer than two tokens has no
    /// such pair and is an error.
    pub fn open_reply_start(&mut self) -> Result<Option<bool>, String> {
        let marker = &self.marks.assistant;
        let [a, b] = match marker.len().checked_sub(2) {
            Some(n) => [marker[n], marker[n + 1]],
            None => return Err(format!("the model marker has {} tokens and a reply start pair needs two", marker.len())),
        };
        let id = |t: u32| i32::try_from(t).map_err(|_| format!("the model marker names token {t}"));
        let key = [id(a)?, id(b)?];
        if let Some(row) = self.opens.iter_mut().find(|row| row[..2] == key) {
            let before = row[2] != 0;
            row[2] = 1;
            return Ok(Some(before));
        }
        let at = self.opens.partition_point(|row| row[..2] < key[..]);
        self.opens.insert(at, [key[0], key[1], 1]);
        Ok(None)
    }
}

/// What a question finds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    /// A place, and the end of the turn content it stands in.
    Place(usize, usize),
    /// The question asks for a stored fact and no place holds it.
    NotTold,
    /// The turn asks to have its own rest repeated (claude-364): that rest, as the positions
    /// of its first token and of the turn content's end.
    Repeat(usize, usize),
    Nothing,
}

/// The tables as a session reads them.
#[derive(Clone, Debug)]
pub struct AsksIndex {
    start: u32,
    link: Vec<bool>,
    shared: Map<u64, [i32; 2]>,
    unseen: [[i32; 2]; ASKS_FRAME],
    pairs: Map<u64, i32>,
    floor: i32,
    floor_other: i32,
    margin: i32,
    opens: Map<u64, bool>,
    asking: Map<u32, i32>,
    asking_prior: i32,
    asking_floor: i32,
    not_told: Vec<u32>,
    /// The keys of the lead-in pairs, ascending.
    repeat: Vec<u64>,
    /// The kept lead-in triples as [`triple_key`]s, ascending.
    repeat3: Vec<u64>,
    /// What of a frame and a question the read uses: all of both but in a census's cut.
    cut: Cut,
}

/// The frame of a candidate place `s` in a turn whose content is `lo..hi`. First the token at
/// each offset before the place, nearest first, with `start` standing once for the turn's start.
/// Then the turn's last tokens, the last one first, as far as they stand after the place.
/// `start + 1` fills every offset with no token, so a short statement is scored on as many
/// offsets as a long one.
fn frame(tokens: &[u32], lo: usize, hi: usize, s: usize, start: u32) -> ([u32; ASKS_FRAME], usize) {
    let mut out = [start + 1; ASKS_FRAME];
    for (n, slot) in out[..ASKS_BEFORE].iter_mut().enumerate() {
        if s < lo + n + 1 {
            *slot = start;
            break;
        }
        // A position past the tokens has no token before it to read.
        match tokens.get(s - n - 1) {
            Some(&token) => *slot = token,
            None => break,
        }
    }
    for (n, slot) in out[ASKS_BEFORE..].iter_mut().enumerate() {
        match hi.checked_sub(n + 1).filter(|&at| at > s && at < tokens.len()) {
            Some(at) => *slot = tokens[at],
            None => break,
        }
    }
    (out, ASKS_FRAME)
}

/// The distinct tokens of the part of a question `cut` reads, ascending; none when the range is
/// not inside the tokens.
fn question(tokens: &[u32], asked: &Range<usize>, cut: Cut) -> Vec<u32> {
    let mut q = tokens.get(cut.asked(asked)).unwrap_or(&[]).to_vec();
    q.sort_unstable();
    q.dedup();
    q
}

impl AsksIndex {
    pub fn new(asks: &PlaceAsks) -> Result<Self, String> {
        let vocab = asks.vocab;
        if !fits_the_keys(vocab) {
            return Err(format!("a vocabulary of {vocab} does not fit the asks' keys"));
        }
        if asks.unseen.len() != ASKS_FRAME {
            return Err(format!("the asks have {} unseen rows for a frame of {ASKS_FRAME}", asks.unseen.len()));
        }
        // A frame token is a token, the turn's start (`vocab`) or an offset past it (`vocab + 1`).
        let token = |v: i32| u32::try_from(v).ok().filter(|&t| t <= vocab + 1).ok_or_else(|| format!("the asks name token {v} past a vocabulary of {vocab}"));
        let word = |v: i32| u32::try_from(v).ok().filter(|&t| t < vocab).ok_or_else(|| format!("the asks name token {v} past a vocabulary of {vocab}"));
        let offset = |v: i32| usize::try_from(v).ok().filter(|j| (1..=ASKS_FRAME).contains(j)).ok_or_else(|| format!("the asks name offset {v} outside the frame"));
        let mut link = vec![false; vocab as usize + 1];
        for &t in &asks.link {
            *link.get_mut(t as usize).ok_or_else(|| format!("the asks link token {t} past a vocabulary of {vocab}"))? = true;
        }
        let mut shared = Map::default();
        for row in &asks.shared {
            shared.insert(key2(offset(row[0])?, token(row[1])?), [row[2], row[3]]);
        }
        let mut pairs = Map::default();
        for row in &asks.pairs {
            pairs.insert(key3(token(row[0])?, offset(row[1])?, token(row[2])?), row[3]);
        }
        let mut asking = Map::default();
        for row in &asks.asking {
            asking.insert(token(row[0])?, row[1]);
        }
        let mut opens = Map::default();
        for row in &asks.opens {
            opens.insert((u64::from(word(row[0])?) << 16) | u64::from(word(row[1])?), row[2] != 0);
        }
        if asks.margin < 0 {
            return Err(format!("the asks' margin is {}", asks.margin));
        }
        if asks.not_told.last().is_some_and(|&t| t != asks.marks.eos) {
            return Err("the asks' stored reply does not end at the end mark".to_owned());
        }
        let mut repeat = Vec::with_capacity(asks.repeat.len());
        for &[before, token] in &asks.repeat {
            if before >= vocab || token >= vocab {
                return Err(format!("the asks name the lead-in pair {before} {token} past a vocabulary of {vocab}"));
            }
            repeat.push(pair_key(before, token));
        }
        repeat.sort_unstable();
        repeat.dedup();
        let mut repeat3 = Vec::with_capacity(asks.repeat3.len());
        for &[first, before, token] in &asks.repeat3 {
            if first >= vocab || before >= vocab || token >= vocab {
                return Err(format!("the asks name the lead-in triple {first} {before} {token} past a vocabulary of {vocab}"));
            }
            repeat3.push(triple_key(first, before, token));
        }
        repeat3.sort_unstable();
        repeat3.dedup();
        let mut unseen = [[0i32; 2]; ASKS_FRAME];
        unseen.copy_from_slice(&asks.unseen);
        Ok(Self {
            start: vocab,
            link,
            shared,
            unseen,
            pairs,
            floor: asks.floor,
            floor_other: asks.floor_other,
            margin: asks.margin,
            opens,
            asking,
            asking_prior: asks.asking_prior,
            asking_floor: asks.asking_floor,
            not_told: asks.not_told.clone(),
            repeat,
            repeat3,
            cut: Cut::ALL,
        })
    }

    /// The same tables read under `cut`, for tables fitted under it ([`fit_cut`]): a census's
    /// reader. A session never calls this.
    pub fn with_cut(mut self, cut: Cut) -> Self {
        self.cut = cut;
        self
    }

    /// The stored reply for a fact never told; empty when the tables hold none.
    pub fn not_told(&self) -> &[u32] {
        &self.not_told
    }

    /// Whether a reply row with `token` after `before` is open, when the tables say.
    pub fn opens(&self, before: u32, token: u32) -> Option<bool> {
        // A value past the vocabulary is no token, and its key would be another pair's.
        if before >= self.start || token >= self.start {
            return None;
        }
        self.opens.get(&((u64::from(before) << 16) | u64::from(token))).copied()
    }

    /// The place among `scored` candidates, each `(score, place, ..)` in the order of the
    /// conversation: none unless the best score reaches `floor`, and then the latest candidate
    /// within the margin of the best.
    fn pick<T: Copy>(&self, scored: &[(i32, T)], floor: i32) -> Option<T> {
        let best = scored.iter().map(|c| c.0).max().filter(|&best| best >= floor)?;
        scored.iter().rev().find(|c| c.0 >= best.saturating_sub(self.margin)).map(|c| c.1)
    }

    /// Whether a question of the distinct tokens `q` asks for a stored fact.
    pub fn asks_a_fact(&self, q: &[u32]) -> bool {
        self.asking(q) >= self.asking_floor
    }

    /// The score of the candidate place `s` in a turn whose content is `lo..hi`, for a question
    /// of the distinct tokens `q`; `None` when `s` is no candidate.
    pub fn score(&self, tokens: &[u32], lo: usize, hi: usize, s: usize, q: &[u32]) -> Option<i32> {
        // A place is a token of the turn: a position outside the turn or the tokens is none.
        if s < lo || s >= hi || s >= tokens.len() {
            return None;
        }
        let (f, n) = frame(tokens, lo, hi, s, self.start);
        if !self.link.get(f[0] as usize).copied().unwrap_or(false) {
            return None;
        }
        let f = self.cut.frame(f, self.start);
        // The weights are a file's: the sum stops at the type's ends and never wraps.
        let mut score = 0i32;
        for (j, &c) in f[..n].iter().enumerate() {
            let held = q.binary_search(&c).is_ok();
            let side = usize::from(!held);
            score = score.saturating_add(self.shared.get(&key2(j + 1, c)).map_or(self.unseen[j][side], |w| w[side]));
            for &qt in q {
                if qt != c {
                    if let Some(&w) = self.pairs.get(&key3(qt, j + 1, c)) {
                        score = score.saturating_add(w);
                    }
                }
            }
        }
        Some(score)
    }

    /// The score that a question of the distinct tokens `q` asks for a stored fact.
    pub fn asking(&self, q: &[u32]) -> i32 {
        q.iter().filter_map(|t| self.asking.get(t)).fold(self.asking_prior, |sum, &w| sum.saturating_add(w))
    }

    /// Every candidate of the finished user turns `prior`, in order, with its score.
    fn candidates(&self, tokens: &[u32], prior: &[Range<usize>], q: &[u32], mut each: impl FnMut(usize, usize, i32)) {
        for turn in prior {
            for s in turn.clone() {
                if let Some(score) = self.score(tokens, turn.start, turn.end, s, q) {
                    each(s, turn.end, score);
                }
            }
        }
    }

    /// The distinct tokens this reader reads of the question in `asked`.
    fn asked(&self, tokens: &[u32], asked: &Range<usize>) -> Vec<u32> {
        question(tokens, asked, self.cut)
    }

    /// Where the phrase starts that the turn in `asked` asks to have repeated (claude-364): the
    /// token after the first lead-in in the turn, when a token of the turn stands there. A
    /// lead-in is a kept pair or a kept triple, and the first is the one that ends first.
    /// `None` with no lead-in in the turn, with nothing after the first one, or with tables
    /// that hold none.
    pub fn repeated(&self, tokens: &[u32], asked: &Range<usize>) -> Option<usize> {
        if self.repeat.is_empty() && self.repeat3.is_empty() {
            return None;
        }
        let turn = tokens.get(asked.clone())?;
        // A value past the vocabulary is no token, and its key would be another lead-in's.
        let token = |w: &[u32]| w.iter().all(|&t| t < self.start);
        let pair = |w: &[u32]| token(w) && self.repeat.binary_search(&pair_key(w[0], w[1])).is_ok();
        let triple = |w: &[u32]| token(w) && self.repeat3.binary_search(&triple_key(w[0], w[1], w[2])).is_ok();
        // `last` is the lead-in's last token: a pair starts one before it, a triple two.
        let last = (1..turn.len()).find(|&last| pair(&turn[last - 1..=last]) || (last >= 2 && triple(&turn[last - 2..=last])))?;
        Some(asked.start + last + 1).filter(|&start| start < asked.end)
    }

    /// What the question in `asked` finds among the finished user turns `prior`. A turn that asks
    /// to have its own rest repeated finds that rest, before any place is looked for: the answer
    /// is in the turn itself.
    pub fn find(&self, tokens: &[u32], prior: &[Range<usize>], asked: &Range<usize>) -> Asked {
        if let Some(start) = self.repeated(tokens, asked) {
            return Asked::Repeat(start, asked.end);
        }
        let q = self.asked(tokens, asked);
        let mut scored: Vec<(i32, (usize, usize))> = Vec::new();
        self.candidates(tokens, prior, &q, |s, end, score| scored.push((score, (s, end))));
        let asking = self.asks_a_fact(&q);
        match self.pick(&scored, if asking { self.floor } else { self.floor_other }) {
            Some((s, end)) => Asked::Place(s, end),
            None if asking && !self.not_told.is_empty() => Asked::NotTold,
            None => Asked::Nothing,
        }
    }
}

/// A reply that answers a user turn: that turn's content, the finished user turns before it, and
/// the reply's first row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskCase {
    pub asked: Range<usize>,
    pub prior: Vec<Range<usize>>,
    pub reply: usize,
}

/// Every reply of `tokens` that answers a user turn, as the place register sees it: the first
/// model turn after each user turn.
pub fn ask_cases(tokens: &[u32], marks: &PlaceMarks) -> Vec<AskCase> {
    let mut turns = PlaceTurns::default();
    let mut answered = false;
    let mut out = Vec::new();
    for t in 0..tokens.len() {
        if tokens[t] == marks.bos {
            answered = false;
        }
        match turns.push(&tokens[..=t], marks) {
            Some(PlaceRole::User) => answered = false,
            Some(PlaceRole::Assistant) => {
                if !answered {
                    if let Some((asked, prior)) = turns.users().split_last() {
                        out.push(AskCase {
                            asked: asked.clone(),
                            prior: prior.to_vec(),
                            reply: t + 1,
                        });
                    }
                }
                answered = true;
            }
            Some(PlaceRole::System) | None => {}
        }
    }
    out
}

/// The reply that starts at row `a`, to its end mark.
fn reply_form<'a>(tokens: &'a [u32], a: usize, marks: &PlaceMarks) -> Option<&'a [u32]> {
    let rest = tokens.get(a..)?;
    let end = rest.iter().take(REPLY_CAP).position(|&x| x == marks.eos)?;
    Some(&rest[..=end])
}

fn bin(score: i32) -> usize {
    (score.clamp(-LIM, LIM) + LIM) as usize
}

/// The floor that keeps the most right picks over wrong ones, from each side's change at each
/// bin: the count of picks a floor at that bin keeps is the running sum up to it.
fn best_floor(right: &[i64], wrong: &[i64]) -> (i32, i64, i64) {
    let (mut r, mut w) = (0i64, 0i64);
    let mut best = (i32::MAX, 0i64, 0i64);
    for (i, (dr, dw)) in right.iter().zip(wrong).enumerate() {
        r += dr;
        w += dw;
        // Bin 0 holds every score at or under the least one, which no floor the find reads keeps:
        // the floors start one above it.
        if (1..=2 * LIM as usize).contains(&i) && r - w >= best.1 - best.2 {
            best = (i as i32 - LIM, r, w);
        }
    }
    best
}

/// What the asks do on a split whose true places are known.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AskCensus {
    /// Replies with a place: found right, found wrong, raised "not told", nothing.
    pub recall: [usize; 4],
    /// Replies that say a fact was not told: a place set, "not told" raised, nothing.
    pub abstain: [usize; 3],
    /// Every other reply: a place set, "not told" raised, nothing.
    pub general: [usize; 3],
    /// Replies with a place, by the row before their value: a fitted pair opens it, a fitted
    /// pair does not, no fitted pair; and the replies where a fitted pair opens an earlier row.
    pub opens: [usize; 4],
    /// Replies whose turn the find answers with a repeat (claude-364): among the replies with a
    /// place, those that say a fact was not told, and every other. Each is also counted under
    /// "nothing" above: a repeat sets no place and raises no "not told".
    pub repeat: [usize; 3],
}

impl AskCensus {
    pub fn lines(&self, split: &str) -> Vec<String> {
        let (r, a, g) = (self.recall, self.abstain, self.general);
        let mut lines = vec![
            format!("asks-census split={split} recall={} true_place={} wrong_place={} not_told={} nothing={}", r.iter().sum::<usize>(), r[0], r[1], r[2], r[3]),
            format!("asks-census split={split} abstain={} place_set={} not_told={} nothing={}", a.iter().sum::<usize>(), a[0], a[1], a[2]),
            format!("asks-census split={split} general={} place_set={} not_told={} nothing={}", g.iter().sum::<usize>(), g[0], g[1], g[2]),
            format!(
                "asks-census split={split} value_row_open={} value_row_not_open={} value_row_no_pair={} an_earlier_row_open={}",
                self.opens[0], self.opens[1], self.opens[2], self.opens[3]
            ),
        ];
        // Tables with no lead-in pair print the lines they always did.
        if self.repeat != [0; 3] {
            lines.push(format!("asks-census split={split} repeat recall={} abstain={} general={}", self.repeat[0], self.repeat[1], self.repeat[2]));
        }
        lines
    }
}

/// The asks on a split: `places` are its true places, `[reply start, place, ..]`, and a reply
/// with no place counts as one that says a fact was not told when it is one of `forms`. The
/// second value lists up to `show` replies whose place was missed, as token ids.
pub fn census(tokens: &[u32], marks: &PlaceMarks, places: &[[u32; 4]], index: &AsksIndex, forms: &[Vec<u32>], show: usize) -> (AskCensus, Vec<String>) {
    let truth: Map<usize, usize> = places.iter().map(|p| (p[0] as usize, p[1] as usize)).collect();
    let forms: HashSet<&[u32]> = forms.iter().map(Vec::as_slice).collect();
    let mut out = AskCensus::default();
    let mut missed = Vec::new();
    for p in places {
        let (reply, value) = (p[0] as usize, (p[2] as usize).min(tokens.len()));
        let rows = reply.saturating_sub(1).max(1)..value;
        let open = |row: usize| index.opens(tokens[row - 1], tokens[row]);
        if let Some(last) = rows.clone().last() {
            out.opens[match open(last) {
                Some(true) => 0,
                Some(false) => 1,
                None => 2,
            }] += 1;
            out.opens[3] += usize::from(rows.clone().any(|row| row != last && open(row) == Some(true)));
        }
    }
    for case in ask_cases(tokens, marks) {
        let found = index.find(tokens, &case.prior, &case.asked);
        match truth.get(&case.reply) {
            Some(&place) => {
                let slot = match found {
                    Asked::Place(s, _) if s == place => 0,
                    Asked::Place(..) => 1,
                    Asked::NotTold => 2,
                    Asked::Nothing | Asked::Repeat(..) => 3,
                };
                out.recall[slot] += 1;
                out.repeat[0] += usize::from(matches!(found, Asked::Repeat(..)));
                if slot != 0 && missed.len() < show {
                    let stated = case.prior.iter().find(|turn| turn.contains(&place)).cloned().unwrap_or(place..place);
                    let at = index.score(tokens, stated.start, stated.end, place, &index.asked(tokens, &case.asked));
                    missed.push(format!(
                        "asks-miss {} | asked={:?} | stated={:?} | place_offset={} | got={found:?} | place_score={at:?} floor={}",
                        ["right", "wrong", "not_told", "nothing"][slot],
                        &tokens[case.asked.clone()],
                        // A kit's place may lie past the tokens: then no statement is shown.
                        tokens.get(stated.clone()).unwrap_or(&[]),
                        place - stated.start,
                        index.floor
                    ));
                }
            }
            None => {
                let slot = match found {
                    Asked::Place(..) => 0,
                    Asked::NotTold => 1,
                    Asked::Nothing | Asked::Repeat(..) => 2,
                };
                let says_not_told = reply_form(tokens, case.reply, marks).is_some_and(|form| forms.contains(form));
                out.repeat[if says_not_told { 1 } else { 2 }] += usize::from(matches!(found, Asked::Repeat(..)));
                if says_not_told {
                    out.abstain[slot] += 1;
                    // A place set where the reply says the fact was never told: the turn it
                    // was taken from, in the same line form as a missed place.
                    if let (Asked::Place(s, _), true) = (found, missed.len() < show) {
                        let taken = case.prior.iter().find(|turn| turn.contains(&s)).cloned().unwrap_or(s..s);
                        let at = index.score(tokens, taken.start, taken.end, s, &index.asked(tokens, &case.asked));
                        missed.push(format!(
                            "asks-miss invented | asked={:?} | stated={:?} | place_offset={} | got={found:?} | place_score={at:?} floor={}",
                            &tokens[case.asked.clone()],
                            &tokens[taken.clone()],
                            s - taken.start,
                            index.floor
                        ));
                    }
                } else {
                    out.general[slot] += 1;
                }
            }
        }
    }
    (out, missed)
}

/// Count the lead-ins of a split (claude-364): the pairs and the triples of neighbouring tokens
/// in a user turn after which the rest of the turn is the reply. Each is counted once in each
/// turn that holds it with a token after it, at its first place there: `turns` such turns, and
/// `replies` of them answered with exactly the rest of the turn from that place and the end
/// mark. A pair is kept when `replies` is at least [`REPEAT_MIN`] and at least
/// [`REPEAT_KEEP`]`.0` in every `REPEAT_KEEP.1` of `turns`; a triple by the same two counts
/// when the pair it ends on is not kept. Returns the kept pairs and the kept triples, each
/// ascending, and the count's own account, line by line.
#[allow(clippy::type_complexity)]
pub fn fit_repeat(tokens: &[u32], marks: &PlaceMarks, vocab: u32) -> Result<(Vec<[u32; 2]>, Vec<[u32; 3]>, Vec<String>), String> {
    if !fits_the_keys(vocab) {
        return Err(format!("a vocabulary of {vocab} does not fit the asks' keys"));
    }
    if let Some(&t) = tokens.iter().find(|&&t| t >= vocab) {
        return Err(format!("token {t} is past a vocabulary of {vocab}"));
    }
    let cases = ask_cases(tokens, marks);
    // For each pair and each triple: the turns that hold it with a token after it, and those
    // answered so.
    let mut counts: Map<u64, [u32; 2]> = Map::default();
    let mut counts3: Map<u64, [u32; 2]> = Map::default();
    let mut met: HashSet<u64> = HashSet::new();
    let mut met3: HashSet<u64> = HashSet::new();
    for case in &cases {
        let turn = &tokens[case.asked.clone()];
        // The reply without its end mark; a reply with none in reach is no repeat of anything.
        let said = reply_form(tokens, case.reply, marks).map(|form| &form[..form.len() - 1]);
        met.clear();
        met3.clear();
        for at in 0..turn.len().saturating_sub(2) {
            let rest = said == Some(&turn[at + 2..]);
            if met.insert(pair_key(turn[at], turn[at + 1])) {
                let n = counts.entry(pair_key(turn[at], turn[at + 1])).or_default();
                bump(&mut n[0]);
                if rest {
                    bump(&mut n[1]);
                }
            }
            // The triple that ends on this pair, when the turn holds a token before it.
            if at >= 1 && met3.insert(triple_key(turn[at - 1], turn[at], turn[at + 1])) {
                let n = counts3.entry(triple_key(turn[at - 1], turn[at], turn[at + 1])).or_default();
                bump(&mut n[0]);
                if rest {
                    bump(&mut n[1]);
                }
            }
        }
    }
    let keep = |n: &[u32; 2]| n[1] >= REPEAT_MIN && u64::from(n[1]) * REPEAT_KEEP.1 >= u64::from(n[0]) * REPEAT_KEEP.0;
    let mut shown: Vec<(u64, [u32; 2])> = counts.iter().filter(|(_, n)| n[1] >= REPEAT_SHOWN).map(|(&key, &n)| (key, n)).collect();
    shown.sort_unstable();
    let kept: Vec<[u32; 2]> = shown.iter().filter(|(_, n)| keep(n)).map(|&(key, _)| [(key >> 16) as u32, (key & 0xffff) as u32]).collect();
    let mut lines = vec![format!("asks-repeat replies={} pairs_counted={} kept={}", cases.len(), counts.len(), kept.len())];
    for (key, n) in &shown {
        lines.push(format!(
            "asks-repeat pair before={} token={} turns={} replies_the_rest={} kept={}",
            key >> 16,
            key & 0xffff,
            n[0],
            n[1],
            u8::from(keep(n))
        ));
    }
    // A triple is for the table only where the pair it ends on is not kept.
    let pair_kept = |key: u64| counts.get(&(key & 0xffff_ffff)).is_some_and(|n| keep(n));
    let mut shown3: Vec<(u64, [u32; 2])> = counts3.iter().filter(|(&key, n)| n[1] >= REPEAT_SHOWN && !pair_kept(key)).map(|(&key, &n)| (key, n)).collect();
    shown3.sort_unstable();
    let kept3: Vec<[u32; 3]> = shown3.iter().filter(|(_, n)| keep(n)).map(|&(key, _)| [(key >> 32) as u32, ((key >> 16) & 0xffff) as u32, (key & 0xffff) as u32]).collect();
    lines.push(format!("asks-repeat triples_counted={} kept={}", counts3.len(), kept3.len()));
    for (key, n) in &shown3 {
        lines.push(format!(
            "asks-repeat triple first={} before={} token={} turns={} replies_the_rest={} kept={}",
            key >> 32,
            (key >> 16) & 0xffff,
            key & 0xffff,
            n[0],
            n[1],
            u8::from(keep(n))
        ));
    }
    Ok((kept, kept3, lines))
}

/// Fit the asks on a split by counting: `places` are its true places, `[reply start, place, ..]`
/// in the split's positions. The second value is the fit's own account, line by line.
pub fn fit(tokens: &[u32], marks: &PlaceMarks, vocab: u32, places: &[[u32; 4]], tokens_sha256: String) -> Result<(PlaceAsks, Vec<String>), String> {
    fit_cut(tokens, marks, vocab, places, tokens_sha256, Cut::ALL)
}

/// [`fit`] under `cut`: the counts, the weights, the floors and the margin are all taken from
/// what the cut leaves of each frame and each question. The tables are for a reader under the
/// same cut ([`AsksIndex::with_cut`]) and for a census; they are not a session's.
pub fn fit_cut(tokens: &[u32], marks: &PlaceMarks, vocab: u32, places: &[[u32; 4]], tokens_sha256: String, cut: Cut) -> Result<(PlaceAsks, Vec<String>), String> {
    if !fits_the_keys(vocab) {
        return Err(format!("a vocabulary of {vocab} does not fit the asks' keys"));
    }
    if let Some(&t) = tokens.iter().find(|&&t| t >= vocab) {
        return Err(format!("token {t} is past a vocabulary of {vocab}"));
    }
    if places.is_empty() {
        return Err("the kit holds no place: there is nothing to fit".to_owned());
    }
    let start = vocab;
    let truth: Map<usize, usize> = places.iter().map(|p| (p[0] as usize, p[1] as usize)).collect();
    let cases = ask_cases(tokens, marks);
    let placed = cases.iter().filter(|c| truth.contains_key(&c.reply)).count();
    if placed != places.len() {
        return Err(format!("{} of the kit's {} places start a reply the turns give", placed, places.len()));
    }
    let mut lines = vec![format!("asks-fit replies={} with_a_place={placed}", cases.len())];

    // The tokens that may stand before a place.
    let mut before = vec![0u32; vocab as usize + 1];
    let mut outside = 0usize;
    for case in &cases {
        let Some(&place) = truth.get(&case.reply) else { continue };
        match case.prior.iter().find(|turn| turn.contains(&place)) {
            Some(turn) => bump(&mut before[frame(tokens, turn.start, turn.end, place, start).0[0] as usize]),
            None => outside += 1,
        }
    }
    let link: Vec<bool> = before.iter().map(|&n| n >= LINK_MIN).collect();
    lines.push(format!("asks-fit link tokens={} place_outside_the_earlier_user_turns={outside}", link.iter().filter(|&&on| on).count()));

    // Where a reply starts its value: each pair of a reply row's token and the one before it,
    // from the role marker's last row on, by whether the value starts at the next row.
    let mut open: Map<u64, [u32; 2]> = Map::default();
    for p in places {
        let (reply, value) = (p[0] as usize, p[2] as usize);
        if value < reply || value > tokens.len() {
            return Err(format!("the kit's reply at {reply} has its value at {value}"));
        }
        for row in reply.saturating_sub(1).max(1)..value {
            bump(&mut open.entry((u64::from(tokens[row - 1]) << 16) | u64::from(tokens[row])).or_default()[usize::from(row + 1 == value)]);
        }
    }
    let mut open_rows: Vec<[i32; 3]> = open
        .iter()
        .filter(|(_, n)| n[0].saturating_add(n[1]) >= OPEN_MIN)
        .map(|(&key, n)| [(key >> 16) as i32, (key & 0xFFFF) as i32, i32::from(n[1] > n[0])])
        .collect();
    open_rows.sort_unstable();
    lines.push(format!("asks-fit open pairs={} that_open={}", open_rows.len(), open_rows.iter().filter(|row| row[2] == 1).count()));

    // The pairs that stand at a true place, and the questions that have one.
    let mut pair: Map<u64, [u32; 2]> = Map::default();
    let mut recall_questions: HashSet<&[u32]> = HashSet::new();
    for case in &cases {
        let Some(&place) = truth.get(&case.reply) else { continue };
        recall_questions.insert(&tokens[case.asked.clone()]);
        let Some(turn) = case.prior.iter().find(|turn| turn.contains(&place)) else { continue };
        let (f, n) = frame(tokens, turn.start, turn.end, place, start);
        if !link[f[0] as usize] {
            continue;
        }
        let f = cut.frame(f, start);
        let q = question(tokens, &case.asked, cut);
        for (j, &c) in f[..n].iter().enumerate() {
            for &qt in &q {
                if qt != c {
                    bump(&mut pair.entry(key3(qt, j + 1, c)).or_default()[0]);
                }
            }
        }
    }

    // Every candidate of every reply: the frame tokens by side, and the pairs away from a place.
    // `[held at a true place, not held there, held at another candidate, not held there]`.
    let mut shared: Map<u64, [u32; 4]> = Map::default();
    let (mut true_total, mut other_total) = (0u64, 0u64);
    let mut asking: Map<u32, [u32; 2]> = Map::default();
    let mut asking_total = [0u64; 2];
    let mut forms: HashMap<&[u32], u32> = HashMap::new();
    let asks_a_fact = |case: &AskCase| truth.contains_key(&case.reply) || recall_questions.contains(&tokens[case.asked.clone()]);
    for case in &cases {
        let q = question(tokens, &case.asked, cut);
        let place = truth.get(&case.reply).copied();
        let side = usize::from(!asks_a_fact(case));
        asking_total[side] += 1;
        for &qt in &q {
            bump(&mut asking.entry(qt).or_default()[side]);
        }
        if place.is_none() && side == 0 {
            if let Some(form) = reply_form(tokens, case.reply, marks) {
                bump(forms.entry(form).or_default());
            }
        }
        for turn in &case.prior {
            for s in turn.clone() {
                let (f, n) = frame(tokens, turn.start, turn.end, s, start);
                if !link[f[0] as usize] {
                    continue;
                }
                let f = cut.frame(f, start);
                let is_true = place == Some(s);
                if is_true {
                    true_total += 1;
                } else {
                    other_total += 1;
                }
                for (j, &c) in f[..n].iter().enumerate() {
                    let held = q.binary_search(&c).is_ok();
                    bump(&mut shared.entry(key2(j + 1, c)).or_default()[usize::from(!held) + 2 * usize::from(!is_true)]);
                    if !is_true {
                        for &qt in &q {
                            if qt != c {
                                if let Some(seen) = pair.get_mut(&key3(qt, j + 1, c)) {
                                    bump(&mut seen[1]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    lines.push(format!("asks-fit candidates true={true_total} other={other_total} frame_tokens={} pairs_at_a_true_place={}", shared.len(), pair.len()));

    // The weights.
    let mut shared_rows = Vec::new();
    let mut low = [[0u32; 4]; ASKS_FRAME];
    for (&key, n) in &shared {
        let (j, c) = ((key >> 16) as usize, (key & 0xFFFF) as i32);
        if n.iter().fold(0u32, |sum, &count| sum.saturating_add(count)) >= SHARED_MIN {
            shared_rows.push([j as i32, c, odds(n[0], true_total, n[2], other_total), odds(n[1], true_total, n[3], other_total)]);
        } else {
            for (sum, &count) in low[j - 1].iter_mut().zip(n) {
                *sum = sum.saturating_add(count);
            }
        }
    }
    shared_rows.sort_unstable();
    let unseen: Vec<[i32; 2]> = low.iter().map(|n| [odds(n[0], true_total, n[2], other_total), odds(n[1], true_total, n[3], other_total)]).collect();
    let mut pair_rows = Vec::new();
    for (&key, seen) in &pair {
        if seen[0] < PAIR_MIN {
            continue;
        }
        let base = shared.get(&(key & 0xFF_FFFF)).copied().unwrap_or_default();
        let weight = odds(seen[0], u64::from(base[0]) + u64::from(base[1]) + 1, seen[1], u64::from(base[2]) + u64::from(base[3]) + 1);
        if weight.abs() >= PAIR_KEEP {
            pair_rows.push([(key >> 24) as i32, ((key >> 16) & 0xFF) as i32, (key & 0xFFFF) as i32, weight]);
        }
    }
    pair_rows.sort_unstable();
    let mut asking_rows: Vec<[i32; 2]> = asking
        .iter()
        .filter(|(_, n)| n[0].saturating_add(n[1]) >= ASKING_MIN)
        .map(|(&t, n)| [t as i32, odds(n[0], asking_total[0], n[1], asking_total[1])])
        .collect();
    asking_rows.sort_unstable();
    let asking_prior = lg(asking_total[0] + 1) - lg(asking_total[1] + 1);
    let mut ranked: Vec<(&[u32], u32)> = forms.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let said: u64 = ranked.iter().map(|&(_, n)| u64::from(n)).sum();
    let kept: Vec<Vec<u32>> = ranked.iter().filter(|&&(_, n)| u64::from(n) * u64::from(FORM_SHARE) >= said).map(|(form, _)| form.to_vec()).collect();
    lines.push(format!(
        "asks-fit weights shared={} pairs={} asking={} asks_a_fact={} other={} not_told_replies={said} forms_kept={} top_forms={:?}",
        shared_rows.len(),
        pair_rows.len(),
        asking_rows.len(),
        asking_total[0],
        asking_total[1],
        kept.len(),
        ranked.iter().take(4).map(|(_, n)| *n).collect::<Vec<_>>()
    ));
    let mut asks = PlaceAsks {
        tokens_sha256,
        marks: marks.clone(),
        vocab,
        link: (0..=vocab).filter(|&t| link[t as usize]).collect(),
        shared: shared_rows,
        unseen,
        pairs: pair_rows,
        floor: 0,
        floor_other: 0,
        margin: 0,
        opens: open_rows,
        asking: asking_rows,
        asking_prior,
        asking_floor: 0,
        not_told: kept.first().cloned().unwrap_or_default(),
        forms: kept,
        repeat: Vec::new(),
        repeat3: Vec::new(),
    };

    // First, which questions ask for a stored fact: the asking floor, from every reply's
    // question scored with the asking weights.
    let index = AsksIndex::new(&asks)?.with_cut(cut);
    let bins = 2 * LIM as usize + 2;
    let mut asked_hist = [vec![0i64; bins], vec![0i64; bins]];
    for case in &cases {
        let side = usize::from(!asks_a_fact(case));
        asked_hist[side][0] += 1;
        asked_hist[side][bin(index.asking(&index.asked(tokens, &case.asked))) + 1] -= 1;
    }
    let (asking_at, asks_right, asks_wrong) = best_floor(&asked_hist[0], &asked_hist[1]);
    asks.asking_floor = asking_at;
    lines.push(format!("asks-fit asking floor={asking_at} asks_a_fact_kept={asks_right} other_kept={asks_wrong}"));

    // Then the two floors and the margin, from every reply's candidates scored with the
    // weights. A question that asks for a stored fact is one group and every other question the
    // second: the second is most of a split and almost none of it has a place, so one floor for
    // both would be set by it and would deny plain statements to the first. For each margin and
    // group, each side's change at each bin: a pick whose best score is `a` is kept by every
    // floor up to `a`.
    let mut picked = vec![[[vec![0i64; bins], vec![0i64; bins]], [vec![0i64; bins], vec![0i64; bins]]]; MARGINS.len()];
    let mut scored: Vec<(i32, bool)> = Vec::new();
    for case in &cases {
        let q = index.asked(tokens, &case.asked);
        let place = truth.get(&case.reply).copied();
        let group = usize::from(index.asking(&q) < asking_at);
        scored.clear();
        index.candidates(tokens, &case.prior, &q, |s, _, score| scored.push((score, place == Some(s))));
        let Some(best) = scored.iter().map(|c| c.0).max() else { continue };
        for (hist, &margin) in picked.iter_mut().zip(&MARGINS) {
            // The latest candidate within the margin of the best, as the find takes it.
            let right = scored.iter().rev().find(|c| c.0 >= best.saturating_sub(margin)).is_some_and(|c| c.1);
            hist[group][usize::from(!right)][0] += 1;
            hist[group][usize::from(!right)][bin(best) + 1] -= 1;
        }
    }
    let mut decided = (0i32, 0i32, 0i32, i64::MIN);
    for (hist, &margin) in picked.iter().zip(&MARGINS) {
        let (at, right, wrong) = best_floor(&hist[0][0], &hist[0][1]);
        let (other_at, other_right, other_wrong) = best_floor(&hist[1][0], &hist[1][1]);
        lines.push(format!(
            "asks-fit margin={margin} floor={at} right={right} wrong={wrong} floor_other={other_at} right={other_right} wrong={other_wrong}"
        ));
        let gain = right - wrong + other_right - other_wrong;
        if gain > decided.3 {
            decided = (margin, at, other_at, gain);
        }
    }
    asks.margin = decided.0;
    asks.floor = decided.1;
    asks.floor_other = decided.2;
    lines.push(format!("asks-fit decide margin={} floor={} floor_other={}", asks.margin, asks.floor, asks.floor_other));
    Ok((asks, lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOS: u32 = 0;
    const EOS: u32 = 1;
    const SEP: u32 = 2;
    const USER: u32 = 4;
    const MODEL: u32 = 5;
    const SAYS: u32 = 10;
    const IS: u32 = 11;
    const WHAT: u32 = 12;
    const OK: u32 = 13;
    const NOT: u32 = 14;
    const TOLD: u32 = 15;
    const VOCAB: u32 = 64;

    fn marks() -> PlaceMarks {
        PlaceMarks {
            bos: BOS,
            eos: EOS,
            sep: vec![SEP],
            system: vec![3],
            user: vec![USER],
            assistant: vec![MODEL],
        }
    }

    /// Documents that each state two keyed values in two user turns and then ask for one key:
    /// a stated one, an unstated one, or a general question. Returns the tokens and the places.
    fn corpus(docs: usize) -> (Vec<u32>, Vec<[u32; 4]>) {
        corpus_from(0, docs)
    }

    /// The documents numbered `first..first + docs`. A document's keys, values and kind repeat
    /// only every 420, so two ranges inside one period share no document.
    fn corpus_from(first: usize, docs: usize) -> (Vec<u32>, Vec<[u32; 4]>) {
        let (mut x, mut places) = (Vec::new(), Vec::new());
        for d in first..first + docs {
            let keys = [20 + (d % 5) as u32, 20 + ((d + 2) % 5) as u32];
            let values = [30 + (d % 7) as u32, 40 + (d % 3) as u32];
            x.push(BOS);
            let mut at = [0usize; 2];
            for i in 0..2 {
                x.extend([USER, SAYS, keys[i], IS]);
                at[i] = x.len();
                x.extend([values[i], SEP, MODEL, OK, EOS, SEP]);
            }
            match d % 4 {
                0 | 1 => {
                    let i = d % 2;
                    x.extend([USER, WHAT, keys[i], SEP, MODEL]);
                    places.push([x.len() as u32, at[i] as u32, x.len() as u32, 1]);
                    x.extend([values[i], EOS, SEP]);
                }
                2 => {
                    let other = 20 + ((d + 1) % 5) as u32;
                    x.extend([USER, WHAT, other, SEP, MODEL, NOT, TOLD, EOS, SEP]);
                }
                _ => x.extend([USER, 16, 17, SEP, MODEL, 18, EOS, SEP]),
            }
        }
        (x, places)
    }

    #[test]
    fn lg_rises_with_its_argument_and_doubles_by_sixteen() {
        assert_eq!(lg(1), 0);
        assert_eq!(lg(2), 16);
        assert_eq!(lg(3), 24);
        assert_eq!(lg(1 << 20), 320);
        assert!((1..4096u64).all(|n| lg(n) <= lg(n + 1)));
    }

    #[test]
    fn a_fitted_table_finds_the_stated_value_and_raises_not_told() {
        let (x, places) = corpus(240);
        let (asks, lines) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        assert_eq!(asks.not_told, vec![NOT, TOLD, EOS], "{lines:#?}");
        let index = AsksIndex::new(&asks).expect("the index");
        // Documents the fit never counted.
        let (held, held_places) = corpus_from(240, 60);
        let (counts, missed) = census(&held, &marks(), &held_places, &index, &asks.forms, 4);
        assert_eq!(counts.recall, [30, 0, 0, 0], "{missed:#?} {lines:#?}");
        assert_eq!(counts.abstain, [0, 15, 0]);
        assert_eq!(counts.general, [0, 0, 15 + 120]);
    }

    fn written(asks: &PlaceAsks) -> String {
        serde_json::to_string(asks).expect("written")
    }

    /// The first reply of the corpus that has a place, and its question's key.
    fn a_recall(x: &[u32], places: &[[u32; 4]]) -> (AskCase, u32) {
        let case = ask_cases(x, &marks()).into_iter().find(|c| places.iter().any(|p| p[0] as usize == c.reply)).expect("a reply with a place");
        let key = x[case.asked.end - 1];
        (case, key)
    }

    #[test]
    fn a_cut_that_keeps_everything_writes_the_table_of_the_fit() {
        let (x, places) = corpus(240);
        let whole = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit").0;
        // More last tokens than a question has is the whole question.
        let cut = Cut { slots: u8::MAX, question_last: 64 };
        let kept = fit_cut(&x, &marks(), VOCAB, &places, "test".to_owned(), cut).expect("the fit").0;
        assert_eq!(written(&kept), written(&whole));
    }

    #[test]
    fn a_cut_fit_counts_no_token_at_an_offset_it_drops_and_no_question_token_it_does_not_read() {
        let (x, places) = corpus(240);
        let table = |cut: Cut| fit_cut(&x, &marks(), VOCAB, &places, "test".to_owned(), cut).expect("the fit").0;
        // A row at an offset past the second that holds a token and not the filler.
        let holds = |offset: i32, token: i32| offset > 2 && token != VOCAB as i32 + 1;
        let past_two = |asks: &PlaceAsks| asks.shared.iter().filter(|row| holds(row[0], row[1])).count() + asks.pairs.iter().filter(|row| holds(row[1], row[2])).count();
        let whole = table(Cut::ALL);
        assert_ne!(past_two(&whole), 0);
        assert_eq!(past_two(&table(Cut { slots: 0b11, question_last: 0 })), 0);
        // The last token of a question is its key: the word before it is not read.
        let reads_what = |asks: &PlaceAsks| asks.pairs.iter().any(|row| row[0] == WHAT as i32) || asks.asking.iter().any(|row| row[0] == WHAT as i32);
        assert!(reads_what(&whole));
        assert!(!reads_what(&table(Cut { slots: u8::MAX, question_last: 1 })));
    }

    #[test]
    fn a_cut_reader_reads_no_offset_it_drops_and_no_question_token_past_its_count() {
        let (x, places) = corpus(240);
        let (asks, _) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        let (case, key) = a_recall(&x, &places);
        let reader = |cut: Cut| AsksIndex::new(&asks).expect("the index").with_cut(cut);
        // The scores of the candidates of the two statements before the question.
        let scores = |cut: Cut| {
            let index = reader(cut);
            let q = index.asked(&x, &case.asked);
            let mut out = Vec::new();
            index.candidates(&x, &case.prior, &q, |_, _, score| out.push(score));
            out
        };
        let whole = scores(Cut::ALL);
        assert_eq!(whole.len(), 2, "one candidate in each statement");
        assert_ne!(whole[0], whole[1], "the key two tokens before a value tells the two apart");
        // The token before both values is the same one.
        let cut = scores(Cut { slots: 0b1, question_last: 0 });
        assert_eq!(cut[0], cut[1]);
        let mut all = vec![WHAT, key];
        all.sort_unstable();
        assert_eq!(reader(Cut::ALL).asked(&x, &case.asked), all);
        assert_eq!(reader(Cut { slots: u8::MAX, question_last: 1 }).asked(&x, &case.asked), vec![key]);
    }

    #[test]
    fn a_cut_census_finds_the_places_only_with_the_offset_that_holds_the_key() {
        let (x, places) = corpus(240);
        let (held, held_places) = corpus_from(240, 60);
        let found = |slots: u8| {
            let cut = Cut { slots, question_last: 0 };
            let (asks, _) = fit_cut(&x, &marks(), VOCAB, &places, "test".to_owned(), cut).expect("the fit");
            let index = AsksIndex::new(&asks).expect("the index").with_cut(cut);
            census(&held, &marks(), &held_places, &index, &asks.forms, 0).0.recall
        };
        assert_eq!(found(0b11), [30, 0, 0, 0]);
        assert!(found(0b1)[0] < 30, "{:?}", found(0b1));
    }

    #[test]
    fn a_fit_writes_the_same_bytes_twice() {
        let (x, places) = corpus(240);
        let text = || serde_json::to_string(&fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit").0).expect("written");
        assert_eq!(text(), text());
    }

    #[test]
    fn a_fit_refuses_what_it_cannot_count() {
        let (x, places) = corpus(240);
        let refused = |vocab: u32, places: &[[u32; 4]]| fit(&x, &marks(), vocab, places, "test".to_owned()).is_err();
        assert!(!refused(VOCAB, &places));
        // A vocabulary whose tokens do not fit a key, a token past the vocabulary, a kit with no
        // place, a place that starts no reply, a value before its reply.
        assert!(refused(u32::from(u16::MAX), &places));
        assert!(refused(u32::MAX, &places));
        assert!(refused(30, &places));
        assert!(refused(VOCAB, &[]));
        let mut moved = places.clone();
        moved[0][0] += 1;
        assert!(refused(VOCAB, &moved));
        let mut early = places.clone();
        early[0][2] = early[0][0] - 1;
        assert!(refused(VOCAB, &early));
    }

    #[test]
    fn the_read_takes_any_table_and_any_range_without_a_wrap_or_a_panic() {
        let (x, places) = corpus(240);
        let (asks, _) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        // Weights at the type's top: the sums stop there.
        let mut heavy = asks.clone();
        heavy.unseen = vec![[i32::MAX; 2]; ASKS_FRAME];
        heavy.shared.clear();
        heavy.asking_prior = i32::MAX;
        heavy.asking = vec![[SAYS as i32, i32::MAX], [WHAT as i32, i32::MAX]];
        let index = AsksIndex::new(&heavy).expect("the index");
        let case = ask_cases(&x, &marks()).into_iter().find(|c| !c.prior.is_empty()).expect("a reply with an earlier turn");
        assert_eq!(index.asking(&question(&x, &case.asked, Cut::ALL)), i32::MAX);
        assert!(matches!(index.find(&x, &case.prior, &case.asked), Asked::Place(..)));
        // Ranges past the tokens, and one that runs backwards: no place is found and nothing is
        // read past the end.
        let index = AsksIndex::new(&asks).expect("the index");
        assert!(!matches!(index.find(&x[..3], &[0..50], &(2..90)), Asked::Place(..)));
        assert!(!matches!(index.find(&x[..3], &[0..5], &(3..1)), Asked::Place(..)));
        assert_eq!(index.score(&x[..3], 0, 9, 4, &[]), None);
        // A candidate place of the whole tokens is none when the tokens end just before it: the
        // token before it is still a link token, and the place itself is not there to be read.
        let q = question(&x, &case.asked, Cut::ALL);
        let (turn, p) = case
            .prior
            .iter()
            .flat_map(|turn| turn.clone().map(move |s| (turn.clone(), s)))
            .find(|(turn, s)| index.score(&x, turn.start, turn.end, *s, &q).is_some())
            .expect("a candidate place");
        assert_eq!(index.score(&x[..p], turn.start, turn.end, p, &q), None, "a place one past the tokens");
        // A vocabulary past the keys is refused, and a value past the vocabulary opens nothing.
        let mut wide = asks.clone();
        wide.vocab = u32::from(u16::MAX);
        assert!(AsksIndex::new(&wide).is_err());
        let mut pair = asks;
        pair.opens = vec![[1, 7, 1]];
        let index = AsksIndex::new(&pair).expect("the index");
        assert_eq!(index.opens(1, 7), Some(true));
        assert_eq!(index.opens(0, (1 << 16) + 7), None);
    }

    #[test]
    fn the_census_reads_a_kit_whose_place_lies_past_the_tokens() {
        let (x, places) = corpus(240);
        let (asks, _) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        let index = AsksIndex::new(&asks).expect("the index");
        let (held, mut held_places) = corpus_from(240, 60);
        held_places[0][1] = held.len() as u32 + 9;
        let (counts, missed) = census(&held, &marks(), &held_places, &index, &asks.forms, 4);
        assert_eq!(counts.recall.iter().sum::<usize>(), 30);
        assert_eq!(counts.recall[0], 29, "{missed:#?}");
        assert!(missed.iter().any(|line| line.contains("stated=[]")), "{missed:#?}");
    }

    #[test]
    fn the_tables_round_trip_and_refuse_a_token_past_the_vocabulary() {
        let (x, places) = corpus(240);
        let (asks, _) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        let text = serde_json::to_string(&asks).expect("written");
        assert_eq!(serde_json::from_str::<PlaceAsks>(&text).expect("read"), asks);
        let mut bad = asks.clone();
        bad.link.push(VOCAB + 1);
        assert!(AsksIndex::new(&bad).is_err());
        let mut bad = asks;
        bad.not_told = vec![NOT, TOLD];
        assert!(AsksIndex::new(&bad).is_err());
    }

    #[test]
    fn the_reply_start_pair_is_set_open_once_and_nothing_else_moves() {
        let (x, places) = corpus(240);
        let (mut asks, _) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        asks.marks.assistant = vec![SEP, MODEL];
        let pair = [SEP as i32, MODEL as i32];
        // Three rows around the pair's sorted place, the pair counted not open among them.
        asks.opens = vec![[1, 7, 1], [pair[0], pair[1], 0], [pair[0], pair[1] + 1, 0]];
        let mut once = asks.clone();
        assert_eq!(once.open_reply_start(), Ok(Some(false)));
        assert_eq!(once.opens, vec![[1, 7, 1], [pair[0], pair[1], 1], [pair[0], pair[1] + 1, 0]]);
        let mut twice = once.clone();
        assert_eq!(twice.open_reply_start(), Ok(Some(true)));
        assert_eq!(twice, once);
        // Every other field is the fit's.
        let mut back = once.clone();
        back.opens = asks.opens.clone();
        assert_eq!(back, asks);
        assert_eq!(AsksIndex::new(&once).expect("the index").opens(SEP, MODEL), Some(true));
        // A pair the fit never counted is put in its sorted place.
        let mut missing = asks.clone();
        missing.opens.remove(1);
        assert_eq!(missing.open_reply_start(), Ok(None));
        assert_eq!(missing.opens, once.opens);
        let mut last = asks.clone();
        last.opens = vec![[1, 7, 1]];
        assert_eq!(last.open_reply_start(), Ok(None));
        assert_eq!(last.opens, vec![[1, 7, 1], [pair[0], pair[1], 1]]);
        // A marker of one token has no pair, and the table is left as it was.
        let mut one = asks.clone();
        one.marks.assistant = vec![MODEL];
        assert!(one.open_reply_start().is_err());
        one.marks.assistant = asks.marks.assistant.clone();
        assert_eq!(one, asks);
    }

    // claude-364: the lead-in pair of the tests, "say" and ":".
    const SAY: u32 = 50;
    const COLON: u32 = 51;

    /// A table written by hand: a question that holds `WHAT` asks for a stored fact and finds
    /// the position after `IS` in an earlier user turn (a pair weight of 100 over a floor of 50).
    fn hand_table(repeat: Vec<[u32; 2]>) -> PlaceAsks {
        hand_table3(repeat, Vec::new())
    }

    /// [`hand_table`] with the lead-in triples `repeat3` beside its pairs.
    fn hand_table3(repeat: Vec<[u32; 2]>, repeat3: Vec<[u32; 3]>) -> PlaceAsks {
        PlaceAsks {
            tokens_sha256: "test".to_owned(),
            marks: marks(),
            vocab: VOCAB,
            link: vec![IS],
            shared: Vec::new(),
            unseen: vec![[0, 0]; ASKS_FRAME],
            pairs: vec![[WHAT as i32, 1, IS as i32, 100]],
            floor: 50,
            floor_other: 50,
            margin: 0,
            opens: Vec::new(),
            asking: vec![[WHAT as i32, 10]],
            asking_prior: 0,
            asking_floor: 5,
            not_told: vec![NOT, TOLD, EOS],
            forms: vec![vec![NOT, TOLD, EOS]],
            repeat,
            repeat3,
        }
    }

    /// What the last reply's turn finds in `x` with the lead-in pairs `repeat`.
    fn found(x: &[u32], repeat: Vec<[u32; 2]>) -> (Asked, AskCase) {
        let index = AsksIndex::new(&hand_table(repeat)).expect("the index");
        let case = ask_cases(x, &marks()).pop().expect("a reply");
        (index.find(x, &case.prior, &case.asked), case)
    }

    #[test]
    fn a_turn_with_a_lead_in_pair_finds_its_own_rest_before_any_place() {
        // "<u> says K is V <sep> <m> ok </s> <sep> <u> what K say : A B <sep> <m>"
        let x = [BOS, USER, SAYS, 20, IS, 30, SEP, MODEL, OK, EOS, SEP, USER, WHAT, 20, SAY, COLON, 40, 41, SEP, MODEL];
        let (got, case) = found(&x, vec![[SAY, COLON]]);
        assert_eq!(case.asked, 12..18);
        assert_eq!(got, Asked::Repeat(16, 18));
        // With no pair the same turn asks for the stored value, and finds it.
        assert_eq!(found(&x, Vec::new()).0, Asked::Place(5, 6));
        // Another pair, and a pair only an earlier turn holds, change nothing.
        assert_eq!(found(&x, vec![[COLON, SAY]]).0, Asked::Place(5, 6));
        assert_eq!(found(&x, vec![[SAYS, 20]]).0, Asked::Place(5, 6));
        // The first pair in the turn is the lead-in: the phrase may hold the pair again.
        let twice = [BOS, USER, SAY, COLON, 40, SAY, COLON, 41, SEP, MODEL];
        assert_eq!(found(&twice, vec![[SAY, COLON]]).0, Asked::Repeat(4, 8));
        // A pair that ends the turn leads in nothing.
        let bare = [BOS, USER, WHAT, 20, SAY, COLON, SEP, MODEL];
        assert_eq!(found(&bare, vec![[SAY, COLON]]).0, Asked::NotTold);
    }

    #[test]
    fn a_table_with_no_lead_in_pair_is_written_and_read_as_before() {
        let plain = hand_table(Vec::new());
        let text = written(&plain);
        assert!(!text.contains("repeat"), "{text}");
        assert_eq!(serde_json::from_str::<PlaceAsks>(&text).expect("read"), plain);
        let with = hand_table(vec![[SAY, COLON]]);
        let text = written(&with);
        assert!(text.ends_with(",\"repeat\":[[50,51]]}"), "{text}");
        assert_eq!(serde_json::from_str::<PlaceAsks>(&text).expect("read"), with);
        // A pair past the vocabulary is no pair of tokens.
        assert!(AsksIndex::new(&hand_table(vec![[SAY, VOCAB]])).is_err());
        assert!(AsksIndex::new(&hand_table(vec![[VOCAB, COLON]])).is_err());
    }

    /// `repeats` documents whose user turn is "say : A B" answered "A B", then `others` whose
    /// turn holds the same pair and is answered "ok".
    fn lead_in_corpus(repeats: usize, others: usize) -> Vec<u32> {
        let mut x = Vec::new();
        for d in 0..repeats + others {
            let (a, b) = (30 + (d % 7) as u32, 40 + (d % 3) as u32);
            x.extend([BOS, USER, WHAT, SAY, COLON, a, b, SEP, MODEL]);
            if d < repeats {
                x.extend([a, b, EOS, SEP]);
            } else {
                x.extend([OK, EOS, SEP]);
            }
        }
        x
    }

    #[test]
    fn the_lead_in_count_keeps_a_pair_by_both_of_its_counts() {
        let kept = |repeats, others| fit_repeat(&lead_in_corpus(repeats, others), &marks(), VOCAB).expect("the count");
        // 16 replies that are the rest of their turn, of 18 turns with the pair: 7 in 8 or more.
        let (pairs, triples, lines) = kept(16, 2);
        assert_eq!(pairs, vec![[SAY, COLON]], "{lines:#?}");
        // A triple that ends on a kept pair is not written beside it.
        assert_eq!(triples, Vec::<[u32; 3]>::new(), "{lines:#?}");
        // "what say", "say :" and ": A" for each of seven A.
        assert_eq!(lines[0], "asks-repeat replies=18 pairs_counted=9 kept=1");
        assert!(lines.contains(&"asks-repeat pair before=50 token=51 turns=18 replies_the_rest=16 kept=1".to_owned()), "{lines:#?}");
        // One reply fewer is under the least count.
        assert_eq!(kept(15, 0).0, Vec::<[u32; 2]>::new());
        // 16 of 19 is under 7 in 8.
        let (pairs, triples, lines) = kept(16, 3);
        assert_eq!(pairs, Vec::<[u32; 2]>::new());
        // Every turn holds the same token before the pair, so the triple has the pair's counts.
        assert_eq!(triples, Vec::<[u32; 3]>::new());
        assert!(lines.contains(&"asks-repeat triple first=12 before=50 token=51 turns=19 replies_the_rest=16 kept=0".to_owned()), "{lines:#?}");
        assert!(lines.contains(&"asks-repeat pair before=50 token=51 turns=19 replies_the_rest=16 kept=0".to_owned()), "{lines:#?}");
        // The pair before it, "what say", is never followed by the reply alone: ": A B" is not it.
        assert!(!lines.iter().any(|line| line.contains("before=12 token=50")), "{lines:#?}");
        // The kept pair, in a table, finds the phrase in a turn the count never saw.
        let x = [BOS, USER, 16, SAY, COLON, 44, 45, 46, SEP, MODEL];
        assert_eq!(found(&x, kept(16, 2).0).0, Asked::Repeat(5, 8));
    }

    /// What the last reply's turn finds in `x` with the lead-in pairs and triples given.
    fn found3(x: &[u32], repeat: Vec<[u32; 2]>, repeat3: Vec<[u32; 3]>) -> Asked {
        let index = AsksIndex::new(&hand_table3(repeat, repeat3)).expect("the index");
        let case = ask_cases(x, &marks()).pop().expect("a reply");
        index.find(x, &case.prior, &case.asked)
    }

    #[test]
    fn a_kept_triple_finds_the_rest_where_its_pair_is_not_kept() {
        let triple = || vec![[WHAT, SAY, COLON]];
        // "<u> what say : A B <sep> <m>": the triple leads in the two tokens after it.
        let x = [BOS, USER, WHAT, SAY, COLON, 40, 41, SEP, MODEL];
        assert_eq!(found3(&x, Vec::new(), triple()), Asked::Repeat(5, 7));
        // The same pair after another token is no lead-in: only the triple is kept.
        let other = [BOS, USER, 16, SAY, COLON, 40, 41, SEP, MODEL];
        assert!(!matches!(found3(&other, Vec::new(), triple()), Asked::Repeat(..)));
        // The lead-in that ends first is the one read, pair or triple.
        let both = [BOS, USER, WHAT, SAY, COLON, 40, 44, 45, 41, SEP, MODEL];
        assert_eq!(found3(&both, vec![[44, 45]], triple()), Asked::Repeat(5, 9));
        assert_eq!(found3(&both, vec![[44, 45]], Vec::new()), Asked::Repeat(8, 9));
        let pair_first = [BOS, USER, 44, 45, 40, WHAT, SAY, COLON, 41, SEP, MODEL];
        assert_eq!(found3(&pair_first, vec![[44, 45]], triple()), Asked::Repeat(4, 9));
        // A triple that ends the turn leads in nothing.
        let bare = [BOS, USER, WHAT, SAY, COLON, SEP, MODEL];
        assert!(!matches!(found3(&bare, Vec::new(), triple()), Asked::Repeat(..)));
        // The list is written last, only when it holds a triple, and read back the same.
        let with = hand_table3(Vec::new(), triple());
        let text = written(&with);
        assert!(text.ends_with(",\"repeat3\":[[12,50,51]]}") && !text.contains("\"repeat\""), "{text}");
        assert_eq!(serde_json::from_str::<PlaceAsks>(&text).expect("read"), with);
        // A triple past the vocabulary is no triple of tokens.
        assert!(AsksIndex::new(&hand_table3(Vec::new(), vec![[WHAT, SAY, VOCAB]])).is_err());
        assert!(AsksIndex::new(&hand_table3(Vec::new(), vec![[VOCAB, SAY, COLON]])).is_err());
    }

    #[test]
    fn a_lead_in_is_counted_once_in_a_turn_at_its_first_place() {
        // "<u> what say : what say : A B" answered "A B": the pair and the triple stand twice
        // in the turn, and the rest after their first place is not the reply.
        let mut x = Vec::new();
        for d in 0..16u32 {
            let (a, b) = (30 + d % 7, 40 + d % 3);
            x.extend([BOS, USER, WHAT, SAY, COLON, WHAT, SAY, COLON, a, b, SEP, MODEL, a, b, EOS, SEP]);
        }
        let (pairs, triples, lines) = fit_repeat(&x, &marks(), VOCAB).expect("the count");
        assert_eq!((pairs, triples), (Vec::new(), Vec::new()), "{lines:#?}");
        // Counted at every place, each would be answered by the rest in 16 of 32 and be named.
        assert!(!lines.iter().any(|line| line.contains("before=50 token=51")), "{lines:#?}");
    }

    /// `repeats` documents whose user turn is "what say : A B" answered "A B", `strays` with
    /// the same turn answered "ok", and `others` whose turn is "16 say : A B" answered "ok".
    fn longer_corpus(repeats: usize, strays: usize, others: usize) -> Vec<u32> {
        let mut x = Vec::new();
        for d in 0..repeats + strays + others {
            let (a, b) = (30 + (d % 7) as u32, 40 + (d % 3) as u32);
            let first = if d < repeats + strays { WHAT } else { 16 };
            x.extend([BOS, USER, first, SAY, COLON, a, b, SEP, MODEL]);
            if d < repeats {
                x.extend([a, b, EOS, SEP]);
            } else {
                x.extend([OK, EOS, SEP]);
            }
        }
        x
    }

    #[test]
    fn the_lead_in_count_keeps_a_triple_where_its_pair_fails_its_share() {
        let kept = |repeats, strays, others| fit_repeat(&longer_corpus(repeats, strays, others), &marks(), VOCAB).expect("the count");
        // The pair is answered by the rest in 16 of 26 turns; the triple in 16 of its 18.
        let (pairs, triples, lines) = kept(16, 2, 8);
        assert_eq!(pairs, Vec::<[u32; 2]>::new(), "{lines:#?}");
        assert_eq!(triples, vec![[WHAT, SAY, COLON]], "{lines:#?}");
        assert!(lines.contains(&"asks-repeat pair before=50 token=51 turns=26 replies_the_rest=16 kept=0".to_owned()), "{lines:#?}");
        // "what say :", "16 say :" and "say : A" for each of seven A.
        assert!(lines.contains(&"asks-repeat triples_counted=9 kept=1".to_owned()), "{lines:#?}");
        assert!(lines.contains(&"asks-repeat triple first=12 before=50 token=51 turns=18 replies_the_rest=16 kept=1".to_owned()), "{lines:#?}");
        // One reply fewer is under the least count, and 16 of 19 is under 7 in 8.
        assert_eq!(kept(15, 0, 8).1, Vec::<[u32; 3]>::new());
        assert_eq!(kept(16, 3, 8).1, Vec::<[u32; 3]>::new());
        // The kept triple, in a table, finds the phrase in a turn the count never saw.
        let x = [BOS, USER, WHAT, SAY, COLON, 44, 45, 46, SEP, MODEL];
        assert_eq!(found3(&x, pairs, triples), Asked::Repeat(5, 8));
    }

    #[test]
    fn the_census_counts_a_repeat_apart_and_prints_it_only_when_there_is_one() {
        let x = [BOS, USER, 16, SAY, COLON, 44, 45, SEP, MODEL, 44, 45, EOS, SEP];
        let count = |repeat| {
            let table = hand_table(repeat);
            census(&x, &marks(), &[], &AsksIndex::new(&table).expect("the index"), &table.forms, 0).0
        };
        let with = count(vec![[SAY, COLON]]);
        assert_eq!((with.general, with.repeat), ([0, 0, 1], [0, 0, 1]));
        assert_eq!(with.lines("held").len(), 5);
        assert_eq!(with.lines("held")[4], "asks-census split=held repeat recall=0 abstain=0 general=1");
        let without = count(Vec::new());
        assert_eq!((without.general, without.repeat), ([0, 0, 1], [0; 3]));
        assert_eq!(without.lines("held").len(), 4);
    }
}
