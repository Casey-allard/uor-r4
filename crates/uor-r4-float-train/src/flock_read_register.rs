//! The prompt register (in-space-trunk-to-text, claude-155).
//!
//! For each position `t` of a window, the register keeps `q` slots, each a weighted mean over
//! positions of the current turn's span `[a, t]` (the span starts after the last `<|bos|>` or
//! `<|eos|>` at or before `t`, the opening read's turn anchor; a `<|bos|>` or `<|eos|>` reads
//! nothing):
//! 1. **Keys.** Each lane of the stream is turned by a learned quaternion (the Hamilton product,
//!    rotation on the left, as in the opening read); the real part of each lane gives a
//!    straight-through ±1 code, one bit per lane.
//! 2. **Queries.** Each slot has a learned ±1 code over the lanes (straight-through sign).
//! 3. **Learned select.** Each slot picks the `picks` span positions nearest its query in Hamming
//!    distance, ranked on the host from detached distances (ties to the earlier position), and
//!    weighs each pick `relu(m − d) / max(m, 1)` with the distance recomputed on the device, so the
//!    weights carry gradient. The slot is the weighted mean of the picks' turned values, or zero
//!    when no pick is inside the margin.
//! 4. **All select** (one slot): the uniform mean of the turned values over the whole span, from
//!    a running sum along time built by a doubling scan.
//!
//! The stream and the readout each add `Σ_q W_q ⊙ S_q` with their own weight, starting at 0, so
//! at step 0 nothing changes. Only per-lane quaternion products, sums of squared differences of
//! ±1 codes, sums and elementwise weights; nothing is exponential and there is no matrix product.
//!
//! The register is active for the thread that sets it ([`set_prompt_register`]); the slots the
//! stream computed are kept for the readout of the same forward ([`set_register_slots`]).
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};

use crate::flock_read_hamming::hamming_bits;
use crate::flock_read_opening::SHEET_EOS_ID;
use crate::hamilton_sheet::SHEET_BOS_ID;
use crate::{invalid, Result};

/// The most slots a register may keep.
const MAX_SLOTS: usize = 16;
/// The most picks a slot may weigh.
const MAX_PICKS: usize = 32;
const FILE: &str = "prompt_register.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<PromptRegister>> = const { RefCell::new(None) };
    static SLOTS: RefCell<Option<Tensor>> = const { RefCell::new(None) };
}

/// Make `register` the prompt register applied on this thread, or none.
pub fn set_prompt_register(register: Option<PromptRegister>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = register);
}

/// The prompt register applied on this thread, if set.
pub fn prompt_register() -> Option<PromptRegister> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// Keep the slots the stream computed, for the readout of the same forward.
pub fn set_register_slots(slots: Option<Tensor>) {
    SLOTS.with(|cell| *cell.borrow_mut() = slots);
}

/// The slots kept by [`set_register_slots`], taken (the store is left empty).
pub fn take_register_slots() -> Option<Tensor> {
    SLOTS.with(|cell| cell.borrow_mut().take())
}

/// How a slot chooses the span positions it reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegisterSelect {
    /// The `picks` positions nearest the slot's query, weighed by the margin.
    #[default]
    Learned,
    /// Every position of the span, uniformly (one slot only).
    All,
}

impl RegisterSelect {
    fn code(self) -> u32 {
        match self {
            Self::Learned => 0,
            Self::All => 1,
        }
    }

    fn from_code(code: u32) -> Result<Self> {
        match code {
            0 => Ok(Self::Learned),
            1 => Ok(Self::All),
            _ => Err(invalid(format!(
                "the saved register select {code} is unknown"
            ))),
        }
    }
}

/// The span of each position `t` of one window row `ids`: `Some((a, t))` with `a` the first
/// position after the last `<|bos|>` or `<|eos|>` at or before `t` (or 0 when there is none),
/// and `None` when `ids[t]` is itself `<|bos|>` or `<|eos|>` (the opening read's turn rule).
pub fn register_spans(ids: &[u32]) -> Vec<Option<(usize, usize)>> {
    let mut start = 0;
    ids.iter()
        .enumerate()
        .map(|(t, &id)| {
            if id == SHEET_BOS_ID || id == SHEET_EOS_ID {
                start = t + 1;
                None
            } else {
                Some((start, t))
            }
        })
        .collect()
}

