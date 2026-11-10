//! The turn memory in the integer artifact (claude-198): claude-161's opening table and
//! claude-193's reply table, with their move weights, as a `uor-r4.lut-stack/7` container stores
//! them.
//!
//! [`super::IntegerStackModel::parse`] builds the served tables ([ServedTurnMemory]); the session
//! forms the keys and applies the moves (units 2-3 of `turn-memory-integer`).

use super::format::{Container, StackMemoryChain, StackTurnMemory, MEMORY_PAD, MEMORY_REPLY_KEY_LEN, MEMORY_WEIGHT_EXP};
use super::hamming::round_shift;
use super::kernels::NORM_FINE_EXP;
use super::point_key::{point_image, PointKeyImage, ServedPointKey};
use super::StackError;

/// One memory table: its keys (pads cut) in strictly ascending order, the id each names, and the
/// move's weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryTableImage {
    pub keys: Vec<Vec<u32>>,
    pub values: Vec<u32>,
    /// `width` values at `2^MEMORY_WEIGHT_EXP`.
    pub weight: Vec<i32>,
}

/// A stack's turn memory: the header record and its tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryImage {
    pub record: StackTurnMemory,
    pub opening: MemoryTableImage,
    pub reply: Option<MemoryTableImage>,
    /// The served point key (claude-222), on a `uor-r4.lut-stack/8` stack.
    pub point: Option<PointKeyImage>,
}

/// The turn memory of a `UORLUT01` stack artifact, every section checked, or `None` for a stack
/// without one.
pub fn read_memory_uorlut01(bytes: &[u8]) -> Result<Option<MemoryImage>, StackError> {
    memory_image(&Container::parse(bytes)?)
}

/// The turn memory of a parsed container, as [`read_memory_uorlut01`]: its reply keys of the
/// forms the container's memory chain record (claude-345) admits, or the head form alone without
/// one.
pub(crate) fn memory_image(artifact: &Container<'_>) -> Result<Option<MemoryImage>, StackError> {
    let chain_tail = artifact.memory_chain.as_ref().map(|chain| chain.tail);
    let Some(record) = artifact.memory.clone() else {
        return Ok(None);
    };
    let (vocab, width) = (artifact.shape.vocab, artifact.shape.width);
    let (k, window) = (record.k, record.key_window);
    let opening = memory_table(artifact, "opening", record.opening, k, width, vocab, &|key| {
        opening_key_ok(key, k, vocab)
    })?;
    // The record's check holds the count and the turn together.
    let reply = match (record.reply, record.reply_turn) {
        (Some(count), Some(turn)) => Some(memory_table(
            artifact,
            "reply",
            count,
            MEMORY_REPLY_KEY_LEN,
            width,
            vocab,
            &|key| reply_key_ok(key, turn, window, vocab, chain_tail),
        )?),
        _ => None,
    };
    Ok(Some(MemoryImage {
        record,
        opening,
        reply,
        point: point_image(artifact)?,
    }))
}

/// An opening key: 1 to `k` turn ids, each a token id (the float memory never stores an empty one).
fn opening_key_ok(key: &[u32], k: usize, vocab: usize) -> bool {
    !key.is_empty() && turn_ok(key, k, vocab)
}

/// At most `k` turn ids, each a token id.
fn turn_ok(ids: &[u32], k: usize, vocab: usize) -> bool {
    ids.len() <= k && ids.iter().all(|&id| (id as usize) < vocab)
}

/// A reply key, `turn ids, MEMORY_PAD, length, MEMORY_PAD, last id` (the float memory's
/// `ReplyKey::TurnPosition(1)`): 0 to `turn` turn ids (the record's `reply_turn`, not `k`), a reply
/// length from 1 to the key window, and a last id that is a token id. With the memory chain's
/// tail `chain_tail` (claude-345) the turn part is instead at most `2 * turn` ids (a head or ends
/// key) or `MEMORY_PAD` and at most `chain_tail` ids (a tail key).
fn reply_key_ok(key: &[u32], turn_len: usize, window: usize, vocab: usize, chain_tail: Option<usize>) -> bool {
    let n = key.len();
    if n < 4 {
        return false;
    }
    let (turn, tail) = key.split_at(n - 4);
    let (length, last) = (tail[1] as usize, tail[3] as usize);
    tail[0] == MEMORY_PAD
        && tail[2] == MEMORY_PAD
        && (1..=window).contains(&length)
        && last < vocab
        && match chain_tail {
            None => turn_ok(turn, turn_len, vocab),
            Some(tail) => match turn.split_first() {
                Some((&MEMORY_PAD, ids)) => turn_ok(ids, tail, vocab),
                _ => turn_ok(turn, 2 * turn_len, vocab),
            },
        }
}

