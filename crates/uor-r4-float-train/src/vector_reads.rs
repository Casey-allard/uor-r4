//! Three state-keyed look-back reads on the trunk's stream (one-model-end-to-end, claude-381).
//!
//! Each read runs after one layer (`depth - 1`), on the stream `x` of shape `(batch, time, width)`.
//! A row `t` of a window looks only at strictly earlier rows `s < t` of the same window:
//!
//! 1. **Address.** `q_t` and `k_s` are the row's point rotated block by block by a learned unit
//!    quaternion per 4-lane block ([`hamilton_gate`]), then folded to [`ADDRESS`] coordinates by a
//!    fixed sum over the `width / 16` groups of 16.
//! 2. **Score.** From elementwise differences only, never a coordinate-sum of products:
//!    - Starling Lorentz: the Minkowski chord between the lifts of `q` and `k` on the hyperboloid,
//!      `sqrt(2 (cosh d - 1))`, written from `|q - k|^2`, `|q|^2` and `|k|^2` alone; it orders
//!      candidates as the hyperbolic distance `d` does, without the logarithm D29 refuses on the
//!      device; score `offset - chord`.
//!    - Starling L2: `d = |q - k|`; score `offset - d`.
//!    - Prime route: surrogate score `-sum |q - k|`; the choice is the least zeta-phase energy
//!      between the icosian cells (and so the primes) the four address blocks of `q_t` and `k_s`
//!      name.
//!    - Flock: score `-mean |q - k|`; the choice keeps the `K` nearest ([`choose_nearest`]).
//! 3. **Choice.** Hard and detached: the best score (Starling: only when it is above 0), the least
//!    energy (prime route) or the `K` nearest (Flock); Starling and prime route break ties to the
//!    latest `s`, Flock (`choose_nearest`, an argmin) to the earliest.
//! 4. **Payload.** The shared table's point for the token after the chosen `s`, `table[ids[s+1]]`,
//!    a known input since `s + 1 <= t`, times the straight-through factor `1 - (d - d.detach())`.
//! 5. **Add.** `x + weight * payload`; `weight` starts at 0, so at step 0 the read changes nothing.
//!
//! Every forward also records a margin credit ([`take_credit`]) that ranks, for each row whose next
//! token is known, the best earlier row followed by the same token above every row followed by
//! another; and its counts ([`take_stats`]). [`reference`] writes every closed form a second time in
//! plain f64 Rust.
use std::cell::{Cell, RefCell};
use std::f64::consts::{PI, TAU};
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};

use crate::flock_read::choose_nearest;
use crate::geometric_stack::hamilton_gate;
use crate::{invalid, Result};

/// Address coordinates: four quaternion blocks.
pub const ADDRESS: usize = 16;
/// The Starling credit margin (`Credit::FIXTURE`).
pub const DEFAULT_MARGIN: f32 = 1.0;
/// The file a read is saved to inside a checkpoint directory.
pub const FILE: &str = "vector_read.safetensors";
/// The Starling score offset at initialisation.
const OFFSET_INIT: f32 = 2.5;
/// The floor under squared distances and the Lorentz excess, kept away from the root's kink.
const FLOOR: f64 = 1e-7;
/// A value no real score or energy reaches: masked candidates.
const FAR: f32 = 1.0e30;
/// The 120 icosian cells.
pub const CELLS: usize = 120;

/// A run's vector read as `config.json` records it: the kind, the layer it follows (the last
/// when absent), the credit's margin and weight.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct VectorReadArm {
    pub kind: VectorReadKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    pub margin: f32,
    pub credit: f32,
}

/// Which read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VectorReadKind {
    StarlingLorentz,
    StarlingL2,
    PrimeRoute,
    Flock { neighbours: usize },
}

impl VectorReadKind {
    pub fn name(&self) -> String {
        match self {
            Self::StarlingLorentz => "starling-lorentz".to_owned(),
            Self::StarlingL2 => "starling-l2".to_owned(),
            Self::PrimeRoute => "prime-route".to_owned(),
            Self::Flock { neighbours } => format!("flock-{neighbours}"),
        }
    }

    /// The Starling kinds carry an offset and the threshold terms of the credit.
    pub fn is_starling(&self) -> bool {
        matches!(self, Self::StarlingLorentz | Self::StarlingL2)
    }

    fn code(&self) -> [u32; 2] {
        match self {
            Self::StarlingLorentz => [0, 0],
            Self::StarlingL2 => [1, 0],
            Self::PrimeRoute => [2, 0],
            Self::Flock { neighbours } => [3, *neighbours as u32],
        }
    }

    fn from_code(code: &[u32]) -> Result<Self> {
        match code {
            [0, _] => Ok(Self::StarlingLorentz),
            [1, _] => Ok(Self::StarlingL2),
            [2, _] => Ok(Self::PrimeRoute),
            [3, n] if *n > 0 => Ok(Self::Flock {
                neighbours: *n as usize,
            }),
            _ => Err(invalid("the saved vector read has an unknown kind")),
        }
    }
}

thread_local! {
    static ACTIVE: RefCell<Option<VectorRead>> = const { RefCell::new(None) };
    static CREDIT: RefCell<Option<Tensor>> = const { RefCell::new(None) };
    static STATS: RefCell<Option<ReadStats>> = const { RefCell::new(None) };
    static COLLECT: Cell<bool> = const { Cell::new(false) };
}

/// Run `f` with credit and counts collected: only forwards inside it record them, so an eval
/// forward never keeps a credit graph alive. The previous setting is restored on return, and on
/// a panic inside `f` too.
pub fn collecting<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            COLLECT.with(|cell| cell.set(self.0));
        }
    }
    let _restore = Restore(COLLECT.with(|cell| cell.replace(true)));
    f()
}

/// Make `read` the vector read the trunk applies on this thread, or none.
pub fn set_vector_read(read: Option<VectorRead>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = read);
}

/// The vector read the trunk applies on this thread, if one is set.
pub fn vector_read() -> Option<VectorRead> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// The credit loss (already times the credit weight) the forwards since the last take recorded on
/// this thread, summed; none when no forward recorded one. Clears it.
pub fn take_credit() -> Option<Tensor> {
    CREDIT.with(|cell| cell.borrow_mut().take())
}

/// Drop any recorded credit.
pub fn clear_credit() {
    CREDIT.with(|cell| *cell.borrow_mut() = None);
}

/// The read's counts since the last take, as detached device scalars; none when no forward ran.
pub fn take_stats() -> Option<ReadStats> {
    STATS.with(|cell| cell.borrow_mut().take())
}

/// Counts over the forwards since the last [`take_stats`], each a detached f32 device scalar.
#[derive(Clone)]
pub struct ReadStats {
    /// Rows with at least one earlier position (`t > 0`).
    pub rows: Tensor,
    /// Rows that read.
    pub read_rows: Tensor,
    /// Rows whose next token followed some earlier position too.
    pub positive_rows: Tensor,
    /// Rows that read and whose chosen position (Flock: the first kept) is such a positive.
    pub chose_positive: Tensor,
    /// Credit terms.
    pub hinge_pairs: Tensor,
}

impl ReadStats {
    fn add(&self, other: &Self) -> Result<Self> {
        Ok(Self {
            rows: self.rows.add(&other.rows)?,
            read_rows: self.read_rows.add(&other.read_rows)?,
            positive_rows: self.positive_rows.add(&other.positive_rows)?,
            chose_positive: self.chose_positive.add(&other.chose_positive)?,
            hinge_pairs: self.hinge_pairs.add(&other.hinge_pairs)?,
        })
    }
}

