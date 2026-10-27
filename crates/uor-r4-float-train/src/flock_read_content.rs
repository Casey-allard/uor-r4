//! A content-keyed read inside the in-space trunk (in-space-trunk-to-text, claude-145).
//!
//! claude-144's Flock read keyed on the trunk's hidden point and added the next token's point after
//! the last layer, where nothing could transform it. A reply opening's hidden point mostly encodes
//! the turn marker, and the tokens just before an opening are always the same marker, so a short
//! reach sees nothing of the prompt (`--opening-bound`: the last 1–4 tokens name 161 openings,
//! the prompt's first 8 name 607 of 1,226). This read finds earlier tokens by what they are.
//!
//! In `StackModel::hidden`, after layer [`CONTENT_AFTER_LAYER`], for each position `t` of a window
//! with residual point `x_t` and input token `x_s` at earlier positions:
//! 1. **Points.** The key of an earlier position is its own token point `e(x_s)` (the shared table,
//!    detached), rotated lane by lane by one learned unit quaternion. The query is `x_t` rotated by
//!    another. Both are Hamilton products ([`crate::geometric_stack::hamilton_gate`]), and each
//!    4-lane block of both is put on the unit sphere S³, so the choice compares directions.
//! 2. **Choice.** Keep the `neighbours` earlier positions `s < t` nearest the query in mean L1:
//!    claude-144's hard, detached choice ([`FlockRead::choose`]).
//! 3. **Payload.** The kept position's own residual point `x_s` at this layer.
//! 4. **Consensus and credit.** The plain mean of the kept payloads, each carrying the
//!    straight-through factor `1 − (d − d.detach())`, exactly 1 forward.
//! 5. **Add.** `x_t + w ⊙ c_t`, the weight `w` starting at 0, so at step 0 the trunk is unchanged.
//!    Layers after [`CONTENT_AFTER_LAYER`] then transform what the read added.
//!
//! The read is active for the thread that sets it ([`set_content_read`]); every caller of
//! `StackModel::hidden` on that thread gets it. Its parameters and optimizer are claude-144's
//! ([`FlockRead`], [`crate::flock_read::FlockAdamW`]).
use std::cell::RefCell;
use std::path::Path;

use candle_core::{DType, Device, Tensor};

use crate::flock_read::FlockRead;
use crate::{invalid, Result};

/// The read runs after this many of the trunk's 6 layers, leaving 3 to transform what it adds.
pub const CONTENT_AFTER_LAYER: usize = 3;
const FILE: &str = "content_read.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<ContentRead>> = const { RefCell::new(None) };
}

/// Make `read` the content read `StackModel::hidden` applies on this thread, or none.
pub fn set_content_read(read: Option<ContentRead>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = read);
}

/// The content read `StackModel::hidden` applies on this thread, if one is set.
pub fn content_read() -> Option<ContentRead> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// The read: claude-144's parameters (two rotations and a weight), keyed on token points.
#[derive(Clone)]
pub struct ContentRead {
    pub read: FlockRead,
}

impl ContentRead {
    /// A read keeping `neighbours` earlier positions on points of `width` coordinates, starting as
    /// no read at all.
    pub fn new(neighbours: usize, width: usize, device: &Device) -> Result<Self> {
        Ok(Self {
            read: FlockRead::new(neighbours, width, device)?,
        })
    }

    /// Write the read to `directory/content_read.safetensors`.
    pub fn save(&self, directory: &Path) -> Result<()> {
        self.read.save_as(directory, FILE)
    }