/// One table's sections `memory.{kind}.{keys,values,weight}`: `count` keys of `row` ids each
/// (padded with `MEMORY_PAD`), `count` values and `width` weights. Refuses a section of the wrong
/// length, a key `key_ok` rejects, keys out of strictly ascending order, or a value past the
/// vocabulary.
fn memory_table(
    artifact: &Container<'_>,
    kind: &str,
    count: usize,
    row: usize,
    width: usize,
    vocab: usize,
    key_ok: &dyn Fn(&[u32]) -> bool,
) -> Result<MemoryTableImage, StackError> {
    let name = |part: &str| format!("memory.{kind}.{part}");
    let section = |part: &str, values: Vec<u32>, len: usize| -> Result<Vec<u32>, StackError> {
        if values.len() == len {
            Ok(values)
        } else {
            Err(StackError::TableLength {
                name: name(part),
                len: values.len(),
                expected: len,
            })
        }
    };
    let raw = section("keys", artifact.table_u32(&name("keys"))?, count * row)?;
    let values = section("values", artifact.table_u32(&name("values"))?, count)?;
    let weight = artifact.table_i32(&name("weight"))?;
    if weight.len() != width {
        return Err(StackError::TableLength {
            name: name("weight"),
            len: weight.len(),
            expected: width,
        });
    }
    let mut keys: Vec<Vec<u32>> = Vec::with_capacity(count);
    for (i, padded) in raw.chunks_exact(row).enumerate() {
        // A key ends in a token id, so cutting the trailing pads reads back exactly what was stored.
        let len = padded.iter().rposition(|&id| id != MEMORY_PAD).map_or(0, |p| p + 1);
        let key = padded[..len].to_vec();
        if !key_ok(&key) {
            return Err(StackError::Numerics(format!(
                "{}: entry {i} is not a key this engine reads",
                name("keys")
            )));
        }
        if keys.last().is_some_and(|previous| key <= *previous) {
            return Err(StackError::Numerics(format!(
                "{}: the keys are not in strictly ascending order at entry {i}",
                name("keys")
            )));
        }
        keys.push(key);
    }
    if let Some(i) = values.iter().position(|&v| v as usize >= vocab) {
        return Err(StackError::Numerics(format!(
            "{}: entry {i} names an id past the vocabulary",
            name("values")
        )));
    }
    Ok(MemoryTableImage {
        keys,
        values,
        weight,
    })
}
/// The bitmap's `<|bos|>` (reserved, never a prompt content id): the turn anchor restarts after
/// it, as the float side's `opening_slots(.., OpeningAnchor::Turn)` does.
pub(super) const MEMORY_BOS_ID: u32 = 0;
/// The sheet's `<|eos|>`: as `<|bos|>`, and it ends a reply.
pub(super) const MEMORY_EOS_ID: u32 = 1;

/// One memory table as the session looks it up: its keys in strictly ascending order (the
/// reader's order), the id each names, and the move's per-coordinate weight at
/// `2^MEMORY_WEIGHT_EXP`.
pub(crate) struct ServedMemoryTable {
    keys: Vec<Vec<u32>>,
    values: Vec<u32>,
    weight: Vec<i32>,
}

impl ServedMemoryTable {
    fn new(table: &MemoryTableImage) -> Self {
        Self {
            keys: table.keys.clone(),
            values: table.values.clone(),
            weight: table.weight.clone(),
        }
    }

    /// How many keys the table holds.
    fn len(&self) -> usize {
        self.keys.len()
    }

    /// The id `key` names, by binary search over the ascending keys.
    #[inline(never)]
    fn stored(&self, key: &[u32]) -> Option<u32> {
        self.keys
            .binary_search_by(|probe| probe.as_slice().cmp(key))
            .ok()
            .map(|i| self.values[i])
    }
}

/// The turn memory as a session serves it (units 2-3 of `turn-memory-integer`): the recorded key
/// sizes and both tables.
pub(crate) struct ServedTurnMemory {
    /// The header record the sections were checked against.
    pub(crate) record: StackTurnMemory,
    /// Turn ids an opening key holds (the record's `k`).
    pub(crate) k: usize,
    /// Training's key window: its 192-token blocks, counted from the conversation's first token.
    pub(crate) key_window: usize,
    /// The most turn ids a reply key holds (the record's `reply_turn`), with the reply table.
    pub(crate) reply_turn: Option<usize>,
    /// The memory chain's tail (claude-345): with it a reply row asks the ends, tail and head
    /// keys in that order; without it the head key alone.
    pub(crate) chain_tail: Option<usize>,
    opening: ServedMemoryTable,
    reply: Option<ServedMemoryTable>,
    /// The served point key (claude-222): it names a reply's first token where the opening
    /// table's exact key misses.
    pub(crate) point: Option<ServedPointKey>,
}

