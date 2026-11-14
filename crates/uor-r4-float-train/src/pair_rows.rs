//! Pair rows (one-model-end-to-end, claude-370): a table of learned rows addressed by the exact
//! pairs of tokens in a user's turn.
//!
//! Replies fail where they need two of the turn's tokens at once (claude-367). A pair of turn
//! tokens names one row here, so a reply can read what the two of them mean together.
//!
//! 1. **Address.** Token `t` stands for the prime `primes[t]`, the first `vocab` primes in order
//!    (`primes[0] = 2`). An unordered pair of different tokens `a`, `b` has the address
//!    `primes[a] · primes[b]`: by unique factorisation no two pairs share one. The address is a
//!    32-bit integer, so a vocabulary whose two largest primes multiply to 2³¹ or more is refused.
//! 2. **Dictionary.** One pass over a token stream ([`PairDictionary::count`]) counts in how many
//!    user turns each pair appears (once per turn however often it repeats). A pair seen in at
//!    least `min` turns gets a row; rows are numbered by ascending address.
//! 3. **Read.** A turn's bag is the sum of its pairs' rows. Every reply position of that turn adds
//!    the bag to its state ([`PairRows::apply`]): the reply runs from the assistant header's last
//!    token up to the next user marker or the window's end. The read is a gather and a
//!    scatter-add; nothing multiplies a state by a table.
//!
//! 4. **Rarity weight** (claude-373, `--pair-weight replies`). Optionally each row is weighed by
//!    the fixed `1 / R`, where `R` counts the different replies ([`reply_counts`]) that follow the
//!    TRAIN turns holding its pair: a pair that names one reply adds its row whole, a pair seen
//!    before many replies adds little. The bag is then the weighted sum of the turn's rows.
//!
//! The rows are active for the thread that sets them ([`set_pair_rows`]).
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

use candle_core::backprop::GradStore;
use candle_core::{DType, Device, Tensor, Var};

use crate::flock_read::FlockAdamW;
use crate::{invalid, Result};

const FILE: &str = "pair_rows.safetensors";
const DICT_FILE: &str = "pair_rows.dict";
const MAGIC: &[u8; 8] = b"UORPAIR1";
const LIMIT: u64 = 1 << 31;
/// The most different replies counted for one pair (claude-373): a pair seen before more keeps
/// this count, so its weight is `1 / REPLY_CAP`.
pub const REPLY_CAP: usize = 4096;

/// The census of reply counts the trainer prints at start-up and the scoring pass prints at load,
/// in one form so the two lines compare as text: the row count and R at the minimum, 50 %, 90 %,
/// 99 % and the maximum (the value at `(n - 1) · q / 100` of the sorted counts).
pub fn reply_census(counts: &[u32]) -> String {
    let mut sorted = counts.to_vec();
    sorted.sort_unstable();
    if sorted.is_empty() {
        return format!("0 pairs (cap {REPLY_CAP})");
    }
    let at = |q: usize| sorted[(sorted.len() - 1) * q / 100];
    format!(
        "{} pairs; R min {} p50 {} p90 {} p99 {} max {} (cap {REPLY_CAP})",
        sorted.len(),
        at(0),
        at(50),
        at(90),
        at(99),
        at(100)
    )
}

thread_local! {
    static ACTIVE: RefCell<Option<PairRows>> = const { RefCell::new(None) };
}

/// Make `rows` the pair rows applied on this thread, or none.
pub fn set_pair_rows(rows: Option<PairRows>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = rows);
}

