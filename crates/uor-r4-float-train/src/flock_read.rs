//! A float-trainable Flock read on the in-space trunk's hidden points (in-space-trunk-to-text,
//! claude-144).
//!
//! The trunk's recurrence learns short memory: its best checkpoint reads about 8–12 tokens back.
//! The Flock read ([flock-read](../../../docs/mechanisms/flock-read.md)) is the project's way of
//! reading context: look at a few nearest earlier positions and move toward their consensus. The
//! integer form lives in the discrete crate; this is a float form for training, on top of the trunk.
//!
//! For each position `t` of a window, with hidden point `h_t` and input token `x_t`:
//! 1. **Points.** The query is `h_t` rotated lane by lane by one learned unit quaternion per 4-lane
//!    block, and the key is `h_s` rotated by another. Both are Hamilton products ([`hamilton_gate`]),
//!    with no matmul.
//! 2. **Choice.** Keep the `neighbours` earlier positions `s < t` whose keys are nearest the query in
//!    mean L1. The choice is hard, exact and detached (repeated argmin), weighting nothing by score. A position
//!    with fewer earlier positions keeps fewer.
//! 3. **Payload.** The shared table's point for the token after each kept `s`, `e(x_{s+1})`, which is
//!    a known input because `s + 1 ≤ t`: what followed a similar state.
//! 4. **Consensus.** The plain mean of the kept payloads. Each payload carries the straight-through
//!    factor `1 − (d − d.detach())`, which is exactly 1 in the forward pass, so the gradient pulls
//!    the key of a neighbour whose payload helped toward the query and pushes away one that hurt.
//! 5. **Add.** The point becomes `h_t + w ⊙ c_t`, where the per-coordinate weight `w` starts at 0, so
//!    at step 0 the read changes nothing.
//!
//! The read is active for the thread that sets it ([`set_flock_read`]); [`points`] is the trunk's
//! output with the read applied when one is set.
use std::cell::RefCell;
use std::path::Path;

use candle_core::backprop::GradStore;
use candle_core::{DType, Device, Tensor, Var};

use crate::geometric_stack::{hamilton_gate, StackModel};
use crate::{invalid, Result};

/// A distance no real pair reaches: masked and already-kept candidates.
const FAR: f32 = 1.0e30;
const FILE: &str = "flock_read.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<FlockRead>> = const { RefCell::new(None) };
}

/// Make `read` the Flock read [`points`] applies on this thread, or none.
pub fn set_flock_read(read: Option<FlockRead>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = read);
}

/// The Flock read [`points`] applies on this thread, if one is set.
pub fn flock_read() -> Option<FlockRead> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// The trunk's predicted points for `ids` (`batch` windows of `time` tokens), as rows
/// `(batch * time, width)`, with the active Flock read applied when one is set.
pub fn points(model: &StackModel, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
    let hidden = model.hidden(ids, batch, time)?;
    match flock_read() {
        None => Ok(hidden),
        Some(read) => {
            let table = model
                .variables()
                .get("embedding.weight")
                .ok_or_else(|| invalid("the model has no shared vector table"))?
                .as_tensor()
                .clone();
            read.apply(&hidden, ids, batch, time, &table)
        }
    }
}

/// The read's learned parameters and its neighbour count.
#[derive(Clone)]
pub struct FlockRead {
    neighbours: usize,
    pub(crate) query_rotation: Var,
    pub(crate) key_rotation: Var,
    pub(crate) weight: Var,
}