/// A session's stored turn state (units 2-3): the caller's reply start, and the anchor's ids at
/// the prompt's last token, kept for the reply table's keys.
#[derive(Default)]
pub(crate) struct MemoryAnchors {
    /// The opening anchor's ids at the prompt's last token, at most `k`.
    pub(crate) opening: Vec<u32>,
    /// The reply anchor's ids at the prompt's last token, at most `reply_turn`.
    pub(crate) reply: Vec<u32>,
    /// With the memory chain (claude-345): the turn's last ids, at most the chain's tail.
    pub(crate) tail: Vec<u32>,
    /// With the memory chain: the turn's first `reply_turn` ids, then the last `reply_turn` of
    /// the rest (the head's ids where the turn has no more).
    pub(crate) ends: Vec<u32>,
}

/// One row's memory move: the key formed and the id it named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MemoryMove {
    pub(crate) key: Vec<u32>,
    pub(crate) stored: Option<u32>,
    /// Whether the row was the reply's first (the opening table's row).
    pub(crate) opening: bool,
}

/// `w ⊙ (c − n)`'s exponent over the final norm's: the weight's `2^MEMORY_WEIGHT_EXP` and `n`'s
/// `2^NORM_FINE_EXP` make the product `2^-48`, 24 bits below `n`'s grid.
const MEMORY_WEIGHT_DROP: u32 = (-MEMORY_WEIGHT_EXP) as u32;
/// The exponent of the artifact's writer table (the stack engine's Q12 points).
const WRITER_EXP: i32 = -12;
/// The writer table's Q12 point raised to the final norm's exponent.
const MEMORY_POINT_RAISE: i32 = WRITER_EXP - NORM_FINE_EXP;

impl ServedTurnMemory {
    /// The served tables of a checked image, with the container's memory chain record.
    pub(crate) fn new(image: MemoryImage, chain: Option<&StackMemoryChain>) -> Self {
        Self {
            record: image.record.clone(),
            k: image.record.k,
            key_window: image.record.key_window,
            reply_turn: image.record.reply_turn,
            chain_tail: chain.map(|c| c.tail),
            opening: ServedMemoryTable::new(&image.opening),
            reply: image.reply.as_ref().map(ServedMemoryTable::new),
            point: image.point.map(ServedPointKey::new),
        }
    }

    /// The number of opening keys the artifact carries.
    pub(crate) fn opening_len(&self) -> usize {
        self.opening.len()
    }

    /// The number of reply keys the artifact carries, or `None` without a reply table.
    pub(crate) fn reply_len(&self) -> Option<usize> {
        self.reply.as_ref().map(ServedMemoryTable::len)
    }

    /// The start of the `key_window`-token block `row` lies in: training's blocks, counted from
    /// the conversation's first token (`row / key_window * key_window`).
    fn block_start(&self, row: usize) -> usize {
        row / self.key_window * self.key_window
    }

    /// The turn anchor's ids at `at`: the first `limit` ids after the last `<|bos|>` or
    /// `<|eos|>` at or before `at`, within `at`'s key block. A `<|bos|>` or `<|eos|>` at `at`
    /// itself reads none, as `opening_slots` gives it no slot.
    fn turn_ids(&self, tokens: &[u32], at: usize, limit: usize) -> Vec<u32> {
        let block_start = self.block_start(at);
        let mut start = block_start;
        for i in (block_start..=at).rev() {
            let id = tokens[i];
            if id == MEMORY_BOS_ID || id == MEMORY_EOS_ID {
                start = i + 1;
                break;
            }
        }
        if start > at {
            return Vec::new();
        }
        tokens[start..=at].iter().copied().take(limit).collect()
    }

    /// The opening table's key at the row of the prompt's last token `at`: the anchor's first
    /// `k` ids (`[]` where the anchor has none, which no table holds).
    pub(crate) fn opening_key(&self, tokens: &[u32], at: usize) -> Vec<u32> {
        self.turn_ids(tokens, at, self.k)
    }

    /// The reply table's key at a reply row `row`: the turn's ids, `MAX`, the reply length the
    /// row's key window shows, `MAX`, the row's id. The turn is `[]` once the prompt has left the
    /// row's block, and the length is the reply tokens the block shows, as training's windows.
    pub(crate) fn reply_key(
        &self,
        tokens: &[u32],
        row: usize,
        reply_start: usize,
        turn: &[u32],
    ) -> Vec<u32> {
        self.chain_key(tokens, row, reply_start, &[], turn)
    }

