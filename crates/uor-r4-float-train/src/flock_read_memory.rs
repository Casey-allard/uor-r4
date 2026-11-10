//! The turn memory (route-plus-memory, claude-161).
//!
//! An exact table built from TRAIN, read at the readout:
//! 1. **Key** ([`TurnMemory::stored`]). At a row that predicts a reply's first token (an
//!    *opening row*, named by the caller), the token ids at the turn anchor's first `k` slots
//!    ([`opening_slots`], claude-154): the `--opening-bound` probe's first-`k` key.
//! 2. **Table** ([`build_table`]). Per key, TRAIN's commonest opening, ties to the smaller id:
//!    exact integer ids.
//! 3. **Move** ([`TurnMemory::apply_readout`]). At an opening row whose key the table holds, the
//!    readout point `n` moves toward the stored token's table point `c` (detached),
//!    `n + w ⊙ (c − n)`, with `w` per coordinate and starting at 0. Every other row, and an
//!    unseen key, is left as it is. At `w = 1` a hit lands on the stored token's point, to within float rounding, so the
//!    nearest readout decodes it. A nearest readout
//!    names the point a prediction is at, so the memory is a move, as the Hamming readout's copy
//!    is; nothing is exponential.
//!
//! The memory, and the opening rows of the next forward, are set per thread
//! ([`set_turn_memory`], [`set_memory_rows`]).
//!
//! **The reply table** (turn-memory-reply, claude-193), optional ([`TurnMemory::with_reply`]):
//! a second table for the rows past a reply's opening, keyed by the turn's first ids, the reply
//! position and the last reply id (the `--reply-bound` probe's `turn-8 + position +
//! reply-last-1`, claude-192). It keeps each key's commonest target only where that target's
//! share of the key's rows is at least the share it was built at ([`build_reply_table`]). Its
//! move is the same, with its own weight, at the rows the caller names with their keys
//! ([`set_reply_rows`]); the caller computes each key, so the read and the probe key alike.
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;

use candle_core::{DType, Device, Tensor, Var};

use crate::flock_read_opening::{opening_slots, OpeningAnchor};
use crate::{invalid, Result};

/// The longest key a memory may use.
const MAX_KEY: usize = 16;
const FILE: &str = "turn_memory.safetensors";
/// The reply table's own file, written only when the memory has one, so a memory without it
/// saves exactly the files it saved before (claude-193).
const REPLY_FILE: &str = "turn_memory_reply.safetensors";
/// The longest reply key: up to `MAX_KEY` turn ids, two separators, a position and one id.
pub const MAX_REPLY_KEY: usize = MAX_KEY + 4;
/// Pads a saved key shorter than `k`; never a token id.
const PAD: u32 = u32::MAX;
/// Joins the candidates of a chained reply key (claude-345): never a token id, and not [`PAD`].
/// A stored key never holds it; [`TurnMemory::reply_stored`] asks the candidates in order.
pub const CHAIN: u32 = u32::MAX - 1;

thread_local! {
    static ACTIVE: RefCell<Option<TurnMemory>> = const { RefCell::new(None) };
    static ROWS: RefCell<Option<Vec<usize>>> = const { RefCell::new(None) };
    static REPLY_ROWS: RefCell<Option<ReplyRows>> = const { RefCell::new(None) };
    static REPLY_TAIL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Reply rows named for a forward: each a `batch · time` row index and its reply key.
pub type ReplyRows = Vec<(usize, Vec<u32>)>;

/// Name the reply rows past their openings for this thread's next forward, each a `batch · time`
/// row index with its reply key, or none. The reply table moves nothing in a forward with none.
pub fn set_reply_rows(rows: Option<ReplyRows>) {
    REPLY_ROWS.with(|cell| *cell.borrow_mut() = rows);
}

/// The reply rows named for this thread's next forward, if any.
pub fn reply_rows() -> Option<ReplyRows> {
    REPLY_ROWS.with(|cell| cell.borrow().clone())
}

/// The reply table of `rows` (each a reply key and its target): for each key, the commonest
/// target, ties to the smaller id, kept only where its count is at least `min_share` of the
/// key's rows (claude-193; the `--reply-bound` probe's rule).
pub fn build_reply_table(rows: &[(Vec<u32>, u32)], min_share: f64) -> HashMap<Vec<u32>, u32> {
    let mut counts: HashMap<Vec<u32>, BTreeMap<u32, usize>> = HashMap::new();
    for (key, target) in rows {
        *counts
            .entry(key.clone())
            .or_default()
            .entry(*target)
            .or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(key, targets)| {
            let total: usize = targets.values().sum();
            targets
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                .filter(|(_, &n)| n as f64 >= min_share * total as f64)
                .map(|(&token, _)| (key, token))
        })
        .collect()
}

/// The keys the copy read owns (claude-345): of `rows`, each a table key and whether the copy
/// read's latest exact match already names that row's target, the keys where it does on more
/// than half of the key's rows. A table that leaves these keys out stores what the context cannot
/// supply. The count is over all of a key's rows, so a key's other rows never take the key over
/// once the copied ones are set aside.
pub fn copy_owned_keys(rows: &[(Vec<u32>, bool)]) -> std::collections::HashSet<Vec<u32>> {
    copy_owned_keys_of(rows.iter().map(|(key, copied)| (key.as_slice(), *copied)))
}

/// [`copy_owned_keys`] over borrowed keys, for a caller whose rows hold several keys each.
pub fn copy_owned_keys_of<'a>(
    rows: impl Iterator<Item = (&'a [u32], bool)>,
) -> std::collections::HashSet<Vec<u32>> {
    let mut counts: HashMap<&[u32], (usize, usize)> = HashMap::new();
    for (key, copied) in rows {
        let (named, total) = counts.entry(key).or_default();
        *named += usize::from(copied);
        *total += 1;
    }
    counts
        .into_iter()
        .filter(|(_, (named, total))| 2 * named > *total)
        .map(|(key, _)| key.to_vec())
        .collect()
}

/// Whether `key` is one a reply table can hold and save: 1 to [`MAX_REPLY_KEY`] ids, the last a
/// token id rather than the pad, so a padded saved key reads back exactly.
fn reply_key_ok(key: &[u32]) -> bool {
    !key.is_empty()
        && key.len() <= MAX_REPLY_KEY
        && key.last() != Some(&PAD)
        && !key.contains(&CHAIN)
}

/// How many of a turn's last ids the reply keys formed on this thread hold (claude-345): 0 keys
/// a reply on the turn's first ids alone, as before. Set before the reply table is built, and by
/// [`set_turn_memory`] from the memory it activates.
pub fn set_reply_tail(tail: usize) {
    REPLY_TAIL.with(|cell| cell.set(tail));
}

/// The tail length reply keys are formed with on this thread ([`set_reply_tail`]).
pub fn reply_tail() -> usize {
    REPLY_TAIL.with(std::cell::Cell::get)
}

/// Make `memory` the turn memory read on this thread, or none.
pub fn set_turn_memory(memory: Option<TurnMemory>) {
    set_reply_tail(memory.as_ref().map_or(0, TurnMemory::reply_tail));
    ACTIVE.with(|cell| *cell.borrow_mut() = memory);
}

/// The turn memory read on this thread, if set.
pub fn turn_memory() -> Option<TurnMemory> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// Name the opening rows of this thread's next forward, as `batch · time` row indices, or none.
/// The memory moves nothing in a forward with none.
pub fn set_memory_rows(rows: Option<Vec<usize>>) {
    ROWS.with(|cell| *cell.borrow_mut() = rows);
}

/// The opening rows named for this thread's next forward, if any.
pub fn memory_rows() -> Option<Vec<usize>> {
    ROWS.with(|cell| cell.borrow().clone())
}

/// The table of `openings` (each a prompt and the token that opened its reply): for each key,
/// the prompt's first `k` ids, the commonest opening, ties to the smaller id. An empty prompt
/// keys nothing.
pub fn build_table(openings: &[(Vec<u32>, u32)], k: usize) -> HashMap<Vec<u32>, u32> {
    let mut counts: HashMap<Vec<u32>, BTreeMap<u32, usize>> = HashMap::new();
    for (prompt, target) in openings {
        let key = &prompt[..k.min(prompt.len())];
        if key.is_empty() {
            continue;
        }
        *counts
            .entry(key.to_vec())
            .or_default()
            .entry(*target)
            .or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(key, targets)| {
            targets
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                .map(|(&token, _)| (key, token))
        })
        .collect()
}

/// One turn memory: its key length, its table, and the per-coordinate weight of its move.
#[derive(Clone)]
pub struct TurnMemory {
    k: usize,
    table: Arc<HashMap<Vec<u32>, u32>>,
    /// `(width,)`.
    pub(crate) weight: Var,
    /// The reply table and its weight (claude-193), when the memory has one.
    pub(crate) reply: Option<ReplyTable>,
}

/// One row's turn-memory lookup (claude-251, Q5's decision trace): the key formed, the token it
/// named (`None` on a miss), and whether the row opens the reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryLookup {
    pub key: Vec<u32>,
    pub stored: Option<u32>,
    pub opening: bool,
}

