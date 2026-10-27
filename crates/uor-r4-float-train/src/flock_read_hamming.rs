//! The induction-keyed Hamming read inside the in-space trunk (in-space-trunk-to-text, claude-146).
//!
//! The trunk is a selective quaternion recurrence: a token 16 back moves the prediction by 1–2%
//! of what the last token does. Reads keyed on the trunk's own state never learned where to look
//! (claude-144, claude-145, and this read's first form). The project's 26 Sep retrieval head
//! (`archive/integration/geometric-attention-2026-09-26.md` §3) recalled 100% at 512 pairs with
//! Hamming codes and hard top-1 because its key named the *previous* token and its query the
//! current one: an earlier copy of the current token is then an exact match from the first step.
//! This read builds that in ([RESEARCH](../../../docs/loop/tasks/in-space-trunk-to-text/claude-146/RESEARCH.md)).
//!
//! After layer 2 of 6, inside the stream, and on the final norm's output, the points the nearest
//! readout decodes ([`HAMMING_DEPTHS`]), for each position `t` of a window with point `x_t`, and
//! for each of `heads` heads matching on its own block of coordinates:
//! 1. **Points.** The key of an earlier position `s` is the shared table's point of token
//!    `x_{s−1}`, rotated lane by lane by a learned unit quaternion. The query is, for the first
//!    half of the heads (**copy heads**), the point of the current token `x_t` through that same
//!    rotation, so a repeated token is an exact match however the rotation learns; for the rest
//!    (**type heads**), the residual through a second rotation, which must learn what to ask for.
//!    No point is turned by position: the owner's direction is SpiralCore's transport spin for
//!    position, not a rotary code ("we do not need rope", 13 Oct), and the trunk's recurrence
//!    already carries order. The value is `x` rotated by a third quaternion.
//! 2. **Codes.** Each coordinate's bit is 1 at or above 0, as [`pack_signs`] packs it. The forward
//!    pass compares the ±1 codes, so the distance is exactly the Hamming distance
//!    `popcount(q XOR k)` that serving computes. The backward pass treats the code as the identity
//!    (straight through), so the distance's gradient `(a − b) / 2` reaches the rotated points.
//! 3. **Choice.** The [`HAMMING_KEPT`] earlier positions `1 ≤ s < t` nearest in Hamming distance,
//!    hard and detached, by one sort per head ([`choose_sorted`]). Position 0 has no previous token.
//! 4. **Weights.** `max(0, m − d) / m`, between 0 and 1, with `d` in bits and a learned margin `m`
//!    per head: an exact match counts in full, every candidate inside the margin gets graded
//!    credit, and nothing is exponential. A head reports the mean over the candidates it reads (a
//!    count), and reads nothing when none is inside its margin.
//! 5. **Payload,** at full width. Because the key at `s` named token `s − 1`, the kept position
//!    holds what followed the earlier match. A copy head brings back the shared table's point of
//!    token `x_s` itself: the token after the last copy of the current token, as the very point the
//!    nearest readout names it by (an induction head with a tied readout, built in rather than
//!    grown; the residual there failed the recall probe, since the trunk reads a token point added
//!    to its stream as its own input, not as its answer). A type head brings back `v_s`.
//! 6. **Combine.** `c_t` and `y_t` are the copy and type heads' means over the heads that found a
//!    candidate, each with its own weight row, both starting at 0, so at step 0 the trunk is
//!    unchanged. In the stream, `x_t + w_c ⊙ c_t + w_y ⊙ y_t`. On the readout, where a copy head
//!    found something, `n_t + w_c ⊙ (c_t − n_t) + w_y ⊙ y_t`: a nearest readout names the point a
//!    prediction is at, so a copy moves the prediction toward the copied point (at `w_c = 1`,
//!    onto it). Adding the point instead failed the recall probe: next to the stream's larger
//!    parts it barely moves where the prediction lands.
//!
//! The reads are active for the thread that sets them ([`set_hamming_reads`]); every caller of
//! `StackModel::hidden` on that thread gets them. They train with claude-144's AdamW.
use std::cell::RefCell;
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};

use crate::geometric_stack::hamilton_gate;
use crate::{invalid, Result};