    /// A reply key with `lead` before the turn's ids (`[MEMORY_PAD]` for the memory chain's tail
    /// key, `[]` for the head and ends keys): `lead`, the turn's ids while the prompt is in the
    /// row's block, `MAX`, the reply length, `MAX`, the row's id.
    fn chain_key(
        &self,
        tokens: &[u32],
        row: usize,
        reply_start: usize,
        lead: &[u32],
        turn: &[u32],
    ) -> Vec<u32> {
        let block_start = self.block_start(row);
        let prompt_last = reply_start.saturating_sub(1);
        let mut key = Vec::with_capacity(lead.len() + turn.len() + 4);
        key.extend_from_slice(lead);
        if reply_start > 0 && prompt_last >= block_start {
            key.extend_from_slice(turn);
        }
        let visible = block_start.max(reply_start);
        let length = row + 1 - visible;
        key.extend([MEMORY_PAD, length as u32, MEMORY_PAD, tokens[row]]);
        key
    }

    /// One row's memory move. `row` is the token index being stepped (its row predicts the next
    /// token), `reply_start` the index of the reply's first generated token, `anchors` the session's
    /// stored turn state, and `writer` the artifact's Q12 writer table. The opening table moves
    /// the reply's first step's row and the reply table each later row; elsewhere, and where the
    /// table does not hold the key, nothing moves. Returns the key formed and the id it named.
    #[inline(never)]
    pub(crate) fn apply(
        &self,
        tokens: &[u32],
        row: usize,
        reply_start: usize,
        anchors: &mut MemoryAnchors,
        writer: &[i16],
        n: &mut [i64],
    ) -> Option<MemoryMove> {
        if row + 1 == reply_start {
            anchors.opening = self.opening_key(tokens, row);
            anchors.reply = match self.reply_turn {
                Some(turn) => self.turn_ids(tokens, row, turn),
                None => Vec::new(),
            };
            (anchors.tail, anchors.ends) = match (self.chain_tail, self.reply_turn) {
                (Some(tail), Some(turn)) => {
                    let all = self.turn_ids(tokens, row, usize::MAX);
                    let n = all.len();
                    let (head, rest) = all.split_at(turn.min(n));
                    let mut ends = head.to_vec();
                    ends.extend_from_slice(&rest[rest.len() - turn.min(rest.len())..]);
                    (all[n - tail.min(n)..].to_vec(), ends)
                }
                _ => (Vec::new(), Vec::new()),
            };
            let key = anchors.opening.clone();
            let stored = self.opening.stored(&key);
            if let Some(id) = stored {
                self.move_point(&self.opening, id, writer, n);
            }
            return Some(MemoryMove {
                key,
                stored,
                opening: true,
            });
        }
        if row < reply_start {
            return None;
        }
        let reply = self.reply.as_ref()?;
        let head = self.reply_key(tokens, row, reply_start, &anchors.reply);
        let (key, stored) = self.chain_lookup(reply, tokens, row, reply_start, anchors, head);
        if let Some(id) = stored {
            self.move_point(reply, id, writer, n);
        }
        Some(MemoryMove {
            key,
            stored,
            opening: false,
        })
    }

    /// The reply row's key and the id it names: with the memory chain (claude-345), the first of
    /// the ends, tail and head keys the table stores, or the head key where none is; without it,
    /// the head key alone.
    fn chain_lookup(
        &self,
        reply: &ServedMemoryTable,
        tokens: &[u32],
        row: usize,
        reply_start: usize,
        anchors: &MemoryAnchors,
        head: Vec<u32>,
    ) -> (Vec<u32>, Option<u32>) {
        if self.chain_tail.is_some() {
            let ends = self.chain_key(tokens, row, reply_start, &[], &anchors.ends);
            let tail = self.chain_key(tokens, row, reply_start, &[MEMORY_PAD], &anchors.tail);
            for key in [ends, tail] {
                if let Some(id) = reply.stored(&key) {
                    return (key, Some(id));
                }
            }
        }
        let stored = reply.stored(&head);
        (head, stored)
    }

    /// The move `n + w ⊙ (c − n)` at a hit, `c` the stored token's writer-table row at
    /// `NORM_FINE_EXP`: each coordinate's whole sum is formed exactly (i128) and rounded once to
    /// `n`'s grid, half away from zero, as the float side's `apply_readout` leaves the moved
    /// point to be rounded by its readout.
    #[inline(never)]
    fn move_point(&self, table: &ServedMemoryTable, stored: u32, writer: &[i16], n: &mut [i64]) {
        let width = n.len();
        let at = stored as usize * width;
        let Some(row) = writer.get(at..at + width) else {
            return;
        };
        for ((slot, &target), &weight) in n.iter_mut().zip(row).zip(&table.weight) {
            let point = i64::from(target) << MEMORY_POINT_RAISE;
            let sum = (i128::from(*slot) << MEMORY_WEIGHT_DROP)
                + i128::from(weight) * i128::from(point - *slot);
            let moved = round_shift(sum, MEMORY_WEIGHT_DROP);
            *slot = moved.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
        }
    }
}
