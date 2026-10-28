//! The opening read (in-space-trunk-to-text, claude-154).
//!
//! For each position `t` of a window, the read looks back at up to `k` token points that open
//! the current stretch of text and adds their mean to the stream:
//! 1. **Slots** ([`opening_slots`]). With the **turn** anchor, the slots are the first `k`
//!    positions after the last `<|bos|>` or `<|eos|>` at or before `t` (or after the window's
//!    start when there is none), kept only up to `t`; a `<|bos|>` or `<|eos|>` itself reads
//!    nothing. With the **last** anchor, the slots are the last `k` positions up to `t`.
//! 2. **Points.** The shared table's point of each slot's token, detached, turned lane by lane by
//!    a learned quaternion per slot index, starting at the identity (the Hamming read's Hamilton
//!    product order: rotation on the left).
//! 3. **Combine.** The mean over the kept slots (zero where none), times a learned weight per
//!    coordinate that starts at 0: `x + w ⊙ mean`, so at step 0 the trunk is unchanged. Only
//!    per-lane quaternion products, a mean and an elementwise weight; nothing is exponential.
//!
//! The read is active for the thread that sets it ([`set_opening_read`]).
use std::cell::RefCell;
use std::path::Path;

use candle_core::{DType, Device, Tensor, Var};

use crate::hamilton_sheet::SHEET_BOS_ID;
use crate::{invalid, Result};

/// The sheet's `<|eos|>` id; `<|bos|>` is [`SHEET_BOS_ID`].
pub const SHEET_EOS_ID: u32 = 1;
/// The most slots a read may keep.
const MAX_SLOTS: usize = 16;
const FILE: &str = "opening_read.safetensors";

thread_local! {
    static ACTIVE: RefCell<Option<OpeningRead>> = const { RefCell::new(None) };
}

/// Make `read` the opening read applied on this thread, or none.
pub fn set_opening_read(read: Option<OpeningRead>) {
    ACTIVE.with(|cell| *cell.borrow_mut() = read);
}

/// The opening read applied on this thread, if set.
pub fn opening_read() -> Option<OpeningRead> {
    ACTIVE.with(|cell| cell.borrow().clone())
}

/// Where a position's slots start.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpeningAnchor {
    /// The first `k` positions after the last `<|bos|>` or `<|eos|>`.
    #[default]
    Turn,
    /// The last `k` positions up to and including `t`.
    Last,
}

impl OpeningAnchor {
    fn code(self) -> u32 {
        match self {
            Self::Turn => 0,
            Self::Last => 1,
        }
    }

    fn from_code(code: u32) -> Result<Self> {
        match code {
            0 => Ok(Self::Turn),
            1 => Ok(Self::Last),
            _ => Err(invalid(format!(
                "the saved opening anchor {code} is unknown"
            ))),
        }
    }
}

/// The slot positions of each position `t` of one window row `ids`, in slot order:
/// `result[t][j]` is the position slot `j` reads for `t`.
pub fn opening_slots(ids: &[u32], k: usize, anchor: OpeningAnchor) -> Vec<Vec<usize>> {
    let mut start = 0;
    (0..ids.len())
        .map(|t| match anchor {
            OpeningAnchor::Turn => {
                // A `<|bos|>` or `<|eos|>` moves the anchor past itself, so it reads nothing.
                if ids[t] == SHEET_BOS_ID || ids[t] == SHEET_EOS_ID {
                    start = t + 1;
                }
                (start..=t).take(k).collect()
            }
            OpeningAnchor::Last => ((t + 1).saturating_sub(k)..=t).collect(),
        })
        .collect()
}

/// One opening read: a quaternion per slot and lane, and the per-coordinate weight of what it
/// adds.
#[derive(Clone)]
pub struct OpeningRead {
    k: usize,
    anchor: OpeningAnchor,
    /// `(k, lanes, 4)`.
    pub(crate) rotation: Var,
    /// `(width,)`.
    pub(crate) weight: Var,
}