/// For one slot over one row, with `dist[s]` the slot's distance to position `s`: per `t`, the at
/// most `k` positions of `spans[t]` with the smallest distance, ties to the earlier position, in
/// that order; none where the span is `None`.
pub fn register_picks(dist: &[u32], spans: &[Option<(usize, usize)>], k: usize) -> Vec<Vec<usize>> {
    let rank = |s: usize| (dist[s], s);
    let mut best: Vec<usize> = Vec::new();
    let mut previous: Option<(usize, usize)> = None;
    spans
        .iter()
        .map(|span| {
            let Some((a, b)) = *span else {
                previous = None;
                return Vec::new();
            };
            if previous.is_some_and(|(pa, pb)| pa == a && pb + 1 == b) {
                // The span grew by `b` alone: the nearest k of the span are the nearest k of the
                // previous nearest k and `b`, which ranks after every equal distance.
                let at = best.partition_point(|&s| rank(s) < rank(b));
                best.insert(at, b);
                best.truncate(k);
            } else {
                best = (a..=b).collect();
                best.sort_by_key(|&s| rank(s));
                best.truncate(k);
            }
            previous = Some((a, b));
            best.clone()
        })
        .collect()
}

/// One prompt register: `q` slots, each with a query code, a margin and a value rotation, over a
/// shared key rotation, and the per-slot weights of what the stream and the readout add.
#[derive(Clone)]
pub struct PromptRegister {
    q: usize,
    picks: usize,
    select: RegisterSelect,
    /// `(lanes, 4)`.
    pub(crate) key_rotation: Var,
    /// `(q, lanes)`.
    pub(crate) queries: Var,
    /// `(q,)`.
    pub(crate) margins: Var,
    /// `(q, lanes, 4)`.
    pub(crate) value_rotation: Var,
    /// `U`, `(q, width)`.
    pub(crate) stream_weight: Var,
    /// `V`, `(q, width)`.
    pub(crate) readout_weight: Var,
}

fn check_settings(q: usize, picks: usize, select: RegisterSelect, width: usize) -> Result<()> {
    if !(1..=MAX_SLOTS).contains(&q) {
        return Err(invalid(format!(
            "a prompt register keeps 1 to {MAX_SLOTS} slots, not {q}"
        )));
    }
    if !(1..=MAX_PICKS).contains(&picks) {
        return Err(invalid(format!(
            "a prompt register slot weighs 1 to {MAX_PICKS} picks, not {picks}"
        )));
    }
    if width == 0 || !width.is_multiple_of(4) {
        return Err(invalid("a prompt register needs whole 4-lane blocks"));
    }
    if select == RegisterSelect::All && q != 1 {
        return Err(invalid("the all-span register keeps exactly one slot"));
    }
    Ok(())
}

impl PromptRegister {
    /// A register of `q` slots of `picks` picks on a stream of `width` coordinates. Nothing is
    /// random: identity rotations, margins `lanes / 2`, zero weights (so it starts as no read),
    /// and slot `j`'s query on lane `l` is `+1` where `((l·(2j + 1) + j) / 3)` is even and `−1`
    /// elsewhere. Known flaw (claude-155 REPORT): the pattern repeats every 6 lanes, so slot
    /// `j + 6` starts equal to slot `j` and slot `j + 3` opposite to it; only 4 starts are
    /// distinct, and equal slots train as copies. A rerun of the learned select needs a start rule
    /// that is distinct for every slot, with a test.
    pub fn new(
        q: usize,
        picks: usize,
        select: RegisterSelect,
        width: usize,
        device: &Device,
    ) -> Result<Self> {
        check_settings(q, picks, select, width)?;
        let lanes = width / 4;
        let identity =
            |n: usize| -> Vec<f32> { (0..n).flat_map(|_| [1.0f32, 0.0, 0.0, 0.0]).collect() };
        let queries: Vec<f32> = (0..q)
            .flat_map(|j| {
                (0..lanes).map(move |l| {
                    if ((l * (2 * j + 1) + j) / 3) % 2 == 0 {
                        1.0
                    } else {
                        -1.0
                    }
                })
            })
            .collect();
        let var = |values: Vec<f32>, shape: &[usize]| -> Result<Var> {
            Ok(Var::from_tensor(&Tensor::from_vec(values, shape, device)?)?)
        };
        Ok(Self {
            q,
            picks,
            select,
            key_rotation: var(identity(lanes), &[lanes, 4])?,
            queries: var(queries, &[q, lanes])?,
            margins: var(vec![(lanes / 2) as f32; q], &[q])?,
            value_rotation: var(identity(q * lanes), &[q, lanes, 4])?,
            stream_weight: Var::zeros((q, width), DType::F32, device)?,
            readout_weight: Var::zeros((q, width), DType::F32, device)?,
        })
    }

    pub fn q(&self) -> usize {
        self.q
    }

    pub fn picks(&self) -> usize {
        self.picks
    }

    pub fn select(&self) -> RegisterSelect {
        self.select
    }

    fn width(&self) -> Result<usize> {
        Ok(self.stream_weight.dim(1)?)
    }