impl FlockRead {
    /// A read keeping `neighbours` earlier positions, on points of `width` coordinates: identity
    /// rotations and a zero weight, so it starts as no read at all.
    pub fn new(neighbours: usize, width: usize, device: &Device) -> Result<Self> {
        if neighbours == 0 || width == 0 || !width.is_multiple_of(4) {
            return Err(invalid(
                "a Flock read needs a positive neighbour count and whole 4-lane blocks",
            ));
        }
        let lanes = width / 4;
        let identity: Vec<f32> = (0..lanes).flat_map(|_| [1.0f32, 0.0, 0.0, 0.0]).collect();
        Ok(Self {
            neighbours,
            query_rotation: Var::from_tensor(&Tensor::from_vec(
                identity.clone(),
                (lanes, 4),
                device,
            )?)?,
            key_rotation: Var::from_tensor(&Tensor::from_vec(identity, (lanes, 4), device)?)?,
            weight: Var::zeros(width, DType::F32, device)?,
        })
    }

    pub fn neighbours(&self) -> usize {
        self.neighbours
    }

    /// The mean absolute value of the per-coordinate weight, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        Ok(self.weight.as_tensor().abs()?.mean_all()?)
    }

    /// The learned parameters, by name.
    pub fn variables(&self) -> [(&'static str, &Var); 3] {
        [
            ("query_rotation", &self.query_rotation),
            ("key_rotation", &self.key_rotation),
            ("weight", &self.weight),
        ]
    }

    /// Write the read's parameters to `directory/flock_read.safetensors`, the neighbour count with them.
    pub fn save(&self, directory: &Path) -> Result<()> {
        self.save_as(directory, FILE)
    }

    /// Write the read's parameters to `directory/file`, the neighbour count with them.
    pub(crate) fn save_as(&self, directory: &Path, file: &str) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        let mut tensors = std::collections::HashMap::new();
        for (name, var) in self.variables() {
            tensors.insert(name.to_owned(), var.as_tensor().clone());
        }
        let count = Tensor::new(&[self.neighbours as u32], self.weight.device())?;
        tensors.insert("neighbours".to_owned(), count);
        candle_core::safetensors::save(&tensors, directory.join(file))?;
        Ok(())
    }

    /// The read saved in `directory`, or none when the checkpoint has no read.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        Self::load_from(directory, FILE, device)
    }

    /// The read saved as `directory/file`, or none when there is no such file.
    pub(crate) fn load_from(directory: &Path, file: &str, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(file);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<Tensor> {
            tensors
                .get(name)
                .cloned()
                .ok_or_else(|| invalid(format!("the saved Flock read has no {name}")))
        };
        let neighbours = get("neighbours")?.to_vec1::<u32>()?;
        Ok(Some(Self {
            neighbours: *neighbours
                .first()
                .ok_or_else(|| invalid("the saved neighbour count is empty"))?
                as usize,
            query_rotation: Var::from_tensor(&get("query_rotation")?)?,
            key_rotation: Var::from_tensor(&get("key_rotation")?)?,
            weight: Var::from_tensor(&get("weight")?)?,
        }))
    }

    /// `hidden` rows rotated lane by lane by `rotation`'s unit quaternions.
    pub(crate) fn rotate(&self, rotation: &Var, hidden: &Tensor) -> Result<Tensor> {
        let gate = rotation
            .as_tensor()
            .reshape((1, hidden.dim(1)?))?
            .broadcast_as(hidden.shape())?
            .contiguous()?;
        hamilton_gate(&gate, hidden)
    }

    /// For each row of `query` (`batch` windows of `time`), the global row indices of the kept
    /// earlier keys, `(batch * time, kept)`, and which of them are real, `(batch * time, kept)` u8.
    /// Detached: the choice carries no gradient.
    pub(crate) fn choose(
        &self,
        query: &Tensor,
        key: &Tensor,
        batch: usize,
        time: usize,
    ) -> Result<(Tensor, Tensor)> {
        let device = query.device();
        let kept = self.neighbours.min(time.saturating_sub(1)).max(1);
        let earlier: Vec<u8> = (0..time * time)
            .map(|i| u8::from(i % time < i / time))
            .collect();
        let earlier = Tensor::from_vec(earlier, (time, time), device)?;
        let far = Tensor::full(FAR, (time, time), device)?;
        let columns = Tensor::arange(0u32, time as u32, device)?
            .unsqueeze(0)?
            .broadcast_as((time, time))?;
        let (query, key) = (query.detach(), key.detach());
        let (mut picks, mut real) = (Vec::with_capacity(batch), Vec::with_capacity(batch));
        for b in 0..batch {
            let q = query.narrow(0, b * time, time)?;
            let k = key.narrow(0, b * time, time)?;
            let distance = q
                .unsqueeze(1)?
                .broadcast_sub(&k.unsqueeze(0)?)?
                .abs()?
                .mean(2)?;
            let mut left = earlier.where_cond(&distance, &far)?;
            let offset = Tensor::new(&[(b * time) as u32], device)?;
            let (mut at_all, mut ok_all) = (Vec::with_capacity(kept), Vec::with_capacity(kept));
            for _ in 0..kept {
                let at = left.argmin_keepdim(1)?;
                ok_all.push(left.gather(&at, 1)?.lt(FAR / 2.0)?);
                left = columns.broadcast_eq(&at)?.where_cond(&far, &left)?;
                at_all.push(at.broadcast_add(&offset)?);
            }
            picks.push(Tensor::cat(&at_all, 1)?);
            real.push(Tensor::cat(&ok_all, 1)?);
        }
        Ok((Tensor::cat(&picks, 0)?, Tensor::cat(&real, 0)?))
    }

    /// The read applied to `hidden` rows `(batch * time, width)` of the windows `ids`, against the
    /// shared `table`. Returns rows of the same shape.
    pub fn apply(
        &self,
        hidden: &Tensor,
        ids: &[u32],
        batch: usize,
        time: usize,
        table: &Tensor,
    ) -> Result<Tensor> {
        let (rows, width) = hidden.dims2()?;
        if rows != batch * time || ids.len() != rows {
            return Err(invalid("the Flock read needs whole windows"));
        }
        // A window of one token has no earlier position to read.
        if time < 2 {
            return Ok(hidden.clone());
        }
        let device = hidden.device();
        let query = self.rotate(&self.query_rotation, hidden)?;
        let key = self.rotate(&self.key_rotation, hidden)?;
        let (picks, real) = self.choose(&query, &key, batch, time)?;
        // A pick that is not real (a row with fewer earlier positions than kept) points at its
        // window's first position, so its next-token index stays inside the window; it carries weight 0.
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
        // Each kept s < t has s + 1 ≤ t in its own window, so the next token is a known input.
        let ids = Tensor::from_vec(ids.to_vec(), rows, device)?;
        let next = flat.broadcast_add(&Tensor::new(&[1u32], device)?)?;
        let payload = table
            .index_select(&ids.index_select(&next, 0)?, 0)?
            .reshape((rows, kept, width))?;
        let real = real.to_dtype(DType::F32)?;
        let count = real.sum_keepdim(1)?.maximum(1.0)?;
        let consensus = payload
            .broadcast_mul(&real.mul(&factor)?.unsqueeze(2)?)?
            .sum(1)?
            .broadcast_div(&count)?;
        Ok(hidden.add(&consensus.broadcast_mul(&self.weight.as_tensor().unsqueeze(0)?)?)?)
    }
}