    /// The read saved in `directory`, or none when the checkpoint has no content read.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        Ok(FlockRead::load_from(directory, FILE, device)?.map(|read| Self { read }))
    }

    /// The read applied to the residual `(batch, time, width)` of the windows `ids`, with keys
    /// from the shared `table`. Returns a residual of the same shape.
    pub fn apply(&self, residual: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = residual.dims3()?;
        let rows = batch * time;
        if ids.len() != rows {
            return Err(invalid("the content read needs whole windows"));
        }
        // A window of one token has no earlier position to read.
        if time < 2 {
            return Ok(residual.clone());
        }
        let device = residual.device();
        let points = residual.reshape((rows, width))?;
        let tokens = table
            .detach()
            .index_select(&Tensor::from_vec(ids.to_vec(), rows, device)?, 0)?;
        let read = &self.read;
        let query = on_sphere(&read.rotate(&read.query_rotation, &points)?)?;
        let key = on_sphere(&read.rotate(&read.key_rotation, &tokens)?)?;
        let (picks, real) = read.choose(&query, &key, batch, time)?;
        // A pick that is not real points at its window's first position and carries weight 0.
        let starts: Vec<u32> = (0..rows).map(|r| ((r / time) * time) as u32).collect();
        let starts = Tensor::from_vec(starts, (rows, 1), device)?.broadcast_as(picks.shape())?;
        let picks = real.where_cond(&picks, &starts)?;
        let kept = picks.dim(1)?;
        let flat = picks.flatten_all()?;
        let kept_keys = key.index_select(&flat, 0)?.reshape((rows, kept, width))?;
        let distance = query
            .unsqueeze(1)?
            .broadcast_sub(&kept_keys)?
            .abs()?
            .mean(2)?;
        // 1 − (d − d̄): exactly 1 forward; its gradient credits each kept key by its payload's use.
        let factor = distance.detach().sub(&distance)?.affine(1.0, 1.0)?;
        let payload = points
            .index_select(&flat, 0)?
            .reshape((rows, kept, width))?;
        let real = real.to_dtype(DType::F32)?;
        let count = real.sum_keepdim(1)?.maximum(1.0)?;
        let consensus = payload
            .broadcast_mul(&real.mul(&factor)?.unsqueeze(2)?)?
            .sum(1)?
            .broadcast_div(&count)?;
        let added = consensus.broadcast_mul(&read.weight.as_tensor().unsqueeze(0)?)?;
        Ok(points.add(&added)?.reshape((batch, time, width))?)
    }
}