/// Each read runs after this many of the trunk's 6 layers; the last one, on the readout.
pub const HAMMING_DEPTHS: [usize; 2] = [2, 6];
/// The read at this depth runs on the final norm's output ([`HammingRead::apply_readout`]).
pub const HAMMING_READOUT_DEPTH: usize = HAMMING_DEPTHS[1];
/// Earlier positions each head keeps.
pub const HAMMING_KEPT: usize = 16;
const FILE: &str = "hamming_read.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<HammingReads>> = const { RefCell::new(None) };
}

/// Make `reads` the Hamming reads `StackModel::hidden` applies on this thread, or none.
pub fn set_hamming_reads(reads: Option<HammingReads>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = reads);
}

/// The Hamming reads `StackModel::hidden` applies on this thread, if set.
pub fn hamming_reads() -> Option<HammingReads> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// Each coordinate's sign bit, packed 64 to a word in coordinate order: the serving form of a
/// head's code, compared by `(a ^ b).count_ones()`. A coordinate at or above 0 is a 1.
pub fn pack_signs(coordinates: &[f32]) -> Vec<u64> {
    coordinates
        .chunks(64)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0u64, |word, (i, &c)| word | (u64::from(c >= 0.0) << i))
        })
        .collect()
}

/// One read: learned per-lane rotations for the query, key and value, a margin per head (bits),
/// and the per-coordinate weights of what it adds: row 0 the copy heads', row 1 the type heads'.
#[derive(Clone)]
pub struct HammingRead {
    heads: usize,
    pub(crate) query_rotation: Var,
    pub(crate) key_rotation: Var,
    pub(crate) value_rotation: Var,
    pub(crate) margin: Var,
    pub(crate) weight: Var,
}

/// The reads at every depth in [`HAMMING_DEPTHS`], in that order.
#[derive(Clone)]
pub struct HammingReads {
    pub reads: Vec<HammingRead>,
}

impl HammingReads {
    /// Reads with `heads` heads on points of `width` coordinates: identity rotations, margins at
    /// half a head's bits, and zero weights, so they start as no read at all.
    pub fn new(heads: usize, width: usize, device: &Device) -> Result<Self> {
        let reads = HAMMING_DEPTHS
            .iter()
            .map(|_| HammingRead::new(heads, width, device))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { reads })
    }

    /// The read inside the stream after `depth` layers, if one runs there.
    pub fn at_depth(&self, depth: usize) -> Option<&HammingRead> {
        if depth == HAMMING_READOUT_DEPTH {
            return None;
        }
        HAMMING_DEPTHS
            .iter()
            .position(|&d| d == depth)
            .and_then(|i| self.reads.get(i))
    }

    /// The read on the final norm's output.
    pub fn readout(&self) -> Option<&HammingRead> {
        HAMMING_DEPTHS
            .iter()
            .position(|&d| d == HAMMING_READOUT_DEPTH)
            .and_then(|i| self.reads.get(i))
    }

    /// Every learned parameter, in a fixed order for the optimizer.
    pub fn variables(&self) -> Vec<&Var> {
        self.reads.iter().flat_map(|r| r.variables()).collect()
    }

    /// The mean absolute value of every read's weight, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        let parts = self
            .reads
            .iter()
            .map(|r| r.weight.as_tensor().abs()?.mean_all())
            .collect::<candle_core::Result<Vec<_>>>()?;
        Ok(Tensor::stack(&parts, 0)?.mean_all()?)
    }

    /// Every read's margins in bits, read by read and head by head.
    pub fn margins(&self) -> Result<Vec<f64>> {
        let mut margins = Vec::new();
        for read in &self.reads {
            for m in read.margin.as_tensor().to_vec1::<f32>()? {
                margins.push(f64::from(m));
            }
        }
        Ok(margins)
    }

    /// Write the reads to `directory/hamming_read.safetensors`, with their head count.
    pub fn save(&self, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        let mut tensors = std::collections::HashMap::new();
        for (depth, read) in HAMMING_DEPTHS.iter().zip(&self.reads) {
            for (name, var) in read.named() {
                tensors.insert(format!("d{depth}.{name}"), var.as_tensor().clone());
            }
        }
        let heads = self.reads.first().map_or(0, |r| r.heads) as u32;
        let device = self.reads[0].weight.device();
        tensors.insert("heads".to_owned(), Tensor::new(&[heads], device)?);
        candle_core::safetensors::save(&tensors, directory.join(FILE))?;
        Ok(())
    }

    /// The reads saved in `directory`, or none when the checkpoint has none.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<Var> {
            let tensor = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("the saved Hamming read has no {name}")))?;
            Ok(Var::from_tensor(tensor)?)
        };
        let heads = get("heads")?.as_tensor().to_vec1::<u32>()?;
        let heads = *heads
            .first()
            .ok_or_else(|| invalid("the saved head count is empty"))? as usize;
        let mut reads = Vec::with_capacity(HAMMING_DEPTHS.len());
        for depth in HAMMING_DEPTHS {
            reads.push(HammingRead {
                heads,
                query_rotation: get(&format!("d{depth}.query_rotation"))?,
                key_rotation: get(&format!("d{depth}.key_rotation"))?,
                value_rotation: get(&format!("d{depth}.value_rotation"))?,
                margin: get(&format!("d{depth}.margin"))?,
                weight: get(&format!("d{depth}.weight"))?,
            });
        }
        Ok(Some(Self { reads }))
    }
}