/// The reply table of a [`TurnMemory`] and the per-coordinate weight of its move.
#[derive(Clone)]
pub(crate) struct ReplyTable {
    table: Arc<HashMap<Vec<u32>, u32>>,
    /// `(width,)`.
    pub(crate) weight: Var,
    /// How many of a turn's last ids the table's tail keys hold (claude-345); 0 when the table
    /// keys on the turn's first ids alone.
    tail: usize,
}

impl TurnMemory {
    /// A memory keyed on `k` ids over points of `width` coordinates, with a zero weight, so it
    /// starts as no read at all. An empty `table` is the no-memory control.
    pub fn new(
        k: usize,
        table: HashMap<Vec<u32>, u32>,
        width: usize,
        device: &Device,
    ) -> Result<Self> {
        if !(1..=MAX_KEY).contains(&k) {
            return Err(invalid(format!("a turn memory keys on 1 to {MAX_KEY} ids")));
        }
        if width == 0 {
            return Err(invalid("a turn memory needs a width"));
        }
        if table.keys().any(|key| key.is_empty() || key.len() > k) {
            return Err(invalid(format!("a turn memory's keys hold 1 to {k} ids")));
        }
        Ok(Self {
            k,
            table: Arc::new(table),
            weight: Var::zeros(width, DType::F32, device)?,
            reply: None,
        })
    }

    /// This memory with a reply table (claude-193) and a zero reply weight, so the table starts
    /// as no read at all. An empty `table` is the no-memory control's.
    pub fn with_reply(mut self, table: HashMap<Vec<u32>, u32>) -> Result<Self> {
        if !table.keys().all(|key| reply_key_ok(key)) {
            return Err(invalid(format!(
                "a reply table's keys hold 1 to {MAX_REPLY_KEY} ids and end in a token id"
            )));
        }
        let weight = Var::zeros(self.weight.dim(0)?, DType::F32, self.weight.device())?;
        self.reply = Some(ReplyTable {
            table: Arc::new(table),
            weight,
            tail: 0,
        });
        Ok(self)
    }

    /// This memory with its reply table's tail keys holding a turn's last `tail` ids
    /// (claude-345). The longest tail key is the form's mark, the ids, two separators, a position
    /// and one id, so `tail` is 1 to `MAX_REPLY_KEY - 5`. Refused without a reply table.
    pub fn with_reply_tail(mut self, tail: usize) -> Result<Self> {
        let Some(reply) = self.reply.as_mut() else {
            return Err(invalid("a reply tail needs a reply table"));
        };
        if !(1..=MAX_REPLY_KEY - 5).contains(&tail) {
            return Err(invalid(format!(
                "a reply tail holds 1 to {} ids",
                MAX_REPLY_KEY - 5
            )));
        }
        reply.tail = tail;
        Ok(self)
    }

    /// The reply table's tail length ([`with_reply_tail`](Self::with_reply_tail)); 0 without one.
    pub fn reply_tail(&self) -> usize {
        self.reply.as_ref().map_or(0, |reply| reply.tail)
    }

    pub fn k(&self) -> usize {
        self.k
    }

    /// How many keys the reply table holds, or none when the memory has no reply table.
    pub fn reply_len(&self) -> Option<usize> {
        self.reply.as_ref().map(|reply| reply.table.len())
    }

    /// The reply table's stored token for `key`, if it holds the key. A chained key (claude-345)
    /// is its candidates joined by [`CHAIN`], asked in order: the first the table holds names the
    /// token. A key without the join is one candidate, read as before.
    pub fn reply_stored(&self, key: &[u32]) -> Option<u32> {
        self.reply_lookup(key).1
    }