    /// The slots of `x` `(batch, time, width)` for the windows `ids`: `(batch, time, q, width)`.
    pub fn slots(&self, x: &Tensor, ids: &[u32]) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        let rows = batch * time;
        if ids.len() != rows {
            return Err(invalid("the prompt register needs whole windows"));
        }
        if width != self.width()? {
            return Err(invalid(
                "the prompt register's width differs from the stream's",
            ));
        }
        let device = x.device();
        let lanes = width / 4;
        let points = x.reshape((rows, lanes, 4))?;
        let turned_value = |j: usize| -> Result<Tensor> {
            let rotation = self.value_rotation.as_tensor().get(j)?.unsqueeze(0)?;
            Ok(hamilton_product(&rotation, &points)?.reshape((rows, width))?)
        };
        let slots = match self.select {
            RegisterSelect::Learned => {
                let rotation = self.key_rotation.as_tensor().unsqueeze(0)?;
                let key = sign_code(
                    &hamilton_product(&rotation, &points)?
                        .narrow(2, 0, 1)?
                        .reshape((rows, lanes))?,
                )?;
                let query = sign_code(self.queries.as_tensor())?;
                // Every slot's distance to every position, detached and on the host: `(rows, q)`.
                let distance =
                    hamming_bits(&query.detach().unsqueeze(0)?, &key.detach().unsqueeze(1)?)?
                        .to_vec2::<f32>()?;
                let (index, valid) = self.pick_tables(&distance, ids, batch, time);
                let per_slot = rows * self.picks;
                let mut slots = Vec::with_capacity(self.q);
                for j in 0..self.q {
                    let range = j * per_slot..(j + 1) * per_slot;
                    let index = Tensor::from_vec(index[range.clone()].to_vec(), per_slot, device)?;
                    let valid =
                        Tensor::from_vec(valid[range].to_vec(), (rows, self.picks), device)?;
                    let picked = key
                        .index_select(&index, 0)?
                        .reshape((rows, self.picks, lanes))?;
                    let code = query.narrow(0, j, 1)?.reshape((1, 1, lanes))?;
                    let d = hamming_bits(&code, &picked)?;
                    let margin = self.margins.as_tensor().narrow(0, j, 1)?.reshape((1, 1))?;
                    let weight = margin
                        .broadcast_sub(&d)?
                        .relu()?
                        .broadcast_div(&margin.maximum(1.0)?)?
                        .mul(&valid)?;
                    let values = turned_value(j)?
                        .index_select(&index, 0)?
                        .reshape((rows, self.picks, width))?;
                    let sum = values.broadcast_mul(&weight.unsqueeze(2)?)?.sum(1)?;
                    let total = weight.sum_keepdim(1)?;
                    // Zero where no pick is inside the margin: the zero sum over 1.
                    let total = total.gt(0.0)?.where_cond(&total, &total.ones_like()?)?;
                    slots.push(sum.broadcast_div(&total)?);
                }
                Tensor::stack(&slots, 1)?
            }
            RegisterSelect::All => {
                let values = turned_value(0)?.reshape((batch, time, width))?;
                // The running sum along time by doubling: after step i every position holds the
                // sum of the 2^(i+1) positions ending at it.
                let mut prefix = values;
                let mut shift = 1;
                while shift < time {
                    let shifted = Tensor::cat(
                        &[
                            &Tensor::zeros((batch, shift, width), DType::F32, device)?,
                            &prefix.narrow(1, 0, time - shift)?,
                        ],
                        1,
                    )?;
                    prefix = prefix.add(&shifted)?;
                    shift *= 2;
                }
                let prefix = prefix.reshape((rows, width))?;
                let (high, high_mask, low, low_mask, count) = span_tables(ids, batch, time);
                let at = |index: Vec<u32>, mask: Vec<f32>| -> Result<Tensor> {
                    Ok(prefix
                        .index_select(&Tensor::from_vec(index, rows, device)?, 0)?
                        .broadcast_mul(&Tensor::from_vec(mask, (rows, 1), device)?)?)
                };
                at(high, high_mask)?
                    .sub(&at(low, low_mask)?)?
                    .broadcast_div(&Tensor::from_vec(count, (rows, 1), device)?)?
                    .unsqueeze(1)?
            }
        };
        Ok(slots.reshape((batch, time, self.q, width))?)
    }

    /// The host side of the learned select: per slot, row and pick, the flat row index of the
    /// picked position and a 0/1 validity (a short span leaves picks unused, pointing at the
    /// window's first position).
    fn pick_tables(
        &self,
        distance: &[Vec<f32>],
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> (Vec<u32>, Vec<f32>) {
        let rows = batch * time;
        let mut index = vec![0u32; self.q * rows * self.picks];
        let mut valid = vec![0f32; self.q * rows * self.picks];
        for b in 0..batch {
            let base = b * time;
            let spans = register_spans(&ids[base..base + time]);
            for j in 0..self.q {
                let dist: Vec<u32> = (0..time)
                    .map(|s| distance[base + s][j].round() as u32)
                    .collect();
                for (t, chosen) in register_picks(&dist, &spans, self.picks).iter().enumerate() {
                    for p in 0..self.picks {
                        let at = (j * rows + base + t) * self.picks + p;
                        index[at] = base as u32;
                        if let Some(&s) = chosen.get(p) {
                            index[at] = (base + s) as u32;
                            valid[at] = 1.0;
                        }
                    }
                }
            }
        }
        (index, valid)
    }

    /// `x + Σ_q U_q ⊙ S_q`.
    pub fn apply_stream(&self, x: &Tensor, slots: &Tensor) -> Result<Tensor> {
        self.add(&self.stream_weight, x, slots)
    }

    /// `n + Σ_q V_q ⊙ S_q`, `n` the same shape as the stream.
    pub fn apply_readout(&self, n: &Tensor, slots: &Tensor) -> Result<Tensor> {
        self.add(&self.readout_weight, n, slots)
    }

    fn add(&self, weight: &Var, base: &Tensor, slots: &Tensor) -> Result<Tensor> {
        let width = self.width()?;
        let weight = weight.as_tensor().reshape((1, 1, self.q, width))?;
        Ok(base.add(&slots.broadcast_mul(&weight)?.sum(2)?)?)
    }

    /// A copy with `U_q` and `V_q` zeroed for `slot`, or for every slot when `None`. The copy
    /// shares the rotations, queries and margins with `self`; its weights are its own.
    pub fn without_slot(&self, slot: Option<usize>) -> Result<Self> {
        if slot.is_some_and(|j| j >= self.q) {
            return Err(invalid("the prompt register has no such slot"));
        }
        let zeroed = |weight: &Var| -> Result<Var> {
            let device = weight.device();
            let mut rows = weight.as_tensor().to_vec2::<f32>()?;
            for (j, row) in rows.iter_mut().enumerate() {
                if slot.is_none_or(|k| k == j) {
                    row.iter_mut().for_each(|v| *v = 0.0);
                }
            }
            let shape = weight.dims().to_vec();
            let flat: Vec<f32> = rows.into_iter().flatten().collect();
            Ok(Var::from_tensor(&Tensor::from_vec(flat, shape, device)?)?)
        };
        Ok(Self {
            stream_weight: zeroed(&self.stream_weight)?,
            readout_weight: zeroed(&self.readout_weight)?,
            ..self.clone()
        })
    }

    /// Every learned parameter, in a fixed order for the optimizer.
    pub fn variables(&self) -> Vec<&Var> {
        vec![
            &self.key_rotation,
            &self.queries,
            &self.margins,
            &self.value_rotation,
            &self.stream_weight,
            &self.readout_weight,
        ]
    }

    /// The mean absolute value of `U` and `V` together, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        Ok(Tensor::cat(
            &[
                self.stream_weight.as_tensor().flatten_all()?,
                self.readout_weight.as_tensor().flatten_all()?,
            ],
            0,
        )?
        .abs()?
        .mean_all()?)
    }

    fn tensors(&self) -> Result<HashMap<String, Tensor>> {
        let device = self.stream_weight.device();
        let mut tensors = HashMap::new();
        for (name, var) in [
            ("key_rotation", &self.key_rotation),
            ("queries", &self.queries),
            ("margins", &self.margins),
            ("value_rotation", &self.value_rotation),
            ("stream_weight", &self.stream_weight),
            ("readout_weight", &self.readout_weight),
        ] {
            tensors.insert(name.to_owned(), var.as_tensor().clone());
        }
        for (name, value) in [
            ("q", self.q as u32),
            ("picks", self.picks as u32),
            ("select", self.select.code()),
        ] {
            tensors.insert(name.to_owned(), Tensor::new(&[value], device)?);
        }
        Ok(tensors)
    }

    /// Write the register to `directory/prompt_register.safetensors`, with its `q`, `picks` and
    /// select.
    pub fn save(&self, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        candle_core::safetensors::save(&self.tensors()?, directory.join(FILE))?;
        Ok(())
    }

    /// The register saved in `directory`, or none when the checkpoint has none. A file whose
    /// settings are out of range or whose tensors do not match them is refused.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<&Tensor> {
            tensors
                .get(name)
                .ok_or_else(|| invalid(format!("the saved prompt register has no {name}")))
        };
        let scalar = |name: &str| -> Result<u32> {
            get(name)?
                .to_vec1::<u32>()?
                .first()
                .copied()
                .ok_or_else(|| invalid(format!("the saved prompt register's {name} is empty")))
        };
        let var = |name: &str| -> Result<Var> { Ok(Var::from_tensor(get(name)?)?) };
        let register = Self {
            q: scalar("q")? as usize,
            picks: scalar("picks")? as usize,
            select: RegisterSelect::from_code(scalar("select")?)?,
            key_rotation: var("key_rotation")?,
            queries: var("queries")?,
            margins: var("margins")?,
            value_rotation: var("value_rotation")?,
            stream_weight: var("stream_weight")?,
            readout_weight: var("readout_weight")?,
        };
        let width = match register.stream_weight.dims() {
            [_, width] => *width,
            _ => 0,
        };
        check_settings(register.q, register.picks, register.select, width)?;
        let (q, lanes) = (register.q, width / 4);
        let shapes: [(&Var, Vec<usize>); 6] = [
            (&register.key_rotation, vec![lanes, 4]),
            (&register.queries, vec![q, lanes]),
            (&register.margins, vec![q]),
            (&register.value_rotation, vec![q, lanes, 4]),
            (&register.stream_weight, vec![q, width]),
            (&register.readout_weight, vec![q, width]),
        ];
        if shapes
            .iter()
            .any(|(var, shape)| var.dims() != shape.as_slice())
        {
            return Err(invalid(
                "the saved prompt register's tensors do not match its settings",
            ));
        }
        Ok(Some(register))
    }
}