fn record(credit: Option<Tensor>, stats: ReadStats) -> Result<()> {
    if let Some(credit) = credit {
        let sum = match take_credit() {
            Some(old) => old.add(&credit)?,
            None => credit,
        };
        CREDIT.with(|cell| *cell.borrow_mut() = Some(sum));
    }
    let sum = match take_stats() {
        Some(old) => old.add(&stats)?,
        None => stats,
    };
    STATS.with(|cell| *cell.borrow_mut() = Some(sum));
    Ok(())
}

/// The credit's pair sets for windows of `ids`, built on the host from the ids alone. Index
/// `(b * time + t) * time + s`; only `s < t` and rows with `t + 1 < time` carry terms.
pub struct CreditMasks {
    /// `s` is a positive of `t`: `ids[s+1] == ids[t+1]`.
    pub positive: Vec<u8>,
    /// `s` is a negative of a row that has a positive.
    pub negative: Vec<f32>,
    /// Starling only: `s` is an earlier position of a row with no positive.
    pub empty: Vec<f32>,
    /// Per row `b * time + t`: the row has a positive.
    pub has_positive: Vec<f32>,
    /// Number of credit terms.
    pub terms: usize,
    /// Rows with a positive.
    pub positive_rows: usize,
}

impl CreditMasks {
    pub fn build(ids: &[u32], batch: usize, time: usize, starling: bool) -> Self {
        let mut positive = vec![0u8; batch * time * time];
        let mut negative = vec![0f32; batch * time * time];
        let mut empty = vec![0f32; batch * time * time];
        let mut has_positive = vec![0f32; batch * time];
        let (mut terms, mut positive_rows) = (0, 0);
        for b in 0..batch {
            let w = &ids[b * time..(b + 1) * time];
            for t in 1..time.saturating_sub(1) {
                let row = (b * time + t) * time;
                let target = w[t + 1];
                let pos: Vec<bool> = (0..t).map(|s| w[s + 1] == target).collect();
                if pos.iter().any(|&p| p) {
                    has_positive[b * time + t] = 1.0;
                    positive_rows += 1;
                    for (s, &p) in pos.iter().enumerate() {
                        if p {
                            positive[row + s] = 1;
                        } else {
                            negative[row + s] = 1.0;
                            terms += 1;
                        }
                    }
                    if starling {
                        terms += 1;
                    }
                } else if starling {
                    empty[row..row + t].fill(1.0);
                    terms += t;
                }
            }
        }
        Self {
            positive,
            negative,
            empty,
            has_positive,
            terms,
            positive_rows,
        }
    }
}

/// One read: its kind, the layer it follows, the credit's margin and weight, and its parameters.
#[derive(Clone)]
pub struct VectorRead {
    kind: VectorReadKind,
    depth: usize,
    margin: f32,
    credit: f32,
    query_rotation: Var,
    key_rotation: Var,
    weight: Var,
    offset: Option<Var>,
    /// Prime route: the unit icosians `(120, 4)` and the zeta phases of their primes `(120, 8)`.
    cells: Option<(Tensor, Tensor)>,
}

impl VectorRead {
    /// A read after layer `depth - 1` (`depth >= 1`) on points of `width` coordinates (a multiple
    /// of 16): identity rotations and a zero weight, so it starts as no read at all. A depth past
    /// the model's layers never fires.
    pub fn new(
        kind: VectorReadKind,
        depth: usize,
        width: usize,
        margin: f32,
        credit: f32,
        device: &Device,
    ) -> Result<Self> {
        Self::validate(kind, depth, width)?;
        let lanes = width / 4;
        let identity: Vec<f32> = (0..lanes).flat_map(|_| [1.0f32, 0.0, 0.0, 0.0]).collect();
        let offset = if kind.is_starling() {
            Some(Var::from_tensor(&Tensor::new(&[OFFSET_INIT], device)?)?)
        } else {
            None
        };
        Ok(Self {
            kind,
            depth,
            margin,
            credit,
            query_rotation: Var::from_tensor(&Tensor::from_vec(
                identity.clone(),
                (lanes, 4),
                device,
            )?)?,
            key_rotation: Var::from_tensor(&Tensor::from_vec(identity, (lanes, 4), device)?)?,
            weight: Var::zeros(width, DType::F32, device)?,
            offset,
            cells: Self::cells(kind, device)?,
        })
    }

    fn validate(kind: VectorReadKind, depth: usize, width: usize) -> Result<()> {
        if depth == 0 {
            return Err(invalid(
                "a vector read runs after a layer: depth starts at 1",
            ));
        }
        if width == 0 || !width.is_multiple_of(ADDRESS) {
            return Err(invalid("a vector read needs whole 16-coordinate groups"));
        }
        if kind == (VectorReadKind::Flock { neighbours: 0 }) {
            return Err(invalid("a Flock vector read keeps at least one neighbour"));
        }
        Ok(())
    }

    fn cells(kind: VectorReadKind, device: &Device) -> Result<Option<(Tensor, Tensor)>> {
        if kind != VectorReadKind::PrimeRoute {
            return Ok(None);
        }
        let units: Vec<f32> = reference::icosians()
            .iter()
            .flat_map(|q| q.map(|c| c as f32))
            .collect();
        let table = reference::phase_table();
        let phases: Vec<f32> = (0..CELLS)
            .flat_map(|i| table.iter().map(move |row| row[i] as f32))
            .collect();
        Ok(Some((
            Tensor::from_vec(units, (CELLS, 4), device)?,
            Tensor::from_vec(phases, (CELLS, table.len()), device)?,
        )))
    }

    pub fn kind(&self) -> VectorReadKind {
        self.kind
    }

    /// The read runs after layer index `depth - 1`.
    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn margin(&self) -> f32 {
        self.margin
    }

    pub fn credit_weight(&self) -> f32 {
        self.credit
    }

    /// Tests: every weight coordinate set to `weight`, and the Starling offset to `offset`.
    #[cfg(all(test, feature = "cuda"))]
    pub(crate) fn set_for_test(&self, weight: f32, offset: Option<f32>) -> Result<()> {
        let device = self.weight.device();
        let width = self.weight.as_tensor().elem_count();
        self.weight
            .set(&Tensor::from_vec(vec![weight; width], width, device)?)?;
        if let (Some(var), Some(value)) = (&self.offset, offset) {
            var.set(&Tensor::new(&[value], device)?)?;
        }
        Ok(())
    }

    /// The learned parameters: query and key rotations, weight, and the Starling offset.
    pub fn variables(&self) -> Vec<&Var> {
        let mut vars = vec![&self.query_rotation, &self.key_rotation, &self.weight];
        vars.extend(self.offset.as_ref());
        vars
    }