    /// The candidate of `key` the reply table holds and its stored token; where it holds none,
    /// the last candidate (the key on the turn's first ids) and nothing.
    pub fn reply_lookup<'a>(&self, key: &'a [u32]) -> (&'a [u32], Option<u32>) {
        let mut last = key;
        for candidate in key.split(|&id| id == CHAIN) {
            last = candidate;
            let stored = self.reply.as_ref().and_then(|reply| reply.table.get(candidate));
            if let Some(&token) = stored {
                return (candidate, Some(token));
            }
        }
        (last, None)
    }

    /// How many keys the table holds.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// The opening table's entries in ascending key order, the order [`save`](Self::save) writes
    /// (claude-198, for the export).
    pub fn opening_entries(&self) -> Vec<(Vec<u32>, u32)> {
        sorted_entries(&self.table)
    }

    /// The opening move's per-coordinate weight.
    pub fn opening_weight(&self) -> Result<Vec<f32>> {
        Ok(self.weight.as_tensor().to_vec1::<f32>()?)
    }

    /// The reply table's entries in ascending key order and the reply move's weight, or none
    /// when the memory has no reply table (claude-198, for the export).
    pub fn reply_parts(&self) -> Result<Option<(Vec<(Vec<u32>, u32)>, Vec<f32>)>> {
        self.reply
            .as_ref()
            .map(|reply| Ok((sorted_entries(&reply.table), reply.weight.as_tensor().to_vec1::<f32>()?)))
            .transpose()
    }

    /// The host side of [`apply_readout`](Self::apply_readout) for windows `ids` of `time`
    /// positions: for every `batch · time` row, the stored token (0 where none) and a 0/1 hit,
    /// set only at the opening `rows` whose key the table holds.
    pub fn stored(&self, ids: &[u32], time: usize, rows: &[usize]) -> Result<(Vec<u32>, Vec<f32>)> {
        let total = ids.len();
        let (mut stored, mut hit) = (vec![0u32; total], vec![0f32; total]);
        for (r, key) in self.opening_keys(ids, time, rows)? {
            if let Some(&token) = self.table.get(&key) {
                stored[r] = token;
                hit[r] = 1.0;
            }
        }
        Ok((stored, hit))
    }

    /// Each opening row's key, `(row, key)`: the ids at the turn anchor's first `k` slots of the
    /// row's window, the key [`stored`](Self::stored) looks up.
    fn opening_keys(
        &self,
        ids: &[u32],
        time: usize,
        rows: &[usize],
    ) -> Result<Vec<(usize, Vec<u32>)>> {
        if time == 0 || !ids.len().is_multiple_of(time) {
            return Err(invalid("the turn memory needs whole windows"));
        }
        let total = ids.len();
        let mut by_window: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for &r in rows {
            if r >= total {
                return Err(invalid(format!(
                    "opening row {r} is outside the {total} rows of this forward"
                )));
            }
            by_window.entry(r / time).or_default().push(r % time);
        }
        let mut keys = Vec::with_capacity(rows.len());
        for (b, offsets) in by_window {
            let window = &ids[b * time..(b + 1) * time];
            let slots = opening_slots(window, self.k, OpeningAnchor::Turn);
            for t in offsets {
                keys.push((b * time + t, slots[t].iter().map(|&s| window[s]).collect()));
            }
        }
        Ok(keys)
    }

    /// claude-251, Q5's decision trace: each row's lookup in a forward of windows `ids` of
    /// `time` positions, as [`apply_readout`](Self::apply_readout) makes it. An opening row
    /// looks its key up in the opening table; a named reply row, in the reply table where the
    /// memory has one; every other row looks nothing up.
    pub fn lookups(
        &self,
        ids: &[u32],
        time: usize,
        rows: &[usize],
        reply_rows: &[(usize, Vec<u32>)],
    ) -> Result<Vec<Option<MemoryLookup>>> {
        let total = ids.len();
        let mut lookups = vec![None; total];
        for (r, key) in self.opening_keys(ids, time, rows)? {
            let stored = self.table.get(&key).copied();
            lookups[r] = Some(MemoryLookup { key, stored, opening: true });
        }
        if self.reply.is_some() {
            for (r, key) in reply_rows {
                if *r >= total {
                    return Err(invalid(format!(
                        "reply row {r} is outside the {total} rows of this forward"
                    )));
                }
                // The candidate that was held, as the integer session reports its move's key.
                let (key, stored) = self.reply_lookup(key);
                lookups[*r] = Some(MemoryLookup { key: key.to_vec(), stored, opening: false });
            }
        }
        Ok(lookups)
    }

    /// The host side of the reply move for a forward of `total` rows: for every row, the reply
    /// table's stored token (0 where none) and a 0/1 hit, set only at the named `rows` whose key
    /// the reply table holds. A memory without a reply table hits nothing.
    pub fn stored_reply(
        &self,
        total: usize,
        rows: &[(usize, Vec<u32>)],
    ) -> Result<(Vec<u32>, Vec<f32>)> {
        let (mut stored, mut hit) = (vec![0u32; total], vec![0f32; total]);
        for (r, key) in rows {
            if *r >= total {
                return Err(invalid(format!(
                    "reply row {r} is outside the {total} rows of this forward"
                )));
            }
            if let Some(token) = self.reply_stored(key) {
                stored[*r] = token;
                hit[*r] = 1.0;
            }
        }
        Ok((stored, hit))
    }

    /// The memory read on the final norm's output `(batch, time, width)` of the windows `ids`,
    /// with token points from the shared `table`: at the opening `rows` whose key the table
    /// holds, `n + w ⊙ (c − n)`; at the named `reply_rows` whose key the reply table holds,
    /// `n + w_r ⊙ (c_r − n)` (claude-193); elsewhere `n`. The two sets of rows must not share a
    /// row. Returns the same shape.
    pub fn apply_readout(
        &self,
        output: &Tensor,
        ids: &[u32],
        rows: &[usize],
        reply_rows: &[(usize, Vec<u32>)],
        table: &Tensor,
    ) -> Result<Tensor> {
        let (batch, time, width) = output.dims3()?;
        let total = batch * time;
        if ids.len() != total {
            return Err(invalid("the turn memory needs whole windows"));
        }
        if width != self.weight.dim(0)? {
            return Err(invalid(
                "the turn memory's width differs from the readout's",
            ));
        }
        let device = output.device();
        let (stored, hit) = self.stored(ids, time, rows)?;
        let points = output.reshape((total, width))?;
        let target = table
            .detach()
            .index_select(&Tensor::from_vec(stored, total, device)?, 0)?;
        let hit = Tensor::from_vec(hit, (total, 1), device)?;
        let pull = target
            .sub(&points)?
            .broadcast_mul(&hit)?
            .broadcast_mul(self.weight.as_tensor())?;
        let mut moved = points.add(&pull)?;
        if let Some(reply) = self.reply.as_ref().filter(|_| !reply_rows.is_empty()) {
            if reply_rows.iter().any(|(r, _)| rows.contains(r)) {
                return Err(invalid(
                    "a row is named both as an opening and as a later reply row",
                ));
            }
            let (stored, hit) = self.stored_reply(total, reply_rows)?;
            let target = table
                .detach()
                .index_select(&Tensor::from_vec(stored, total, device)?, 0)?;
            let hit = Tensor::from_vec(hit, (total, 1), device)?;
            let pull = target
                .sub(&points)?
                .broadcast_mul(&hit)?
                .broadcast_mul(reply.weight.as_tensor())?;
            moved = moved.add(&pull)?;
        }
        Ok(moved.reshape((batch, time, width))?)
    }

    /// Every learned parameter, in a fixed order for the optimizer: the weight, then the reply
    /// weight when the memory has a reply table.
    pub fn variables(&self) -> Vec<&Var> {
        let mut vars = vec![&self.weight];
        if let Some(reply) = &self.reply {
            vars.push(&reply.weight);
        }
        vars
    }

    /// The mean absolute value of the weight, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        Ok(self.weight.as_tensor().abs()?.mean_all()?)
    }

    /// The mean absolute value of the reply weight, or none without a reply table.
    pub fn reply_weight_mean_abs(&self) -> Result<Option<Tensor>> {
        self.reply
            .as_ref()
            .map(|reply| Ok(reply.weight.as_tensor().abs()?.mean_all()?))
            .transpose()
    }

    /// Write the memory to `directory/turn_memory.safetensors`: its weight, `k`, and the table
    /// as `keys` `(n, k)` (padded with `u32::MAX`) and `values` `(n,)`, sorted by key. An empty
    /// table is stored as one padded row with `count` 0.
    pub fn save(&self, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        let device = self.weight.device();
        let mut entries: Vec<(&Vec<u32>, &u32)> = self.table.iter().collect();
        entries.sort();
        let count = entries.len();
        let mut keys = Vec::with_capacity(count.max(1) * self.k);
        let mut values = Vec::with_capacity(count.max(1));
        for (key, &value) in &entries {
            keys.extend(
                key.iter()
                    .copied()
                    .chain(std::iter::repeat(PAD))
                    .take(self.k),
            );
            values.push(value);
        }
        if count == 0 {
            keys.extend(std::iter::repeat_n(PAD, self.k));
            values.push(PAD);
        }
        let rows = values.len();
        let mut tensors = std::collections::HashMap::new();
        tensors.insert("weight".to_owned(), self.weight.as_tensor().clone());
        tensors.insert("k".to_owned(), Tensor::new(&[self.k as u32], device)?);
        tensors.insert("count".to_owned(), Tensor::new(&[count as u32], device)?);
        tensors.insert(
            "keys".to_owned(),
            Tensor::from_vec(keys, (rows, self.k), device)?,
        );
        tensors.insert("values".to_owned(), Tensor::from_vec(values, rows, device)?);
        candle_core::safetensors::save(&tensors, directory.join(FILE))?;
        if let Some(reply) = &self.reply {
            save_reply(reply, &directory.join(REPLY_FILE))?;
        }
        Ok(())
    }

    /// The memory saved in `directory`, or none when the checkpoint has none.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<&Tensor> {
            tensors
                .get(name)
                .ok_or_else(|| invalid(format!("the saved turn memory has no {name}")))
        };
        let scalar = |name: &str| -> Result<usize> {
            get(name)?
                .to_vec1::<u32>()?
                .first()
                .map(|&v| v as usize)
                .ok_or_else(|| invalid(format!("the saved turn memory's {name} is empty")))
        };
        let (k, count) = (scalar("k")?, scalar("count")?);
        let keys = get("keys")?.to_vec2::<u32>()?;
        let values = get("values")?.to_vec1::<u32>()?;
        if keys.len() != values.len() || keys.len() < count || keys.iter().any(|row| row.len() != k)
        {
            return Err(invalid(
                "the saved turn memory's keys and values do not match its k and count",
            ));
        }
        let table: HashMap<Vec<u32>, u32> = keys
            .into_iter()
            .zip(values)
            .take(count)
            .map(|(key, value)| (key.into_iter().take_while(|&id| id != PAD).collect(), value))
            .collect();
        if table.len() != count {
            return Err(invalid("the saved turn memory repeats a key"));
        }
        let weight = Var::from_tensor(get("weight")?)?;
        let mut memory = Self::new(k, table, weight.dim(0)?, device)?;
        memory.weight = weight;
        let reply_path = directory.join(REPLY_FILE);
        if reply_path.exists() {
            let (table, reply_weight, tail) = load_reply(&reply_path, device)?;
            if reply_weight.dim(0)? != memory.weight.dim(0)? {
                return Err(invalid(
                    "the saved reply table's weight differs in width from the memory's",
                ));
            }
            memory = memory.with_reply(table)?;
            if tail > 0 {
                memory = memory.with_reply_tail(tail)?;
            }
            if let Some(reply) = memory.reply.as_mut() {
                reply.weight = reply_weight;
            }
        }
        Ok(Some(memory))
    }
}