impl HammingRead {
    fn new(heads: usize, width: usize, device: &Device) -> Result<Self> {
        if heads < 2 || !width.is_multiple_of(heads) {
            return Err(invalid(
                "a Hamming read needs at least 2 heads (copy and type), a count that divides the width",
            ));
        }
        let head_width = width / heads;
        if !head_width.is_multiple_of(4) || head_width > 64 {
            return Err(invalid(
                "a Hamming head needs whole 4-lane blocks and at most 64 coordinates (one word)",
            ));
        }
        let lanes = width / 4;
        let identity: Vec<f32> = (0..lanes).flat_map(|_| [1.0f32, 0.0, 0.0, 0.0]).collect();
        let rotation = || -> Result<Var> {
            Ok(Var::from_tensor(&Tensor::from_vec(
                identity.clone(),
                (lanes, 4),
                device,
            )?)?)
        };
        // Copy heads start with a quarter of a head's bits as margin, so only near-exact repeats
        // count; type heads start with half, so they read widely while they learn what to ask.
        let margins: Vec<f32> = (0..heads)
            .map(|h| {
                if h < heads / 2 {
                    (head_width / 4) as f32
                } else {
                    (head_width / 2) as f32
                }
            })
            .collect();
        Ok(Self {
            heads,
            query_rotation: rotation()?,
            key_rotation: rotation()?,
            value_rotation: rotation()?,
            margin: Var::from_tensor(&Tensor::from_vec(margins, heads, device)?)?,
            weight: Var::zeros((2, width), DType::F32, device)?,
        })
    }