    /// Write the read to `dir/vector_read.safetensors`, its kind, depth, margin and credit weight
    /// with it.
    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let device = self.weight.device();
        let mut tensors = std::collections::HashMap::new();
        tensors.insert(
            "query_rotation".to_owned(),
            self.query_rotation.as_tensor().clone(),
        );
        tensors.insert(
            "key_rotation".to_owned(),
            self.key_rotation.as_tensor().clone(),
        );
        tensors.insert("weight".to_owned(), self.weight.as_tensor().clone());
        if let Some(offset) = &self.offset {
            tensors.insert("offset".to_owned(), offset.as_tensor().clone());
        }
        tensors.insert("kind".to_owned(), Tensor::new(&self.kind.code(), device)?);
        tensors.insert(
            "depth".to_owned(),
            Tensor::new(&[self.depth as u32], device)?,
        );
        tensors.insert("margin".to_owned(), Tensor::new(&[self.margin], device)?);
        tensors.insert("credit".to_owned(), Tensor::new(&[self.credit], device)?);
        candle_core::safetensors::save(&tensors, dir.join(FILE))?;
        Ok(())
    }

    /// The read saved in `dir`, or none when the checkpoint has no vector read.
    pub fn load(dir: &Path, device: &Device) -> Result<Option<Self>> {
        let path = dir.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<Tensor> {
            tensors
                .get(name)
                .cloned()
                .ok_or_else(|| invalid(format!("the saved vector read has no {name}")))
        };
        let first_u32 = |name: &str| -> Result<u32> {
            get(name)?
                .to_vec1::<u32>()?
                .first()
                .copied()
                .ok_or_else(|| invalid(format!("the saved vector read's {name} is empty")))
        };
        let first_f32 = |name: &str| -> Result<f32> {
            get(name)?
                .to_vec1::<f32>()?
                .first()
                .copied()
                .ok_or_else(|| invalid(format!("the saved vector read's {name} is empty")))
        };
        let kind = VectorReadKind::from_code(&get("kind")?.to_vec1::<u32>()?)?;
        let depth = first_u32("depth")? as usize;
        let weight = Var::from_tensor(&get("weight")?)?;
        Self::validate(kind, depth, weight.as_tensor().elem_count())?;
        let lanes = weight.as_tensor().elem_count() / 4;
        let rotation = |name: &str| -> Result<Var> {
            let t = get(name)?;
            if t.dims() != [lanes, 4] {
                return Err(invalid(format!(
                    "the saved vector read's {name} is {:?}, not [{lanes}, 4]",
                    t.dims()
                )));
            }
            Ok(Var::from_tensor(&t)?)
        };
        let offset = if kind.is_starling() {
            let t = get("offset")?;
            if t.dims() != [1] {
                return Err(invalid(format!(
                    "the saved vector read's offset is {:?}, not [1]",
                    t.dims()
                )));
            }
            Some(Var::from_tensor(&t)?)
        } else {
            None
        };
        Ok(Some(Self {
            kind,
            depth,
            margin: first_f32("margin")?,
            credit: first_f32("credit")?,
            query_rotation: rotation("query_rotation")?,
            key_rotation: rotation("key_rotation")?,
            weight,
            offset,
            cells: Self::cells(kind, device)?,
        }))
    }

    /// Rows `(rows, width)` rotated block by block by `rotation`, folded to `(rows, 16)`.
    fn address(&self, rotation: &Var, flat: &Tensor) -> Result<Tensor> {
        let (rows, width) = flat.dims2()?;
        let gate = rotation
            .as_tensor()
            .reshape((1, width))?
            .broadcast_as((rows, width))?
            .contiguous()?;
        let turned = hamilton_gate(&gate, flat)?;
        Ok(turned.reshape((rows, width / ADDRESS, ADDRESS))?.sum(1)?)
    }

    /// Prime route: each 4-block's icosian cell's zeta phases, `(rows, 4 * 8)`. Detached.
    fn phases(&self, address: &Tensor) -> Result<Tensor> {
        let (units, phases) = self
            .cells
            .as_ref()
            .ok_or_else(|| invalid("the prime-route read has no cell table"))?;
        let rows = address.dim(0)?;
        let blocks = address.detach().reshape((rows * 4, 4))?;
        let norm = blocks.sqr()?.sum_keepdim(1)?.sqrt()?.maximum(1e-12)?;
        let unit = blocks.broadcast_div(&norm)?;
        let cell = unit
            .unsqueeze(1)?
            .broadcast_sub(&units.unsqueeze(0)?)?
            .sqr()?
            .sum(2)?
            .argmin(1)?;
        let channels = phases.dim(1)?;
        Ok(phases
            .index_select(&cell, 0)?
            .reshape((rows, 4 * channels))?)
    }

    /// The read applied to the stream `x` `(batch, time, width)` of the windows `ids`, against the
    /// shared `table`. Returns the same shape, and records this forward's credit and counts.
    pub fn apply(&self, x: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        let rows = batch * time;
        if ids.len() != rows || width != self.weight.as_tensor().elem_count() {
            return Err(invalid("the vector read needs whole windows of its width"));
        }
        // A window of one token has no earlier position to read.
        if time < 2 {
            return Ok(x.clone());
        }
        let device = x.device();
        let f32s = |v: Vec<f32>, shape: (usize, usize, usize)| Tensor::from_vec(v, shape, device);
        let flat = x.reshape((rows, width))?;
        let q = self.address(&self.query_rotation, &flat)?;
        let k = self.address(&self.key_rotation, &flat)?;
        let (q3, k3) = (
            q.reshape((batch, time, ADDRESS))?,
            k.reshape((batch, time, ADDRESS))?,
        );
        let earlier: Vec<u8> = (0..time * time)
            .map(|i| u8::from(i % time < i / time))
            .collect();
        let earlier = Tensor::from_vec(earlier, (time, time), device)?
            .broadcast_as((batch, time, time))?
            .contiguous()?;
        let columns = Tensor::arange(1f32, (time + 1) as f32, device)?.reshape((1, 1, time))?;
        // [b, t, s] = q_t - k_s: (batch, time, time, 16), never (.., width).
        let diff = q3.unsqueeze(2)?.broadcast_sub(&k3.unsqueeze(1)?)?;
        let starts: Vec<f32> = (0..rows).map(|r| ((r / time) * time) as f32).collect();
        let starts = Tensor::from_vec(starts, rows, device)?;
        let ids_t = Tensor::from_vec(ids.to_vec(), rows, device)?;
        let weight = self.weight.as_tensor().unsqueeze(0)?;

        // The score with gradient, and for every kind but Flock the chosen pair's distance,
        // payload rows, and the chosen position + 1 (0: none) with whether the row reads.
        let (score, contribution, plus1, reads) = match self.kind {
            VectorReadKind::Flock { neighbours } => {
                let score = diff.abs()?.mean(3)?.neg()?;
                let (picks, real) = choose_nearest(&q, &k, batch, time, neighbours)?;
                // A pick that is not real points at its window's first position, so its next
                // token stays inside the window; it carries weight 0.
                let start_rows = starts.to_dtype(DType::U32)?.reshape((rows, 1))?;
                let picks = real.where_cond(&picks, &start_rows.broadcast_as(picks.shape())?)?;
                let kept = picks.dim(1)?;
                let all = picks.flatten_all()?;
                let kept_keys = k.index_select(&all, 0)?.reshape((rows, kept, ADDRESS))?;
                let distance = q.unsqueeze(1)?.broadcast_sub(&kept_keys)?.abs()?.mean(2)?;
                let factor = distance.detach().sub(&distance)?.affine(1.0, 1.0)?;
                let next = all.broadcast_add(&Tensor::new(&[1u32], device)?)?;
                let payload = table
                    .index_select(&ids_t.index_select(&next, 0)?, 0)?
                    .reshape((rows, kept, width))?;
                let real_f = real.to_dtype(DType::F32)?;
                let count = real_f.sum_keepdim(1)?.maximum(1.0)?;
                let contribution = payload
                    .broadcast_mul(&real_f.mul(&factor)?.unsqueeze(2)?)?
                    .sum(1)?
                    .broadcast_div(&count)?;
                let first = picks.narrow(1, 0, 1)?.squeeze(1)?.to_dtype(DType::F32)?;
                let reads = real_f.narrow(1, 0, 1)?.squeeze(1)?;
                let plus1 = first.sub(&starts)?.affine(1.0, 1.0)?.mul(&reads)?;
                (score, contribution, plus1, reads)
            }
            kind => {
                let (score, distance) = match kind {
                    VectorReadKind::PrimeRoute => {
                        let l1 = diff.abs()?.sum(3)?;
                        (l1.neg()?, l1)
                    }
                    _ => {
                        let n2 = diff.sqr()?.sum(3)?;
                        let d = if kind == VectorReadKind::StarlingLorentz {
                            lorentz_from_distances(&q3, &k3, &n2)?
                        } else {
                            n2.maximum(FLOOR)?.sqrt()?
                        };
                        let offset = self
                            .offset
                            .as_ref()
                            .ok_or_else(|| invalid("a Starling read has an offset"))?
                            .as_tensor()
                            .reshape((1, 1, 1))?;
                        (d.neg()?.broadcast_add(&offset)?, d)
                    }
                };
                let (plus1, reads) = if kind == VectorReadKind::PrimeRoute {
                    let (pq, pk) = (self.phases(&q)?, self.phases(&k)?);
                    let lanes = pq.dim(1)?;
                    let gap = pq
                        .reshape((batch, time, 1, lanes))?
                        .broadcast_sub(&pk.reshape((batch, 1, time, lanes))?)?
                        .abs()?;
                    // |wrap(a - b)| for phases in (-pi, pi]: the shorter way round the circle.
                    let energy = gap.minimum(&gap.affine(-1.0, TAU)?)?.sum(3)?;
                    let (_, plus1) = latest_extreme(&energy, &earlier, false, &columns)?;
                    let reads = plus1.gt(0.5)?.to_dtype(DType::F32)?;
                    (plus1, reads)
                } else {
                    let (best, plus1) = latest_extreme(&score.detach(), &earlier, true, &columns)?;
                    let reads = best.gt(0.0)?.to_dtype(DType::F32)?;
                    (plus1.mul(&reads)?, reads)
                };
                let chosen = columns
                    .broadcast_eq(&plus1.unsqueeze(2)?)?
                    .to_dtype(DType::F32)?;
                let picked = distance.mul(&chosen)?.sum(2)?.reshape(rows)?;
                let factor = picked.detach().sub(&picked)?.affine(1.0, 1.0)?;
                let next = starts.add(&plus1.reshape(rows)?)?.to_dtype(DType::U32)?;
                let payload = table.index_select(&ids_t.index_select(&next, 0)?, 0)?;
                let gain = factor.mul(&reads.reshape(rows)?)?.unsqueeze(1)?;
                (score, payload.broadcast_mul(&gain)?, plus1, reads)
            }
        };
        let out = flat
            .add(&contribution.broadcast_mul(&weight)?)?
            .reshape((batch, time, width))?;
        if !COLLECT.with(Cell::get) {
            return Ok(out);
        }

        // The margin credit over every earlier position, masks from the ids.
        let masks = CreditMasks::build(ids, batch, time, self.kind.is_starling());
        let shape = (batch, time, time);
        let positive = Tensor::from_vec(masks.positive.clone(), shape, device)?;
        let positive_f = positive.to_dtype(DType::F32)?;
        let credit = if masks.terms == 0 {
            None
        } else {
            let margin = f64::from(self.margin);
            let (_, best) = latest_extreme(&score.detach(), &positive, true, &columns)?;
            let star = columns
                .broadcast_eq(&best.unsqueeze(2)?)?
                .to_dtype(DType::F32)?;
            let star_score = score.mul(&star)?.sum(2)?;
            let mut total = score
                .broadcast_sub(&star_score.unsqueeze(2)?)?
                .affine(1.0, margin)?
                .relu()?
                .mul(&f32s(masks.negative.clone(), shape)?)?
                .sum_all()?;
            if self.kind.is_starling() {
                let has = Tensor::from_vec(masks.has_positive.clone(), (batch, time), device)?;
                let threshold = star_score
                    .affine(-1.0, margin)?
                    .relu()?
                    .mul(&has)?
                    .sum_all()?;
                let empty = score
                    .affine(1.0, margin)?
                    .relu()?
                    .mul(&f32s(masks.empty.clone(), shape)?)?
                    .sum_all()?;
                total = total.add(&threshold)?.add(&empty)?;
            }
            let scale = f64::from(self.credit) / masks.terms.max(1) as f64;
            Some(total.affine(scale, 0.0)?)
        };
        let chosen = columns
            .broadcast_eq(&plus1.reshape((batch, time))?.unsqueeze(2)?)?
            .to_dtype(DType::F32)?;
        let scalar = |v: usize| Tensor::new(v as f32, device);
        let stats = ReadStats {
            rows: scalar(batch * (time - 1))?,
            read_rows: reads.detach().sum_all()?,
            positive_rows: scalar(masks.positive_rows)?,
            chose_positive: chosen.mul(&positive_f)?.sum_all()?.detach(),
            hinge_pairs: scalar(masks.terms)?,
        };
        record(credit, stats)?;
        Ok(out)
    }
}

