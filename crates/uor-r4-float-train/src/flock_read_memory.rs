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
/// Pads a saved key shorter than `k`; never a token id.
const PAD: u32 = u32::MAX;

thread_local! {
    static ACTIVE: RefCell<Option<TurnMemory>> = const { RefCell::new(None) };
    static ROWS: RefCell<Option<Vec<usize>>> = const { RefCell::new(None) };
}

/// Make `memory` the turn memory read on this thread, or none.
pub fn set_turn_memory(memory: Option<TurnMemory>) {
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
        })
    }

    pub fn k(&self) -> usize {
        self.k
    }

    /// How many keys the table holds.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// The host side of [`apply_readout`](Self::apply_readout) for windows `ids` of `time`
    /// positions: for every `batch · time` row, the stored token (0 where none) and a 0/1 hit,
    /// set only at the opening `rows` whose key the table holds.
    pub fn stored(&self, ids: &[u32], time: usize, rows: &[usize]) -> Result<(Vec<u32>, Vec<f32>)> {
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
        let (mut stored, mut hit) = (vec![0u32; total], vec![0f32; total]);
        for (b, offsets) in by_window {
            let window = &ids[b * time..(b + 1) * time];
            let slots = opening_slots(window, self.k, OpeningAnchor::Turn);
            for t in offsets {
                let key: Vec<u32> = slots[t].iter().map(|&s| window[s]).collect();
                if let Some(&token) = self.table.get(&key) {
                    stored[b * time + t] = token;
                    hit[b * time + t] = 1.0;
                }
            }
        }
        Ok((stored, hit))
    }

    /// The memory read on the final norm's output `(batch, time, width)` of the windows `ids`,
    /// with token points from the shared `table`: at the opening `rows` whose key the table
    /// holds, `n + w ⊙ (c − n)`; elsewhere `n`. Returns the same shape.
    pub fn apply_readout(
        &self,
        output: &Tensor,
        ids: &[u32],
        rows: &[usize],
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
        Ok(points.add(&pull)?.reshape((batch, time, width))?)
    }

    /// Every learned parameter, in a fixed order for the optimizer.
    pub fn variables(&self) -> Vec<&Var> {
        vec![&self.weight]
    }

    /// The mean absolute value of the weight, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        Ok(self.weight.as_tensor().abs()?.mean_all()?)
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
        Ok(Some(memory))
    }
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
            m.apply_readout(&output, &ids, &rows, &table)
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
            .apply_readout(&output, &ids, &rows, &table)
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