/// AdamW for the read's three parameters, in tensor arithmetic on their device, with the trunk's
/// constants (beta1 0.9, beta2 0.95, epsilon 1e-8) and its own global-norm clip at 1. No weight
/// decay: [`hamilton_gate`] divides each quaternion by its norm, so the rotations stay unit whatever
/// their stored scale, and the weight starts at 0.
pub struct FlockAdamW {
    first: Vec<Tensor>,
    second: Vec<Tensor>,
    step: i32,
}

pub const FLOCK_BETA1: f64 = 0.9;
pub const FLOCK_BETA2: f64 = 0.95;
pub const FLOCK_EPSILON: f64 = 1e-8;
pub const FLOCK_CLIP: f64 = 1.0;

impl FlockAdamW {
    pub fn new(read: &FlockRead) -> Result<Self> {
        let zeros = |v: &Var| v.as_tensor().zeros_like();
        let mut first = Vec::new();
        let mut second = Vec::new();
        for (_, var) in read.variables() {
            first.push(zeros(var)?);
            second.push(zeros(var)?);
        }
        Ok(Self {
            first,
            second,
            step: 0,
        })
    }

    /// One update of `read`'s parameters from `grads` at learning rate `lr`. A parameter with no
    /// gradient is left as it is.
    pub fn step(&mut self, read: &FlockRead, grads: &GradStore, lr: f64) -> Result<()> {
        let present: Vec<Option<Tensor>> = read
            .variables()
            .iter()
            .map(|(_, var)| grads.get(var.as_tensor()).cloned())
            .collect();
        let mut squared = 0.0f64;
        for grad in present.iter().flatten() {
            squared += f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
        }
        let norm = squared.sqrt();
        if !norm.is_finite() {
            return Err(invalid("non-finite Flock read gradient"));
        }
        let scale = if norm > FLOCK_CLIP {
            FLOCK_CLIP / norm
        } else {
            1.0
        };
        self.step += 1;
        let correct1 = 1.0 - FLOCK_BETA1.powi(self.step);
        let correct2 = 1.0 - FLOCK_BETA2.powi(self.step);
        for (i, ((_, var), grad)) in read.variables().iter().zip(&present).enumerate() {
            let Some(grad) = grad else { continue };
            let grad = grad.affine(scale, 0.0)?;
            self.first[i] = self.first[i]
                .affine(FLOCK_BETA1, 0.0)?
                .add(&grad.affine(1.0 - FLOCK_BETA1, 0.0)?)?;
            self.second[i] = self.second[i]
                .affine(FLOCK_BETA2, 0.0)?
                .add(&grad.sqr()?.affine(1.0 - FLOCK_BETA2, 0.0)?)?;
            let update = self.first[i].affine(1.0 / correct1, 0.0)?.div(
                &self.second[i]
                    .affine(1.0 / correct2, 0.0)?
                    .sqrt()?
                    .affine(1.0, FLOCK_EPSILON)?,
            )?;
            var.set(&var.as_tensor().sub(&update.affine(lr, 0.0)?)?)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "cuda")]
    use super::*;