/// The Minkowski chord between the lifts `(sqrt(1 + |v|^2), v)` of `q_t` and `k_s` on the
/// hyperboloid, from the squared difference `n2` `(batch, time, time)` and the squared norms only:
/// `e = lift_q lift_k - (|q|^2 + |k|^2 - n2) / 2 - 1 = cosh d - 1`, chord `sqrt(2 e)`. The chord
/// rises with the hyperbolic distance `d = arcosh(1 + e)`, so it orders candidates exactly as `d`
/// does; `d` itself needs a logarithm, whose kernel D29 refuses on the device.
fn lorentz_from_distances(q3: &Tensor, k3: &Tensor, n2: &Tensor) -> Result<Tensor> {
    let qn = q3.sqr()?.sum(2)?;
    let kn = k3.sqr()?.sum(2)?;
    let lift_q = qn.affine(1.0, 1.0)?.sqrt()?.unsqueeze(2)?;
    let lift_k = kn.affine(1.0, 1.0)?.sqrt()?.unsqueeze(1)?;
    let half = qn
        .unsqueeze(2)?
        .broadcast_add(&kn.unsqueeze(1)?)?
        .sub(n2)?
        .affine(0.5, 0.0)?;
    let e = lift_q
        .broadcast_mul(&lift_k)?
        .sub(&half)?
        .affine(1.0, -1.0)?
        .maximum(FLOOR)?;
    Ok(e.affine(2.0, 0.0)?.sqrt()?)
}

/// Over `mask`ed `values` `(batch, time, time)` (detached), the largest (or least) per row, and the
/// latest column holding it, as `column + 1` f32 `(batch, time)`, 0 when the row has none.
fn latest_extreme(
    values: &Tensor,
    mask: &Tensor,
    largest: bool,
    columns: &Tensor,
) -> Result<(Tensor, Tensor)> {
    let fill = Tensor::full(
        if largest { -FAR } else { FAR },
        values.shape(),
        values.device(),
    )?;
    let masked = mask.where_cond(values, &fill)?;
    let best = if largest {
        masked.max_keepdim(2)?
    } else {
        masked.min_keepdim(2)?
    };
    let hit = masked
        .broadcast_eq(&best)?
        .to_dtype(DType::F32)?
        .mul(&mask.to_dtype(DType::F32)?)?;
    let plus1 = hit.broadcast_mul(columns)?.max(2)?;
    Ok((best.squeeze(2)?, plus1))
}

/// Every closed form of the reads a second time, in plain f64 Rust, for one window.
pub mod reference {
    use super::*;

