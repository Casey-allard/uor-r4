//! Three state-keyed look-back reads on the trunk's stream (one-model-end-to-end, claude-381).
//!
//! Each read runs after one layer (`depth - 1`), on the stream `x` of shape `(batch, time, width)`.
//! A row `t` of a window looks only at strictly earlier rows `s < t` of the same window:
//!
//! 1. **Address.** `q_t` and `k_s` are the row's point rotated block by block by a learned unit
//!    quaternion per 4-lane block ([`hamilton_gate`]), then folded to [`ADDRESS`] coordinates by a
//!    fixed sum over the `width / 16` groups of 16. By default (claude-385) the address reads a
//!    detached copy of the stream, so the credit trains the read's own rotations, offset and
//!    weight and never the trunk under it; [`VectorRead::with_stream_grad`] turns that off.
//! 2. **Score.** From elementwise differences only, never a coordinate-sum of products:
//!    - Starling Lorentz: the Minkowski chord between the lifts `l = sqrt(1 + |v|^2)` of `q` and
//!      `k` on the hyperboloid, `sqrt(2 (cosh d - 1))`, with the excess written without a product
//!      of the two lifts (claude-385): `cosh d - 1 = (|q - k|^2 - (l_q - l_k)^2) / 2` and
//!      `l_q - l_k = (|q|^2 - |k|^2) / (l_q + l_k)`, which keeps its precision at the norms the
//!      trunk's states reach. The chord orders candidates as the hyperbolic distance `d` does,
//!      without the logarithm D29 refuses on the device; score `offset - chord`.
//!    - Starling L2: `d = |q - k|`; score `offset - d`.
//!    - Prime route: surrogate score `-sum |q - k|`; the choice is the least zeta-phase energy
//!      between the icosian cells (and so the primes) the four address blocks of `q_t` and `k_s`
//!      name, read from a detached `120 x 120` table of cell-pair energies (claude-385).
//!    - Flock: score `-mean |q - k|`; the choice keeps the `K` best scores.
//!
//!    A Starling offset can start from the address scale: [`calibrate_next_offset`] sets it at the
//!    next forward to the median over rows of the nearest earlier distance.
//! 3. **Choice.** Hard and detached: the best score (Starling: only when it is above 0), the least
//!    energy (prime route) or the `K` best (Flock); every kind breaks ties to the latest `s`
//!    (claude-385).
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
/// when absent), the credit's margin and weight, and whether the credit reaches the stream.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct VectorReadArm {
    pub kind: VectorReadKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    pub margin: f32,
    pub credit: f32,
    pub stream_grad: bool,
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
    static CALIBRATE: Cell<bool> = const { Cell::new(false) };
    static CALIBRATED: Cell<Option<f32>> = const { Cell::new(None) };
    static PICKS: RefCell<Option<Vec<u32>>> = const { RefCell::new(None) };
    static PROBE: RefCell<Option<Vec<PayloadRow>>> = const { RefCell::new(None) };
    static PAYLOAD_SCALE: Cell<f32> = const { Cell::new(1.0) };
    static FIRST_PICK: Cell<bool> = const { Cell::new(false) };
}

/// claude-387: one row's payload reading, recorded while a probe is open ([`probing_payload`]).
#[derive(Clone, Debug, PartialEq)]
pub struct PayloadRow {
    /// `|x|`: the stream row where the read adds.
    pub stream_norm: f32,
    /// `|p|`: the row the read adds (`w ⊙` its kept picks' mean, times the eval payload scale).
    pub payload_norm: f32,
    /// `x · p`.
    pub dot: f32,
    /// The next token of each kept pick that is real, best first (one at most for every kind
    /// but Flock); empty when the row read nothing.
    pub next_ids: Vec<u32>,
}

/// claude-387: run `f` with every forward's rows recorded as [`PayloadRow`]s, in forward order,
/// and return them with its result. Eval only: the reading is detached. The previous recording
/// is restored on return.
pub fn probing_payload<T>(f: impl FnOnce() -> Result<T>) -> Result<(T, Vec<PayloadRow>)> {
    struct Restore(Option<Vec<PayloadRow>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let old = self.0.take();
            PROBE.with(|cell| *cell.borrow_mut() = old);
        }
    }
    let _restore = Restore(PROBE.with(|cell| cell.borrow_mut().replace(Vec::new())));
    let out = f()?;
    let rows = PROBE.with(|cell| cell.borrow_mut().take()).unwrap_or_default();
    Ok((out, rows))
}

/// claude-387: run `f` with the read's payload multiplied by `scale` (eval only; positive and
/// finite). At the default 1 the forward skips the multiply, so it is the unscaled forward bit
/// for bit. The previous scale is restored on return.
pub fn with_payload_scale<T>(scale: f32, f: impl FnOnce() -> Result<T>) -> Result<T> {
    if !(scale.is_finite() && scale > 0.0) {
        return Err(invalid("a payload scale is positive and finite"));
    }
    struct Restore(f32);
    impl Drop for Restore {
        fn drop(&mut self) {
            PAYLOAD_SCALE.with(|cell| cell.set(self.0));
        }
    }
    let _restore = Restore(PAYLOAD_SCALE.with(|cell| cell.replace(scale)));
    f()
}

/// claude-388: run `f` with Flock's payload copied from its first real kept pick alone (`w ⊙` that
/// pick's next-token point) instead of the mean of its kept picks: the hard copy. Eval only.
/// Every other kind already reads one pick and is unchanged. Off by default, and the default
/// forward skips it, so it is the plain forward bit for bit. The previous setting is restored on
/// return.
pub fn with_first_pick_payload<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            FIRST_PICK.with(|cell| cell.set(self.0));
        }
    }
    let _restore = Restore(FIRST_PICK.with(|cell| cell.replace(true)));
    f()
}

/// claude-385: the next training forward (inside [`collecting`]) of a Starling read sets its
/// offset from the address scale before it scores: the median over the forward's rows of the
/// nearest earlier position's distance ([`median_nearest`]), so about half the rows read from the
/// first step. An eval forward, or a forward of another kind, leaves it pending; unsetting the
/// read ([`set_vector_read`] with none) clears it.
pub fn calibrate_next_offset() {
    CALIBRATE.with(|cell| cell.set(true));
}

/// The offset the last calibration set, once; none when no calibration ran since the last take.
pub fn take_calibrated_offset() -> Option<f32> {
    CALIBRATED.with(Cell::take)
}