/// `points` rows with each 4-lane block put on the unit sphere S³. The small constant under the
/// root keeps a zero block's gradient finite.
fn on_sphere(points: &Tensor) -> Result<Tensor> {
    let (rows, width) = points.dims2()?;
    let blocks = points.reshape((rows, width / 4, 4))?;
    let norm = blocks.sqr()?.sum_keepdim(2)?.affine(1.0, 1e-12)?.sqrt()?;
    Ok(blocks.broadcast_div(&norm)?.reshape((rows, width))?)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "cuda")]
    use super::*;
    #[cfg(feature = "cuda")]
    use candle_core::Var;

    /// A 4-token table whose rows point along the four axes, scaled small like the trunk's table.
    #[cfg(feature = "cuda")]
    fn axis_table(device: &Device) -> Tensor {
        Tensor::new(
            &[
                [0.01f32, 0.0, 0.0, 0.0],
                [0.0, 0.01, 0.0, 0.0],
                [0.0, 0.0, 0.01, 0.0],
                [0.0, 0.0, 0.0, 0.01],
            ],
            device,
        )
        .unwrap()
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn keys_come_from_token_points_and_the_payload_is_the_kept_residual() {
        let device = Device::new_cuda(0).unwrap();
        let read = ContentRead::new(1, 4, &device).unwrap();
        read.read
            .weight
            .set(&Tensor::ones(4, DType::F32, &device).unwrap())
            .unwrap();
        // Position 2's query points along axis 0. By residual, position 1 matches it; by token,
        // position 0 does (token 0 points along axis 0, token 1 along axis 1).
        let residual = Tensor::new(
            &[[
                [0.0f32, 0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
            ]],
            &device,
        )
        .unwrap();
        let ids = [0u32, 1, 3];
        let out = read
            .apply(&residual, &ids, &axis_table(&device))
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Position 2 keeps position 0 and adds its residual, not position 1's.
        assert_eq!(out[2], vec![1.0, 0.0, 1.0, 0.0]);
        // Position 1 has only position 0 to keep; position 0 keeps nothing.
        assert_eq!(out[1], vec![1.0, 0.0, 1.0, 0.0]);
        assert_eq!(out[0], vec![0.0, 0.0, 1.0, 0.0]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_zero_weight_leaves_the_residual_unchanged() {
        let device = Device::new_cuda(0).unwrap();
        let read = ContentRead::new(2, 4, &device).unwrap();
        let residual = Tensor::new(
            &[[
                [0.3f32, -0.2, 1.0, 0.0],
                [1.0, 0.5, 0.0, -1.0],
                [0.2, 0.0, 0.7, 0.1],
            ]],
            &device,
        )
        .unwrap();
        let out = read
            .apply(&residual, &[2, 1, 0], &axis_table(&device))
            .unwrap();
        assert_eq!(
            out.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            residual.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_credit_pulls_the_query_toward_a_helpful_key() {
        let device = Device::new_cuda(0).unwrap();
        let read = ContentRead::new(1, 4, &device).unwrap();
        read.read
            .weight
            .set(&Tensor::ones(4, DType::F32, &device).unwrap())
            .unwrap();
        // Position 2's query (1, 0.5) is nearer token 0's key (axis 0) than token 1's (axis 1), so
        // it keeps position 0, whose residual (axis 2) is the payload.
        let residual = Var::new(
            &[[
                [0.0f32, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
                [1.0, 0.5, 0.0, 0.0],
            ]],
            &device,
        )
        .unwrap();
        let out = read
            .apply(residual.as_tensor(), &[0, 1, 3], &axis_table(&device))
            .unwrap();
        // A loss that rewards the payload's coordinate in position 2's output: the payload helps.
        let loss = out
            .narrow(1, 2, 1)
            .unwrap()
            .narrow(2, 2, 1)
            .unwrap()
            .neg()
            .unwrap()
            .sum_all()
            .unwrap();
        let grads = loss.backward().unwrap();
        let grad = grads
            .get(residual.as_tensor())
            .unwrap()
            .squeeze(0)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // dL/dd = +1, so descent shortens the distance. With q = x/|x| = (0.894, 0.447) and the key
        // (1, 0): dd/dq = (−1/4, 1/4), and through the sphere (I − q qᵀ) / |x| that is
        // (−0.1342, 0.2683) on x. Descent raises x[0] and lowers x[1], toward the key.
        assert!((grad[2][0] + 0.134_164_1).abs() < 1e-5, "{grad:?}");
        assert!((grad[2][1] - 0.268_328_2).abs() < 1e-5, "{grad:?}");
        // The payload is live: position 0's residual enters position 2's output at weight 1 and
        // count 1, and only its axis-2 coordinate is in the loss. A detached payload gives zeros.
        assert_eq!(grad[0], vec![0.0, 0.0, -1.0, 0.0]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_trunk_with_a_zero_weight_read_is_unchanged_and_a_moved_read_changes_it() {
        let device = Device::new_cuda(0).unwrap();
        let model = crate::spiralcore_fbs_sheet::build_fbs_model(
            crate::spiralcore_fbs_arms::FbsArm::OneAtomActive,
            1,
            &device,
        )
        .unwrap();
        let ids: Vec<u32> = (0..16u32).map(|i| (i * 37 + 5) % 4096).collect();
        let hidden = |m: &crate::geometric_stack::StackModel| {
            crate::hamilton_sheet::with_gate_mode(
                crate::hamilton_sheet::HamiltonArm::HamiltonActive,
                1,
                0,
                false,
                || m.hidden(&ids, 1, 16),
            )
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap()
        };
        let plain = hidden(&model);
        let read = ContentRead::new(4, crate::hamilton_sheet::SHEET_WIDTH, &device).unwrap();
        set_content_read(Some(read.clone()));
        let zero = hidden(&model);
        read.read
            .weight
            .set(&Tensor::ones(crate::hamilton_sheet::SHEET_WIDTH, DType::F32, &device).unwrap())
            .unwrap();
        let moved = hidden(&model);
        set_content_read(None);
        assert_eq!(zero, plain);
        assert_ne!(moved, plain);
    }
}