    /// The Hamilton product `a b`.
    pub fn hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
        [
            a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
            a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
            a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
            a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
        ]
    }

    /// The 120 unit icosians: the 8 axis units, the 16 `(+-1/2)^4`, and the 96 even permutations of
    /// `(+-phi/2, +-1/2, +-1/(2 phi), 0)`.
    pub fn icosians() -> Vec<[f64; 4]> {
        let phi = (1.0 + 5f64.sqrt()) / 2.0;
        let mut out = Vec::with_capacity(CELLS);
        for axis in 0..4 {
            for sign in [1.0, -1.0] {
                out.push(std::array::from_fn(|i| if i == axis { sign } else { 0.0 }));
            }
        }
        for signs in 0..16u32 {
            out.push(std::array::from_fn(|i| {
                if (signs >> i) & 1 == 1 {
                    -0.5
                } else {
                    0.5
                }
            }));
        }
        let base = [phi / 2.0, 0.5, 1.0 / (2.0 * phi), 0.0];
        for perm in even_permutations() {
            for signs in 0..8u32 {
                let mut v = [0.0; 4];
                for (i, &place) in perm.iter().enumerate() {
                    let sign = if i < 3 && (signs >> i) & 1 == 1 {
                        -1.0
                    } else {
                        1.0
                    };
                    v[place] = sign * base[i];
                }
                out.push(v);
            }
        }
        out
    }

    fn even_permutations() -> Vec<[usize; 4]> {
        let mut out = Vec::new();
        for a in 0..4 {
            for b in 0..4 {
                for c in 0..4 {
                    for d in 0..4 {
                        let p = [a, b, c, d];
                        let distinct = (0..4).all(|i| (i + 1..4).all(|j| p[i] != p[j]));
                        let inversions = (0..4)
                            .flat_map(|i| (i + 1..4).map(move |j| (i, j)))
                            .filter(|&(i, j)| p[i] > p[j])
                            .count();
                        if distinct && inversions % 2 == 0 {
                            out.push(p);
                        }
                    }
                }
            }
        }
        out
    }

    /// The first `n` primes.
    pub fn primes(n: usize) -> Vec<u64> {
        let mut out: Vec<u64> = Vec::with_capacity(n);
        let mut candidate = 2u64;
        while out.len() < n {
            if out
                .iter()
                .take_while(|&&p| p * p <= candidate)
                .all(|&p| !candidate.is_multiple_of(p))
            {
                out.push(candidate);
            }
            candidate += 1;
        }
        out
    }

    /// `x` wrapped into `(-pi, pi]`.
    pub fn wrap(x: f64) -> f64 {
        let y = x.rem_euclid(TAU);
        if y > PI {
            y - TAU
        } else {
            y
        }
    }

    /// `Phi[c][i] = wrap(gamma_{ch_c} ln p_i)` over the attention's zeta channels and the 120 cells'
    /// primes.
    pub fn phase_table() -> Vec<Vec<f64>> {
        // `uor_r4_core::prime_route_geometric_attention::ATTENTION_ZETA_CHANNELS`, copied: that
        // module is behind a core feature this crate does not turn on.
        const ATTENTION_ZETA_CHANNELS: [u16; 8] = [0, 1, 2, 3, 5, 8, 13, 21];
        use uor_r4_core::zeta_zeros::ZETA_ZEROS;
        let primes = primes(CELLS);
        ATTENTION_ZETA_CHANNELS
            .iter()
            .map(|&ch| {
                primes
                    .iter()
                    .map(|&p| wrap(ZETA_ZEROS[usize::from(ch)] * (p as f64).ln()))
                    .collect()
            })
            .collect()
    }

    /// The zeta-phase energy between two cells: `sum_c |wrap(Phi[c][a] - Phi[c][b])|`.
    pub fn cell_energy(table: &[Vec<f64>], a: usize, b: usize) -> f64 {
        table.iter().map(|row| wrap(row[a] - row[b]).abs()).sum()
    }

    /// The energy between two addresses' four cells.
    pub fn energy(table: &[Vec<f64>], q: [usize; 4], k: [usize; 4]) -> f64 {
        q.iter()
            .zip(k)
            .map(|(&a, b)| cell_energy(table, a, b))
            .sum()
    }

    /// The nearest icosian cell of a block normalised to unit length (ties to the lowest index).
    pub fn cell(units: &[[f64; 4]], block: &[f64]) -> usize {
        let norm = block.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-12);
        let mut best = (0, f64::INFINITY);
        for (i, u) in units.iter().enumerate() {
            let d: f64 = (0..4).map(|j| (block[j] / norm - u[j]).powi(2)).sum();
            if d < best.1 {
                best = (i, d);
            }
        }
        best.0
    }

    fn squared(v: &[f64]) -> f64 {
        v.iter().map(|x| x * x).sum()
    }

    fn squared_gap(q: &[f64], k: &[f64]) -> f64 {
        q.iter().zip(k).map(|(a, b)| (a - b).powi(2)).sum()
    }

    /// The exact hyperboloid distance from `|q - k|^2` and the norms alone.
    pub fn lorentz_distance(q: &[f64], k: &[f64]) -> f64 {
        let (qn, kn, n2) = (squared(q), squared(k), squared_gap(q, k));
        let e = ((1.0 + qn).sqrt() * (1.0 + kn).sqrt() - (qn + kn - n2) / 2.0 - 1.0).max(0.0);
        (1.0 + e + (e * (e + 2.0)).sqrt()).ln()
    }

    /// The same with the tensor code's floor under the excess.
    pub fn lorentz_distance_floored(q: &[f64], k: &[f64]) -> f64 {
        let (qn, kn, n2) = (squared(q), squared(k), squared_gap(q, k));
        let e = ((1.0 + qn).sqrt() * (1.0 + kn).sqrt() - (qn + kn - n2) / 2.0 - 1.0).max(FLOOR);
        (1.0 + e + (e * (e + 2.0)).sqrt()).ln()
    }

    /// The tensor code's score distance: the Minkowski chord `sqrt(2 e)` between the two lifts,
    /// with its floor under the excess `e = cosh d - 1`.
    pub fn lorentz_chord(q: &[f64], k: &[f64]) -> f64 {
        let (qn, kn, n2) = (squared(q), squared(k), squared_gap(q, k));
        let e = ((1.0 + qn).sqrt() * (1.0 + kn).sqrt() - (qn + kn - n2) / 2.0 - 1.0).max(FLOOR);
        (2.0 * e).sqrt()
    }

    /// One read in f64: its kind, credit margin, Starling offset, and parameters as flat slices.
    pub struct Reference<'a> {
        pub kind: VectorReadKind,
        pub margin: f64,
        pub offset: f64,
        pub query_rotation: &'a [f64],
        pub key_rotation: &'a [f64],
        pub weight: &'a [f64],
    }

    /// The reference forward's output rows, chosen positions per row, and credit (before the
    /// credit weight: the terms' sum over their count).
    pub struct ReferenceOutput {
        pub out: Vec<f64>,
        pub chosen: Vec<Vec<usize>>,
        pub credit: f64,
    }

    fn address(rotation: &[f64], row: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0; ADDRESS];
        for (j, (g, u)) in rotation.chunks(4).zip(row.chunks(4)).enumerate() {
            let r = (g.iter().map(|v| v * v).sum::<f64>() + 1e-12).sqrt();
            let turned = hamilton(
                [g[0] / r, g[1] / r, g[2] / r, g[3] / r],
                [u[0], u[1], u[2], u[3]],
            );
            let at = (j * 4) % ADDRESS;
            for (o, v) in out[at..at + 4].iter_mut().zip(turned) {
                *o += v;
            }
        }
        out
    }

    impl Reference<'_> {
        /// The read on one window of `ids.len()` rows of `x`, against `table` rows of `width`.
        pub fn forward(
            &self,
            x: &[f64],
            ids: &[u32],
            table: &[f64],
            width: usize,
        ) -> ReferenceOutput {
            let time = ids.len();
            let q: Vec<Vec<f64>> = x
                .chunks(width)
                .map(|r| address(self.query_rotation, r))
                .collect();
            let k: Vec<Vec<f64>> = x
                .chunks(width)
                .map(|r| address(self.key_rotation, r))
                .collect();
            let score = |t: usize, s: usize| -> f64 {
                match self.kind {
                    VectorReadKind::StarlingLorentz => self.offset - lorentz_chord(&q[t], &k[s]),
                    VectorReadKind::StarlingL2 => {
                        self.offset - squared_gap(&q[t], &k[s]).max(FLOOR).sqrt()
                    }
                    VectorReadKind::PrimeRoute => -q[t]
                        .iter()
                        .zip(&k[s])
                        .map(|(a, b)| (a - b).abs())
                        .sum::<f64>(),
                    VectorReadKind::Flock { .. } => {
                        -q[t]
                            .iter()
                            .zip(&k[s])
                            .map(|(a, b)| (a - b).abs())
                            .sum::<f64>()
                            / ADDRESS as f64
                    }
                }
            };
            let units = icosians();
            let table_phase = phase_table();
            let cells = |a: &[f64]| -> [usize; 4] {
                std::array::from_fn(|j| cell(&units, &a[4 * j..4 * j + 4]))
            };
            let mut out = x.to_vec();
            let mut chosen = Vec::with_capacity(time);
            for t in 0..time {
                let picks: Vec<usize> = match self.kind {
                    VectorReadKind::Flock { neighbours } => {
                        let mut order: Vec<usize> = (0..t).collect();
                        order.sort_by(|&a, &b| score(t, b).total_cmp(&score(t, a)));
                        order.truncate(neighbours);
                        order
                    }
                    VectorReadKind::PrimeRoute => {
                        let cq = cells(&q[t]);
                        let mut best: Option<(usize, f64)> = None;
                        for (s, ks) in k.iter().enumerate().take(t) {
                            let e = energy(&table_phase, cq, cells(ks));
                            if best.is_none_or(|(_, b)| e <= b) {
                                best = Some((s, e));
                            }
                        }
                        best.map(|(s, _)| s).into_iter().collect()
                    }
                    _ => {
                        let mut best: Option<(usize, f64)> = None;
                        for s in 0..t {
                            let v = score(t, s);
                            if best.is_none_or(|(_, b)| v >= b) {
                                best = Some((s, v));
                            }
                        }
                        best.filter(|&(_, v)| v > 0.0)
                            .map(|(s, _)| s)
                            .into_iter()
                            .collect()
                    }
                };
                if !picks.is_empty() {
                    let share = 1.0 / picks.len() as f64;
                    for &s in &picks {
                        let row = ids[s + 1] as usize * width;
                        let point = &table[row..row + width];
                        let target = &mut out[t * width..(t + 1) * width];
                        for ((o, w), p) in target.iter_mut().zip(self.weight).zip(point) {
                            *o += w * share * p;
                        }
                    }
                }
                chosen.push(picks);
            }
            let masks = CreditMasks::build(ids, 1, time, self.kind.is_starling());
            let mut total = 0.0;
            for t in 1..time.saturating_sub(1) {
                let row = t * time;
                let star = (0..t).filter(|&s| masks.positive[row + s] == 1).fold(
                    None,
                    |best: Option<usize>, s| match best {
                        Some(b) if score(t, b) > score(t, s) => Some(b),
                        _ => Some(s),
                    },
                );
                if let Some(p) = star {
                    let sp = score(t, p);
                    for s in 0..t {
                        if masks.negative[row + s] > 0.0 {
                            total += (self.margin - sp + score(t, s)).max(0.0);
                        }
                    }
                    if self.kind.is_starling() {
                        total += (self.margin - sp).max(0.0);
                    }
                } else if self.kind.is_starling() {
                    for s in 0..t {
                        total += (self.margin + score(t, s)).max(0.0);
                    }
                }
            }
            ReferenceOutput {
                out,
                chosen,
                credit: total / masks.terms.max(1) as f64,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::reference::*;
    use super::*;

    #[test]
    fn the_icosians_are_120_distinct_unit_quaternions_closed_under_products() {
        let units = icosians();
        assert_eq!(units.len(), CELLS);
        for u in &units {
            let norm: f64 = u.iter().map(|v| v * v).sum();
            assert!((norm - 1.0).abs() < 1e-12, "{u:?}");
        }
        let gap = |a: &[f64; 4], b: &[f64; 4]| -> f64 {
            a.iter()
                .zip(b)
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f64>()
                .sqrt()
        };
        for (i, a) in units.iter().enumerate() {
            for b in &units[i + 1..] {
                assert!(gap(a, b) > 0.1, "{a:?} {b:?}");
            }
        }
        for &a in &units {
            for &b in &units {
                let p = hamilton(a, b);
                let nearest = units
                    .iter()
                    .map(|u| gap(u, &p))
                    .fold(f64::INFINITY, f64::min);
                assert!(nearest < 1e-12, "{a:?} * {b:?} = {p:?}");
            }
        }
    }

    #[test]
    fn the_lorentz_distance_from_differences_is_the_hyperboloid_distance() {
        let pairs: [([f64; 4], [f64; 4]); 6] = [
            ([0.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]),
            ([0.5, -0.2, 0.1, 0.3], [-0.4, 0.6, 0.0, 0.2]),
            ([2.0, 1.0, -1.0, 0.5], [1.5, -0.5, 0.25, 0.0]),
            ([-3.0, 0.0, 2.0, 1.0], [3.0, 0.0, -2.0, -1.0]),
            ([0.1, 0.1, 0.1, 0.1], [0.2, 0.1, 0.0, 0.1]),
            ([5.0, -4.0, 3.0, -2.0], [0.0, 1.0, 0.0, 1.0]),
        ];
        let direct = |q: &[f64; 4], k: &[f64; 4]| -> f64 {
            let lq = (1.0 + q.iter().map(|v| v * v).sum::<f64>()).sqrt();
            let lk = (1.0 + k.iter().map(|v| v * v).sum::<f64>()).sqrt();
            let inner: f64 = q.iter().zip(k).map(|(a, b)| a * b).sum();
            (lq * lk - inner).acosh()
        };
        for (q, k) in &pairs {
            let d = lorentz_distance(q, k);
            assert!((d - direct(q, k)).abs() < 1e-9, "{q:?} {k:?}");
            assert!((d - lorentz_distance(k, q)).abs() < 1e-12);
            assert!((d - lorentz_distance_floored(q, k)).abs() < 1e-9);
            // The excess cancels to rounding, so the distance is of order sqrt(epsilon).
            assert!(lorentz_distance(q, q) < 1e-7);
            // The device's score distance is the chord between the lifts, sqrt(2 (cosh d - 1)).
            let chord = (2.0 * (direct(q, k).cosh() - 1.0)).sqrt();
            assert!((lorentz_chord(q, k) - chord).abs() < 1e-7, "{q:?} {k:?}");
        }
        // The chord orders every pair exactly as the hyperbolic distance does.
        let mut by_distance: Vec<usize> = (0..pairs.len()).collect();
        let mut by_chord = by_distance.clone();
        by_distance.sort_by(|&a, &b| {
            lorentz_distance(&pairs[a].0, &pairs[a].1)
                .total_cmp(&lorentz_distance(&pairs[b].0, &pairs[b].1))
        });
        by_chord.sort_by(|&a, &b| {
            lorentz_chord(&pairs[a].0, &pairs[a].1)
                .total_cmp(&lorentz_chord(&pairs[b].0, &pairs[b].1))
        });
        assert_eq!(by_distance, by_chord);
        let points = [
            [0.0, 0.0, 0.0, 0.0],
            [1.0, 2.0, 0.0, -1.0],
            [-2.0, 0.5, 1.0, 0.0],
        ];
        for a in &points {
            for b in &points {
                for c in &points {
                    let lhs = lorentz_distance(a, c);
                    assert!(lhs <= lorentz_distance(a, b) + lorentz_distance(b, c) + 1e-12);
                }
            }
        }
    }

    #[test]
    fn the_phase_table_wraps_and_its_energy_is_zero_only_on_equal_cells() {
        let table = phase_table();
        assert_eq!(table.len(), 8);
        assert_eq!(primes(5), vec![2, 3, 5, 7, 11]);
        for row in &table {
            assert_eq!(row.len(), CELLS);
            assert!(row.iter().all(|&v| v > -PI && v <= PI));
        }
        for a in 0..CELLS {
            assert_eq!(cell_energy(&table, a, a), 0.0);
            for b in 0..CELLS {
                let e = cell_energy(&table, a, b);
                assert_eq!(e, cell_energy(&table, b, a));
                if a != b {
                    assert!(e > 1e-6, "cells {a} {b}");
                }
            }
        }
        let q = [3, 17, 64, 119];
        assert_eq!(energy(&table, q, q), 0.0);
        assert!(energy(&table, q, [3, 17, 64, 118]) > 0.0);
        assert_eq!(
            energy(&table, q, [0, 1, 2, 3]),
            energy(&table, [0, 1, 2, 3], q)
        );
        // The wrap's range is half-open at -pi.
        assert_eq!(wrap(-PI), PI);
        assert!((wrap(3.0 * PI + 0.5) - (-PI + 0.5)).abs() < 1e-12);
    }

    #[test]
    fn the_credit_pairs_on_a_hand_written_window() {
        // Next tokens ids[1..] = 1 7 1 7 2 1 7. Rows t = 1..=6 carry terms:
        // t1 target 7: none; t2 target 1: pos {0} neg {1}; t3 target 7: pos {1} neg {0,2};
        // t4 target 2: none (4 earlier); t5 target 1: pos {0,2} neg {1,3,4};
        // t6 target 7: pos {1,3} neg {0,2,4,5}.
        let ids = [5u32, 1, 7, 1, 7, 2, 1, 7];
        let time = ids.len();
        let positives = |m: &CreditMasks, t: usize| -> Vec<usize> {
            (0..t).filter(|&s| m.positive[t * time + s] == 1).collect()
        };
        let negatives = |m: &CreditMasks, t: usize| -> Vec<usize> {
            (0..t).filter(|&s| m.negative[t * time + s] > 0.0).collect()
        };
        let lorentz = CreditMasks::build(&ids, 1, time, true);
        let flock = CreditMasks::build(&ids, 1, time, false);
        for m in [&lorentz, &flock] {
            assert_eq!(positives(m, 1), Vec::<usize>::new());
            assert_eq!(positives(m, 2), vec![0]);
            assert_eq!(negatives(m, 2), vec![1]);
            assert_eq!(positives(m, 3), vec![1]);
            assert_eq!(negatives(m, 3), vec![0, 2]);
            assert_eq!(positives(m, 4), Vec::<usize>::new());
            assert_eq!(negatives(m, 4), Vec::<usize>::new());
            assert_eq!(positives(m, 5), vec![0, 2]);
            assert_eq!(negatives(m, 5), vec![1, 3, 4]);
            assert_eq!(positives(m, 6), vec![1, 3]);
            assert_eq!(negatives(m, 6), vec![0, 2, 4, 5]);
            // The last row's next token is unknown: no terms.
            assert!(positives(m, 7).is_empty() && negatives(m, 7).is_empty());
            assert_eq!(m.positive_rows, 4);
        }
        // Lorentz: 10 pair terms, 4 threshold terms, 1 + 4 empty-row terms.
        assert_eq!(lorentz.terms, 19);
        assert_eq!(lorentz.empty.iter().filter(|&&v| v > 0.0).count(), 5);
        assert_eq!(flock.terms, 10);
        assert!(flock.empty.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn kinds_are_named_and_depth_zero_is_refused() {
        assert_eq!(VectorReadKind::StarlingLorentz.name(), "starling-lorentz");
        assert_eq!(VectorReadKind::StarlingL2.name(), "starling-l2");
        assert_eq!(VectorReadKind::PrimeRoute.name(), "prime-route");
        assert_eq!(VectorReadKind::Flock { neighbours: 4 }.name(), "flock-4");
        for kind in [
            VectorReadKind::StarlingLorentz,
            VectorReadKind::Flock { neighbours: 2 },
        ] {
            assert_eq!(VectorReadKind::from_code(&kind.code()).unwrap(), kind);
        }
        let cpu = Device::Cpu;
        assert!(VectorRead::new(VectorReadKind::StarlingL2, 0, 16, 1.0, 1.0, &cpu).is_err());
        assert!(VectorRead::new(VectorReadKind::StarlingL2, 1, 12, 1.0, 1.0, &cpu).is_err());
        assert!(VectorRead::new(
            VectorReadKind::Flock { neighbours: 0 },
            1,
            16,
            1.0,
            1.0,
            &cpu
        )
        .is_err());
    }

    #[cfg(feature = "cuda")]
    mod cuda {
        use super::*;

        const T: usize = 6;
        const W: usize = 16;
        const V: usize = 8;
        const IDS: [u32; T] = [1, 2, 3, 2, 3, 2];

        struct Lcg(u64);
        impl Lcg {
            fn unit(&mut self) -> f64 {
                self.0 = self
                    .0
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((self.0 >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
            }
            fn vec(&mut self, n: usize, scale: f64) -> Vec<f64> {
                (0..n).map(|_| self.unit() * scale).collect()
            }
        }

        fn kinds() -> [VectorReadKind; 4] {
            [
                VectorReadKind::StarlingLorentz,
                VectorReadKind::StarlingL2,
                VectorReadKind::PrimeRoute,
                VectorReadKind::Flock { neighbours: 2 },
            ]
        }

        fn f32s(v: &[f64]) -> Vec<f32> {
            v.iter().map(|&x| x as f32).collect()
        }

        fn back(v: &[f32]) -> Vec<f64> {
            v.iter().map(|&x| f64::from(x)).collect()
        }

        struct Fixture {
            read: VectorRead,
            x: Tensor,
            table: Tensor,
        }

        fn fixture(kind: VectorReadKind, margin: f32, seed: u64) -> Result<Fixture> {
            let device = Device::new_cuda(0)?;
            let mut rng = Lcg(seed);
            let read = VectorRead::new(kind, 1, W, margin, 1.0, &device)?;
            let set = |var: &Var, v: Vec<f64>, rows: usize, cols: usize| -> Result<()> {
                Ok(var.set(&Tensor::from_vec(f32s(&v), (rows, cols), &device)?)?)
            };
            let mut rot = rng.vec(W, 0.5);
            rot.iter_mut().step_by(4).for_each(|v| *v += 1.0);
            set(&read.query_rotation, rot, W / 4, 4)?;
            let mut rot = rng.vec(W, 0.5);
            rot.iter_mut().step_by(4).for_each(|v| *v += 1.0);
            set(&read.key_rotation, rot, W / 4, 4)?;
            read.weight
                .set(&Tensor::from_vec(f32s(&rng.vec(W, 1.0)), W, &device)?)?;
            let x = Tensor::from_vec(f32s(&rng.vec(T * W, 0.4)), (1, T, W), &device)?;
            let table = Tensor::from_vec(f32s(&rng.vec(V * W, 1.0)), (V, W), &device)?;
            Ok(Fixture { read, x, table })
        }

        fn host(t: &Tensor) -> Result<Vec<f64>> {
            Ok(back(&t.flatten_all()?.to_vec1::<f32>()?))
        }

        fn reference_of(f: &Fixture) -> Result<ReferenceOutput> {
            let offset = match &f.read.offset {
                Some(o) => host(o.as_tensor())?[0],
                None => 0.0,
            };
            let (qr, kr, w) = (
                host(f.read.query_rotation.as_tensor())?,
                host(f.read.key_rotation.as_tensor())?,
                host(f.read.weight.as_tensor())?,
            );
            let r = Reference {
                kind: f.read.kind,
                margin: f64::from(f.read.margin),
                offset,
                query_rotation: &qr,
                key_rotation: &kr,
                weight: &w,
            };
            Ok(r.forward(&host(&f.x)?, &IDS, &host(&f.table)?, W))
        }

        #[test]
        fn the_forward_matches_the_plain_reference_for_every_kind() -> Result<()> {
            for kind in kinds() {
                clear_credit();
                let _ = take_stats();
                let f = fixture(kind, DEFAULT_MARGIN, 381)?;
                let out = host(&collecting(|| f.read.apply(&f.x, &IDS, &f.table))?)?;
                let want = reference_of(&f)?;
                for (i, (a, b)) in out.iter().zip(&want.out).enumerate() {
                    assert!((a - b).abs() < 1e-5, "{kind:?} [{i}]: {a} vs {b}");
                }
                let credit = take_credit().map(|c| c.to_scalar::<f32>()).transpose()?;
                let credit = credit.map(f64::from).unwrap_or(0.0);
                assert!(
                    (credit - want.credit).abs() < 1e-5,
                    "{kind:?}: {credit} vs {}",
                    want.credit
                );
                let stats = take_stats().expect("stats");
                let reads = want.chosen.iter().filter(|c| !c.is_empty()).count();
                assert_eq!(
                    stats.read_rows.to_scalar::<f32>()?,
                    reads as f32,
                    "{kind:?}"
                );
                assert_eq!(stats.rows.to_scalar::<f32>()?, (T - 1) as f32);
            }
            Ok(())
        }

        /// The Starling gate: at an offset no score clears, no row reads and the stream comes
        /// back unchanged; between, some rows read and some do not, and the device agrees with
        /// the reference on every offset of the sweep.
        #[test]
        fn the_starling_gate_reads_only_above_zero() -> Result<()> {
            for kind in [VectorReadKind::StarlingLorentz, VectorReadKind::StarlingL2] {
                let f = fixture(kind, DEFAULT_MARGIN, 385)?;
                let mut counts = Vec::new();
                for offset in [0.0f32, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5] {
                    let o = f.read.offset.as_ref().expect("a Starling offset");
                    o.set(&Tensor::new(&[offset], o.device())?)?;
                    clear_credit();
                    let _ = take_stats();
                    let out = host(&collecting(|| f.read.apply(&f.x, &IDS, &f.table))?)?;
                    let want = reference_of(&f)?;
                    let reads = want.chosen.iter().filter(|c| !c.is_empty()).count();
                    let stats = take_stats().expect("stats");
                    assert_eq!(
                        stats.read_rows.to_scalar::<f32>()?,
                        reads as f32,
                        "{kind:?} offset {offset}"
                    );
                    for (i, (a, b)) in out.iter().zip(&want.out).enumerate() {
                        assert!((a - b).abs() < 1e-5, "{kind:?} offset {offset} [{i}]");
                    }
                    if reads == 0 {
                        assert_eq!(out, host(&f.x)?, "{kind:?} offset {offset}");
                    }
                    counts.push(reads);
                }
                clear_credit();
                assert_eq!(counts[0], 0, "{kind:?}: a zero offset read {counts:?}");
                assert!(
                    counts.iter().any(|&c| c > 0 && c < T - 1),
                    "{kind:?}: no offset of the sweep read some rows and not others: {counts:?}"
                );
            }
            Ok(())
        }

        #[test]
        fn the_credit_gradient_matches_central_differences() -> Result<()> {
            for kind in kinds() {
                let f = fixture(kind, 5.0, 382)?;
                let loss = |f: &Fixture| -> Result<Tensor> {
                    clear_credit();
                    collecting(|| f.read.apply(&f.x, &IDS, &f.table))?;
                    take_credit().ok_or_else(|| invalid("no credit"))
                };
                let grads = loss(&f)?.backward()?;
                let mut vars = vec![
                    (&f.read.query_rotation, [1usize, 5, 10]),
                    (&f.read.key_rotation, [2, 7, 13]),
                ];
                if let Some(o) = &f.read.offset {
                    vars.push((o, [0, 0, 0]));
                }
                for (var, entries) in vars {
                    let grad = host(grads.get(var.as_tensor()).expect("a gradient"))?;
                    let base = var.as_tensor().clone();
                    let flat = host(&base)?;
                    for &e in &entries {
                        let h = 1e-3;
                        let at = |delta: f64| -> Result<f64> {
                            let mut v = flat.clone();
                            v[e] += delta;
                            var.set(&Tensor::from_vec(f32s(&v), base.shape(), base.device())?)?;
                            // The difference is taken on the f64 reference credit, which the
                            // forward test ties to the device's: an f32 loss differenced at h
                            // carries rounding of order 1e-6 |L| / h, as large as the margin here.
                            Ok(reference_of(&f)?.credit)
                        };
                        let fd = (at(h)? - at(-h)?) / (2.0 * h);
                        // A copy, not `base`, which shares the variable's own storage.
                        var.set(&Tensor::from_vec(f32s(&flat), base.shape(), base.device())?)?;
                        let ad = grad[e];
                        let tol = 2e-2 * fd.abs().max(ad.abs()) + 1e-4;
                        assert!(
                            (fd - ad).abs() <= tol,
                            "{kind:?} entry {e}: fd {fd} ad {ad}"
                        );
                    }
                }
            }
            clear_credit();
            Ok(())
        }

        #[test]
        fn a_zero_weight_leaves_the_stream_bit_for_bit() -> Result<()> {
            for kind in kinds() {
                let f = fixture(kind, DEFAULT_MARGIN, 383)?;
                f.read
                    .weight
                    .set(&f.read.weight.as_tensor().zeros_like()?)?;
                let out = f.read.apply(&f.x, &IDS, &f.table)?;
                let bits = |t: &Tensor| -> Result<Vec<u32>> {
                    Ok(t.flatten_all()?
                        .to_vec1::<f32>()?
                        .iter()
                        .map(|v| v.to_bits())
                        .collect())
                };
                assert_eq!(bits(&out)?, bits(&f.x)?, "{kind:?}");
            }
            // Outside `collecting` (an eval forward) nothing is recorded.
            assert!(take_credit().is_none());
            assert!(take_stats().is_none());
            set_vector_read(Some(fixture(kinds()[0], 1.0, 384)?.read));
            assert!(vector_read().is_some());
            set_vector_read(None);
            assert!(vector_read().is_none());
            Ok(())
        }
    }
}