/// A table's entries in ascending key order, as `save` and `save_reply` write them.
fn sorted_entries(table: &HashMap<Vec<u32>, u32>) -> Vec<(Vec<u32>, u32)> {
    let mut entries: Vec<(Vec<u32>, u32)> = table.iter().map(|(k, &v)| (k.clone(), v)).collect();
    entries.sort();
    entries
}

/// Write `reply` to `path`: its weight, and the table as `keys` `(n, MAX_REPLY_KEY)` (padded
/// with `u32::MAX`) and `values` `(n,)`, sorted by key, with `count`. An empty table is stored
/// as one padded row with `count` 0.
fn save_reply(reply: &ReplyTable, path: &Path) -> Result<()> {
    let device = reply.weight.device();
    let mut entries: Vec<(&Vec<u32>, &u32)> = reply.table.iter().collect();
    entries.sort();
    let count = entries.len();
    let mut keys = Vec::with_capacity(count.max(1) * MAX_REPLY_KEY);
    let mut values = Vec::with_capacity(count.max(1));
    for (key, &value) in &entries {
        keys.extend(
            key.iter()
                .copied()
                .chain(std::iter::repeat(PAD))
                .take(MAX_REPLY_KEY),
        );
        values.push(value);
    }
    if count == 0 {
        keys.extend(std::iter::repeat_n(PAD, MAX_REPLY_KEY));
        values.push(PAD);
    }
    let rows = values.len();
    let mut tensors = std::collections::HashMap::new();
    tensors.insert("weight".to_owned(), reply.weight.as_tensor().clone());
    tensors.insert("count".to_owned(), Tensor::new(&[count as u32], device)?);
    tensors.insert(
        "keys".to_owned(),
        Tensor::from_vec(keys, (rows, MAX_REPLY_KEY), device)?,
    );
    tensors.insert("values".to_owned(), Tensor::from_vec(values, rows, device)?);
    // Written only with a tail, so a table without one saves the file it saved before.
    if reply.tail > 0 {
        tensors.insert("tail".to_owned(), Tensor::new(&[reply.tail as u32], device)?);
    }
    candle_core::safetensors::save(&tensors, path)?;
    Ok(())
}