impl OpeningRead {
    /// A read of `k` slots on points of `width` coordinates: identity rotations and a zero weight,
    /// so it starts as no read at all.
    pub fn new(k: usize, width: usize, anchor: OpeningAnchor, device: &Device) -> Result<Self> {
        if !(1..=MAX_SLOTS).contains(&k) {
            return Err(invalid(format!(
                "an opening read keeps 1 to {MAX_SLOTS} slots"
            )));
        }
        if width == 0 || !width.is_multiple_of(4) {
            return Err(invalid("an opening read needs whole 4-lane blocks"));
        }
        let lanes = width / 4;
        let identity: Vec<f32> = (0..k * lanes)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect();
        Ok(Self {
            k,
            anchor,
            rotation: Var::from_tensor(&Tensor::from_vec(identity, (k, lanes, 4), device)?)?,
            weight: Var::zeros(width, DType::F32, device)?,
        })
    }

    pub fn k(&self) -> usize {
        self.k
    }

    pub fn anchor(&self) -> OpeningAnchor {
        self.anchor
    }

    /// The read applied to `x` `(batch, time, width)` of the windows `ids`, with token points
    /// from the shared `table`: `x + w ⊙ mean`. Returns the same shape.
    pub fn apply(&self, x: &Tensor, ids: &[u32], table: &Tensor) -> Result<Tensor> {
        let (batch, time, width) = x.dims3()?;
        let rows = batch * time;
        if ids.len() != rows {
            return Err(invalid("the opening read needs whole windows"));
        }
        if width != self.weight.dim(0)? {
            return Err(invalid(
                "the opening read's width differs from the stream's",
            ));
        }
        let device = x.device();
        let lanes = width / 4;
        let (slot_ids, slot_mask, count) = self.slot_tables(ids, batch, time);
        let table = table.detach();
        let mut sum = Tensor::zeros((rows, lanes, 4), DType::F32, device)?;
        for (j, (slot_ids, slot_mask)) in slot_ids.into_iter().zip(slot_mask).enumerate() {
            let points = table
                .index_select(&Tensor::from_vec(slot_ids, rows, device)?, 0)?
                .reshape((rows, lanes, 4))?;
            let rotation = self.rotation.as_tensor().get(j)?.unsqueeze(0)?;
            let turned = hamilton_product(&rotation, &points)?;
            let mask = Tensor::from_vec(slot_mask, (rows, 1, 1), device)?;
            sum = sum.add(&turned.broadcast_mul(&mask)?)?;
        }
        let count = Tensor::from_vec(count, (rows, 1, 1), device)?;
        let mean = sum.broadcast_div(&count)?.reshape((batch, time, width))?;
        Ok(x.add(&mean.broadcast_mul(self.weight.as_tensor())?)?)
    }

    /// The host side of [`apply`](Self::apply) for `batch` windows of `time` positions: per slot
    /// index, each row's token id and a 0/1 mask, and each row's kept-slot count (at least 1, so
    /// a position with no slot divides its zero sum by 1).
    fn slot_tables(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> (Vec<Vec<u32>>, Vec<Vec<f32>>, Vec<f32>) {
        let rows = batch * time;
        let mut slot_ids = vec![vec![0u32; rows]; self.k];
        let mut slot_mask = vec![vec![0f32; rows]; self.k];
        let mut count = vec![1f32; rows];
        for b in 0..batch {
            let row = &ids[b * time..(b + 1) * time];
            for (t, slots) in opening_slots(row, self.k, self.anchor).iter().enumerate() {
                let r = b * time + t;
                for (j, &s) in slots.iter().enumerate() {
                    slot_ids[j][r] = row[s];
                    slot_mask[j][r] = 1.0;
                }
                count[r] = slots.len().max(1) as f32;
            }
        }
        (slot_ids, slot_mask, count)
    }

    /// Every learned parameter, in a fixed order for the optimizer.
    pub fn variables(&self) -> Vec<&Var> {
        vec![&self.rotation, &self.weight]
    }

    /// The mean absolute value of the weight, as a scalar tensor.
    pub fn weight_mean_abs(&self) -> Result<Tensor> {
        Ok(self.weight.as_tensor().abs()?.mean_all()?)
    }

    /// Write the read to `directory/opening_read.safetensors`, with its `k` and anchor.
    pub fn save(&self, directory: &Path) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        let device = self.weight.device();
        let mut tensors = std::collections::HashMap::new();
        tensors.insert("rotation".to_owned(), self.rotation.as_tensor().clone());
        tensors.insert("weight".to_owned(), self.weight.as_tensor().clone());
        tensors.insert("k".to_owned(), Tensor::new(&[self.k as u32], device)?);
        tensors.insert(
            "anchor".to_owned(),
            Tensor::new(&[self.anchor.code()], device)?,
        );
        candle_core::safetensors::save(&tensors, directory.join(FILE))?;
        Ok(())
    }