/// The host side of the all-span select for `batch` windows of `time` positions: per row, the
/// prefix row ending the span and its 0/1 mask, the prefix row just before the span and its mask
/// (0 when the span starts the window), and the span's length (1 for an empty span).
fn span_tables(
    ids: &[u32],
    batch: usize,
    time: usize,
) -> (Vec<u32>, Vec<f32>, Vec<u32>, Vec<f32>, Vec<f32>) {
    let rows = batch * time;
    let (mut high, mut high_mask) = (vec![0u32; rows], vec![0f32; rows]);
    let (mut low, mut low_mask) = (vec![0u32; rows], vec![0f32; rows]);
    let mut count = vec![1f32; rows];
    for b in 0..batch {
        let base = b * time;
        for (t, span) in register_spans(&ids[base..base + time]).iter().enumerate() {
            let r = base + t;
            high[r] = r as u32;
            low[r] = r as u32;
            if let Some((a, t)) = *span {
                high_mask[r] = 1.0;
                if a > 0 {
                    low[r] = (base + a - 1) as u32;
                    low_mask[r] = 1.0;
                }
                count[r] = (t - a + 1) as f32;
            }
        }
    }
    (high, high_mask, low, low_mask, count)
}

/// ±1 by `p ≥ 0` forward, the identity backward (the Hamming read's straight-through code).
fn sign_code(p: &Tensor) -> Result<Tensor> {
    let bits = p.ge(0.0)?.to_dtype(DType::F32)?.affine(2.0, -1.0)?;
    Ok(p.add(&bits.sub(p)?.detach())?)
}