/// The reply table and weight saved at `path` by [`save_reply`]. A key's trailing pads are cut;
/// a key ends in a token id, so this reads back exactly what was saved.
fn load_reply(path: &Path, device: &Device) -> Result<(HashMap<Vec<u32>, u32>, Var, usize)> {
    let tensors = candle_core::safetensors::load(path, device)?;
    let get = |name: &str| -> Result<&Tensor> {
        tensors
            .get(name)
            .ok_or_else(|| invalid(format!("the saved reply table has no {name}")))
    };
    let count = get("count")?
        .to_vec1::<u32>()?
        .first()
        .map(|&v| v as usize)
        .ok_or_else(|| invalid("the saved reply table's count is empty"))?;
    let keys = get("keys")?.to_vec2::<u32>()?;
    let values = get("values")?.to_vec1::<u32>()?;
    if keys.len() != values.len()
        || keys.len() < count
        || keys.iter().any(|row| row.len() != MAX_REPLY_KEY)
    {
        return Err(invalid(
            "the saved reply table's keys and values do not match its count",
        ));
    }
    let table: HashMap<Vec<u32>, u32> = keys
        .into_iter()
        .zip(values)
        .take(count)
        .map(|(mut key, value)| {
            while key.last() == Some(&PAD) {
                key.pop();
            }
            (key, value)
        })
        .collect();
    if table.len() != count {
        return Err(invalid("the saved reply table repeats a key"));
    }
    let tail = match tensors.get("tail") {
        Some(tail) => tail.to_vec1::<u32>()?.first().map_or(0, |&v| v as usize),
        None => 0,
    };
    Ok((table, Var::from_tensor(get("weight")?)?, tail))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flock_read_opening::SHEET_EOS_ID;
    use crate::hamilton_sheet::SHEET_BOS_ID;

    #[test]
    fn the_table_keeps_each_keys_commonest_opening_with_ties_to_the_smaller_id() {
        let openings = vec![
            (vec![5, 6, 7], 40),
            (vec![5, 6, 9], 41),
            (vec![5, 6], 41),
            // A tie between 30 and 20 under one key goes to 20.
            (vec![8], 30),
            (vec![8], 20),
            // An empty prompt keys nothing.
            (vec![], 99),
        ];
        let table = build_table(&openings, 2);
        assert_eq!(table.len(), 2);
        // The key is the first 2 ids: three openings share [5, 6], two of them 41.
        assert_eq!(table[&vec![5, 6]], 41);
        assert_eq!(table[&vec![8]], 20);
        // A longer key splits them.
        let longer = build_table(&openings, 3);
        assert_eq!(longer[&vec![5, 6, 7]], 40);
        assert_eq!(longer[&vec![5, 6, 9]], 41);
        assert_eq!(longer[&vec![5, 6]], 41);
    }

    #[test]
    fn the_reply_table_keeps_each_keys_commonest_target_at_its_share() {
        let rows = vec![
            // Key [1]: 7 twice, 8 once, a 2/3 share.
            (vec![1], 7),
            (vec![1], 7),
            (vec![1], 8),
            // Key [2]: 4 and 2 once each, exactly 1/2; the tie goes to the smaller id.
            (vec![2], 4),
            (vec![2], 2),
            // Key [3]: three targets once each, 1/3.
            (vec![3], 9),
            (vec![3], 8),
            (vec![3], 7),
        ];
        let half = build_reply_table(&rows, 0.5);
        assert_eq!(half.len(), 2);
        assert_eq!((half[&vec![1]], half[&vec![2]]), (7, 2));
        // A share exactly at the threshold is kept, and dropped just above it.
        assert!(!build_reply_table(&rows, 0.51).contains_key(&vec![2]));
        // At share 0 every key keeps its commonest target.
        let all = build_reply_table(&rows, 0.0);
        assert_eq!((all.len(), all[&vec![3]]), (3, 7));
    }

    #[test]
    fn a_reply_row_moves_only_when_named_and_its_key_is_held() {
        let mut table = HashMap::new();
        table.insert(vec![11, PAD, 2, PAD, 51], 52u32);
        let memory = TurnMemory::new(2, HashMap::new(), 4, &Device::Cpu)
            .unwrap()
            .with_reply(table)
            .unwrap();
        assert_eq!(memory.reply_len(), Some(1));
        let held = vec![11, PAD, 2, PAD, 51];
        let unseen = vec![11, PAD, 2, PAD, 99];
        let (stored, hit) = memory
            .stored_reply(6, &[(4, held.clone()), (5, unseen)])
            .unwrap();
        assert_eq!((stored[4], hit[4]), (52, 1.0));
        assert_eq!(hit.iter().filter(|&&h| h == 1.0).count(), 1);
        // A row past the forward is refused; a memory with no reply table hits nothing.
        assert!(memory.stored_reply(6, &[(6, held.clone())]).is_err());
        let plain = TurnMemory::new(2, HashMap::new(), 4, &Device::Cpu).unwrap();
        assert_eq!(plain.reply_len(), None);
        let (_, hit) = plain.stored_reply(6, &[(4, held)]).unwrap();
        assert!(hit.iter().all(|&h| h == 0.0));
    }

    #[test]
    fn a_reply_table_refuses_keys_it_cannot_save() {
        let plain = || TurnMemory::new(2, HashMap::new(), 4, &Device::Cpu).unwrap();
        for key in [vec![], vec![1; MAX_REPLY_KEY + 1], vec![1, 2, PAD]] {
            let mut table = HashMap::new();
            table.insert(key.clone(), 3u32);
            assert!(plain().with_reply(table).is_err(), "{key:?}");
        }
        let empty = plain().with_reply(HashMap::new()).unwrap();
        assert_eq!(empty.reply_len(), Some(0));
        assert_eq!(empty.variables().len(), 2);
    }

    #[test]
    fn a_saved_reply_table_loads_and_leaves_the_opening_file_as_it_was() {
        let device = Device::Cpu;
        let dir = std::env::temp_dir().join(format!("claude-193-reply-{}", std::process::id()));
        let mut openings = HashMap::new();
        openings.insert(vec![3, 4], 6u32);
        let plain = TurnMemory::new(2, openings, 4, &device).unwrap();
        let mut table = HashMap::new();
        table.insert(vec![3, 4, PAD, 1, PAD, 6], 7u32);
        table.insert(vec![PAD, 2, PAD, 9], 8u32);
        let with = plain.clone().with_reply(table.clone()).unwrap();
        let reply_weight = Tensor::new(&[0.5f32, -1.0, 0.0, 2.0], &device).unwrap();
        with.reply
            .as_ref()
            .unwrap()
            .weight
            .set(&reply_weight)
            .unwrap();
        plain.save(&dir.join("plain")).unwrap();
        with.save(&dir.join("with")).unwrap();
        // The opening file is the same bytes with or without a reply table, and only the
        // memory with one writes the reply file.
        assert_eq!(
            std::fs::read(dir.join("plain").join(FILE)).unwrap(),
            std::fs::read(dir.join("with").join(FILE)).unwrap()
        );
        assert!(!dir.join("plain").join(REPLY_FILE).exists());
        let loaded = TurnMemory::load(&dir.join("with"), &device)
            .unwrap()
            .expect("saved");
        let reply = loaded.reply.as_ref().expect("a reply table");
        assert_eq!(*reply.table, table);
        assert_eq!(
            reply.weight.as_tensor().to_vec1::<f32>().unwrap(),
            vec![0.5, -1.0, 0.0, 2.0]
        );
        let loaded = TurnMemory::load(&dir.join("plain"), &device)
            .unwrap()
            .expect("saved");
        assert!(loaded.reply.is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// claude-345: a chained key is its candidates asked in order; the tail is saved with the
    /// reply table, and a table without one saves the file it saved before.
    #[test]
    fn a_reply_tail_is_saved_with_the_table_and_a_chained_key_is_asked_in_order() {
        let device = Device::Cpu;
        let dir = std::env::temp_dir().join(format!("claude-345-tail-{}", std::process::id()));
        let mut openings = HashMap::new();
        openings.insert(vec![3, 4], 6u32);
        let plain = TurnMemory::new(2, openings, 4, &device).unwrap();
        let ends = vec![3, 4, 5, 6, PAD, 1, PAD, 6];
        let tail = vec![PAD, 5, 6, PAD, 1, PAD, 6];
        let head = vec![3, 4, PAD, 1, PAD, 6];
        let mut table = HashMap::new();
        table.insert(tail.clone(), 8u32);
        table.insert(head.clone(), 9u32);
        let without = plain.clone().with_reply(table.clone()).unwrap();
        let with = without.clone().with_reply_tail(2).unwrap();
        assert_eq!((without.reply_tail(), with.reply_tail()), (0, 2));
        // A tail needs a reply table and a length a key can hold.
        assert!(plain.clone().with_reply_tail(2).is_err());
        assert!(without.clone().with_reply_tail(0).is_err());
        assert!(without.clone().with_reply_tail(MAX_REPLY_KEY - 4).is_err());
        // The first candidate the table holds names the token; one candidate reads as before.
        let wide = [&ends[..], &[CHAIN], &tail, &[CHAIN], &head].concat();
        assert_eq!(with.reply_lookup(&wide), (&tail[..], Some(8)));
        assert_eq!(with.reply_stored(&[&ends[..], &[CHAIN], &head].concat()), Some(9));
        assert_eq!(with.reply_lookup(&ends), (&ends[..], None));
        assert_eq!(with.reply_stored(&head), Some(9));
        // A table cannot hold the mark that joins candidates.
        let mut marked = HashMap::new();
        marked.insert(vec![3, CHAIN, 4], 1u32);
        assert!(plain.clone().with_reply(marked).is_err());
        // Saved and loaded, the tail and the table come back, and the thread keys by them.
        with.save(&dir.join("with")).unwrap();
        without.save(&dir.join("without")).unwrap();
        let loaded = TurnMemory::load(&dir.join("with"), &device).unwrap().expect("saved");
        assert_eq!(loaded.reply_tail(), 2);
        assert_eq!(*loaded.reply.as_ref().expect("a reply table").table, table);
        set_turn_memory(Some(loaded));
        assert_eq!(reply_tail(), 2);
        let loaded = TurnMemory::load(&dir.join("without"), &device).unwrap().expect("saved");
        assert_eq!(loaded.reply_tail(), 0);
        set_turn_memory(Some(loaded));
        assert_eq!(reply_tail(), 0);
        set_turn_memory(None);
        let file = dir.join("without").join(REPLY_FILE);
        let saved = candle_core::safetensors::load(&file, &device).unwrap();
        assert!(!saved.contains_key("tail"), "a table without a tail saves no tail");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_opening_row_keys_on_its_turns_first_ids_and_only_named_rows_hit() {
        let device = Device::Cpu;
        // One window of 10: BOS, a turn of 3 prompt ids, then the reply; EOS, a second turn.
        let window = vec![SHEET_BOS_ID, 11, 12, 13, 50, 51, SHEET_EOS_ID, 21, 22, 60];
        let mut table = HashMap::new();
        table.insert(vec![11, 12], 50);
        table.insert(vec![21, 22], 60);
        let memory = TurnMemory::new(2, table, 4, &device).unwrap();
        // Row 3 predicts the first reply's opening; row 8 the second's; row 5 is inside a reply.
        let (stored, hit) = memory.stored(&window, 10, &[3, 8]).unwrap();
        assert_eq!((stored[3], hit[3]), (50, 1.0));
        assert_eq!((stored[8], hit[8]), (60, 1.0));
        assert_eq!(hit.iter().filter(|&&h| h == 1.0).count(), 2);
        // Row 5 holds the same turn's key, and moves only when the caller names it.
        let (_, hit) = memory.stored(&window, 10, &[3]).unwrap();
        assert_eq!((hit[3], hit[5]), (1.0, 0.0));
        let (stored, hit) = memory.stored(&window, 10, &[5]).unwrap();
        assert_eq!((stored[5], hit[5]), (50, 1.0));
        // An unseen key moves nothing.
        let (_, hit) = memory.stored(&[SHEET_BOS_ID, 11, 99, 98], 4, &[3]).unwrap();
        assert!(hit.iter().all(|&h| h == 0.0));
        // Rows past the forward, or a ragged window, are refused.
        assert!(memory.stored(&window, 10, &[10]).is_err());
        assert!(memory.stored(&window, 3, &[0]).is_err());
    }

    /// claude-251: each row's lookup is the key the move reads and the entry it names, and a
    /// memory without a reply table looks no reply row up.
    #[test]
    fn each_rows_lookup_is_the_key_and_entry_the_move_reads() {
        let window = vec![SHEET_BOS_ID, 11, 12, 13, 50, 51, SHEET_EOS_ID, 21, 22, 60];
        let mut table = HashMap::new();
        table.insert(vec![11, 12], 50);
        let mut replies = HashMap::new();
        replies.insert(vec![11, PAD, 2, PAD, 51], 52u32);
        let memory = TurnMemory::new(2, table.clone(), 4, &Device::Cpu)
            .unwrap()
            .with_reply(replies)
            .unwrap();
        let reply = vec![(5, vec![11, PAD, 2, PAD, 51]), (9, vec![21, PAD, 1, PAD, 60])];
        let lookups = memory.lookups(&window, 10, &[3, 8], &reply).unwrap();
        let lookup = |key: Vec<u32>, stored, opening| Some(MemoryLookup { key, stored, opening });
        assert_eq!(lookups[3], lookup(vec![11, 12], Some(50), true));
        // An opening the table misses is still a lookup, with no entry.
        assert_eq!(lookups[8], lookup(vec![21, 22], None, true));
        assert_eq!(lookups[5], lookup(vec![11, PAD, 2, PAD, 51], Some(52), false));
        assert_eq!(lookups[9], lookup(vec![21, PAD, 1, PAD, 60], None, false));
        assert_eq!(lookups.iter().flatten().count(), 4);
        // The opening lookups name what `stored` moves toward.
        let (stored, hit) = memory.stored(&window, 10, &[3, 8]).unwrap();
        for r in [3, 8] {
            let named = lookups[r].as_ref().unwrap().stored;
            assert_eq!(named, (hit[r] == 1.0).then_some(stored[r]));
        }
        let plain = TurnMemory::new(2, table, 4, &Device::Cpu).unwrap();
        assert_eq!(plain.lookups(&window, 10, &[3], &reply).unwrap().iter().flatten().count(), 1);
        assert!(memory.lookups(&window, 10, &[3], &[(10, vec![1])]).is_err());
    }

    #[test]
    fn the_key_equals_the_probes_first_k_prompt_ids() {
        // The probe's prompt runs back from the opening to BOS; the anchor's slots start after it.
        let window = vec![SHEET_BOS_ID, 31, 32, 33, 34, 35, 70];
        let prompt = &window[1..=5];
        let mut table = HashMap::new();
        table.insert(prompt[..4].to_vec(), 70);
        let memory = TurnMemory::new(4, table, 4, &Device::Cpu).unwrap();
        let (stored, hit) = memory.stored(&window, 7, &[5]).unwrap();
        assert_eq!((stored[5], hit[5]), (70, 1.0));
    }

    /// claude-198: the memory's keys fit the integer artifact's rows, so every key this crate can
    /// save, the export can carry.
    #[test]
    fn the_memorys_key_shapes_are_the_integer_artifacts() {
        use uor_r4_integer::fbs_snapshot::{MEMORY_MAX_K, MEMORY_PAD, MEMORY_REPLY_KEY_LEN};
        assert_eq!(MAX_KEY, MEMORY_MAX_K);
        assert_eq!(MAX_REPLY_KEY, MEMORY_REPLY_KEY_LEN);
        assert_eq!(PAD, MEMORY_PAD);
    }

    #[test]
    fn a_memory_refuses_keys_it_cannot_hold() {
        let device = Device::Cpu;
        assert!(TurnMemory::new(0, HashMap::new(), 4, &device).is_err());
        assert!(TurnMemory::new(MAX_KEY + 1, HashMap::new(), 4, &device).is_err());
        let mut table = HashMap::new();
        table.insert(vec![1, 2, 3], 4);
        assert!(TurnMemory::new(2, table, 4, &device).is_err());
        assert!(TurnMemory::new(2, HashMap::new(), 4, &device)
            .unwrap()
            .is_empty());
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn zero_weight_is_no_read_and_the_move_is_the_stored_points_pull() {
        let device = Device::new_cuda(0).unwrap();
        let width = 4;
        let table = Tensor::from_vec(
            (0..8 * width)
                .map(|v| v as f32 * 0.25 - 3.0)
                .collect::<Vec<_>>(),
            (8, width),
            &device,
        )
        .unwrap();
        let ids = vec![SHEET_BOS_ID, 1 + 2, 4, 5, 6, 7];
        let output = Tensor::from_vec(
            (0..ids.len() * width)
                .map(|v| (v as f32).sin())
                .collect::<Vec<_>>(),
            (1, ids.len(), width),
            &device,
        )
        .unwrap();
        let mut entries = HashMap::new();
        entries.insert(vec![3, 4], 6u32);
        let memory = TurnMemory::new(2, entries, width, &device).unwrap();
        let rows = [2usize];
        let read = |m: &TurnMemory| -> Vec<Vec<f32>> {
            m.apply_readout(&output, &ids, &rows, &[], &table)
                .unwrap()
                .squeeze(0)
                .unwrap()
                .to_vec2::<f32>()
                .unwrap()
        };
        let before = output.squeeze(0).unwrap().to_vec2::<f32>().unwrap();
        // Zero weight: no read, bit for bit.
        assert_eq!(read(&memory), before);
        // Weight 1: the hit row lands on the stored token's point, to within float rounding
        // (n + (c − n) can be one ulp from c); no other row moves at all.
        memory
            .weight
            .set(&Tensor::ones(width, DType::F32, &device).unwrap())
            .unwrap();
        let moved = read(&memory);
        let stored = table.get(6).unwrap().to_vec1::<f32>().unwrap();
        for (i, (&m, &c)) in moved[2].iter().zip(&stored).enumerate() {
            assert!((m - c).abs() < 1e-6, "coordinate {i}: {m} != {c}");
        }
        for r in [0, 1, 3, 4, 5] {
            assert_eq!(moved[r], before[r], "row {r}");
        }
        // A quarter: n + 0.25 (c − n).
        memory
            .weight
            .set(&Tensor::full(0.25f32, width, &device).unwrap())
            .unwrap();
        let quarter = read(&memory);
        for (i, (&q, (&n, &c))) in quarter[2]
            .iter()
            .zip(before[2].iter().zip(&stored))
            .enumerate()
        {
            assert!((q - (n + 0.25 * (c - n))).abs() < 1e-6, "coordinate {i}");
        }
        // The no-memory control moves nothing, even at weight 1.
        let empty = TurnMemory::new(2, HashMap::new(), width, &device).unwrap();
        empty
            .weight
            .set(&Tensor::ones(width, DType::F32, &device).unwrap())
            .unwrap();
        assert_eq!(read(&empty), before);
        // The gradient reaches the weight only through hit rows.
        let fresh = TurnMemory::new(2, memory.table.as_ref().clone(), width, &device).unwrap();
        let grads = fresh
            .apply_readout(&output, &ids, &rows, &[], &table)
            .unwrap()
            .sum_all()
            .unwrap()
            .backward()
            .unwrap();
        let g = grads
            .get(fresh.weight.as_tensor())
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let want: Vec<f32> = stored.iter().zip(&before[2]).map(|(c, n)| c - n).collect();
        for (a, b) in g.iter().zip(&want) {
            assert!((a - b).abs() < 1e-5, "{g:?} != {want:?}");
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn zero_reply_weight_is_no_read_and_the_reply_move_is_its_pull() {
        let device = Device::new_cuda(0).unwrap();
        let width = 4;
        let table = Tensor::from_vec(
            (0..8 * width)
                .map(|v| v as f32 * 0.25 - 3.0)
                .collect::<Vec<_>>(),
            (8, width),
            &device,
        )
        .unwrap();
        let ids = vec![SHEET_BOS_ID, 3, 4, 5, 6, 7];
        let output = Tensor::from_vec(
            (0..ids.len() * width)
                .map(|v| (v as f32).cos())
                .collect::<Vec<_>>(),
            (1, ids.len(), width),
            &device,
        )
        .unwrap();
        let key = vec![3, 4, PAD, 2, PAD, 5];
        let mut entries = HashMap::new();
        entries.insert(key.clone(), 6u32);
        let memory = TurnMemory::new(2, HashMap::new(), width, &device)
            .unwrap()
            .with_reply(entries)
            .unwrap();
        let reply_rows = vec![(4usize, key.clone()), (5usize, vec![9, PAD, 1, PAD, 2])];
        let read = |m: &TurnMemory| -> Vec<Vec<f32>> {
            m.apply_readout(&output, &ids, &[], &reply_rows, &table)
                .unwrap()
                .squeeze(0)
                .unwrap()
                .to_vec2::<f32>()
                .unwrap()
        };
        let before = output.squeeze(0).unwrap().to_vec2::<f32>().unwrap();
        // Zero reply weight: no read, bit for bit.
        assert_eq!(read(&memory), before);
        // Reply weight 1: the held row lands on the stored token's point; the unseen key's row
        // and every other row do not move.
        let reply = memory.reply.as_ref().unwrap();
        reply
            .weight
            .set(&Tensor::ones(width, DType::F32, &device).unwrap())
            .unwrap();
        let moved = read(&memory);
        let stored = table.get(6).unwrap().to_vec1::<f32>().unwrap();
        for (i, (&m, &c)) in moved[4].iter().zip(&stored).enumerate() {
            assert!((m - c).abs() < 1e-6, "coordinate {i}: {m} != {c}");
        }
        for r in [0, 1, 2, 3, 5] {
            assert_eq!(moved[r], before[r], "row {r}");
        }
        // A row named both as an opening and as a reply row is refused.
        assert!(memory
            .apply_readout(&output, &ids, &[4], &reply_rows, &table)
            .is_err());
        // The gradient reaches the reply weight only through the held row.
        let fresh = TurnMemory::new(2, HashMap::new(), width, &device)
            .unwrap()
            .with_reply(memory.reply.as_ref().unwrap().table.as_ref().clone())
            .unwrap();
        let grads = fresh
            .apply_readout(&output, &ids, &[], &reply_rows, &table)
            .unwrap()
            .sum_all()
            .unwrap()
            .backward()
            .unwrap();
        let g = grads
            .get(fresh.reply.as_ref().unwrap().weight.as_tensor())
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let want: Vec<f32> = stored.iter().zip(&before[4]).map(|(c, n)| c - n).collect();
        for (a, b) in g.iter().zip(&want) {
            assert!((a - b).abs() < 1e-5, "{g:?} != {want:?}");
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_saved_memory_loads_with_its_table_and_weight() {
        let device = Device::new_cuda(0).unwrap();
        let dir = std::env::temp_dir().join(format!("claude-161-memory-{}", std::process::id()));
        let mut entries = HashMap::new();
        entries.insert(vec![3, 4], 6u32);
        entries.insert(vec![9], 2u32);
        let memory = TurnMemory::new(2, entries, 4, &device).unwrap();
        memory
            .weight
            .set(&Tensor::new(&[0.5f32, -1.0, 0.0, 2.0], &device).unwrap())
            .unwrap();
        memory.save(&dir).unwrap();
        let loaded = TurnMemory::load(&dir, &device).unwrap().expect("saved");
        assert_eq!(loaded.k(), 2);
        assert_eq!(*loaded.table, *memory.table);
        assert_eq!(
            loaded.weight.as_tensor().to_vec1::<f32>().unwrap(),
            vec![0.5, -1.0, 0.0, 2.0]
        );
        // An empty table round-trips as empty.
        let empty = TurnMemory::new(3, HashMap::new(), 4, &device).unwrap();
        let empty_dir = dir.join("empty");
        empty.save(&empty_dir).unwrap();
        let loaded = TurnMemory::load(&empty_dir, &device)
            .unwrap()
            .expect("saved");
        assert!(loaded.is_empty());
        assert_eq!(loaded.k(), 3);
        // No file: no memory.
        assert!(TurnMemory::load(&dir.join("none"), &device)
            .unwrap()
            .is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

#[cfg(test)]
mod copy_owned_tests {
    use super::copy_owned_keys;

    /// claude-345: a key is the copy read's when it names the target on more than half of the
    /// key's rows; an even split or a minority leaves the key with the table.
    #[test]
    fn the_copy_read_owns_a_key_it_names_on_most_rows() {
        let row = |key: u32, copied: bool| (vec![key, 7], copied);
        let rows = vec![
            row(1, true),
            row(1, true),
            row(1, false),
            row(2, true),
            row(2, false),
            row(3, false),
            row(4, true),
        ];
        let owned = copy_owned_keys(&rows);
        assert!(owned.contains(&vec![1, 7]) && owned.contains(&vec![4, 7]));
        assert!(!owned.contains(&vec![2, 7]) && !owned.contains(&vec![3, 7]));
        assert_eq!(owned.len(), 2);
        assert!(copy_owned_keys(&[]).is_empty());
    }
}