    fn named(&self) -> [(&'static str, &Var); 5] {
        [
            ("query_rotation", &self.query_rotation),
            ("key_rotation", &self.key_rotation),
            ("value_rotation", &self.value_rotation),
            ("margin", &self.margin),
            ("weight", &self.weight),
        ]
    }

    fn variables(&self) -> [&Var; 5] {
        self.named().map(|(_, var)| var)
    }

    /// `points` rows rotated lane by lane by `rotation`'s unit quaternions.
    fn rotate(rotation: &Var, points: &Tensor) -> Result<Tensor> {
        let gate = rotation
            .as_tensor()
            .reshape((1, points.dim(1)?))?
            .broadcast_as(points.shape())?
            .contiguous()?;
        hamilton_gate(&gate, points)
    }

    /// The first half of the heads are copy heads; the rest are type heads.
    fn copy_heads(&self) -> usize {
        self.heads / 2
    }

    /// The read applied inside the stream, to the residual `(batch, time, width)` of the windows
    /// `ids`, with token points from the shared `table`: `x + w_c ⊙ c + w_y ⊙ y`. Returns the
    /// same shape.
    pub fn apply(&self, residual: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        self.read(residual, ids, table, false)
    }

    /// The read applied to the final norm's output `(batch, time, width)`, the points the
    /// nearest readout decodes: where a copy head found an earlier copy of the current token,
    /// the prediction moves toward the copied point, `n + w_c ⊙ (c − n) + w_y ⊙ y`, and stays
    /// `n + w_y ⊙ y` where none did. At `w_c = 1` a found copy is decoded exactly. A nearest
    /// readout names the point a prediction is at, so a copy is a move, not an added term.
    pub fn apply_readout(&self, output: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        self.read(output, ids, table, true)
    }

    fn read(
        &self,
        residual: &Tensor,
        ids: &[u32],
        table: &Tensor,
        readout: bool,
    ) -> Result<Tensor> {
        let (batch, time, width) = residual.dims3()?;
        let rows = batch * time;
        if ids.len() != rows {
            return Err(invalid("the Hamming read needs whole windows"));
        }
        // A window of one token has no earlier position to read.
        if time < 2 {
            return Ok(residual.clone());
        }
        let device = residual.device();
        let head_width = width / self.heads;
        let copy_width = self.copy_heads() * head_width;
        let points = residual.reshape((rows, width))?;
        // Each position's own token and the token before it, as the shared table's points,
        // detached. A window's first position has none and takes its own; it is never a candidate.
        let previous: Vec<u32> = (0..rows)
            .map(|r| if r % time == 0 { ids[r] } else { ids[r - 1] })
            .collect();
        let table = table.detach();
        let token_points = |ids: Vec<u32>| -> Result<Tensor> {
            Ok(table.index_select(&Tensor::from_vec(ids, rows, device)?, 0)?)
        };
        let current = token_points(ids.to_vec())?;
        let previous = token_points(previous)?;
        // Copy heads ask with the current token's point through the key's own rotation, so a
        // repeated token stays an exact match however that rotation learns; type heads ask with
        // the residual through the query's. Rotations act lane by lane, so each half is cut after.
        let copy_asks = Self::rotate(&self.key_rotation, &current)?;
        let type_asks = Self::rotate(&self.query_rotation, &points)?;
        let query = Tensor::cat(
            &[
                &copy_asks.narrow(1, 0, copy_width)?,
                &type_asks.narrow(1, copy_width, width - copy_width)?,
            ],
            1,
        )?;
        let key = Self::rotate(&self.key_rotation, &previous)?;
        let value = Self::rotate(&self.value_rotation, &points)?;
        // ±1 by `p ≥ 0` forward, as `pack_signs` packs it, so 0 is a 1 bit; the identity backward.
        let code = |p: &Tensor| -> Result<Tensor> {
            let bits = p.ge(0.0)?.to_dtype(DType::F32)?.affine(2.0, -1.0)?;
            Ok(p.add(&bits.sub(p)?.detach())?)
        };
        let (query, key) = (code(&query)?, code(&key)?);
        let zero = Tensor::new(&[0u32], device)?;
        let margins = self.margin.as_tensor();
        // What each kind of head brings back, at full width: a copy head the table's point of the
        // kept position's own token, the readout's own point for it (the 26 Sep head's tied
        // readout); a type head the kept position's rotated residual.
        let sources = [
            current.reshape((batch, time, width))?,
            value.reshape((batch, time, width))?,
        ];
        let mut sums = [
            Tensor::zeros((rows, width), DType::F32, device)?,
            Tensor::zeros((rows, width), DType::F32, device)?,
        ];
        // How many heads of each kind found any candidate inside their margin, per row.
        let mut found = [
            Tensor::zeros((rows, 1), DType::F32, device)?,
            Tensor::zeros((rows, 1), DType::F32, device)?,
        ];
        for h in 0..self.heads {
            let q = query.narrow(1, h * head_width, head_width)?.contiguous()?;
            let k = key.narrow(1, h * head_width, head_width)?.contiguous()?;
            // A window's first position has no previous token, so its key names nothing: the
            // candidates start at 1.
            let (picks, real) = choose_sorted(&q, &k, batch, time, HAMMING_KEPT, 1)?;
            // A pick that is not real points at its window's first position, whose successor is
            // still inside the window, and carries weight 0.
            let picks = real.where_cond(&picks, &zero.broadcast_as(picks.shape())?)?;
            let kept = picks.dim(1)?;
            // Gather inside each window, `(batch, time · kept, columns)`: the backward then
            // spreads over windows and coordinates instead of looping over every pick.
            let index = |columns: usize| -> Result<Tensor> {
                Ok(picks
                    .reshape((batch, time * kept, 1))?
                    .broadcast_as((batch, time * kept, columns))?
                    .contiguous()?)
            };
            let kept_keys = k
                .reshape((batch, time, head_width))?
                .gather(&index(head_width)?, 1)?
                .reshape((rows, kept, head_width))?;
            let bits = hamming_bits(&q.unsqueeze(1)?, &kept_keys)?;
            // Each candidate weighs `max(0, m − d) / m`, between 0 and 1, so an exact match
            // (`d = 0`) counts in full. The head reports the mean over the candidates it reads,
            // a count and not a learned sum, and reads nothing when none is inside its margin.
            let margin = margins.narrow(0, h, 1)?.reshape((1, 1))?;
            let weights = margin
                .broadcast_sub(&bits)?
                .relu()?
                .broadcast_div(&margin.maximum(1.0)?)?
                .mul(&real.to_dtype(DType::F32)?)?;
            let count = weights
                .gt(0.0)?
                .to_dtype(DType::F32)?
                .sum_keepdim(1)?
                .maximum(1.0)?;
            // The key at `s` names token `s − 1`, so the kept position is already the one after
            // the earlier match: what it holds is what followed.
            let kind = usize::from(h >= self.copy_heads());
            let payload = sources[kind]
                .gather(&index(width)?, 1)?
                .reshape((rows, kept, width))?;
            sums[kind] = sums[kind].add(
                &payload
                    .broadcast_mul(&weights.unsqueeze(2)?)?
                    .sum(1)?
                    .broadcast_div(&count)?,
            )?;
            found[kind] =
                found[kind].add(&weights.gt(0.0)?.to_dtype(DType::F32)?.max_keepdim(1)?)?;
        }
        // Each kind's mean over the heads that found something, so a head that found nothing does
        // not dilute one that did.
        let means = (0..2)
            .map(|kind| Ok(sums[kind].broadcast_div(&found[kind].maximum(1.0)?)?))
            .collect::<Result<Vec<_>>>()?;
        let weight = self.weight.as_tensor();
        let (copy_weight, type_weight) = (weight.narrow(0, 0, 1)?, weight.narrow(0, 1, 1)?);
        let copied = if readout {
            // Only where a copy head found an earlier copy does the prediction move.
            let matched = found[0].gt(0.0)?.to_dtype(DType::F32)?;
            means[0].sub(&points)?.broadcast_mul(&matched)?
        } else {
            means[0].clone()
        };
        let added = copied
            .broadcast_mul(&copy_weight)?
            .add(&means[1].broadcast_mul(&type_weight)?)?;
        Ok(points.add(&added)?.reshape((batch, time, width))?)
    }
}

/// A distance no real pair reaches: positions that are not earlier.
const FAR: f32 = 1.0e30;

/// The `neighbours` earlier positions `first ≤ s < t` nearest each row `t` of `query` in mean L1,
/// for all `batch` windows of `time` at once, as positions inside the row's own window
/// `(batch * time, kept)`, and which are real.
/// One sort per call, where claude-144's choice (`flock_read::choose_nearest`) takes `kept` argmin
/// rounds per window; with `first` 0, the same nearest set, hard and detached. Ties may come out in
/// another order.
pub(crate) fn choose_sorted(
    query: &Tensor,
    key: &Tensor,
    batch: usize,
    time: usize,
    neighbours: usize,
    first: usize,
) -> Result<(Tensor, Tensor)> {
    let device = query.device();
    let kept = neighbours.min(time.saturating_sub(1)).max(1);
    let width = query.dim(1)?;
    let rows = batch * time;
    let q = query.detach().reshape((batch, time, 1, width))?;
    let k = key.detach().reshape((batch, 1, time, width))?;
    let distance = q.broadcast_sub(&k)?.abs()?.mean(3)?.reshape((rows, time))?;
    let earlier: Vec<u8> = (0..rows * time)
        .map(|i| u8::from(i % time >= first && i % time < (i / time) % time))
        .collect();
    let earlier = Tensor::from_vec(earlier, (rows, time), device)?;
    let far = Tensor::full(FAR, (rows, time), device)?;
    let masked = earlier.where_cond(&distance, &far)?.contiguous()?;
    let order = masked
        .arg_sort_last_dim(true)?
        .narrow(1, 0, kept)?
        .contiguous()?;
    let real = masked.gather(&order, 1)?.lt(FAR / 2.0)?;
    Ok((order, real))
}

/// The Hamming distance between ±1 codes, `(a − b)² / 4` summed over the last dimension: the
/// popcount of the XOR forward. Its gradient is zero on agreeing bits and pulls a disagreeing bit
/// toward the other code.
pub(crate) fn hamming_bits(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    let last = b.rank() - 1;
    Ok(a.broadcast_sub(b)?.sqr()?.sum(last)?.affine(0.25, 0.0)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_sign_codes_pack_one_bit_per_coordinate_in_order() {
        let mut coordinates = vec![-1.0f32; 70];
        coordinates[0] = 0.5;
        coordinates[63] = 0.0;
        coordinates[65] = 2.0;
        assert_eq!(pack_signs(&coordinates), vec![1 | (1 << 63), 1 << 1]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_sorted_choice_keeps_the_same_nearest_set_as_claude_144s() {
        let device = Device::new_cuda(0).unwrap();
        let (batch, time, width) = (2, 20, 8);
        let values: Vec<f32> = (0..batch * time * width)
            .map(|i| (((i * 2_654_435_761) % 10_007) as f32) / 997.0)
            .collect();
        let points = Tensor::from_vec(values, (batch * time, width), &device).unwrap();
        let (local, fast_real) = choose_sorted(&points, &points, batch, time, 4, 0).unwrap();
        // The sorted choice gives positions inside each window; claude-144's gives global rows.
        let starts: Vec<u32> = (0..batch * time)
            .map(|r| ((r / time) * time) as u32)
            .collect();
        let starts = Tensor::from_vec(starts, (batch * time, 1), &device).unwrap();
        let fast = local.broadcast_add(&starts).unwrap();
        let (slow, slow_real) =
            crate::flock_read::choose_nearest(&points, &points, batch, time, 4).unwrap();
        let sets = |picks: Tensor, real: Tensor| -> Vec<Vec<u32>> {
            let (picks, real) = (
                picks.to_vec2::<u32>().unwrap(),
                real.to_vec2::<u8>().unwrap(),
            );
            picks
                .into_iter()
                .zip(real)
                .map(|(row, ok)| {
                    let mut kept: Vec<u32> = row
                        .into_iter()
                        .zip(ok)
                        .filter(|(_, o)| *o == 1)
                        .map(|(p, _)| p)
                        .collect();
                    kept.sort_unstable();
                    kept
                })
                .collect()
        };
        assert_eq!(sets(fast, fast_real), sets(slow, slow_real));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_forward_distance_is_the_popcount_of_the_xor_of_the_sign_codes() {
        let device = Device::new_cuda(0).unwrap();
        // Two 64-coordinate points with no zero coordinate.
        let a: Vec<f32> = (0..64).map(|i| ((i * 37 + 11) % 19) as f32 - 9.5).collect();
        let b: Vec<f32> = (0..64).map(|i| ((i * 53 + 7) % 23) as f32 - 11.5).collect();
        let code = |v: &[f32]| {
            Tensor::from_vec(v.to_vec(), (1, 64), &device)
                .unwrap()
                .sign()
                .unwrap()
        };
        let bits = hamming_bits(&code(&a), &code(&b))
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()[0];
        let popcount = (pack_signs(&a)[0] ^ pack_signs(&b)[0]).count_ones();
        assert_eq!(bits, popcount as f32);
        assert!(popcount > 0 && popcount < 64);
    }

    #[cfg(feature = "cuda")]
    fn residual(batch: usize, time: usize, width: usize, device: &Device) -> Tensor {
        let values: Vec<f32> = (0..batch * time * width)
            .map(|i| (((i * 2_654_435_761) % 1_000) as f32 / 500.0) - 0.999)
            .collect();
        Tensor::from_vec(values, (batch, time, width), device).unwrap()
    }

    /// A table of `vocab` token points with independent pseudo-random signs (a splitmix64 hash),
    /// so two different tokens' codes differ in about half their bits.
    #[cfg(feature = "cuda")]
    fn table(vocab: usize, width: usize, device: &Device) -> Tensor {
        let values: Vec<f32> = (0..vocab * width)
            .map(|i| {
                let mut z = (i as u64)
                    .wrapping_add(1)
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15);
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^= z >> 31;
                ((z >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
            })
            .collect();
        Tensor::from_vec(values, (vocab, width), device).unwrap()
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_zero_weight_leaves_the_residual_unchanged() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let x = residual(2, 24, 256, &device);
        let ids: Vec<u32> = (0..48).map(|i| (i % 5) as u32).collect();
        let out = reads.reads[0]
            .apply(&x, &ids, &table(8, 256, &device))
            .unwrap();
        assert_eq!(
            out.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            x.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_copy_head_reads_what_followed_the_earlier_copy_of_its_token() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        // Only the copy heads' row of weights on, so the added part is theirs alone.
        let copy_only: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i < 256))).collect();
        read.weight
            .set(&Tensor::from_vec(copy_only, (2, 256), &device).unwrap())
            .unwrap();
        // Learned rotations, as after training: two different unit quaternions on every lane. A
        // copy query goes through the key's rotation, so it still matches exactly.
        let turned = |q: [f32; 4]| Tensor::from_vec(q.repeat(64), (64, 4), &device).unwrap();
        read.key_rotation
            .set(&turned([0.6, 0.8, 0.0, 0.0]))
            .unwrap();
        read.query_rotation
            .set(&turned([0.8, 0.0, 0.6, 0.0]))
            .unwrap();
        // Three windows of three tokens; heads 0 and 1 are the copy heads.
        // Window 0, [5 7 5]: at t = 2 the key of s = 1 names token 5, an exact match, so the copy
        // heads bring token 7's own table point, in full and alone (s = 0 names no earlier token).
        // Window 1, [9 11 9]: the same in its own window, token 11.
        // Window 2, [3 5 5]: at t = 2 no earlier key names 5; only t itself would.
        let ids = [5u32, 7, 5, 9, 11, 9, 3, 5, 5];
        let x = residual(3, 3, 256, &device);
        let table = table(16, 256, &device);
        let points = table.to_vec2::<f32>().unwrap();
        let added = read
            .apply(&x, &ids, &table)
            .unwrap()
            .sub(&x)
            .unwrap()
            .reshape((9, 256))
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        for c in 0..256 {
            assert!((added[2][c] - points[7][c]).abs() < 1e-6, "window 0, {c}");
            assert!((added[5][c] - points[11][c]).abs() < 1e-6, "window 1, {c}");
            assert_eq!(added[8][c], 0.0, "window 2, {c}");
        }
        // Positions 0 and 1 have no candidate (1 ≤ s < t) and read nothing in any head.
        for r in [0usize, 1, 3, 4, 6, 7] {
            assert!(added[r].iter().all(|&v| v == 0.0), "row {r}");
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_readout_moves_onto_a_found_copy_and_leaves_the_rest() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = reads.readout().unwrap();
        let copy_only: Vec<f32> = (0..512).map(|i| f32::from(u8::from(i < 256))).collect();
        read.weight
            .set(&Tensor::from_vec(copy_only, (2, 256), &device).unwrap())
            .unwrap();
        // The windows of the copy test: window 0 finds token 7 at t = 2, window 1 token 11,
        // window 2 nothing. With the copy weight at 1 the found rows decode exactly as the copied
        // token's point; every other row is the readout's own point, unchanged.
        let ids = [5u32, 7, 5, 9, 11, 9, 3, 5, 5];
        let n = residual(3, 3, 256, &device);
        let table = table(16, 256, &device);
        let points = table.to_vec2::<f32>().unwrap();
        let own = n.reshape((9, 256)).unwrap().to_vec2::<f32>().unwrap();
        let out = read
            .apply_readout(&n, &ids, &table)
            .unwrap()
            .reshape((9, 256))
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        for c in 0..256 {
            assert!((out[2][c] - points[7][c]).abs() < 1e-6, "window 0, {c}");
            assert!((out[5][c] - points[11][c]).abs() < 1e-6, "window 1, {c}");
        }
        for r in [0usize, 1, 3, 4, 6, 7, 8] {
            assert_eq!(out[r], own[r], "row {r}");
        }
        // The stream read is the other one; the readout is not offered at its depth.
        assert!(reads.at_depth(HAMMING_READOUT_DEPTH).is_none());
        assert!(reads.at_depth(HAMMING_DEPTHS[0]).is_some());
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn no_position_reads_a_later_one_and_a_zero_margin_reads_nothing() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        read.weight
            .set(&Tensor::ones((2, 256), DType::F32, &device).unwrap())
            .unwrap();
        let x = residual(1, 24, 256, &device);
        let table = table(32, 256, &device);
        // Repeated tokens, so the copy heads have exact matches to read.
        let ids: Vec<u32> = (0..24).map(|i| (i % 5) as u32 * 3).collect();
        let base = read
            .apply(&x, &ids, &table)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Change only the last position, its point and its token: every earlier output must stay
        // exactly as it was.
        let last = Tensor::ones((1, 1, 256), DType::F32, &device).unwrap();
        let changed = Tensor::cat(&[&x.narrow(1, 0, 23).unwrap(), &last], 1).unwrap();
        let mut changed_ids = ids.clone();
        changed_ids[23] = 31;
        let moved = read
            .apply(&changed, &changed_ids, &table)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        assert_eq!(&moved[..23], &base[..23]);
        assert_ne!(moved[23], base[23]);
        // A margin of 0 bits weighs every candidate at 0, an exact match too: the read adds nothing.
        read.margin
            .set(&Tensor::zeros(4, DType::F32, &device).unwrap())
            .unwrap();
        let none = read.apply(&x, &ids, &table).unwrap();
        assert_eq!(
            none.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            x.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_helpful_payload_raises_the_margin_under_descent() {
        let device = Device::new_cuda(0).unwrap();
        let reads = HammingReads::new(4, 256, &device).unwrap();
        let read = &reads.reads[0];
        read.weight
            .set(&Tensor::ones((2, 256), DType::F32, &device).unwrap())
            .unwrap();
        // A residual of all-positive points, and a table shifted up by 0.5: a quarter of its signs
        // stay negative, so token codes still differ, but every point's sum is positive. Every
        // payload, a type head's or a copy head's, then adds to the sum of the read's output, and
        // a loss that rewards that sum rewards each.
        let x = residual(1, 24, 256, &device)
            .abs()
            .unwrap()
            .affine(1.0, 0.1)
            .unwrap();
        // Margins at a head's full 64 bits, so every candidate but a full mismatch is inside it,
        // and a wider margin raises each one's weight `(m − d) / m` by `d / m²`.
        read.margin
            .set(&Tensor::full(64.0f32, 4, &device).unwrap())
            .unwrap();
        let ids: Vec<u32> = (0..24).map(|i| (i * 5 % 11) as u32).collect();
        let out = read
            .apply(&x, &ids, &table(16, 256, &device).affine(1.0, 0.5).unwrap())
            .unwrap();
        let loss = out.sum_all().unwrap().neg().unwrap();
        let grads = loss.backward().unwrap();
        let margin = grads
            .get(read.margin.as_tensor())
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        // A wider margin weighs the helpful payloads more, so descent raises every head's margin.
        assert!(margin.iter().all(|&g| g < 0.0), "{margin:?}");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_trunk_with_zero_weight_reads_is_unchanged_and_moved_reads_change_it() {
        let device = Device::new_cuda(0).unwrap();
        let model = crate::spiralcore_fbs_sheet::build_fbs_model(
            crate::spiralcore_fbs_arms::FbsArm::OneAtomActive,
            1,
            &device,
        )
        .unwrap();
        // Seven tokens repeated, so the copy heads find exact matches.
        let ids: Vec<u32> = (0..32u32).map(|i| (i % 7) * 37 + 5).collect();
        let hidden = |m: &crate::geometric_stack::StackModel| {
            crate::hamilton_sheet::with_gate_mode(
                crate::hamilton_sheet::HamiltonArm::HamiltonActive,
                1,
                0,
                false,
                || m.hidden(&ids, 1, 32),
            )
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()
        };
        let plain = hidden(&model);
        let reads = HammingReads::new(4, crate::hamilton_sheet::SHEET_WIDTH, &device).unwrap();
        set_hamming_reads(Some(reads.clone()));
        let zero = hidden(&model);
        for read in &reads.reads {
            read.weight
                .set(
                    &Tensor::ones((2, crate::hamilton_sheet::SHEET_WIDTH), DType::F32, &device)
                        .unwrap(),
                )
                .unwrap();
        }
        let moved = hidden(&model);
        set_hamming_reads(None);
        assert_eq!(zero, plain);
        assert_ne!(moved, plain);
    }
}