    #[cfg(feature = "cuda")]
    #[test]
    fn the_choice_keeps_the_nearest_earlier_positions_and_never_the_position_itself() {
        let device = Device::new_cuda(0).unwrap();
        let read = FlockRead::new(2, 4, &device).unwrap();
        // One window of five points on a line; with identity rotations query = key = hidden.
        let hidden = Tensor::new(
            &[
                [0.0f32, 0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
                [9.0, 0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0, 0.0],
            ],
            &device,
        )
        .unwrap();
        let (picks, real) = read.choose(&hidden, &hidden, 1, 5).unwrap();
        let picks = picks.to_vec2::<u32>().unwrap();
        let real = real.to_vec2::<u8>().unwrap();
        // t = 0 has no earlier position; t = 1 has one.
        assert_eq!(real[0], vec![0, 0]);
        assert_eq!(picks[1][0], 0);
        assert_eq!(real[1], vec![1, 0]);
        // t = 4 (value 2): nearest earlier are 2 (value 1) and 0 (value 0), never 3 (value 9) or 4.
        assert_eq!(picks[4], vec![2, 0]);
        assert_eq!(real[4], vec![1, 1]);
        for (t, row) in picks.iter().enumerate() {
            for (j, &s) in row.iter().enumerate() {
                if real[t][j] == 1 {
                    assert!((s as usize) < t, "row {t} kept {s}");
                }
            }
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_zero_weight_leaves_the_points_unchanged_and_a_unit_weight_adds_the_payload_mean() {
        let device = Device::new_cuda(0).unwrap();
        let read = FlockRead::new(1, 4, &device).unwrap();
        let hidden = Tensor::new(
            &[
                [0.0f32, 0.0, 0.0, 0.0],
                [5.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
            ],
            &device,
        )
        .unwrap();
        let table = Tensor::new(
            &[
                [0.0f32, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            &device,
        )
        .unwrap();
        let ids = [1u32, 2, 3];
        let same = read.apply(&hidden, &ids, 1, 3, &table).unwrap();
        assert_eq!(
            same.to_vec2::<f32>().unwrap(),
            hidden.to_vec2::<f32>().unwrap()
        );
        read.weight
            .set(&Tensor::ones(4, DType::F32, &device).unwrap())
            .unwrap();
        let moved = read
            .apply(&hidden, &ids, 1, 3, &table)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // t = 2 (value 1) keeps s = 0 (value 0, nearer than 5), whose next token is ids[1] = 2.
        assert_eq!(moved[2], vec![1.0, 0.0, 1.0, 0.0]);
        // t = 1 keeps s = 0 too; t = 0 keeps nothing.
        assert_eq!(moved[1], vec![5.0, 0.0, 1.0, 0.0]);
        assert_eq!(moved[0], vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_credit_pulls_the_key_of_a_helpful_neighbour_toward_the_query() {
        let device = Device::new_cuda(0).unwrap();
        let read = FlockRead::new(1, 4, &device).unwrap();
        read.weight
            .set(&Tensor::ones(4, DType::F32, &device).unwrap())
            .unwrap();
        // t = 2 (value 1) keeps s = 0 (value 0); its payload is the table row of ids[1] = 2.
        let hidden = Var::new(
            &[
                [0.0f32, 0.0, 0.0, 0.0],
                [5.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
            ],
            &device,
        )
        .unwrap();
        let table = Tensor::new(
            &[
                [0.0f32, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            &device,
        )
        .unwrap();
        let out = read
            .apply(hidden.as_tensor(), &[1, 2, 3], 1, 3, &table)
            .unwrap();
        // A loss that rewards that payload's coordinate in row 2: the payload helps.
        let loss = out
            .narrow(0, 2, 1)
            .unwrap()
            .narrow(1, 2, 1)
            .unwrap()
            .neg()
            .unwrap()
            .sum_all()
            .unwrap();
        let grads = loss.backward().unwrap();
        let grad = grads
            .get(hidden.as_tensor())
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Descent moves the kept key, row 0, toward the query at 1 along coordinate 0: its gradient
        // there is negative (1 − (d − d̄) gives dL/dd = −dL/dfactor = +1, and dd/dk = −1/4).
        assert!(grad[0][0] < 0.0, "{grad:?}");
        assert!((grad[0][0] + 0.25).abs() < 1e-6, "{grad:?}");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn one_adamw_step_matches_the_hand_computed_step() {
        let device = Device::new_cuda(0).unwrap();
        let read = FlockRead::new(1, 4, &device).unwrap();
        let mut opt = FlockAdamW::new(&read).unwrap();
        // A loss whose gradient on the weight is [0.5, -0.25, 0, 0] (norm under the clip).
        let loss = read
            .weight
            .as_tensor()
            .mul(&Tensor::new(&[0.5f32, -0.25, 0.0, 0.0], &device).unwrap())
            .unwrap()
            .sum_all()
            .unwrap();
        let grads = loss.backward().unwrap();
        opt.step(&read, &grads, 0.1).unwrap();
        // Step 1: m̂ = g, v̂ = g², update = g / (|g| + ε) ≈ sign(g), so w = −0.1·sign(g).
        let w = read.weight.as_tensor().to_vec1::<f32>().unwrap();
        assert!((w[0] + 0.1).abs() < 1e-6, "{w:?}");
        assert!((w[1] - 0.1).abs() < 1e-6, "{w:?}");
        assert_eq!(w[2], 0.0);
        assert_eq!(w[3], 0.0);
    }
}