    /// The read saved in `directory`, or none when the checkpoint has none.
    pub fn load(directory: &Path, device: &Device) -> Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.exists() {
            return Ok(None);
        }
        let tensors = candle_core::safetensors::load(path, device)?;
        let get = |name: &str| -> Result<&Tensor> {
            tensors
                .get(name)
                .ok_or_else(|| invalid(format!("the saved opening read has no {name}")))
        };
        let scalar = |name: &str| -> Result<u32> {
            get(name)?
                .to_vec1::<u32>()?
                .first()
                .copied()
                .ok_or_else(|| invalid(format!("the saved opening read's {name} is empty")))
        };
        let read = Self {
            k: scalar("k")? as usize,
            anchor: OpeningAnchor::from_code(scalar("anchor")?)?,
            rotation: Var::from_tensor(get("rotation")?)?,
            weight: Var::from_tensor(get("weight")?)?,
        };
        let width = read.weight.dim(0)?;
        if !(1..=MAX_SLOTS).contains(&read.k) || width == 0 || !width.is_multiple_of(4) {
            return Err(invalid(format!(
                "the saved opening read keeps {} slots over {width} coordinates; it needs 1 to \
                 {MAX_SLOTS} slots over whole 4-lane blocks",
                read.k
            )));
        }
        if read.rotation.dims() != [read.k, width / 4, 4] {
            return Err(invalid(
                "the saved opening read's rotation does not match its k",
            ));
        }
        Ok(Some(read))
    }
}