/// The lower median of `nearest`, each row's least distance to an earlier position, leaving out
/// rows with none (at or above `FAR / 2`); none when no row has one.
pub fn median_nearest(nearest: &[f32]) -> Option<f32> {
    let mut real: Vec<f32> = nearest
        .iter()
        .copied()
        .filter(|d| d.is_finite() && *d < FAR / 2.0)
        .collect();
    if real.is_empty() {
        return None;
    }
    real.sort_by(f32::total_cmp);
    Some(real[(real.len() - 1) / 2])
}

/// claude-385: run `f` with every forward's choices recorded, and return them with its result:
/// for each row in forward order, the chosen earlier position plus 1 (Flock: the first kept), 0
/// when the row read nothing. The previous recording is restored on return.
pub fn recording_picks<T>(f: impl FnOnce() -> Result<T>) -> Result<(T, Vec<u32>)> {
    struct Restore(Option<Vec<u32>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let old = self.0.take();
            PICKS.with(|cell| *cell.borrow_mut() = old);
        }
    }
    let _restore = Restore(PICKS.with(|cell| cell.borrow_mut().replace(Vec::new())));
    let out = f()?;
    let picks = PICKS.with(|cell| cell.borrow_mut().take()).unwrap_or_default();
    Ok((out, picks))
}