/// `p ⊗ q` lane by lane on `(…, lanes, 4)` tensors, `p` broadcast over `q`'s rows, the rotation
/// on the left (the opening read's product).
fn hamilton_product(p: &Tensor, q: &Tensor) -> Result<Tensor> {
    let part = |t: &Tensor, i: usize| t.narrow(2, i, 1);
    let (a1, b1, c1, d1) = (part(p, 0)?, part(p, 1)?, part(p, 2)?, part(p, 3)?);
    let (a2, b2, c2, d2) = (part(q, 0)?, part(q, 1)?, part(q, 2)?, part(q, 3)?);
    let m = |x: &Tensor, y: &Tensor| x.broadcast_mul(y);
    let a = m(&a1, &a2)?
        .sub(&m(&b1, &b2)?)?
        .sub(&m(&c1, &c2)?)?
        .sub(&m(&d1, &d2)?)?;
    let b = m(&a1, &b2)?
        .add(&m(&b1, &a2)?)?
        .add(&m(&c1, &d2)?)?
        .sub(&m(&d1, &c2)?)?;
    let c = m(&a1, &c2)?
        .sub(&m(&b1, &d2)?)?
        .add(&m(&c1, &a2)?)?
        .add(&m(&d1, &b2)?)?;
    let d = m(&a1, &d2)?
        .add(&m(&b1, &c2)?)?
        .sub(&m(&c1, &b2)?)?
        .add(&m(&d1, &a2)?)?;
    Ok(Tensor::cat(&[&a, &b, &c, &d], 2)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: usize = 8;

    const ROW: [u32; 13] = [0, 10, 11, 12, 20, 21, 1, 13, 14, 22, 1, 0, 0];
    /// A row that starts mid-prompt: no `<|bos|>` or `<|eos|>` before position 3.
    const MID: [u32; 13] = [30, 31, 32, 0, 33, 34, 35, 36, 1, 37, 38, 1, 0];

    fn scratch(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "uor-prompt-register-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    #[test]
    fn register_spans_follow_the_turn_anchor() {
        let s = |a: usize, t: usize| Some((a, t));
        assert_eq!(
            register_spans(&ROW),
            vec![
                None,
                s(1, 1),
                s(1, 2),
                s(1, 3),
                s(1, 4),
                s(1, 5),
                None,
                s(7, 7),
                s(7, 8),
                s(7, 9),
                None,
                None,
                None,
            ]
        );
        assert_eq!(
            register_spans(&MID),
            vec![
                s(0, 0),
                s(0, 1),
                s(0, 2),
                None,
                s(4, 4),
                s(4, 5),
                s(4, 6),
                s(4, 7),
                None,
                s(9, 9),
                s(9, 10),
                None,
                None,
            ]
        );
    }

    #[test]
    fn register_picks_take_the_nearest_k_of_the_span_with_ties_to_the_earlier() {
        // Position 0 and 6 (outside every span) are the nearest of all: a pick from the whole
        // row would take them.
        let dist = [0, 5, 2, 5, 2, 1, 0, 3, 3, 3, 0, 0, 0];
        let picks = register_picks(&dist, &register_spans(&ROW), 3);
        let expect: [&[usize]; 13] = [
            &[],
            &[1],
            &[2, 1],
            &[2, 1, 3],
            &[2, 4, 1],
            &[5, 2, 4],
            &[],
            &[7],
            &[7, 8],
            &[7, 8, 9],
            &[],
            &[],
            &[],
        ];
        for (t, e) in expect.iter().enumerate() {
            assert_eq!(picks[t], *e, "picks at t = {t}");
        }
        // A span handed alone is ranked whole, the same as when it grows one position at a time.
        let alone = register_picks(&dist, &[None, None, None, None, None, Some((1, 5))], 3);
        assert_eq!(alone[5], vec![5, 2, 4]);
    }

    #[test]
    fn new_refuses_out_of_range_settings() {
        let device = Device::Cpu;
        let new = |q, picks, select, width| PromptRegister::new(q, picks, select, width, &device);
        assert!(new(1, 1, RegisterSelect::Learned, 4).is_ok());
        assert!(new(16, 32, RegisterSelect::Learned, WIDTH).is_ok());
        assert!(new(1, 32, RegisterSelect::All, WIDTH).is_ok());
        assert!(new(0, 4, RegisterSelect::Learned, WIDTH).is_err());
        assert!(new(17, 4, RegisterSelect::Learned, WIDTH).is_err());
        assert!(new(2, 0, RegisterSelect::Learned, WIDTH).is_err());
        assert!(new(2, 33, RegisterSelect::Learned, WIDTH).is_err());
        assert!(new(2, 4, RegisterSelect::Learned, 0).is_err());
        assert!(new(2, 4, RegisterSelect::Learned, 6).is_err());
        assert!(new(2, 4, RegisterSelect::All, WIDTH).is_err());
    }

    #[test]
    fn the_register_saves_loads_and_refuses_a_bad_file() {
        let device = Device::Cpu;
        let directory = scratch("saves");
        assert!(PromptRegister::load(&directory, &device).unwrap().is_none());
        let register = PromptRegister::new(3, 5, RegisterSelect::Learned, WIDTH, &device).unwrap();
        let values = |n: usize, salt: usize| -> Vec<f32> {
            (0..n)
                .map(|i| ((i * 37 + salt * 11) % 17) as f32 * 0.1 - 0.8)
                .collect()
        };
        for (salt, var) in register.variables().into_iter().enumerate() {
            let shape = var.dims().to_vec();
            let n = shape.iter().product();
            var.set(&Tensor::from_vec(values(n, salt), shape, &device).unwrap())
                .unwrap();
        }
        register.save(&directory).unwrap();
        let loaded = PromptRegister::load(&directory, &device).unwrap().unwrap();
        assert_eq!(
            (loaded.q(), loaded.picks(), loaded.select()),
            (3, 5, RegisterSelect::Learned)
        );
        let flat = |v: &Var| {
            v.as_tensor()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap()
        };
        for (a, b) in loaded.variables().into_iter().zip(register.variables()) {
            assert_eq!(a.dims(), b.dims());
            assert_eq!(flat(a), flat(b));
        }
        // The same tensors with a setting out of range are refused.
        for (name, value) in [("q", 0u32), ("select", 1), ("picks", 33), ("select", 7)] {
            let mut tensors = register.tensors().unwrap();
            tensors.insert(name.to_owned(), Tensor::new(&[value], &device).unwrap());
            candle_core::safetensors::save(&tensors, directory.join(FILE)).unwrap();
            assert!(
                PromptRegister::load(&directory, &device).is_err(),
                "{name} = {value} is loaded"
            );
        }
        // A tensor that does not match the settings is refused.
        let mut tensors = register.tensors().unwrap();
        tensors.insert(
            "margins".to_owned(),
            Tensor::from_vec(values(2, 1), 2, &device).unwrap(),
        );
        candle_core::safetensors::save(&tensors, directory.join(FILE)).unwrap();
        assert!(PromptRegister::load(&directory, &device).is_err());
        std::fs::remove_file(directory.join(FILE)).unwrap();
        std::fs::remove_dir(&directory).unwrap();
    }

    // The vendored candle refuses tensor compute on the CPU (D24/D29), so the tests that run the
    // register are CUDA tests.
    #[cfg(feature = "cuda")]
    mod cuda {
        use super::*;

        const TIME: usize = 13;
        const LANES: usize = WIDTH / 4;

        fn values(n: usize, salt: usize) -> Vec<f32> {
            (0..n)
                .map(|i| ((i * 37 + salt * 11) % 17) as f32 * 0.1 - 0.8)
                .collect()
        }

        fn ids() -> Vec<u32> {
            ROW.iter().chain(MID.iter()).copied().collect()
        }

        fn set(var: &Var, values: Vec<f32>, device: &Device) {
            let shape = var.dims().to_vec();
            var.set(&Tensor::from_vec(values, shape, device).unwrap())
                .unwrap();
        }

        fn quaternion(p: [f32; 4], q: [f32; 4]) -> [f32; 4] {
            let ([a1, b1, c1, d1], [a2, b2, c2, d2]) = (p, q);
            [
                a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
                a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2,
                a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2,
                a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2,
            ]
        }

        /// `point` turned lane by lane by the `lanes · 4` quaternions of `rotation`.
        fn turn(rotation: &[f32], point: &[f32]) -> Vec<f32> {
            (0..LANES)
                .flat_map(|l| {
                    let r = [0, 1, 2, 3].map(|i| rotation[l * 4 + i]);
                    let p = [0, 1, 2, 3].map(|i| point[l * 4 + i]);
                    quaternion(r, p)
                })
                .collect()
        }

        fn added(register: &PromptRegister, x: &Tensor) -> Vec<Vec<Vec<f32>>> {
            let slots = register.slots(x, &ids()).unwrap();
            register
                .apply_stream(x, &slots)
                .unwrap()
                .sub(x)
                .unwrap()
                .to_vec3::<f32>()
                .unwrap()
        }

        #[test]
        fn zero_weights_change_neither_the_stream_nor_the_readout() {
            let device = Device::new_cuda(0).unwrap();
            let x =
                Tensor::from_vec(values(2 * TIME * WIDTH, 5), (2, TIME, WIDTH), &device).unwrap();
            let n =
                Tensor::from_vec(values(2 * TIME * WIDTH, 3), (2, TIME, WIDTH), &device).unwrap();
            let flat = |t: &Tensor| t.flatten_all().unwrap().to_vec1::<f32>().unwrap();
            for (q, select) in [(3, RegisterSelect::Learned), (1, RegisterSelect::All)] {
                let register = PromptRegister::new(q, 4, select, WIDTH, &device).unwrap();
                let slots = register.slots(&x, &ids()).unwrap();
                assert_eq!(slots.dims(), &[2, TIME, q, WIDTH]);
                assert_eq!(flat(&register.apply_stream(&x, &slots).unwrap()), flat(&x));
                assert_eq!(flat(&register.apply_readout(&n, &slots).unwrap()), flat(&n));
            }
        }

        #[test]
        fn one_slot_is_the_weighted_mean_of_its_picks() {
            let device = Device::new_cuda(0).unwrap();
            let (q, picks) = (2, 3);
            let register =
                PromptRegister::new(q, picks, RegisterSelect::Learned, WIDTH, &device).unwrap();
            // General quaternions, so every Hamilton term counts; margins that let distance 1
            // and 2 weigh in.
            let key_rotation = values(LANES * 4, 4);
            let queries = values(q * LANES, 6);
            let margins = [2.5f32, 1.5];
            let value_rotation = values(q * LANES * 4, 2);
            set(&register.key_rotation, key_rotation.clone(), &device);
            set(&register.queries, queries.clone(), &device);
            set(&register.margins, margins.to_vec(), &device);
            set(&register.value_rotation, value_rotation.clone(), &device);
            let xs = values(2 * TIME * WIDTH, 7);
            let x = Tensor::from_vec(xs.clone(), (2, TIME, WIDTH), &device).unwrap();
            let ids = ids();
            for j in 0..q {
                let mut u = vec![0.0f32; q * WIDTH];
                u[j * WIDTH..(j + 1) * WIDTH].fill(1.0);
                set(&register.stream_weight, u, &device);
                let added = added(&register, &x);
                let sign = |v: f32| if v >= 0.0 { 1.0f32 } else { -1.0 };
                let code: Vec<f32> = (0..LANES).map(|l| sign(queries[j * LANES + l])).collect();
                let rotation = &value_rotation[j * LANES * 4..(j + 1) * LANES * 4];
                for b in 0..2 {
                    let point = |s: usize| &xs[(b * TIME + s) * WIDTH..(b * TIME + s + 1) * WIDTH];
                    let dist: Vec<u32> = (0..TIME)
                        .map(|s| {
                            let turned = turn(&key_rotation, point(s));
                            (0..LANES)
                                .map(|l| (code[l] - sign(turned[l * 4])).powi(2) / 4.0)
                                .sum::<f32>() as u32
                        })
                        .collect();
                    let spans = register_spans(&ids[b * TIME..(b + 1) * TIME]);
                    for (t, chosen) in register_picks(&dist, &spans, picks).iter().enumerate() {
                        let mut sum = vec![0.0f32; WIDTH];
                        let mut total = 0.0f32;
                        for &s in chosen {
                            let w = (margins[j] - dist[s] as f32).max(0.0) / margins[j].max(1.0);
                            for (acc, v) in sum.iter_mut().zip(turn(rotation, point(s))) {
                                *acc += w * v;
                            }
                            total += w;
                        }
                        for (a, e) in added[b][t].iter().zip(&sum) {
                            let e = if total > 0.0 { e / total } else { 0.0 };
                            assert!(
                                (a - e).abs() < 1e-5,
                                "slot {j} b {b} t {t}: added {:?}, sum {sum:?} over {total}",
                                added[b][t]
                            );
                        }
                    }
                }
            }
        }

        #[test]
        fn the_bag_weighs_the_whole_span_uniformly() {
            let device = Device::new_cuda(0).unwrap();
            let register = PromptRegister::new(1, 1, RegisterSelect::All, WIDTH, &device).unwrap();
            let rotation = values(LANES * 4, 2);
            set(&register.value_rotation, rotation.clone(), &device);
            set(&register.stream_weight, vec![1.0; WIDTH], &device);
            let xs = values(2 * TIME * WIDTH, 7);
            let x = Tensor::from_vec(xs.clone(), (2, TIME, WIDTH), &device).unwrap();
            let added = added(&register, &x);
            let ids = ids();
            for b in 0..2 {
                for (t, span) in register_spans(&ids[b * TIME..(b + 1) * TIME])
                    .iter()
                    .enumerate()
                {
                    let mut expect = vec![0.0f32; WIDTH];
                    if let Some((a, t)) = *span {
                        for s in a..=t {
                            let r = b * TIME + s;
                            for (e, v) in expect
                                .iter_mut()
                                .zip(turn(&rotation, &xs[r * WIDTH..(r + 1) * WIDTH]))
                            {
                                *e += v / (t - a + 1) as f32;
                            }
                        }
                    }
                    for (a, e) in added[b][t].iter().zip(&expect) {
                        assert!(
                            (a - e).abs() < 1e-4,
                            "b {b} t {t}: added {:?}, expected {expect:?}",
                            added[b][t]
                        );
                    }
                }
            }
        }

        #[test]
        fn gradients_reach_every_learned_part() {
            let device = Device::new_cuda(0).unwrap();
            let register =
                PromptRegister::new(2, 3, RegisterSelect::Learned, WIDTH, &device).unwrap();
            set(&register.stream_weight, values(2 * WIDTH, 1), &device);
            set(&register.readout_weight, values(2 * WIDTH, 8), &device);
            let x =
                Tensor::from_vec(values(2 * TIME * WIDTH, 7), (2, TIME, WIDTH), &device).unwrap();
            let slots = register.slots(&x, &ids()).unwrap();
            let stream = register.apply_stream(&x, &slots).unwrap();
            let grads = register
                .apply_readout(&stream, &slots)
                .unwrap()
                .sum_all()
                .unwrap()
                .backward()
                .unwrap();
            for (name, var) in [
                "key_rotation",
                "queries",
                "margins",
                "value_rotation",
                "stream_weight",
                "readout_weight",
            ]
            .into_iter()
            .zip(register.variables())
            {
                assert!(
                    grads.get(var.as_tensor()).is_some(),
                    "no gradient for {name}"
                );
            }
        }
    }
}