/// `p ⊗ q` lane by lane on `(…, lanes, 4)` tensors, `p` broadcast over `q`'s rows, in the
/// Hamming read's order (the rotation on the left).
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
    #[cfg(feature = "cuda")]
    const VOCAB: usize = 40;

    /// Deterministic values in [−0.8, 0.8], with no random source.
    fn values(n: usize, salt: usize) -> Vec<f32> {
        (0..n)
            .map(|i| ((i * 37 + salt * 11) % 17) as f32 * 0.1 - 0.8)
            .collect()
    }

    // The vendored candle refuses tensor compute on the CPU (D24/D29), so the tests that run
    // `apply` are CUDA tests; the slot rule and save/load are host tests.
    #[cfg(feature = "cuda")]
    fn table(device: &Device) -> Tensor {
        Tensor::from_vec(values(VOCAB * WIDTH, 3), (VOCAB, WIDTH), device).unwrap()
    }

    #[cfg(feature = "cuda")]
    fn quaternion(p: [f32; 4], q: [f32; 4]) -> [f32; 4] {
        let ([a1, b1, c1, d1], [a2, b2, c2, d2]) = (p, q);
        [
            a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
            a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2,
            a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2,
            a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2,
        ]
    }

    const ROW: [u32; 13] = [0, 10, 11, 12, 20, 21, 1, 13, 14, 22, 1, 0, 0];
    /// A row that starts mid-prompt: no `<|bos|>` or `<|eos|>` before position 3.
    const MID: [u32; 13] = [30, 31, 32, 0, 33, 34, 35, 36, 1, 37, 38, 1, 0];

    #[test]
    fn the_turn_anchor_starts_after_the_last_bos_or_eos() {
        let turn = opening_slots(&ROW, 3, OpeningAnchor::Turn);
        assert_eq!(turn.len(), ROW.len());
        let expect: [&[usize]; 13] = [
            &[],
            &[1],
            &[1, 2],
            &[1, 2, 3],
            &[1, 2, 3],
            &[1, 2, 3],
            &[],
            &[7],
            &[7, 8],
            &[7, 8, 9],
            &[],
            &[],
            &[],
        ];
        for (t, e) in expect.iter().enumerate() {
            assert_eq!(turn[t], *e, "turn slots at t = {t}");
        }
        let mid = opening_slots(&MID, 3, OpeningAnchor::Turn);
        assert_eq!(mid[0], vec![0]);
        assert_eq!(mid[2], vec![0, 1, 2]);
        assert!(mid[3].is_empty());
        assert_eq!(mid[7], vec![4, 5, 6]);
        assert!(mid[8].is_empty());
        assert_eq!(mid[10], vec![9, 10]);
        let last = opening_slots(&ROW, 3, OpeningAnchor::Last);
        assert_eq!(last[0], vec![0]);
        assert_eq!(last[1], vec![0, 1]);
        assert_eq!(last[2], vec![0, 1, 2]);
        assert_eq!(last[6], vec![4, 5, 6]);
        assert_eq!(last[12], vec![10, 11, 12]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_zero_weight_opening_read_is_no_read() {
        let device = Device::new_cuda(0).unwrap();
        let read = OpeningRead::new(3, WIDTH, OpeningAnchor::Turn, &device).unwrap();
        let x = Tensor::from_vec(values(2 * 13 * WIDTH, 5), (2, 13, WIDTH), &device).unwrap();
        let ids: Vec<u32> = ROW.iter().chain(MID.iter()).copied().collect();
        let out = read.apply(&x, &ids, &table(&device)).unwrap();
        assert_eq!(
            out.flatten_all().unwrap().to_vec1::<f32>().unwrap(),
            x.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_opening_read_adds_the_mean_of_its_turned_slots() {
        let device = Device::new_cuda(0).unwrap();
        let table = table(&device);
        let table_rows = table.to_vec2::<f32>().unwrap();
        let ids: Vec<u32> = ROW.iter().chain(MID.iter()).copied().collect();
        let x = Tensor::from_vec(values(2 * 13 * WIDTH, 7), (2, 13, WIDTH), &device).unwrap();
        let lanes = WIDTH / 4;
        // A general quaternion in every slot and lane, so every term of the product counts; and
        // every position of both rows, so partial and empty slot lists count too.
        let rotation = values(3 * lanes * 4, 4);
        for anchor in [OpeningAnchor::Turn, OpeningAnchor::Last] {
            let read = OpeningRead::new(3, WIDTH, anchor, &device).unwrap();
            read.weight
                .set(&Tensor::ones(WIDTH, DType::F32, &device).unwrap())
                .unwrap();
            read.rotation
                .set(&Tensor::from_vec(rotation.clone(), (3, lanes, 4), &device).unwrap())
                .unwrap();
            let out = read.apply(&x, &ids, &table).unwrap();
            let added = out.sub(&x).unwrap().to_vec3::<f32>().unwrap();
            for b in 0..2 {
                let row = &ids[b * 13..(b + 1) * 13];
                for (t, slots) in opening_slots(row, 3, anchor).iter().enumerate() {
                    let mut expect = vec![0.0f32; WIDTH];
                    for (j, &s) in slots.iter().enumerate() {
                        let point = &table_rows[row[s] as usize];
                        for lane in 0..lanes {
                            let at = (j * lanes + lane) * 4;
                            let r = [
                                rotation[at],
                                rotation[at + 1],
                                rotation[at + 2],
                                rotation[at + 3],
                            ];
                            let q = [
                                point[lane * 4],
                                point[lane * 4 + 1],
                                point[lane * 4 + 2],
                                point[lane * 4 + 3],
                            ];
                            let turned = quaternion(r, q);
                            for i in 0..4 {
                                expect[lane * 4 + i] += turned[i] / slots.len() as f32;
                            }
                        }
                    }
                    for (a, e) in added[b][t].iter().zip(&expect) {
                        assert!(
                            (a - e).abs() < 1e-5,
                            "{anchor:?} b {b} t {t}: added {:?}, expected {expect:?}",
                            added[b][t]
                        );
                    }
                }
            }
        }
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_opening_read_trains_its_rotation_and_weight_and_never_the_table() {
        let device = Device::new_cuda(0).unwrap();
        let table = Var::from_tensor(&table(&device)).unwrap();
        let ids: Vec<u32> = ROW.iter().chain(MID.iter()).copied().collect();
        let x = Tensor::from_vec(values(2 * 13 * WIDTH, 7), (2, 13, WIDTH), &device).unwrap();
        let read = OpeningRead::new(3, WIDTH, OpeningAnchor::Turn, &device).unwrap();
        read.weight
            .set(&Tensor::ones(WIDTH, DType::F32, &device).unwrap())
            .unwrap();
        let grads = read
            .apply(&x, &ids, table.as_tensor())
            .unwrap()
            .sum_all()
            .unwrap()
            .backward()
            .unwrap();
        assert!(grads.get(read.rotation.as_tensor()).is_some());
        assert!(grads.get(read.weight.as_tensor()).is_some());
        assert!(grads.get(table.as_tensor()).is_none());
    }

    #[test]
    fn the_slot_tables_count_and_mask_each_kept_slot() {
        let ids: Vec<u32> = ROW.iter().chain(MID.iter()).copied().collect();
        for anchor in [OpeningAnchor::Turn, OpeningAnchor::Last] {
            let read = OpeningRead::new(3, WIDTH, anchor, &Device::Cpu).unwrap();
            let (slot_ids, slot_mask, count) = read.slot_tables(&ids, 2, 13);
            for b in 0..2 {
                let row = &ids[b * 13..(b + 1) * 13];
                for (t, slots) in opening_slots(row, 3, anchor).iter().enumerate() {
                    let r = b * 13 + t;
                    assert_eq!(count[r], slots.len().max(1) as f32, "{anchor:?} r {r}");
                    for j in 0..3 {
                        let kept = j < slots.len();
                        assert_eq!(
                            slot_mask[j][r],
                            f32::from(u8::from(kept)),
                            "{anchor:?} r {r} j {j}"
                        );
                        if kept {
                            assert_eq!(slot_ids[j][r], row[slots[j]], "{anchor:?} r {r} j {j}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_saved_opening_read_out_of_range_is_refused() {
        let device = Device::Cpu;
        let directory = std::env::temp_dir().join(format!(
            "uor-opening-read-refused-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let mut tensors = std::collections::HashMap::new();
        tensors.insert(
            "rotation".to_owned(),
            Tensor::from_vec(Vec::<f32>::new(), (0, 2, 4), &device).unwrap(),
        );
        tensors.insert(
            "weight".to_owned(),
            Tensor::from_vec(values(WIDTH, 1), WIDTH, &device).unwrap(),
        );
        tensors.insert("k".to_owned(), Tensor::new(&[0u32], &device).unwrap());
        tensors.insert("anchor".to_owned(), Tensor::new(&[0u32], &device).unwrap());
        candle_core::safetensors::save(&tensors, directory.join(FILE)).unwrap();
        assert!(OpeningRead::load(&directory, &device).is_err());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn the_opening_read_saves_and_loads() {
        let device = Device::Cpu;
        let directory = std::env::temp_dir().join(format!(
            "uor-opening-read-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let read = OpeningRead::new(5, WIDTH, OpeningAnchor::Last, &device).unwrap();
        read.weight
            .set(&Tensor::from_vec(values(WIDTH, 9), WIDTH, &device).unwrap())
            .unwrap();
        read.rotation
            .set(&Tensor::from_vec(values(5 * 2 * 4, 2), (5, 2, 4), &device).unwrap())
            .unwrap();
        read.save(&directory).unwrap();
        let loaded = OpeningRead::load(&directory, &device).unwrap().unwrap();
        assert_eq!(loaded.k(), 5);
        assert_eq!(loaded.anchor(), OpeningAnchor::Last);
        let flat = |v: &Var| {
            v.as_tensor()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap()
        };
        assert_eq!(flat(&loaded.rotation), flat(&read.rotation));
        assert_eq!(flat(&loaded.weight), flat(&read.weight));
        std::fs::remove_file(directory.join(FILE)).unwrap();
        assert!(OpeningRead::load(&directory, &device).unwrap().is_none());
        std::fs::remove_dir(&directory).unwrap();
    }
}