/// The pair rows applied on this thread, if any are set. They share their variable with every
/// other copy.
pub fn pair_rows() -> Option<PairRows> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// The first `count` primes in order, or fewer: generation stops once the last two multiply to
/// `LIMIT` or more, since no pair address may reach it.
fn first_primes(count: usize) -> Vec<u32> {
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
/// vocabulary whose two largest primes multiply to 2³¹ or more.
pub fn token_primes(vocab: usize) -> Result<Vec<u32>> {
    if vocab < 2 {
        return Err(invalid(format!(
            "pair rows want a vocabulary of at least 2 tokens, not {vocab}"
        )));
    }
    let primes = first_primes(vocab);
    let last = primes.len();
    let top = u64::from(primes[last - 1]) * u64::from(primes[last - 2]);
    if last < vocab {
        return Err(invalid(format!(
            "pair rows hold a pair address in 31 bits; a vocabulary of {vocab} tokens has an \
             address of at least {top}"
        )));
    }
    if top >= LIMIT {
        return Err(invalid(format!(
            "pair rows hold a pair address in 31 bits; a vocabulary of {vocab} tokens has the \
             address {top}"
        )));
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
    fn check(&self) -> Result<()> {
        if self.user.is_empty() || self.assistant.is_empty() {
            return Err(invalid(
                "pair rows want a user marker and an assistant header of at least one token",
            ));
        }
        // A turn is found by the user marker first, so a user marker that begins the header
        // would hide every header. The other order is refused too: the marks stay unambiguous.
        if self.user.starts_with(&self.assistant) || self.assistant.starts_with(&self.user) {
            return Err(invalid(
                "pair rows want a user marker and an assistant header neither of which begins \
                 the other",
            ));
        }
        Ok(())
    }
}

/// One user turn found in a token run, by position in that run.
struct Turn {
    /// The first position after the user marker.
    start: usize,
    /// The assistant header's first position: the turn's tokens are `start..header`.
    header: usize,
    /// The assistant header's last position: the first reply position.
    reply: usize,
}

/// The user turns of `tokens` whose user marker and assistant header both lie inside it, and the
/// start of every user marker. A second user marker before the header starts the turn again.
fn scan(tokens: &[u32], marks: &PairMarks) -> (Vec<Turn>, Vec<usize>) {
    let at = |i: usize, mark: &[u32]| tokens[i..].starts_with(mark);
    let (mut turns, mut users) = (Vec::new(), Vec::new());
    let mut open: Option<usize> = None;
    let mut i = 0;
    while i < tokens.len() {
        if at(i, &marks.user) {
            users.push(i);
            i += marks.user.len();
            open = Some(i);
        } else if at(i, &marks.assistant) {
            if let Some(start) = open.take() {
                turns.push(Turn {
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
    /// Count the pairs of the user turns in `tokens` and keep those seen in at least `min` turns.
    /// A pair counts once per turn. Refuses a turn token outside the vocabulary.
    pub fn count(vocab: usize, tokens: &[u32], marks: &PairMarks, min: u32) -> Result<Self> {
        marks.check()?;
        let primes = token_primes(vocab)?;
        let (turns, _) = scan(tokens, marks);
        let mut seen: HashMap<u32, u32> = HashMap::new();
        for turn in &turns {
            let bag = distinct(&tokens[turn.start..turn.header]);
            if let Some(&token) = bag.last().filter(|&&t| t as usize >= vocab) {
                return Err(invalid(format!(
                    "a turn token {token} lies outside the vocabulary of {vocab}"
                )));
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

    /// The dictionary as bytes: the magic, the vocabulary, the row count and the addresses, each
    /// a little-endian u32.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16 + 4 * self.addresses.len());
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&(self.primes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.addresses.len() as u32).to_le_bytes());
        for address in &self.addresses {
            bytes.extend_from_slice(&address.to_le_bytes());
        }
        bytes
    }

    /// The dictionary [`Self::to_bytes`] wrote. Refuses another magic, a length that is not the
    /// one the row count names, and addresses out of order.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let word = |n: usize| u32::from_le_bytes(bytes[n..n + 4].try_into().unwrap());
        if bytes.len() < 16 || &bytes[..8] != MAGIC {
            return Err(invalid("not a pair dictionary: no magic"));
        }
        let (vocab, count) = (word(8) as usize, word(12) as usize);
        if bytes.len() != 16 + 4 * count {
            return Err(invalid(format!(
                "a pair dictionary of {count} rows wants {} bytes, not {}",
                16 + 4 * count,
                bytes.len()
            )));
        }
        let addresses: Vec<u32> = (0..count).map(|r| word(16 + 4 * r)).collect();
        if addresses.windows(2).any(|w| w[0] >= w[1]) {
            return Err(invalid("a pair dictionary's addresses are not ascending"));
        }
        Ok(Self {
            primes: token_primes(vocab)?,
            addresses,
        })
    }
}

/// Which rows a batch of windows reads and where each turn's bag is added.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// Per position (`batch · time`, row-major): the turn whose bag it adds, or `turns` for none.
    pub turn_of: Vec<u32>,
    /// Per hit: the turn.
    pub hit_turn: Vec<u32>,
    /// Per hit: the dictionary row.
    pub hit_row: Vec<u32>,
    /// The number of turns with at least one hit. Only those get a turn index.
    pub turns: usize,
    /// Per window: the turn whose reply is still open at the window's last position, for a decode
    /// step that continues it.
    pub open: Vec<Option<u32>>,
}

/// The plan for `ids`, `batch` windows of `time` tokens, row-major. Nothing crosses a window: a
/// turn counts only when its user marker and its assistant header both lie inside the window. A
/// turn's reply positions run from the header's last token through the position before the next
/// user marker, or the window's end. A turn none of whose pairs has a row adds nothing and gets no
/// turn index.
pub fn plan(
    dict: &PairDictionary,
    marks: &PairMarks,
    ids: &[u32],
    batch: usize,
    time: usize,
) -> Result<Plan> {
    marks.check()?;
    if ids.len() != batch * time {
        return Err(invalid(format!(
            "pair rows want {batch} windows of {time} tokens, not {} ids",
            ids.len()
        )));
    }
    let (mut spans, mut hit_turn, mut hit_row) = (Vec::new(), Vec::new(), Vec::new());
    let mut open = vec![None; batch];
    for (window, row) in ids.chunks(time.max(1)).take(batch).enumerate() {
        let (turns, users) = scan(row, marks);
        for turn in turns {
            let rows = dict.turn_rows(&row[turn.start..turn.header]);
            if rows.is_empty() {
                continue;
            }
            let end = users
                .iter()
                .copied()
                .find(|&u| u > turn.reply)
                .unwrap_or(time);
            let index = spans.len() as u32;
            if end == time {
                open[window] = Some(index);
            }
            hit_turn.extend(std::iter::repeat_n(index, rows.len()));
            hit_row.extend(rows);
            spans.push(window * time + turn.reply..window * time + end);
        }
    }
    let turns = spans.len();
    let mut turn_of = vec![turns as u32; batch * time];
    for (index, span) in spans.into_iter().enumerate() {
        turn_of[span].fill(index as u32);
    }
    Ok(Plan {
        turn_of,
        hit_turn,
        hit_row,
        turns,
        open,
    })
}

/// The plan's hit vectors and the turn each output position adds, as tensors on `device`:
/// `hit_row`, `hit_turn`, `turn_of`. They are built from host vectors and carry no gradient
/// history.
fn indices(plan: &Plan, turn_of: &[u32], device: &Device) -> Result<[Tensor; 3]> {
    let tensor = |v: &[u32]| Tensor::from_slice(v, v.len(), device).map(|t| t.detach());
    Ok([
        tensor(&plan.hit_row)?,
        tensor(&plan.hit_turn)?,
        tensor(turn_of)?,
    ])
}

/// How many of `prompts` (each the ids fed before a reply's first token) end inside a reply whose
/// turn reads a bag: the turn's marker and header lie in the prompt and a pair of it has a row.
pub fn bag_readers(dict: &PairDictionary, marks: &PairMarks, prompts: &[&[u32]]) -> Result<usize> {
    let mut readers = 0;
    for ids in prompts {
        if plan(dict, marks, ids, 1, ids.len())?.open[0].is_some() {
            readers += 1;
        }
    }
    Ok(readers)
}

/// Per dictionary row: how many different replies follow the user turns of `tokens` that hold
/// the row's pair, at most [`REPLY_CAP`]. A reply is told apart by its exact token run, hashed;
/// it runs from the assistant header's last position up to the next user marker or the end of
/// `tokens`, as in [`plan`]. Each different reply counts once however many turns it follows.
pub fn reply_counts(dict: &PairDictionary, tokens: &[u32], marks: &PairMarks) -> Result<Vec<u32>> {
    marks.check()?;
    let (turns, users) = scan(tokens, marks);
    let mut replies: Vec<HashSet<u64>> = vec![HashSet::new(); dict.len()];
    for turn in &turns {
        let rows = dict.turn_rows(&tokens[turn.start..turn.header]);
        if rows.is_empty() {
            continue;
        }
        let end = users
            .get(users.partition_point(|&u| u <= turn.reply))
            .copied()
            .unwrap_or(tokens.len());
        let mut hasher = DefaultHasher::new();
        tokens[turn.reply..end].hash(&mut hasher);
        let reply = hasher.finish();
        for row in rows {
            let seen = &mut replies[row as usize];
            if seen.len() < REPLY_CAP {
                seen.insert(reply);
            }
        }
    }
    Ok(replies.iter().map(|seen| seen.len() as u32).collect())
}

/// The learned rows, one per dictionary pair, `width` wide, and when weighed (claude-373) a fixed
/// weight per row that takes no gradient.
#[derive(Clone, Debug)]
pub struct PairRows {
    dict: Arc<PairDictionary>,
    marks: PairMarks,
    rows: Var,
    weights: Option<Tensor>,
    /// claude-374: the rows add `(1 + w)` of themselves, `w` the stored `1 / R`, in place of `w`.
    boost: bool,
}

/// The tensor name of a boosted file's mark (claude-374); a `replies` file has none.
const BOOST: &str = "boost";

impl PairRows {
    /// Rows for every pair of `dict`, `width` wide, all zero, so the model starts unchanged.
    pub fn new(
        dict: PairDictionary,
        marks: PairMarks,
        width: usize,
        device: &Device,
    ) -> Result<Self> {
        marks.check()?;
        if dict.is_empty() || width == 0 {
            return Err(invalid(format!(
                "pair rows want at least one pair and a width, not {} pairs of width {width}",
                dict.len()
            )));
        }
        let rows = Var::zeros((dict.len(), width), DType::F32, device)?;
        Ok(Self {
            dict: Arc::new(dict),
            marks,
            rows,
            weights: None,
            boost: false,
        })
    }

    /// The rows at full weight with rare pairs raised (claude-374): row `r` adds
    /// `1 + 1 / counts[r]` times itself, a constant that takes no gradient. The stored weight stays
    /// claude-373's `1 / R`, so the counts read back exactly; the forward adds the one.
    pub fn with_boost_weights(self, counts: &[u32]) -> Result<Self> {
        let mut rows = self.with_reply_weights(counts)?;
        rows.boost = true;
        Ok(rows)
    }

    /// Which weight the rows carry: `none` (claude-370), `replies` (`1 / R`, claude-373) or
    /// `boost` (`1 + 1 / R`, claude-374).
    pub fn weight_kind(&self) -> &'static str {
        match (&self.weights, self.boost) {
            (None, _) => "none",
            (Some(_), false) => "replies",
            (Some(_), true) => "boost",
        }
    }

    /// The rows weighed by rarity (claude-373): row `r` adds `1 / counts[r]` times itself, a
    /// constant that takes no gradient. Refuses counts of another length or a zero count.
    pub fn with_reply_weights(self, counts: &[u32]) -> Result<Self> {
        if counts.contains(&0) {
            return Err(invalid("a pair-rows reply count of zero has no weight"));
        }
        let weights = counts.iter().map(|&r| 1.0 / r as f32).collect();
        self.with_weights(weights)
    }

    /// The rows with `weights`, one per row. Refuses another length or a weight that is not
    /// finite and positive.
    fn with_weights(mut self, weights: Vec<f32>) -> Result<Self> {
        if weights.len() != self.dict.len() {
            return Err(invalid(format!(
                "{} pair-rows weights for a dictionary of {} pairs",
                weights.len(),
                self.dict.len()
            )));
        }
        if let Some(w) = weights.iter().find(|w| !(w.is_finite() && **w > 0.0)) {
            return Err(invalid(format!(
                "a pair-rows weight must be finite and positive, not {w}"
            )));
        }
        let device = self.rows.as_tensor().device().clone();
        let length = weights.len();
        self.weights = Some(Tensor::from_vec(weights, length, &device)?.detach());
        Ok(self)
    }

    /// The fixed row weights, when the rows are weighed.
    pub fn weights(&self) -> Option<&Tensor> {
        self.weights.as_ref()
    }

    /// Each row's count of different replies, read back from its weight (`R = 1 / w`, rounded),
    /// when the rows are weighed. A count up to [`REPLY_CAP`] survives the f32 weight exactly.
    pub fn reply_counts_of_weights(&self) -> Result<Option<Vec<u32>>> {
        let Some(weights) = &self.weights else {
            return Ok(None);
        };
        let weights = weights.to_vec1::<f32>()?;
        Ok(Some(
            weights
                .iter()
                .map(|&w| (1.0f64 / w as f64).round() as u32)
                .collect(),
        ))
    }

    /// The dictionary the rows are numbered by.
    pub fn dict(&self) -> &PairDictionary {
        &self.dict
    }

    /// The marks that find a turn.
    pub fn marks(&self) -> &PairMarks {
        &self.marks
    }

    /// The trained variables: the rows.
    pub fn variables(&self) -> Vec<Var> {
        vec![self.rows.clone()]
    }

    /// How many rows hold a non-zero entry, and the root mean square over every entry. Reduced
    /// on the device: one value per row and one sum come back.
    pub fn census(&self) -> Result<(usize, f64)> {
        let rows = self.rows.as_tensor();
        let peaks = rows.abs()?.max(1)?.to_vec1::<f32>()?;
        let nonzero = peaks.iter().filter(|&&peak| peak != 0.0).count();
        let squares = rows.sqr()?.sum_all()?.to_scalar::<f32>()? as f64;
        Ok((nonzero, (squares / rows.elem_count() as f64).sqrt()))
    }

    /// The trainer's start: the dictionary counted on `tokens` (pairs seen in at least `min` user
    /// turns) and a zero row for each of its pairs, `width` wide.
    pub fn from_tokens(
        vocab: usize,
        tokens: &[u32],
        marks: &PairMarks,
        min: u32,
        width: usize,
        device: &Device,
    ) -> Result<Self> {
        let dict = PairDictionary::count(vocab, tokens, marks, min)?;
        Self::new(dict, marks.clone(), width, device)
    }

    /// One step of `opt` (built over [`PairRows::variables`]) at rate `lr`, only when `grads`
    /// holds the rows' gradient. With none the rows and `opt`'s step count stay as they are.
    /// Whether it stepped.
    pub fn step(&self, opt: &mut FlockAdamW, grads: &GradStore, lr: f64) -> Result<bool> {
        if grads.get(self.rows.as_tensor()).is_none() {
            return Ok(false);
        }
        opt.step_vars(&[&self.rows], grads, lr)?;
        Ok(true)
    }

    /// `x` (`batch × time × width`) with each reply position's turn bag added. With no hit, `x`
    /// itself.
    pub fn apply(&self, x: &Tensor, ids: &[u32]) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        self.check_width(width)?;
        let plan = plan(&self.dict, &self.marks, ids, batch, time)?;
        self.add_bags(x, &plan, &plan.turn_of)
    }

    /// `x` (`batch × 1 × width`, each row's newest position) with the bag of the turn whose reply
    /// that position is in added. `window` holds every row's whole window, newest id last, row
    /// after row, all of one length. With no hit, `x` itself.
    pub fn apply_last(&self, x: &Tensor, window: &[u32]) -> Result<Tensor> {
        let (batch, one, width) = x.dims3()?;
        self.check_width(width)?;
        if one != 1 || batch == 0 || window.is_empty() || window.len() % batch != 0 {
            return Err(invalid(format!(
                "a pair-rows step wants {batch} rows of one position and windows of one length, \
                 not {one} positions and {} ids",
                window.len()
            )));
        }
        let time = window.len() / batch;
        let plan = plan(&self.dict, &self.marks, window, batch, time)?;
        let last: Vec<u32> = (0..batch)
            .map(|b| plan.turn_of[b * time + time - 1])
            .collect();
        self.add_bags(x, &plan, &last)
    }

    /// Refuse a state whose width is not the rows' own.
    fn check_width(&self, width: usize) -> Result<()> {
        let (count, rows_width) = self.rows.as_tensor().dims2()?;
        if width != rows_width || count != self.dict.len() {
            return Err(invalid(format!(
                "pair rows of width {rows_width} meet a state of width {width}"
            )));
        }
        Ok(())
    }

    /// `x` with, at each output position `i`, the bag of turn `turn_of[i]` added (`plan.turns`
    /// adds nothing): the plan's hit rows gathered, summed into one bag per turn, and scattered.
    fn add_bags(&self, x: &Tensor, plan: &Plan, turn_of: &[u32]) -> Result<Tensor> {
        if plan.hit_row.is_empty() {
            return Ok(x.clone());
        }
        let width = x.dim(2)?;
        let [hit_row, hit_turn, turn_of] = indices(plan, turn_of, x.device())?;
        let gathered = self.rows.as_tensor().index_select(&hit_row, 0)?;
        let gathered = match &self.weights {
            Some(weights) => {
                let weights = weights.index_select(&hit_row, 0)?;
                let weights = if self.boost {
                    weights.affine(1.0, 1.0)?
                } else {
                    weights
                };
                gathered.broadcast_mul(&weights.unsqueeze(1)?)?
            }
            None => gathered,
        };
        let bags = Tensor::zeros((plan.turns + 1, width), DType::F32, x.device())?
            .index_add(&hit_turn, &gathered, 0)?;
        let per_position = bags
            .index_select(&turn_of, 0)?
            .reshape(x.dims())?
            .to_dtype(x.dtype())?;
        Ok(x.add(&per_position)?)
    }

    /// Write the rows (`pair_rows.safetensors`, with the marks and, when weighed, the weights) and the dictionary
    /// (`pair_rows.dict`) into `directory`.
    pub fn save(&self, directory: &Path) -> Result<()> {
        let host = Device::Cpu;
        let mut tensors: HashMap<String, Tensor> = HashMap::new();
        tensors.insert("rows".to_owned(), self.rows.as_tensor().detach());
        if let Some(weights) = &self.weights {
            tensors.insert("weights".to_owned(), weights.clone());
        }
        if self.boost {
            tensors.insert(BOOST.to_owned(), Tensor::from_slice(&[1u32], 1, &host)?);
        }
        let marks = [
            ("user", &self.marks.user),
            ("assistant", &self.marks.assistant),
        ];
        for (name, mark) in marks {
            tensors.insert(
                name.to_owned(),
                Tensor::from_slice(mark, mark.len(), &host)?,
            );
        }
        candle_core::safetensors::save(&tensors, directory.join(FILE))?;
        std::fs::write(directory.join(DICT_FILE), self.dict.to_bytes())?;
        Ok(())
    }

    /// The rows saved in `directory`, or none when the checkpoint has neither pair-rows file.
    /// Refuses a checkpoint that holds only one of the two. Weights present load weighed rows;
    /// a weights tensor of another length, or a weight not finite and positive, is refused.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let (path, dict_path) = (directory.join(FILE), directory.join(DICT_FILE));
        if !path.exists() && !dict_path.exists() {
            return Ok(None);
        }
        for name in [FILE, DICT_FILE] {
            if !directory.join(name).exists() {
                return Err(invalid(format!(
                    "{}: half a pair-rows checkpoint, no {name}",
                    directory.display()
                )));
            }
        }
        let dict = PairDictionary::from_bytes(&std::fs::read(directory.join(DICT_FILE))?)?;
        let tensors = candle_core::safetensors::load(&path, device)?;
        let named = |name: &str| {
            tensors
                .get(name)
                .ok_or_else(|| invalid(format!("{}: no tensor {name}", path.display())))
        };
        let marks = PairMarks {
            user: named("user")?.to_vec1::<u32>()?,
            assistant: named("assistant")?.to_vec1::<u32>()?,
        };
        marks.check()?;
        let rows = named("rows")?;
        let (count, _) = rows.dims2()?;
        if count != dict.len() {
            return Err(invalid(format!(
                "{}: {count} rows for a dictionary of {} pairs",
                path.display(),
                dict.len()
            )));
        }
        let pair = Self {
            dict: Arc::new(dict),
            marks,
            rows: Var::from_tensor(rows)?,
            weights: None,
            boost: false,
        };
        // claude-374: a boost mark is exactly [1] and never stands without weights.
        let boost = match tensors.get(BOOST) {
            None => false,
            Some(mark) if mark.to_vec1::<u32>()? == [1] => true,
            Some(_) => {
                return Err(invalid(format!(
                    "{}: a pair-rows boost mark must be [1]",
                    path.display()
                )))
            }
        };
        match tensors.get("weights") {
            None if boost => Err(invalid(format!(
                "{}: a pair-rows boost mark without weights",
                path.display()
            ))),
            None => Ok(Some(pair)),
            Some(weights) => {
                let mut pair = pair.with_weights(weights.to_vec1::<f32>()?)?;
                pair.boost = boost;
                Ok(Some(pair))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Plain tests cover the address, the dictionary and the plan on the host. The tensor read is
    // tested on the GPU (`cuda`).

    /// The GPU the tensor tests run on.
    #[cfg(feature = "cuda")]
    fn device() -> Device {
        Device::new_cuda(0).unwrap()
    }

    const N: u32 = u32::MAX;

    fn marks() -> PairMarks {
        PairMarks {
            user: vec![1, 4],
            assistant: vec![2, 3],
        }
    }

    /// Rows, by ascending address (primes 5→13, 6→17, 7→19, 8→23): (5,6)=0 at 221, (5,7)=1 at
    /// 247, (6,7)=2 at 323, (7,8)=3 at 437.
    fn dict() -> PairDictionary {
        let stream = [1, 4, 5, 6, 2, 3, 1, 4, 7, 8, 2, 3, 1, 4, 5, 6, 7, 2, 3];
        PairDictionary::count(20, &stream, &marks(), 1).unwrap()
    }

    /// `turn_of` with the plan's "none" written as `N`.
    fn turn_of(plan: &Plan) -> Vec<u32> {
        let none = plan.turns as u32;
        plan.turn_of
            .iter()
            .map(|&t| if t == none { N } else { t })
            .collect()
    }

    /// Window A: two turns. Turn 0 (5,6) replies at 6..=8, turn 1 (7,8) at 14..=15, still open.
    const WINDOW_A: [u32; 16] = [9, 1, 4, 5, 6, 2, 3, 10, 11, 1, 4, 7, 8, 2, 3, 12];

    #[test]
    fn primes_are_the_first_in_order_and_a_wide_vocab_is_refused() {
        assert_eq!(token_primes(6).unwrap(), vec![2, 3, 5, 7, 11, 13]);
        assert!(token_primes(1).is_err());
        let primes = first_primes(6_000);
        let wide = (2..=6_000)
            .find(|&v| u64::from(primes[v - 1]) * u64::from(primes[v - 2]) >= LIMIT)
            .unwrap();
        assert!(token_primes(wide - 1).is_ok());
        assert!(token_primes(wide).is_err());
    }

    #[test]
    fn the_address_is_unordered_and_none_for_equal_tokens() {
        let dict = dict();
        assert_eq!(dict.address(1, 3), Some(3 * 7));
        assert_eq!(dict.address(3, 1), Some(3 * 7));
        assert_eq!(dict.address(2, 2), None);
        assert_eq!(dict.address(0, 20), None);
        assert_eq!(dict.row(6, 5), Some(0));
        assert_eq!(dict.row(8, 7), Some(3));
        assert_eq!(dict.row(5, 8), None);
        assert_eq!(dict.len(), 4);
        assert_eq!(dict.turn_rows(&[7, 6, 5, 5]), vec![0, 1, 2]);
    }

    #[test]
    fn count_keeps_pairs_seen_in_min_turns_once_per_turn() {
        // (5,6) appears four times in its first turn and once in a second; (5,7) in one turn;
        // the replies' (8,9) is outside every turn.
        let stream = [
            1, 4, 5, 6, 5, 6, 2, 3, 8, 9, 1, 4, 5, 7, 2, 3, 8, 9, 1, 4, 6, 5, 2, 3,
        ];
        let at = |min| PairDictionary::count(20, &stream, &marks(), min).unwrap();
        let one = at(1);
        assert_eq!(one.len(), 2);
        assert!(one.row(5, 6).is_some() && one.row(5, 7).is_some());
        assert_eq!(one.row(8, 9), None);
        let two = at(2);
        assert_eq!(two.len(), 1);
        assert_eq!(two.row(5, 6), Some(0));
        assert!(at(3).is_empty(), "a pair counts once per turn");
        assert!(PairDictionary::count(5, &stream, &marks(), 1).is_err());
    }

    #[test]
    fn bytes_round_trip_and_a_truncated_file_is_refused() {
        let dict = dict();
        let bytes = dict.to_bytes();
        assert_eq!(PairDictionary::from_bytes(&bytes).unwrap(), dict);
        assert!(PairDictionary::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(PairDictionary::from_bytes(&bytes[..10]).is_err());
        let mut other = bytes.clone();
        other[0] ^= 1;
        assert!(PairDictionary::from_bytes(&other).is_err());
    }

    #[test]
    fn each_reply_takes_its_own_turn_and_nothing_else_does() {
        let plan = plan(&dict(), &marks(), &WINDOW_A, 1, 16).unwrap();
        assert_eq!(plan.turns, 2);
        assert_eq!(
            turn_of(&plan),
            vec![N, N, N, N, N, N, 0, 0, 0, N, N, N, N, N, 1, 1]
        );
        assert_eq!(plan.hit_turn, vec![0, 1]);
        assert_eq!(plan.hit_row, vec![0, 3]);
        assert_eq!(plan.open, vec![Some(1)]);
    }

    #[test]
    fn a_turn_cut_by_the_window_start_gets_none() {
        let plan = plan(&dict(), &marks(), &[4, 5, 6, 2, 3, 10, 11, 12], 1, 8).unwrap();
        assert_eq!(plan.turns, 0);
        assert!(plan.hit_row.is_empty());
        assert_eq!(turn_of(&plan), vec![N; 8]);
        assert_eq!(plan.open, vec![None]);
    }

    #[test]
    fn a_pair_outside_the_dictionary_gives_no_hit() {
        let plan = plan(&dict(), &marks(), &[1, 4, 5, 8, 2, 3, 10], 1, 7).unwrap();
        assert_eq!(plan.turns, 0);
        assert!(plan.hit_row.is_empty() && plan.hit_turn.is_empty());
        assert_eq!(turn_of(&plan), vec![N; 7]);
        assert_eq!(plan.open, vec![None]);
    }

    #[test]
    fn windows_of_a_batch_are_independent() {
        // Window 0 ends inside a user turn; window 1 opens on a header. Neither pairs with the
        // other. Window 1's own turn (7,8) replies at its last position.
        let ids = [9, 9, 9, 9, 1, 4, 5, 6, 2, 3, 1, 4, 7, 8, 2, 3];
        let plan = plan(&dict(), &marks(), &ids, 2, 8).unwrap();
        let mut expect = vec![N; 16];
        expect[15] = 0;
        assert_eq!(turn_of(&plan), expect);
        assert_eq!(
            (plan.hit_turn.clone(), plan.hit_row.clone()),
            (vec![0], vec![3])
        );
        assert_eq!(plan.open, vec![None, Some(0)]);
        let alone = super::plan(&dict(), &marks(), &ids[8..], 1, 8).unwrap();
        assert_eq!(alone.hit_row, plan.hit_row);
        assert_eq!(turn_of(&alone), expect[8..].to_vec());
    }

    #[test]
    fn open_is_none_once_a_new_user_marker_closes_the_reply() {
        let ids = [1, 4, 5, 6, 2, 3, 10, 1, 4, 7];
        let plan = plan(&dict(), &marks(), &ids, 1, 10).unwrap();
        assert_eq!(turn_of(&plan), vec![N, N, N, N, N, 0, 0, N, N, N]);
        assert_eq!(plan.open, vec![None]);
        assert!(super::plan(&dict(), &marks(), &ids, 2, 10).is_err());
    }

    /// Two windows of 16: window A, and a window whose first turn (5,6,7) reads rows 0, 1, 2 at
    /// 6..=8 and whose second turn (5,8) has no row.
    #[cfg(feature = "cuda")]
    fn batch_ids() -> Vec<u32> {
        let mut ids = WINDOW_A.to_vec();
        ids.extend([1, 4, 5, 6, 7, 2, 3, 10, 11, 1, 4, 5, 8, 2, 3, 12]);
        ids
    }

    #[cfg(feature = "cuda")]
    fn state(batch: usize, time: usize, width: usize) -> Vec<f32> {
        (0..batch * time * width)
            .map(|n| ((n * 37 % 101) as f32 - 50.0) / 7.0)
            .collect()
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn rows_at_zero_leave_the_state_bit_for_bit() {
        let device = device();
        let pair = PairRows::new(dict(), marks(), 4, &device).unwrap();
        let values = state(2, 16, 4);
        let x = Tensor::from_vec(values.clone(), (2, 16, 4), &device).unwrap();
        let y = pair.apply(&x, &batch_ids()).unwrap();
        let y = y.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        assert!(y
            .iter()
            .zip(&values)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_read_is_the_sum_of_the_turn_rows_at_reply_positions() {
        let device = device();
        let width = 4;
        let pair = PairRows::new(dict(), marks(), width, &device).unwrap();
        let table: Vec<f32> = (0..4 * width)
            .map(|n| ((n / width + 1) * 10 + n % width) as f32)
            .collect();
        let row = |r: usize| table[r * width..(r + 1) * width].to_vec();
        pair.rows
            .set(&Tensor::from_vec(table.clone(), (4, width), &device).unwrap())
            .unwrap();
        let values = state(2, 16, width);
        let x = Tensor::from_vec(values.clone(), (2, 16, width), &device).unwrap();
        let y = pair.apply(&x, &batch_ids()).unwrap();
        let y = y.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let mut bag = vec![vec![0.0f32; width]; 32];
        for p in [6, 7, 8] {
            bag[p] = row(0);
        }
        for p in [14, 15] {
            bag[p] = row(3);
        }
        for p in [16 + 6, 16 + 7, 16 + 8] {
            bag[p] = (0..width)
                .map(|c| row(0)[c] + row(1)[c] + row(2)[c])
                .collect();
        }
        for (p, add) in bag.iter().enumerate() {
            for c in 0..width {
                let expect = values[p * width + c] + add[c];
                assert_eq!(y[p * width + c], expect, "position {p} coordinate {c}");
            }
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn only_hit_rows_take_a_gradient() {
        let device = device();
        let pair = PairRows::new(dict(), marks(), 4, &device).unwrap();
        let x = Tensor::from_vec(state(1, 16, 4), (1, 16, 4), &device).unwrap();
        let loss = pair.apply(&x, &WINDOW_A).unwrap().sum_all().unwrap();
        let grads = loss.backward().unwrap();
        let grad = grads.get(pair.rows.as_tensor()).unwrap();
        let grad = grad.to_vec2::<f32>().unwrap();
        assert_eq!(grad[0], vec![3.0; 4], "row 0 is read at three positions");
        assert_eq!(grad[1], vec![0.0; 4]);
        assert_eq!(grad[2], vec![0.0; 4]);
        assert_eq!(grad[3], vec![2.0; 4], "row 3 is read at two positions");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_index_tensors_track_no_operation() {
        let device = device();
        let plan = plan(&dict(), &marks(), &batch_ids(), 2, 16).unwrap();
        for index in indices(&plan, &plan.turn_of, &device).unwrap() {
            assert!(!index.track_op());
            assert_eq!(index.dtype(), DType::U32);
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn save_and_load_round_trip() {
        let device = device();
        let directory =
            std::env::temp_dir().join(format!("uor-r4-pair-rows-{}-{:x}", std::process::id(), 370));
        std::fs::create_dir_all(&directory).unwrap();
        assert!(PairRows::load(&directory, &device).unwrap().is_none());
        let pair = PairRows::new(dict(), marks(), 4, &device).unwrap();
        let table: Vec<f32> = (0..16).map(|n| n as f32 * 0.5 - 3.0).collect();
        pair.rows
            .set(&Tensor::from_vec(table.clone(), (4, 4), &device).unwrap())
            .unwrap();
        pair.save(&directory).unwrap();
        let loaded = PairRows::load(&directory, &device).unwrap().unwrap();
        assert_eq!(loaded.dict(), pair.dict());
        assert_eq!(loaded.marks(), pair.marks());
        let back = loaded.rows.as_tensor().flatten_all().unwrap();
        assert_eq!(back.to_vec1::<f32>().unwrap(), table);
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_census_counts_rows_that_hold_a_value() {
        let device = device();
        let rows = PairRows::new(dict(), marks(), 4, &device).unwrap();
        assert_eq!(rows.census().unwrap(), (0, 0.0));
        // Row 1 holds (3, 4, 0, 0): the squares sum to 25 over 16 entries, a mean of 25/16.
        let mut table = vec![0f32; 16];
        (table[4], table[5]) = (3.0, 4.0);
        rows.variables()[0]
            .set(&Tensor::from_vec(table, (4, 4), &device).unwrap())
            .unwrap();
        assert_eq!(rows.census().unwrap(), (1, 1.25));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn pair_start_counts_the_stream_and_holds_zero_rows() {
        // Turns (5,6), (7,8), (5,6,7): only (5,6) is seen in two.
        let stream = [1, 4, 5, 6, 2, 3, 1, 4, 7, 8, 2, 3, 1, 4, 5, 6, 7, 2, 3];
        let rows = PairRows::from_tokens(20, &stream, &marks(), 2, 4, &device()).unwrap();
        assert_eq!(rows.dict().len(), 1);
        assert_eq!(rows.dict().row(6, 5), Some(0));
        assert_eq!(rows.dict().row(7, 8), None);
        assert_eq!(rows.marks(), &marks());
        assert_eq!(rows.variables()[0].as_tensor().dims(), &[1, 4]);
        assert_eq!(rows.census().unwrap(), (0, 0.0));
    }

    #[test]
    fn pair_bag_readers_count_prompts_inside_a_reply_with_rows() {
        let inside: &[u32] = &[9, 1, 4, 5, 6, 2, 3, 10];
        let no_turn: &[u32] = &[9, 10, 11, 12, 5, 6];
        let marker_cut: &[u32] = &[4, 5, 6, 2, 3, 10];
        let count = |prompts: &[&[u32]]| bag_readers(&dict(), &marks(), prompts).unwrap();
        assert_eq!(count(&[inside]), 1);
        assert_eq!(count(&[no_turn]), 0);
        assert_eq!(count(&[marker_cut]), 0);
        assert_eq!(count(&[inside, no_turn, marker_cut, inside]), 2);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn pair_step_moves_the_rows_only_with_their_gradient() {
        let device = device();
        let pair = PairRows::new(dict(), marks(), 4, &device).unwrap();
        let vars = pair.variables();
        let mut opt = FlockAdamW::for_vars(&vars.iter().collect::<Vec<_>>()).unwrap();
        // A backward pass that never touched the rows: no step, the rows stay zero.
        let other = Var::ones((2, 4), DType::F32, &device).unwrap();
        let grads = other.as_tensor().sum_all().unwrap().backward().unwrap();
        assert!(!pair.step(&mut opt, &grads, 0.1).unwrap());
        assert_eq!(pair.census().unwrap(), (0, 0.0));
        // One through the read: rows 0 and 3 are hit and move, rows 1 and 2 stay zero.
        let x = Tensor::from_vec(state(1, 16, 4), (1, 16, 4), &device).unwrap();
        let grads = pair
            .apply(&x, &WINDOW_A)
            .unwrap()
            .sum_all()
            .unwrap()
            .backward()
            .unwrap();
        assert!(pair.step(&mut opt, &grads, 0.1).unwrap());
        assert_eq!(pair.census().unwrap().0, 2);
    }

    #[test]
    fn a_damaged_vocabulary_is_refused_without_counting_its_primes() {
        let start = std::time::Instant::now();
        assert!(token_primes(u32::MAX as usize).is_err());
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
        let mut bytes = dict().to_bytes();
        bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(PairDictionary::from_bytes(&bytes).is_err());
    }

    #[test]
    fn marks_where_one_begins_the_other_are_refused() {
        let check = |user: Vec<u32>, assistant: Vec<u32>| PairMarks { user, assistant }.check();
        assert!(check(vec![1, 4], vec![1, 4]).is_err());
        assert!(check(vec![1], vec![1, 4]).is_err());
        assert!(check(vec![2, 3, 9], vec![2, 3]).is_err());
        assert!(check(vec![1, 4], vec![4, 1]).is_ok());
    }

    #[test]
    fn a_second_user_marker_before_the_header_starts_the_turn_again() {
        // The turn is (7,8), row 3. Kept from the first marker it would also hold (5,6), (5,7)
        // and (6,7), rows 0, 1, 2.
        let ids = [1, 4, 5, 6, 1, 4, 7, 8, 2, 3, 10];
        let plan = plan(&dict(), &marks(), &ids, 1, 11).unwrap();
        assert_eq!(
            (plan.hit_turn.clone(), plan.hit_row.clone()),
            (vec![0], vec![3])
        );
        assert_eq!(turn_of(&plan), vec![N, N, N, N, N, N, N, N, N, 0, 0]);
        assert_eq!(plan.open, vec![Some(0)]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn half_a_checkpoint_is_refused() {
        let device = device();
        let directory =
            std::env::temp_dir().join(format!("uor-r4-pair-rows-{}-{:x}", std::process::id(), 371));
        std::fs::create_dir_all(&directory).unwrap();
        let pair = PairRows::new(dict(), marks(), 4, &device).unwrap();
        let error = |directory: &Path| match PairRows::load(directory, &device) {
            Err(error) => error.to_string(),
            Ok(_) => panic!("half a checkpoint loaded"),
        };
        pair.save(&directory).unwrap();
        std::fs::remove_file(directory.join(FILE)).unwrap();
        assert!(error(&directory).contains(FILE));
        pair.save(&directory).unwrap();
        std::fs::remove_file(directory.join(DICT_FILE)).unwrap();
        assert!(error(&directory).contains(DICT_FILE));
        std::fs::remove_file(directory.join(FILE)).unwrap();
        assert!(PairRows::load(&directory, &device).unwrap().is_none());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    /// claude-373: pair (5,6) is in three turns with replies `3 10`, `3 10`, `3 11`; pair (7,8) in
    /// two turns, both with reply `3 12`, the last of which runs to the stream's end.
    #[test]
    fn the_census_reads_back_from_the_weights_and_names_its_quantiles() {
        // R from 1 to the cap survives 1/R stored as f32 and read back rounded.
        let counts: Vec<u32> = (1..=REPLY_CAP as u32).collect();
        let back: Vec<u32> = counts
            .iter()
            .map(|&r| (1.0f64 / (1.0f32 / r as f32) as f64).round() as u32)
            .collect();
        assert_eq!(back, counts);
        assert_eq!(
            reply_census(&[5, 1, 3, 2, 4]),
            "5 pairs; R min 1 p50 3 p90 4 p99 4 max 5 (cap 4096)"
        );
        assert_eq!(reply_census(&[]), "0 pairs (cap 4096)");
    }

    #[test]
    fn a_reply_count_counts_each_different_reply_once() {
        let stream = [
            1, 4, 5, 6, 2, 3, 10, 1, 4, 5, 6, 2, 3, 10, 1, 4, 5, 6, 2, 3, 11, 1, 4, 7, 8, 2, 3, 12,
            1, 4, 7, 8, 2, 3, 12,
        ];
        let dict = PairDictionary::count(20, &stream, &marks(), 2).unwrap();
        assert_eq!(dict.len(), 2);
        assert_eq!(reply_counts(&dict, &stream, &marks()).unwrap(), vec![2, 1]);
    }

    /// claude-373: the reply counts the rows of [`dict`] are weighed by.
    #[cfg(feature = "cuda")]
    const COUNTS: [u32; 4] = [1, 2, 4, 8];

    /// Rows of [`dict`], width 4, set to known values and weighed by [`COUNTS`].
    #[cfg(feature = "cuda")]
    fn weighed(device: &Device) -> (PairRows, Vec<f32>) {
        let pair = PairRows::new(dict(), marks(), 4, device).unwrap();
        let table: Vec<f32> = (0..16).map(|n| ((n / 4 + 1) * 10 + n % 4) as f32).collect();
        pair.rows
            .set(&Tensor::from_vec(table.clone(), (4, 4), device).unwrap())
            .unwrap();
        (pair.with_reply_weights(&COUNTS).unwrap(), table)
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_weighed_bag_is_the_weighted_sum_of_the_turn_rows() {
        let device = device();
        let (pair, table) = weighed(&device);
        let weights = pair.weights().unwrap().to_vec1::<f32>().unwrap();
        assert_eq!(weights, vec![1.0, 0.5, 0.25, 0.125]);
        let part = |r: usize, c: usize| table[r * 4 + c] / COUNTS[r] as f32;
        let values = state(2, 16, 4);
        let x = Tensor::from_vec(values.clone(), (2, 16, 4), &device).unwrap();
        let y = pair.apply(&x, &batch_ids()).unwrap();
        let y = y.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let mut bag = vec![vec![0.0f32; 4]; 32];
        for p in [6, 7, 8] {
            bag[p] = (0..4).map(|c| part(0, c)).collect();
        }
        for p in [14, 15] {
            bag[p] = (0..4).map(|c| part(3, c)).collect();
        }
        for p in [16 + 6, 16 + 7, 16 + 8] {
            bag[p] = (0..4)
                .map(|c| part(0, c) + part(1, c) + part(2, c))
                .collect();
        }
        for (p, add) in bag.iter().enumerate() {
            for c in 0..4 {
                let expect = values[p * 4 + c] + add[c];
                assert_eq!(y[p * 4 + c], expect, "position {p} coordinate {c}");
            }
        }
    }

    /// A scratch folder for one test, removed by the caller.
    #[cfg(feature = "cuda")]
    fn scratch(tag: u32) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("uor-r4-pair-rows-{}-{tag:x}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn unweighed_rows_save_no_weights_and_load_unweighed() {
        let device = device();
        let directory = scratch(0x3730);
        let pair = PairRows::new(dict(), marks(), 4, &device).unwrap();
        assert!(pair.weights().is_none());
        pair.save(&directory).unwrap();
        let tensors = candle_core::safetensors::load(directory.join(FILE), &device).unwrap();
        let mut names: Vec<&String> = tensors.keys().collect();
        names.sort();
        assert_eq!(names, ["assistant", "rows", "user"]);
        let loaded = PairRows::load(&directory, &device).unwrap().unwrap();
        assert!(loaded.weights().is_none());
        assert_eq!(loaded.reply_counts_of_weights().unwrap(), None);
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn weights_round_trip_and_bad_weights_are_refused() {
        let device = device();
        let directory = scratch(0x3731);
        let (pair, _) = weighed(&device);
        pair.save(&directory).unwrap();
        let loaded = PairRows::load(&directory, &device).unwrap().unwrap();
        let back = loaded.weights().unwrap().to_vec1::<f32>().unwrap();
        assert_eq!(back, vec![1.0, 0.5, 0.25, 0.125]);
        assert_eq!(
            loaded.reply_counts_of_weights().unwrap(),
            Some(vec![1, 2, 4, 8])
        );
        let path = directory.join(FILE);
        let bad: [Vec<f32>; 4] = [
            vec![1.0, 0.5, 0.25],
            vec![1.0, 0.0, 0.25, 0.125],
            vec![1.0, -0.5, 0.25, 0.125],
            vec![1.0, f32::NAN, 0.25, 0.125],
        ];
        for weights in bad {
            let mut tensors = candle_core::safetensors::load(&path, &device).unwrap();
            let length = weights.len();
            let tensor = Tensor::from_vec(weights.clone(), length, &device).unwrap();
            tensors.insert("weights".to_owned(), tensor);
            candle_core::safetensors::save(&tensors, &path).unwrap();
            let error = PairRows::load(&directory, &device).unwrap_err().to_string();
            assert!(error.contains("pair-rows weight"), "{weights:?}: {error}");
        }
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_weighed_cached_step_equals_the_whole_window() {
        let device = device();
        let (pair, _) = weighed(&device);
        let ids = batch_ids();
        let values = state(2, 16, 4);
        let x = Tensor::from_vec(values.clone(), (2, 16, 4), &device).unwrap();
        let whole = pair.apply(&x, &ids).unwrap().narrow(1, 15, 1).unwrap();
        let step = pair.apply_last(&x.narrow(1, 15, 1).unwrap(), &ids).unwrap();
        let whole = whole.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let step = step.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        assert!(whole
            .iter()
            .zip(&step)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
        assert_ne!(
            whole[..4],
            values[15 * 4..16 * 4],
            "window A's last position adds row 3"
        );
    }

    /// claude-374: [`weighed`]'s rows and table, boosted by [`COUNTS`].
    #[cfg(feature = "cuda")]
    fn boosted(device: &Device) -> (PairRows, Vec<f32>) {
        let (pair, table) = weighed(device);
        let mut pair = pair;
        pair.weights = None;
        (pair.with_boost_weights(&COUNTS).unwrap(), table)
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_boosted_bag_is_the_sum_of_one_plus_inverse_count_rows() {
        let device = device();
        let (pair, table) = boosted(&device);
        assert_eq!(pair.weight_kind(), "boost");
        // The stored weight stays 1 / R; the forward adds the one.
        let weights = pair.weights().unwrap().to_vec1::<f32>().unwrap();
        assert_eq!(weights, vec![1.0, 0.5, 0.25, 0.125]);
        let part = |r: usize, c: usize| table[r * 4 + c] * (1.0 + 1.0 / COUNTS[r] as f32);
        let values = state(2, 16, 4);
        let x = Tensor::from_vec(values.clone(), (2, 16, 4), &device).unwrap();
        let y = pair.apply(&x, &batch_ids()).unwrap();
        let y = y.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let mut bag = vec![vec![0.0f32; 4]; 32];
        for p in [6, 7, 8] {
            bag[p] = (0..4).map(|c| part(0, c)).collect();
        }
        for p in [14, 15] {
            bag[p] = (0..4).map(|c| part(3, c)).collect();
        }
        for p in [16 + 6, 16 + 7, 16 + 8] {
            bag[p] = (0..4)
                .map(|c| part(0, c) + part(1, c) + part(2, c))
                .collect();
        }
        for (p, add) in bag.iter().enumerate() {
            for c in 0..4 {
                let expect = values[p * 4 + c] + add[c];
                assert_eq!(y[p * 4 + c], expect, "position {p} coordinate {c}");
            }
        }
        // A pair seen with one reply weighs 2: row 0 alone at position 6 adds twice itself.
        for c in 0..4 {
            assert_eq!(y[6 * 4 + c], values[6 * 4 + c] + 2.0 * table[c]);
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn replies_rows_are_claude_373s_and_write_no_boost_mark() {
        let device = device();
        let directory = scratch(0x3740);
        let (pair, _) = weighed(&device);
        assert_eq!(pair.weight_kind(), "replies");
        pair.save(&directory).unwrap();
        let tensors = candle_core::safetensors::load(directory.join(FILE), &device).unwrap();
        let mut names: Vec<&String> = tensors.keys().collect();
        names.sort();
        assert_eq!(names, ["assistant", "rows", "user", "weights"]);
        let loaded = PairRows::load(&directory, &device).unwrap().unwrap();
        assert_eq!(loaded.weight_kind(), "replies");
        assert_eq!(
            PairRows::new(dict(), marks(), 4, &device)
                .unwrap()
                .weight_kind(),
            "none"
        );
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn boost_round_trips_and_a_mark_without_weights_is_refused() {
        let device = device();
        let directory = scratch(0x3741);
        let (pair, _) = boosted(&device);
        pair.save(&directory).unwrap();
        let loaded = PairRows::load(&directory, &device).unwrap().unwrap();
        assert_eq!(loaded.weight_kind(), "boost");
        assert_eq!(
            loaded.reply_counts_of_weights().unwrap(),
            Some(vec![1, 2, 4, 8])
        );
        let x = Tensor::from_vec(state(2, 16, 4), (2, 16, 4), &device).unwrap();
        let a = pair.apply(&x, &batch_ids()).unwrap().flatten_all().unwrap();
        let b = loaded
            .apply(&x, &batch_ids())
            .unwrap()
            .flatten_all()
            .unwrap();
        assert_eq!(a.to_vec1::<f32>().unwrap(), b.to_vec1::<f32>().unwrap());
        let path = directory.join(FILE);
        let mut tensors = candle_core::safetensors::load(&path, &device).unwrap();
        tensors.insert(
            BOOST.to_owned(),
            Tensor::from_slice(&[2u32], 1, &device).unwrap(),
        );
        candle_core::safetensors::save(&tensors, &path).unwrap();
        let error = PairRows::load(&directory, &device).unwrap_err().to_string();
        assert!(error.contains("boost mark must be [1]"), "{error}");
        tensors.insert(
            BOOST.to_owned(),
            Tensor::from_slice(&[1u32], 1, &device).unwrap(),
        );
        tensors.remove("weights");
        candle_core::safetensors::save(&tensors, &path).unwrap();
        let error = PairRows::load(&directory, &device).unwrap_err().to_string();
        assert!(error.contains("boost mark without weights"), "{error}");
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_boosted_cached_step_equals_the_whole_window() {
        let device = device();
        let (pair, table) = boosted(&device);
        let ids = batch_ids();
        let values = state(2, 16, 4);
        let x = Tensor::from_vec(values.clone(), (2, 16, 4), &device).unwrap();
        let whole = pair.apply(&x, &ids).unwrap().narrow(1, 15, 1).unwrap();
        let step = pair.apply_last(&x.narrow(1, 15, 1).unwrap(), &ids).unwrap();
        let whole = whole.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let step = step.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        assert!(whole
            .iter()
            .zip(&step)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
        // Window A's last position adds row 3 at 1 + 1/8, in the step as in the whole window.
        for c in 0..4 {
            assert_eq!(step[c], values[15 * 4 + c] + table[12 + c] * 1.125);
        }
    }
}