/// claude-385: run `f` with no vector read on this thread (the eval-only cut, `R^{-v}`): the trunk
/// is then the one with the flags off. The read is set again on return, and on an error too.
pub fn without_vector_read<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    struct Restore(Option<VectorRead>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let read = self.0.take();
            ACTIVE.with(|cell| *cell.borrow_mut() = read);
        }
    }
    let _restore = Restore(ACTIVE.with(|cell| cell.borrow_mut().take()));
    f()
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
    if read.is_none() {
        CALIBRATE.with(|cell| cell.set(false));
        CALIBRATED.with(|cell| cell.set(None));
    }
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
    /// The address reads the stream with its gradient (claude-381); off by default since
    /// claude-385, so the credit never reaches the trunk.
    stream_grad: bool,
    /// Prime route: the unit icosians `(120, 4)` and the cell-pair energies `(120 * 120)`,
    /// `[a * 120 + b] = sum_c |wrap(Phi[c][a] - Phi[c][b])|`.
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
            stream_grad: false,
            cells: Self::cells(kind, device)?,
        })
    }

    /// The same read with the address reading the stream with (`true`) or without its gradient.
    pub fn with_stream_grad(mut self, on: bool) -> Self {
        self.stream_grad = on;
        self
    }

    /// Whether the credit's gradient reaches the stream under the address.
    pub fn stream_grad(&self) -> bool {
        self.stream_grad
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
        let energies: Vec<f32> = (0..CELLS * CELLS)
            .map(|i| reference::cell_energy(&table, i / CELLS, i % CELLS) as f32)
            .collect();
        Ok(Some((
            Tensor::from_vec(units, (CELLS, 4), device)?,
            Tensor::from_vec(energies, CELLS * CELLS, device)?,
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

    /// The payload weight's mean absolute value.
    pub fn weight_mean_abs(&self) -> Result<f64> {
        let w = self.weight.as_tensor();
        let sum = f64::from(w.abs()?.sum_all()?.to_scalar::<f32>()?);
        Ok(sum / w.elem_count().max(1) as f64)
    }

    /// claude-387: the payload weight's coordinates, for the probe's census.
    pub fn weight_values(&self) -> Result<Vec<f32>> {
        Ok(self.weight.as_tensor().flatten_all()?.to_vec1::<f32>()?)
    }

    /// The Starling offset's value; none for the other kinds.
    pub fn offset_value(&self) -> Result<Option<f32>> {
        match &self.offset {
            Some(o) => Ok(o.as_tensor().flatten_all()?.to_vec1::<f32>()?.first().copied()),
            None => Ok(None),
        }
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
        tensors.insert(
            "stream_grad".to_owned(),
            Tensor::new(&[u32::from(self.stream_grad)], device)?,
        );
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
            // A claude-381 read, saved before the switch existed, trained with the gradient on.
            stream_grad: match tensors.get("stream_grad") {
                Some(_) => first_u32("stream_grad")? != 0,
                None => true,
            },
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

    fn cell_table(&self) -> Result<&(Tensor, Tensor)> {
        self.cells
            .as_ref()
            .ok_or_else(|| invalid("the prime-route read has no cell table"))
    }

    /// Prime route: each 4-block's nearest icosian cell, `(rows, 4)` as exact f32 integers
    /// (ties to the lowest index). Detached.
    fn cell_ids(&self, address: &Tensor) -> Result<Tensor> {
        let (units, _) = self.cell_table()?;
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
        Ok(cell.to_dtype(DType::F32)?.reshape((rows, 4))?)
    }

    /// Prime route: the zeta-phase energy `(batch, time, time)` between the cells of `q_t` and
    /// `k_s`, summed over the four blocks from the cell-pair table; no `(.., time, time, 32)`
    /// phase gap is ever held. Detached.
    fn prime_energy(&self, q: &Tensor, k: &Tensor, batch: usize, time: usize) -> Result<Tensor> {
        let (_, table) = self.cell_table()?;
        let (cq, ck) = (self.cell_ids(q)?, self.cell_ids(k)?);
        let mut energy: Option<Tensor> = None;
        for block in 0..4 {
            let a = cq.narrow(1, block, 1)?.reshape((batch, time, 1))?;
            let b = ck.narrow(1, block, 1)?.reshape((batch, 1, time))?;
            let index = a
                .affine(CELLS as f64, 0.0)?
                .broadcast_add(&b)?
                .flatten_all()?
                .to_dtype(DType::U32)?;
            let part = table.index_select(&index, 0)?.reshape((batch, time, time))?;
            energy = Some(match energy {
                None => part,
                Some(sum) => sum.add(&part)?,
            });
        }
        energy.ok_or_else(|| invalid("an address has four blocks"))
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
        // claude-385: by default the address reads the stream without its gradient, so the
        // credit and the straight-through factor train the rotations, never the trunk below.
        let source = if self.stream_grad {
            flat.clone()
        } else {
            flat.detach()
        };
        let q = self.address(&self.query_rotation, &source)?;
        let k = self.address(&self.key_rotation, &source)?;
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
        // payload rows, and the chosen position + 1 (0: none) with whether the row reads; and
        // each kept pick's next token with whether it is real, `(rows, kept)`, for the probe.
        let (score, contribution, plus1, reads, next_ids, next_real) = match self.kind {
            VectorReadKind::Flock { neighbours } => {
                let score = diff.abs()?.mean(3)?.neg()?;
                // claude-385: the K best scores, each the latest on a tie, as column + 1
                // (0: the row has fewer earlier positions).
                let kept = neighbours.min(time - 1).max(1);
                let ranked = latest_best(&score.detach(), &earlier, &columns, kept)?;
                let ranked = Tensor::stack(&ranked, 2)?.reshape((rows, kept))?;
                let real = ranked.gt(0.5)?;
                // A pick that is not real points at its window's first position, so its next
                // token stays inside the window; it carries weight 0.
                let all = ranked
                    .affine(1.0, -1.0)?
                    .maximum(0.0)?
                    .broadcast_add(&starts.reshape((rows, 1))?)?
                    .flatten_all()?
                    .to_dtype(DType::U32)?;
                let kept_keys = k.index_select(&all, 0)?.reshape((rows, kept, ADDRESS))?;
                let distance = q.unsqueeze(1)?.broadcast_sub(&kept_keys)?.abs()?.mean(2)?;
                let factor = distance.detach().sub(&distance)?.affine(1.0, 1.0)?;
                let next = all.broadcast_add(&Tensor::new(&[1u32], device)?)?;
                let next_ids = ids_t.index_select(&next, 0)?;
                let payload = table
                    .index_select(&next_ids, 0)?
                    .reshape((rows, kept, width))?;
                let real_f = real.to_dtype(DType::F32)?;
                // claude-388: the hard copy keeps the first pick's weight alone (eval only);
                // the default skips it.
                let weigh = if FIRST_PICK.with(Cell::get) {
                    let mut first = vec![0f32; kept];
                    first[0] = 1.0;
                    let first = Tensor::from_vec(first, (1, kept), device)?;
                    real_f.broadcast_mul(&first)?
                } else {
                    real_f.clone()
                };
                let count = weigh.sum_keepdim(1)?.maximum(1.0)?;
                let contribution = payload
                    .broadcast_mul(&weigh.mul(&factor)?.unsqueeze(2)?)?
                    .sum(1)?
                    .broadcast_div(&count)?;
                let reads = real_f.narrow(1, 0, 1)?.squeeze(1)?;
                let plus1 = ranked.narrow(1, 0, 1)?.squeeze(1)?;
                let next_ids = next_ids.reshape((rows, kept))?;
                (score, contribution, plus1, reads, next_ids, real_f)
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
                            .ok_or_else(|| invalid("a Starling read has an offset"))?;
                        // claude-385: the offset from the address scale, before the first score,
                        // at a training forward only: an eval forward leaves it pending.
                        if COLLECT.with(Cell::get) && CALIBRATE.with(|cell| cell.replace(false)) {
                            let far = Tensor::full(FAR, d.shape(), device)?;
                            let nearest = earlier
                                .where_cond(&d.detach(), &far)?
                                .min(2)?
                                .flatten_all()?
                                .to_vec1::<f32>()?;
                            let median = median_nearest(&nearest).ok_or_else(|| {
                                invalid("no earlier position to start the Starling offset from")
                            })?;
                            offset.set(&Tensor::new(&[median], device)?)?;
                            CALIBRATED.with(|cell| cell.set(Some(median)));
                        }
                        let offset = offset.as_tensor().reshape((1, 1, 1))?;
                        (d.neg()?.broadcast_add(&offset)?, d)
                    }
                };
                let (plus1, reads) = if kind == VectorReadKind::PrimeRoute {
                    let energy = self.prime_energy(&q, &k, batch, time)?;
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
                let next_ids = ids_t.index_select(&next, 0)?;
                let payload = table.index_select(&next_ids, 0)?;
                let gain = factor.mul(&reads.reshape(rows)?)?.unsqueeze(1)?;
                let next_real = reads.reshape((rows, 1))?;
                let next_ids = next_ids.reshape((rows, 1))?;
                (score, payload.broadcast_mul(&gain)?, plus1, reads, next_ids, next_real)
            }
        };
        // claude-387: the eval payload scale, skipped at its default 1.
        let scale = PAYLOAD_SCALE.with(Cell::get);
        let contribution = if scale == 1.0 {
            contribution
        } else {
            contribution.affine(f64::from(scale), 0.0)?
        };
        let added = contribution.broadcast_mul(&weight)?;
        let out = flat.add(&added)?.reshape((batch, time, width))?;
        // claude-387: each row's stream and payload, while a probe is open.
        if PROBE.with(|cell| cell.borrow().is_some()) {
            let (x, p) = (flat.detach(), added.detach());
            let norm = |t: &Tensor| -> Result<Vec<f32>> {
                Ok(t.sqr()?.sum(1)?.sqrt()?.to_vec1::<f32>()?)
            };
            let (xn, pn) = (norm(&x)?, norm(&p)?);
            let dot = x.mul(&p)?.sum(1)?.to_vec1::<f32>()?;
            let ids_kept = next_ids.to_vec2::<u32>()?;
            let real_kept = next_real.to_dtype(DType::F32)?.to_vec2::<f32>()?;
            let found: Vec<PayloadRow> = (0..rows)
                .map(|r| PayloadRow {
                    stream_norm: xn[r],
                    payload_norm: pn[r],
                    dot: dot[r],
                    next_ids: ids_kept[r]
                        .iter()
                        .zip(&real_kept[r])
                        .filter(|(_, &real)| real > 0.5)
                        .map(|(&id, _)| id)
                        .collect(),
                })
                .collect();
            PROBE.with(|cell| {
                if let Some(all) = cell.borrow_mut().as_mut() {
                    all.extend(found);
                }
            });
        }
        // claude-385: the choices, row by row, while a recording is open.
        if PICKS.with(|cell| cell.borrow().is_some()) {
            let picks = plus1.reshape(rows)?.to_dtype(DType::U32)?.to_vec1::<u32>()?;
            PICKS.with(|cell| {
                if let Some(all) = cell.borrow_mut().as_mut() {
                    all.extend(picks);
                }
            });
        }
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

/// The Minkowski chord between the lifts `(l, v)`, `l = sqrt(1 + |v|^2)`, of `q_t` and `k_s` on the
/// hyperboloid, from the squared difference `n2` `(batch, time, time)` and the squared norms only:
/// `e = cosh d - 1 = (n2 - (l_q - l_k)^2) / 2`, with `l_q - l_k = (|q|^2 - |k|^2) / (l_q + l_k)`;
/// chord `sqrt(2 e)`. claude-385: the claude-381 form `l_q l_k - (|q|^2 + |k|^2 - n2) / 2 - 1` is
/// the same number, but subtracts two terms of order `|q| |k|`, which leaves nothing of `e` in f32
/// once the states' norms reach the hundreds. The chord rises with the hyperbolic distance
/// `d = arcosh(1 + e)`, so it orders candidates exactly as `d` does; `d` itself needs a logarithm,
/// whose kernel D29 refuses on the device.
fn lorentz_from_distances(q3: &Tensor, k3: &Tensor, n2: &Tensor) -> Result<Tensor> {
    let qn = q3.sqr()?.sum(2)?;
    let kn = k3.sqr()?.sum(2)?;
    let lift_q = qn.affine(1.0, 1.0)?.sqrt()?.unsqueeze(2)?;
    let lift_k = kn.affine(1.0, 1.0)?.sqrt()?.unsqueeze(1)?;
    let lift_gap = qn
        .unsqueeze(2)?
        .broadcast_sub(&kn.unsqueeze(1)?)?
        .div(&lift_q.broadcast_add(&lift_k)?)?;
    let e = n2
        .sub(&lift_gap.sqr()?)?
        .affine(0.5, 0.0)?
        .maximum(FLOOR)?;
    Ok(e.affine(2.0, 0.0)?.sqrt()?)
}

/// claude-385: over the `mask`ed `values` `(batch, time, time)` (detached), the `kept` largest per
/// row, best first, each the latest column on a tie, as `column + 1` f32 `(batch, time)`, 0 once
/// the row has no column left.
fn latest_best(
    values: &Tensor,
    mask: &Tensor,
    columns: &Tensor,
    kept: usize,
) -> Result<Vec<Tensor>> {
    let mut left = mask.clone();
    let none = mask.zeros_like()?;
    let mut ranked = Vec::with_capacity(kept);
    for _ in 0..kept {
        let (_, plus1) = latest_extreme(values, &left, true, columns)?;
        left = columns
            .broadcast_eq(&plus1.unsqueeze(2)?)?
            .where_cond(&none, &left)?;
        ranked.push(plus1);
    }
    Ok(ranked)
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
    /// with its floor under the excess `e = cosh d - 1`, written as the tensor code writes it.
    pub fn lorentz_chord(q: &[f64], k: &[f64]) -> f64 {
        let (qn, kn, n2) = (squared(q), squared(k), squared_gap(q, k));
        let (lq, lk) = ((1.0 + qn).sqrt(), (1.0 + kn).sqrt());
        let gap = (qn - kn) / (lq + lk);
        let e = ((n2 - gap * gap) / 2.0).max(FLOOR);
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
        /// The distance the straight-through factor carries for row `t` choosing `s`: the
        /// Starling score's distance, the prime route's L1 surrogate, Flock's mean L1.
        pub fn distance(&self, x: &[f64], width: usize, t: usize, s: usize) -> f64 {
            let q = address(self.query_rotation, &x[t * width..(t + 1) * width]);
            let k = address(self.key_rotation, &x[s * width..(s + 1) * width]);
            let l1: f64 = q.iter().zip(&k).map(|(a, b)| (a - b).abs()).sum();
            match self.kind {
                VectorReadKind::StarlingLorentz => lorentz_chord(&q, &k),
                VectorReadKind::StarlingL2 => squared_gap(&q, &k).max(FLOOR).sqrt(),
                VectorReadKind::PrimeRoute => l1,
                VectorReadKind::Flock { .. } => l1 / ADDRESS as f64,
            }
        }

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
                        // Best first; claude-385: the latest first among equal scores.
                        order.sort_by(|&a, &b| score(t, b).total_cmp(&score(t, a)).then(b.cmp(&a)));
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
    fn the_offset_start_is_the_lower_median_of_the_rows_with_an_earlier_position() {
        // claude-385: rows with no earlier position (FAR) are left out; an even count takes the
        // lower of the middle two; no row at all gives none.
        assert_eq!(median_nearest(&[FAR, 3.0, 1.0, 2.0, FAR]), Some(2.0));
        assert_eq!(median_nearest(&[4.0, 1.0, 3.0, 2.0]), Some(2.0));
        assert_eq!(median_nearest(&[FAR, 7.5]), Some(7.5));
        assert_eq!(median_nearest(&[FAR, FAR]), None);
        assert_eq!(median_nearest(&[]), None);
        assert_eq!(median_nearest(&[f32::NAN, 5.0]), Some(5.0));
    }

    #[test]
    fn the_cut_and_the_pick_recording_restore_what_they_found() {
        // claude-385: the cut unsets the read only inside, an error inside still sets it again,
        // and a recording with no forward returns no picks and leaves no recording open.
        let cpu = Device::Cpu;
        let read = VectorRead::new(VectorReadKind::PrimeRoute, 1, 16, 1.0, 1.0, &cpu).unwrap();
        set_vector_read(Some(read));
        let inside = without_vector_read(|| Ok(vector_read().is_none())).unwrap();
        assert!(inside);
        assert!(vector_read().is_some());
        let failed: Result<()> = without_vector_read(|| Err(invalid("inside")));
        assert!(failed.is_err());
        assert!(vector_read().is_some());
        let ((), picks) = recording_picks(|| Ok(())).unwrap();
        assert!(picks.is_empty());
        assert!(PICKS.with(|cell| cell.borrow().is_none()));
        calibrate_next_offset();
        set_vector_read(None);
        assert!(!CALIBRATE.with(Cell::get));
        assert!(vector_read().is_none());
    }

    #[test]
    fn the_payload_scale_and_probe_restore_what_they_found() {
        // claude-387: a scale that is not positive and finite is refused; a scale is set only
        // inside, and set back on an error inside too; a probe with no forward returns no rows.
        for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(with_payload_scale(bad, || Ok(())).is_err(), "{bad}");
        }
        let inside = with_payload_scale(16.0, || Ok(PAYLOAD_SCALE.with(Cell::get))).unwrap();
        assert_eq!(inside, 16.0);
        assert_eq!(PAYLOAD_SCALE.with(Cell::get), 1.0);
        let failed: Result<()> = with_payload_scale(4.0, || Err(invalid("inside")));
        assert!(failed.is_err());
        assert_eq!(PAYLOAD_SCALE.with(Cell::get), 1.0);
        let ((), rows) = probing_payload(|| Ok(())).unwrap();
        assert!(rows.is_empty());
        assert!(PROBE.with(|cell| cell.borrow().is_none()));
        // claude-388: the hard copy is off by default, on only inside, and off again after an
        // error inside.
        assert!(!FIRST_PICK.with(Cell::get));
        assert!(with_first_pick_payload(|| Ok(FIRST_PICK.with(Cell::get))).unwrap());
        assert!(!FIRST_PICK.with(Cell::get));
        let failed: Result<()> = with_first_pick_payload(|| Err(invalid("inside")));
        assert!(failed.is_err());
        assert!(!FIRST_PICK.with(Cell::get));
    }

    /// A small deterministic generator for the device tests below, in [-1, 1).
    #[cfg(feature = "cuda")]
    fn units(seed: u64, n: usize) -> Vec<f64> {
        let mut state = seed;
        (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
            })
            .collect()
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_lorentz_chord_keeps_the_hyperbolic_order_at_the_state_scale() {
        // claude-385: a query at norm 1000 (the trunk's states reach the hundreds) and 16 keys
        // displaced from it, orthogonally, by 0.05 x 1.35^s. The f32 chord must order the keys
        // as the f64 hyperbolic distance of the same f32 points does, and keep 1e-3 of its value;
        // the claude-381 form, a difference of two terms of order 1e6, keeps neither.
        // The device: the CPU backend refuses model compute (D24/D29).
        let cpu = Device::new_cuda(0).unwrap();
        let u = units(3855, ADDRESS);
        let norm = u.iter().map(|v| v * v).sum::<f64>().sqrt();
        let query: Vec<f64> = u.iter().map(|v| 1000.0 * v / norm).collect();
        let keys = 16;
        let mut rows: Vec<f32> = Vec::with_capacity((keys + 1) * ADDRESS);
        for s in 0..keys {
            let mut v = units(3856 + s as u64, ADDRESS);
            let along: f64 = v.iter().zip(&u).map(|(a, b)| a * b).sum::<f64>() / norm;
            v.iter_mut().zip(&u).for_each(|(a, b)| *a -= along * b / norm);
            let size = v.iter().map(|a| a * a).sum::<f64>().sqrt();
            let step = 0.05 * 1.35f64.powi(s as i32);
            rows.extend(query.iter().zip(&v).map(|(q, a)| (q + step * a / size) as f32));
        }
        rows.extend(query.iter().map(|&q| q as f32));
        let time = keys + 1;
        let points = Tensor::from_vec(rows.clone(), (1, time, ADDRESS), &cpu).unwrap();
        let n2 = points
            .unsqueeze(2)
            .unwrap()
            .broadcast_sub(&points.unsqueeze(1).unwrap())
            .unwrap()
            .sqr()
            .unwrap()
            .sum(3)
            .unwrap();
        let chord = lorentz_from_distances(&points, &points, &n2)
            .unwrap()
            .narrow(1, keys, 1)
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let exact: Vec<f64> = rows.iter().map(|&v| f64::from(v)).collect();
        let at = |s: usize| &exact[s * ADDRESS..(s + 1) * ADDRESS];
        let distance: Vec<f64> = (0..keys).map(|s| lorentz_distance(at(keys), at(s))).collect();
        let order = |v: &[f64]| -> Vec<usize> {
            let mut o: Vec<usize> = (0..keys).collect();
            o.sort_by(|&a, &b| v[a].total_cmp(&v[b]));
            o
        };
        let device: Vec<f64> = chord[..keys].iter().map(|&c| f64::from(c)).collect();
        assert_eq!(order(&device), order(&distance), "{device:?}");
        for s in 0..keys {
            let want = (2.0 * (distance[s].cosh() - 1.0)).sqrt();
            assert!(
                (device[s] - want).abs() <= 1e-3 * want,
                "key {s}: {} vs {want}",
                device[s]
            );
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_cell_pair_table_is_the_reference_energy() {
        // claude-385: the prime route reads a (120 x 120) table, (batch, time, time) per block,
        // and its energies are the f64 reference's between the same cells; repeated rows tie.
        let cpu = Device::new_cuda(0).unwrap();
        let read = VectorRead::new(VectorReadKind::PrimeRoute, 1, 16, 1.0, 1.0, &cpu).unwrap();
        let (batch, time) = (2, 5);
        let rows = batch * time;
        let q = units(3857, rows * ADDRESS);
        let mut k = units(3858, rows * ADDRESS);
        // Row 3's key is row 1's query: the same four cells.
        k.splice(3 * ADDRESS..4 * ADDRESS, q[ADDRESS..2 * ADDRESS].to_vec());
        let tensor = |v: &[f64]| {
            let v: Vec<f32> = v.iter().map(|&x| x as f32).collect();
            Tensor::from_vec(v, (rows, ADDRESS), &cpu).unwrap()
        };
        let energy = read.prime_energy(&tensor(&q), &tensor(&k), batch, time).unwrap();
        assert_eq!(energy.dims(), [batch, time, time]);
        let energy = energy.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let (unit_cells, phases) = (icosians(), phase_table());
        let rounded = |v: &[f64], r: usize| -> [usize; 4] {
            std::array::from_fn(|j| {
                let block: Vec<f64> = v[r * ADDRESS + 4 * j..r * ADDRESS + 4 * j + 4]
                    .iter()
                    .map(|&x| f64::from(x as f32))
                    .collect();
                cell(&unit_cells, &block)
            })
        };
        for b in 0..batch {
            for t in 0..time {
                for s in 0..time {
                    let want = energy_of(&phases, rounded(&q, b * time + t), rounded(&k, b * time + s));
                    let got = f64::from(energy[(b * time + t) * time + s]);
                    assert!((got - want).abs() < 1e-4, "({b}, {t}, {s}): {got} vs {want}");
                }
            }
        }
        // Query row 1 against key rows 1 and 3 of window 0: row 3 holds row 1's own cells.
        assert_eq!(energy_of(&phases, rounded(&q, 1), rounded(&k, 3)), 0.0);
        assert_eq!(energy[time + 3], 0.0);
        let (_, table) = read.cell_table().unwrap();
        let table = table.to_vec1::<f32>().unwrap();
        for a in 0..CELLS {
            assert_eq!(table[a * CELLS + a], 0.0);
            for b in 0..CELLS {
                assert_eq!(table[a * CELLS + b], cell_energy(&phases, a, b) as f32);
            }
        }
    }

    /// The reference energy between two addresses' cells (the reference's `energy`).
    #[cfg(feature = "cuda")]
    fn energy_of(phases: &[Vec<f64>], q: [usize; 4], k: [usize; 4]) -> f64 {
        energy(phases, q, k)
    }

    #[test]
    fn a_saved_read_keeps_its_stream_switch() {
        // claude-385: the switch is saved; a claude-381 read, saved before it, trained with it on.
        let cpu = Device::Cpu;
        let dir = std::env::temp_dir().join(format!("uor-vread-385-{}", std::process::id()));
        for on in [false, true] {
            let read = VectorRead::new(VectorReadKind::StarlingL2, 1, 16, 1.0, 1.0, &cpu)
                .unwrap()
                .with_stream_grad(on);
            read.save(&dir).unwrap();
            let back = VectorRead::load(&dir, &cpu).unwrap().expect("a read");
            assert_eq!(back.stream_grad(), on);
        }
        let mut tensors = candle_core::safetensors::load(dir.join(FILE), &cpu).unwrap();
        tensors.remove("stream_grad");
        candle_core::safetensors::save(&tensors, dir.join(FILE)).unwrap();
        let old = VectorRead::load(&dir, &cpu).unwrap().expect("a read");
        assert!(old.stream_grad());
        std::fs::remove_file(dir.join(FILE)).unwrap();
        std::fs::remove_dir(&dir).unwrap();
        assert!(!VectorRead::new(VectorReadKind::PrimeRoute, 1, 16, 1.0, 1.0, &cpu)
            .unwrap()
            .stream_grad());
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

        /// claude-385: the recorded picks are the reference's choices, row by row (Flock: the
        /// first kept), 0 where a row reads nothing.
        #[test]
        fn the_recorded_picks_are_the_reference_choices() -> Result<()> {
            for kind in kinds() {
                let f = fixture(kind, DEFAULT_MARGIN, 385)?;
                if let Some(o) = &f.read.offset {
                    // Midway between the two middle rows' nearest earlier distances (f64), so
                    // some rows read and some do not, none of them near the gate.
                    let (qr, kr, w) = (
                        host(f.read.query_rotation.as_tensor())?,
                        host(f.read.key_rotation.as_tensor())?,
                        host(f.read.weight.as_tensor())?,
                    );
                    let r = Reference {
                        kind,
                        margin: f64::from(f.read.margin),
                        offset: 0.0,
                        query_rotation: &qr,
                        key_rotation: &kr,
                        weight: &w,
                    };
                    let x = host(&f.x)?;
                    let mut nearest: Vec<f64> = (1..T)
                        .map(|t| {
                            (0..t)
                                .map(|s| r.distance(&x, W, t, s))
                                .fold(f64::INFINITY, f64::min)
                        })
                        .collect();
                    nearest.sort_by(f64::total_cmp);
                    let mid = nearest.len() / 2;
                    let offset = (nearest[mid - 1] + nearest[mid]) / 2.0;
                    o.set(&Tensor::new(&[offset as f32], o.device())?)?;
                }
                let (_, picks) = recording_picks(|| f.read.apply(&f.x, &IDS, &f.table))?;
                let want: Vec<u32> = reference_of(&f)?
                    .chosen
                    .iter()
                    .map(|c| c.first().map_or(0, |&s| s as u32 + 1))
                    .collect();
                assert!(want.iter().any(|&p| p > 0), "{kind:?}: no row read");
                if kind.is_starling() {
                    assert!(want[1..].contains(&0), "{kind:?}: every row read");
                }
                assert_eq!(picks, want, "{kind:?}");
            }
            Ok(())
        }

        /// claude-385: a calibrated Starling offset is the lower median of the rows' nearest
        /// earlier distances: strictly below it, (rows - 1) / 2 of the 5 rows read, and the
        /// reference at a hair above it reads one row more.
        #[test]
        fn the_calibrated_offset_is_the_median_nearest_distance() -> Result<()> {
            for kind in [VectorReadKind::StarlingLorentz, VectorReadKind::StarlingL2] {
                let f = fixture(kind, DEFAULT_MARGIN, 3851)?;
                let _ = take_stats();
                calibrate_next_offset();
                // An eval forward (no collecting) leaves the calibration pending and the offset.
                f.read.apply(&f.x, &IDS, &f.table)?;
                assert!(CALIBRATE.with(Cell::get), "{kind:?}: an eval forward took it");
                assert_eq!(take_calibrated_offset(), None, "{kind:?}");
                assert_eq!(f.read.offset_value()?, Some(OFFSET_INIT), "{kind:?}");
                let _ = take_stats();
                let out = host(&collecting(|| f.read.apply(&f.x, &IDS, &f.table))?)?;
                clear_credit();
                let offset = take_calibrated_offset().expect("a calibration");
                assert!(!CALIBRATE.with(Cell::get), "{kind:?}: still pending");
                assert_eq!(f.read.offset_value()?, Some(offset), "{kind:?}");
                assert_ne!(offset, OFFSET_INIT, "{kind:?}");
                // The median row's own score is 0 on the device, so it does not read; the f64
                // reference is read a hair either side of the f32 offset.
                let o = f.read.offset.as_ref().expect("offset");
                let at = |value: f32| -> Result<ReferenceOutput> {
                    o.set(&Tensor::new(&[value], o.device())?)?;
                    reference_of(&f)
                };
                let below = at(offset * (1.0 - 1e-4))?;
                for (i, (a, b)) in out.iter().zip(&below.out).enumerate() {
                    assert!((a - b).abs() < 1e-5, "{kind:?} [{i}]: {a} vs {b}");
                }
                let reads = below.chosen.iter().filter(|c| !c.is_empty()).count();
                assert_eq!(reads, (T - 2) / 2, "{kind:?}: rows below the median");
                let stats = take_stats().expect("stats");
                assert_eq!(stats.read_rows.to_scalar::<f32>()?, reads as f32, "{kind:?}");
                let above = at(offset * (1.0 + 1e-4))?;
                let above = above.chosen.iter().filter(|c| !c.is_empty()).count();
                assert_eq!(above, reads + 1, "{kind:?}: the median row");
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

        /// claude-385, repair 1: six points on a circle of radius 1000 at 0, 40, 85, 135, 86 and
        /// 190 degrees. Row 4's nearest earlier point is row 2, one degree away (a chord of 17.5),
        /// and a positive (`IDS[3] == IDS[5]`); the rows' nearest distances are 684, 765, 845,
        /// 17.5 and 923. At the fixed start 2.5 no row reads; calibrated, the offset is the
        /// median 765, and rows 1 and 4 read their nearest, row 4 the positive.
        #[test]
        fn a_known_positive_opens_the_calibrated_read_and_is_chosen() -> Result<()> {
            let device = Device::new_cuda(0)?;
            let mut x = vec![0f32; T * W];
            for (t, deg) in [0.0f64, 40.0, 85.0, 135.0, 86.0, 190.0].iter().enumerate() {
                x[t * W] = (1000.0 * deg.to_radians().cos()) as f32;
                x[t * W + 1] = (1000.0 * deg.to_radians().sin()) as f32;
            }
            let x = Tensor::from_vec(x, (1, T, W), &device)?;
            let table = Tensor::ones((V, W), DType::F32, &device)?;
            assert_eq!(IDS[3], IDS[5]);
            for kind in [VectorReadKind::StarlingLorentz, VectorReadKind::StarlingL2] {
                let read = VectorRead::new(kind, 1, W, DEFAULT_MARGIN, 1.0, &device)?;
                let (_, fixed) = recording_picks(|| read.apply(&x, &IDS, &table))?;
                assert_eq!(fixed, vec![0; T], "{kind:?}: the fixed start read");
                calibrate_next_offset();
                let (_, picks) =
                    recording_picks(|| collecting(|| read.apply(&x, &IDS, &table)))?;
                clear_credit();
                let _ = take_stats();
                let offset = take_calibrated_offset().expect("a calibration");
                assert!((offset - 765.4).abs() < 0.5, "{kind:?}: offset {offset}");
                assert_eq!(picks, vec![0, 1, 0, 0, 3, 0], "{kind:?}");
            }
            Ok(())
        }

        /// claude-385, repair 2: with the stream stop (the default) the credit sends the stream
        /// under the address no gradient at all, and the read's rotations still learn; with the
        /// switch on, the stream gets one.
        #[test]
        fn the_stream_stop_leaves_the_trunk_no_credit_gradient() -> Result<()> {
            for kind in kinds() {
                for on in [false, true] {
                    let f = fixture(kind, 5.0, 3852)?;
                    let read = f.read.clone().with_stream_grad(on);
                    let x = Var::from_tensor(&f.x)?;
                    clear_credit();
                    collecting(|| read.apply(x.as_tensor(), &IDS, &f.table))?;
                    let grads = take_credit().ok_or_else(|| invalid("no credit"))?.backward()?;
                    let trunk = match grads.get(x.as_tensor()) {
                        Some(g) => host(g)?,
                        None => vec![0.0; T * W],
                    };
                    let largest = trunk.iter().fold(0f64, |m, v| m.max(v.abs()));
                    if on {
                        assert!(largest > 0.0, "{kind:?}: no stream gradient with the switch on");
                    } else {
                        assert_eq!(largest, 0.0, "{kind:?}: the credit reached the stream");
                    }
                    let turn = grads
                        .get(read.query_rotation.as_tensor())
                        .ok_or_else(|| invalid("no rotation gradient"))?;
                    assert!(host(turn)?.iter().any(|v| *v != 0.0), "{kind:?} {on}");
                }
            }
            clear_credit();
            Ok(())
        }

        /// claude-385, repair 7: the straight-through factor `1 - (d - d.detach())` sends each
        /// chosen pair's payload gradient as minus the chosen distance's gradient. With the
        /// choices held, `L = sum(out * c)` has `dL/dtheta = -sum_pairs g_pair dd_pair/dtheta`,
        /// `g_pair` the pair's share of `sum_j w_j table[ids[s+1]]_j c[t, j]`, differenced on the
        /// f64 reference distance.
        #[test]
        fn the_payload_gradient_is_minus_the_chosen_distance_gradient() -> Result<()> {
            for kind in kinds() {
                let f = fixture(kind, DEFAULT_MARGIN, 3853)?;
                let cot = Lcg(3853).vec(T * W, 1.0);
                let c = Tensor::from_vec(f32s(&cot), (1, T, W), f.x.device())?;
                let grads = f.read.apply(&f.x, &IDS, &f.table)?.mul(&c)?.sum_all()?.backward()?;
                let base = reference_of(&f)?;
                let (x, table, w) = (host(&f.x)?, host(&f.table)?, host(f.read.weight.as_tensor())?);
                let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
                for (t, picks) in base.chosen.iter().enumerate() {
                    for &s in picks {
                        let row = IDS[s + 1] as usize * W;
                        let g: f64 = (0..W).map(|j| w[j] * table[row + j] * cot[t * W + j]).sum();
                        pairs.push((t, s, g / picks.len() as f64));
                    }
                }
                assert!(!pairs.is_empty(), "{kind:?}: no row read");
                let offset = match &f.read.offset {
                    Some(o) => host(o.as_tensor())?[0],
                    None => 0.0,
                };
                let held = |f: &Fixture| -> Result<f64> {
                    let (qr, kr) = (
                        host(f.read.query_rotation.as_tensor())?,
                        host(f.read.key_rotation.as_tensor())?,
                    );
                    let r = Reference {
                        kind,
                        margin: f64::from(f.read.margin),
                        offset,
                        query_rotation: &qr,
                        key_rotation: &kr,
                        weight: &w,
                    };
                    Ok(pairs.iter().map(|&(t, s, g)| g * r.distance(&x, W, t, s)).sum())
                };
                for (var, entries) in [
                    (&f.read.query_rotation, [1usize, 5, 10]),
                    (&f.read.key_rotation, [2, 7, 13]),
                ] {
                    let grad = host(grads.get(var.as_tensor()).expect("a gradient"))?;
                    let shape = var.as_tensor().shape().clone();
                    let flat = host(var.as_tensor())?;
                    for &e in &entries {
                        let h = 1e-3;
                        let at = |delta: f64| -> Result<f64> {
                            let mut v = flat.clone();
                            v[e] += delta;
                            var.set(&Tensor::from_vec(f32s(&v), &shape, f.x.device())?)?;
                            held(&f)
                        };
                        let fd = (at(h)? - at(-h)?) / (2.0 * h);
                        var.set(&Tensor::from_vec(f32s(&flat), &shape, f.x.device())?)?;
                        let (want, got) = (-fd, grad[e]);
                        let tol = 2e-2 * want.abs().max(got.abs()) + 1e-4;
                        assert!(
                            (want - got).abs() <= tol,
                            "{kind:?} entry {e}: want {want} got {got}"
                        );
                    }
                }
            }
            Ok(())
        }

        /// claude-385, repair 8: every row equal, so every earlier position scores the same; each
        /// kind chooses the latest (Flock keeps the latest K), on the device and in the reference.
        #[test]
        fn equal_scores_take_the_latest_position_for_every_kind() -> Result<()> {
            for kind in kinds() {
                let f = fixture(kind, DEFAULT_MARGIN, 3854)?;
                let x = f.x.narrow(1, 0, 1)?.broadcast_as((1, T, W))?.contiguous()?;
                let (out, picks) = recording_picks(|| f.read.apply(&x, &IDS, &f.table))?;
                assert_eq!(picks, (0..T as u32).collect::<Vec<u32>>(), "{kind:?}");
                let tied = Fixture {
                    read: f.read.clone(),
                    x: x.clone(),
                    table: f.table.clone(),
                };
                let want = reference_of(&tied)?;
                let keep = match kind {
                    VectorReadKind::Flock { neighbours } => neighbours,
                    _ => 1,
                };
                for (t, chosen) in want.chosen.iter().enumerate() {
                    let latest: Vec<usize> = (0..t).rev().take(keep).collect();
                    assert_eq!(chosen, &latest, "{kind:?} row {t}");
                }
                for (i, (a, b)) in host(&out)?.iter().zip(&want.out).enumerate() {
                    assert!((a - b).abs() < 1e-5, "{kind:?} [{i}]: {a} vs {b}");
                }
            }
            Ok(())
        }

        /// claude-385, repair 8: every rank Flock keeps, not only the first, is the latest left
        /// on equal values, and a row with fewer earlier positions than K gets 0 past them.
        #[test]
        fn every_kept_rank_takes_the_latest_left_on_a_tie() -> Result<()> {
            let device = Device::new_cuda(0)?;
            let time = 5;
            let earlier: Vec<u8> = (0..time * time)
                .map(|i| u8::from(i % time < i / time))
                .collect();
            let earlier = Tensor::from_vec(earlier, (1, time, time), &device)?;
            let columns =
                Tensor::arange(1f32, (time + 1) as f32, &device)?.reshape((1, 1, time))?;
            let values = Tensor::ones((1, time, time), DType::F32, &device)?;
            let ranked = latest_best(&values, &earlier, &columns, 3)?;
            for (rank, plus1) in ranked.iter().enumerate() {
                let got = plus1.flatten_all()?.to_vec1::<f32>()?;
                let want: Vec<f32> = (0..time)
                    .map(|t| if rank < t { (t - rank) as f32 } else { 0.0 })
                    .collect();
                assert_eq!(got, want, "rank {rank}");
            }
            Ok(())
        }

        /// claude-387: inside a probe, and at the eval scale 1, the forward is the plain forward
        /// bit for bit. Each probe row holds the stream row's norm, the added row's norm and
        /// their dot product, and the next tokens of the reference's chosen positions, best
        /// first; at scale 4 the added row is four times the plain one.
        #[test]
        fn the_payload_probe_reads_the_forward_and_leaves_it_bit_for_bit() -> Result<()> {
            let bits = |t: &Tensor| -> Result<Vec<u32>> {
                Ok(t.flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .map(|v| v.to_bits())
                    .collect())
            };
            let norm = |v: &[f64]| v.iter().map(|a| a * a).sum::<f64>().sqrt();
            for kind in kinds() {
                let f = fixture(kind, DEFAULT_MARGIN, 387)?;
                let forward = || f.read.apply(&f.x, &IDS, &f.table);
                let plain = forward()?;
                let (probed, rows) = probing_payload(forward)?;
                let unit = with_payload_scale(1.0, forward)?;
                assert_eq!(bits(&probed)?, bits(&plain)?, "{kind:?}");
                assert_eq!(bits(&unit)?, bits(&plain)?, "{kind:?}");
                assert!(PROBE.with(|cell| cell.borrow().is_none()));
                assert_eq!(rows.len(), T, "{kind:?}");
                let (x, out) = (host(&f.x)?, host(&plain)?);
                let want = reference_of(&f)?;
                let mut read_rows = 0;
                for (r, row) in rows.iter().enumerate() {
                    let xs = &x[r * W..(r + 1) * W];
                    let added: Vec<f64> = out[r * W..(r + 1) * W]
                        .iter()
                        .zip(xs)
                        .map(|(o, a)| o - a)
                        .collect();
                    let dot: f64 = xs.iter().zip(&added).map(|(a, p)| a * p).sum();
                    let close = |got: f32, want: f64| {
                        (f64::from(got) - want).abs() <= 1e-4 * (1.0 + want.abs())
                    };
                    assert!(close(row.stream_norm, norm(xs)), "{kind:?} row {r}");
                    assert!(close(row.payload_norm, norm(&added)), "{kind:?} row {r}");
                    assert!(close(row.dot, dot), "{kind:?} row {r}: {} vs {dot}", row.dot);
                    let next: Vec<u32> = want.chosen[r].iter().map(|&s| IDS[s + 1]).collect();
                    assert_eq!(row.next_ids, next, "{kind:?} row {r}");
                    read_rows += usize::from(!next.is_empty());
                }
                assert!(read_rows > 0, "{kind:?}: the fixture reads somewhere");
                let four = host(&with_payload_scale(4.0, forward)?)?;
                for (i, ((a, o), x)) in four.iter().zip(&out).zip(&x).enumerate() {
                    let want = 4.0 * (o - x);
                    assert!(((a - x) - want).abs() < 1e-4, "{kind:?} [{i}]: {} vs {want}", a - x);
                }
                assert_eq!(PAYLOAD_SCALE.with(Cell::get), 1.0);
            }
            Ok(())
        }

        /// claude-388: under the hard copy, Flock's added row is `w ⊙` the table row of its
        /// first chosen position's next token alone, and the fixture has a row where that differs
        /// from the mean the plain forward adds; every other kind is the plain forward bit for
        /// bit. Off again after it returns.
        #[test]
        fn the_hard_copy_adds_the_first_pick_alone() -> Result<()> {
            let bits = |t: &Tensor| -> Result<Vec<u32>> {
                Ok(t.flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .map(|v| v.to_bits())
                    .collect())
            };
            for kind in kinds() {
                let f = fixture(kind, DEFAULT_MARGIN, 388)?;
                let forward = || f.read.apply(&f.x, &IDS, &f.table);
                let plain = forward()?;
                let copy = with_first_pick_payload(forward)?;
                assert!(!FIRST_PICK.with(Cell::get));
                if !matches!(kind, VectorReadKind::Flock { .. }) {
                    assert_eq!(bits(&copy)?, bits(&plain)?, "{kind:?}");
                    continue;
                }
                let (x, out, table) = (host(&f.x)?, host(&copy)?, host(&f.table)?);
                let (w, plain) = (host(f.read.weight.as_tensor())?, host(&plain)?);
                let want = reference_of(&f)?;
                let mut differs = 0;
                for r in 0..T {
                    let first = want.chosen[r].first().map(|&s| IDS[s + 1] as usize);
                    let mean_rows = want.chosen[r].len();
                    for c in 0..W {
                        let i = r * W + c;
                        let added = first.map_or(0.0, |t| w[c] * table[t * W + c]);
                        let got = out[i] - x[i];
                        assert!((got - added).abs() < 1e-5, "row {r} [{c}]: {got} vs {added}");
                    }
                    if mean_rows > 1 {
                        let gap = (0..W)
                            .map(|c| (out[r * W + c] - plain[r * W + c]).abs())
                            .fold(0.0, f64::max);
                        differs += usize::from(gap > 1e-4);
                    }
                }
                assert!(differs > 0, "the fixture has a row whose picks' mean is not the first");
            }
            Ok(())
        }
    }
}
