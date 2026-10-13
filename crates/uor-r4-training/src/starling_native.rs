//! Native starling / murmuration read (rung 0b; M4 #2032, design card
//! 6146209361 with amendment A1 6147797508).
//!
//! **Forward.** Each (batch row, head) keeps, per query, the `K` sources with
//! the largest address dot among the keys stored in its home 600-cell vertex
//! and that vertex's 12 neighbours (kernel K1, `starling_native.cu`; rings of
//! depth `M`, cleared at every document start). The next `E` candidates are
//! exploration slots with exactly zero forward weight. The kept slots are
//! ranked by their content score `s = <q, k> / sqrt(d_head)` together with a
//! learned per-head null score `b`: every real slot ranked below the null is
//! dropped, and the survivors read `T[h, r]` by their rank `r`. A null ranked
//! first reads exactly 0. No exponential, no normalisation, no gate.
//!
//! **Backward.** The values and the rank table get the exact derivatives of
//! that forward. The hard ranking has none, so the declared credit
//! ([`Credit`]) is injected by [`CreditCapture`], which is the identity on the
//! read and receives the read's upstream gradient `g`:
//! * slot utility `u = -<g, v>` (detached; the null's is 0);
//! * surrogate A: a pairwise hinge `max(0, mu - (s_a - s_b))` over the pairs
//!   of kept, exploration and null candidates with `u_a - u_b > delta`,
//!   divided per query by `max(1, pairs)`, trains the content scores and the
//!   null score only;
//! * surrogate B: when the best-utility candidate is helpful and lies off the
//!   query's home vertex, `max(0, mu_a - (<a, rho_best> - <a, rho_home>))`
//!   trains the query address only.
//!
//! Neither surrogate is the derivative of hard selection or a descent
//! guarantee. Credit adds no value or rank-table gradient. Model arithmetic
//! is CUDA-only (D24); every CPU entry refuses.

use std::sync::{Arc, Mutex};

use candle_core::{CpuStorage, CustomOp3, DType, Layout, Shape, Tensor};
use serde::{Deserialize, Serialize};

use crate::{invalid, Result};

/// The `read_route=` prefix of the native read.
pub const ROUTE_PREFIX: &str = "starling_native";
/// An unfilled slot in K1's output.
pub const ABSENT: u32 = u32::MAX;
/// K + E may not exceed this (the kernel's register list).
pub const MAX_SLOTS: usize = 16;
/// Vertices of the 600-cell (the icosian codebook K1 addresses).
pub const CELLS: usize = 120;

/// The declared surrogate constants (A1 section B). The fixture values are the
/// card's; a training run registers its own in its sheet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Credit {
    /// Score margin `mu` of the pairwise hinge.
    pub margin: f32,
    /// Utility gap `delta` a pair must exceed (strictly).
    pub delta: f32,
    /// Address margin `mu_a`.
    pub address_margin: f32,
    /// Coefficient of the rank hinge.
    pub rank: f32,
    /// Coefficient of the address margin.
    pub address: f32,
}

impl Credit {
    /// The constants of the card's finite fixtures (A1 section B, with the
    /// FIT review's binding correction 6148141029: delta = 0.125, exactly
    /// representable, so the strict boundary case is exact in F32).
    pub const FIXTURE: Credit = Credit {
        margin: 1.0,
        delta: 0.125,
        address_margin: 0.1,
        rank: 1.0,
        address: 1.0,
    };

    fn validate(&self) -> Result<()> {
        let values = [
            self.margin,
            self.delta,
            self.address_margin,
            self.rank,
            self.address,
        ];
        if values.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err(invalid(
                "starling credit constants must be finite and non-negative",
            ));
        }
        Ok(())
    }
}

/// The native read's configuration, bound into the artifact's `config.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeStarling {
    /// Ring depth `M` per vertex.
    pub depth: usize,
    /// Kept slots `K`.
    pub kept: usize,
    /// Exploration slots `E` (zero forward weight).
    pub explore: usize,
    /// The tokenizer's BOS identity: every ring clears at a document start.
    pub bos_id: u32,
    pub credit: Credit,
}

impl NativeStarling {
    /// `starling_native:M:K:E` with the given BOS identity and credit.
    pub fn parse(text: &str, bos_id: u32, credit: Credit) -> Result<Self> {
        let parts: Vec<&str> = text.split(':').collect();
        let numbers = match parts.as_slice() {
            [ROUTE_PREFIX, m, k, e] => [m, k, e]
                .iter()
                .map(|v| v.parse::<usize>())
                .collect::<std::result::Result<Vec<_>, _>>()
                .ok(),
            _ => None,
        };
        let Some(numbers) = numbers else {
            return Err(invalid(format!(
                "invalid native read route {text} (starling_native:M:K:E)"
            )));
        };
        let spec = Self {
            depth: numbers[0],
            kept: numbers[1],
            explore: numbers[2],
            bos_id,
            credit,
        };
        spec.validate()?;
        Ok(spec)
    }

    pub fn validate(&self) -> Result<()> {
        if self.depth == 0 || self.kept == 0 || self.slots() > MAX_SLOTS {
            return Err(invalid(format!(
                "starling_native needs M >= 1, K >= 1 and K + E <= {MAX_SLOTS}"
            )));
        }
        self.credit.validate()
    }

    pub fn name(&self) -> String {
        format!(
            "{ROUTE_PREFIX}:{}:{}:{}",
            self.depth, self.kept, self.explore
        )
    }

    /// Kept plus exploration slots K1 returns per query.
    pub fn slots(&self) -> usize {
        self.kept + self.explore
    }
}

fn require_cuda(tensor: &Tensor, what: &str) -> Result<()> {
    if !tensor.device().is_cuda() {
        return Err(invalid(format!(
            "{what} is GPU-only model compute; no CPU or Metal execution (D24)"
        )));
    }
    Ok(())
}

/// The fixed icosian roots `[120, 4]` and their 12 neighbours `[120, 12]`.
pub(crate) fn geometry() -> Result<(Vec<f32>, Vec<u32>)> {
    let roots = crate::geometric_stack::starling_native_roots();
    let adjacency = crate::geometric_stack::starling_native_adjacency()?;
    if roots.len() != CELLS || adjacency.len() != CELLS {
        return Err(invalid("the 600-cell codebook must have 120 vertices"));
    }
    Ok((
        roots.iter().flatten().copied().collect(),
        adjacency.iter().flatten().map(|&c| c as u32).collect(),
    ))
}

/// Kernel K1 as a graph-free op: selection is hard and takes no gradient.
struct Select {
    spec: NativeStarling,
    heads: usize,
}

impl CustomOp3 for Select {
    fn name(&self) -> &'static str {
        "starling-native-select"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("starling selection is GPU-only; no host fallback (D24)")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        query: &candle_core::CudaStorage,
        ql: &Layout,
        key: &candle_core::CudaStorage,
        kl: &Layout,
        reset: &candle_core::CudaStorage,
        rl: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        crate::geometric_stack::cuda_ops::starling_native_select(
            &self.spec, self.heads, query, ql, key, kl, reset, rl,
        )
    }
}

/// K1: per (row, position), the `K + E` slot positions in rank order
/// ([`ABSENT`] when fewer exist), then the home cell and the key's own cell:
/// `[rows, time, K + E + 2]` u32. `query`/`key` are `[rows, time, 4]` unit
/// addresses (`rows = batch * heads`); `reset` is `[batch, time]` u32, 1 at a
/// document start.
pub fn select(
    spec: &NativeStarling,
    heads: usize,
    query: &Tensor,
    key: &Tensor,
    reset: &Tensor,
) -> Result<Tensor> {
    spec.validate()?;
    require_cuda(query, "starling selection")?;
    let (rows, time, four) = query.dims3()?;
    let (batch, reset_time) = reset.dims2()?;
    if four != 4
        || key.dims() != query.dims()
        || heads == 0
        || rows != batch * heads
        || reset_time != time
        || query.dtype() != DType::F32
        || key.dtype() != DType::F32
        || reset.dtype() != DType::U32
    {
        return Err(invalid(
            "starling selection needs [batch*heads, time, 4] F32 addresses and a [batch, time] U32 reset",
        ));
    }
    Ok(query.detach().contiguous()?.apply_op3_no_bwd(
        &key.detach().contiguous()?,
        &reset.contiguous()?,
        &Select { spec: *spec, heads },
    )?)
}

/// Unit 4-d addresses `raw / sqrt(|raw|^2 + 1e-12)` over the last axis: the
/// normalisation the read layer applies to its FBS address maps, shared with
/// the composed credit control so both run the same arithmetic.
pub fn unit_address(raw: &Tensor) -> Result<Tensor> {
    let raw = raw.to_dtype(DType::F32)?;
    let last = raw
        .rank()
        .checked_sub(1)
        .ok_or_else(|| invalid("address needs an axis"))?;
    let norm = raw.sqr()?.sum_keepdim(last)?.affine(1.0, 1e-12)?.sqrt()?;
    Ok(raw.broadcast_div(&norm)?.contiguous()?)
}

/// The document-start mask of `ids` (`[batch * time]`): 1 where the token is
/// the BOS identity. Index metadata only; no model arithmetic.
pub fn reset_mask(ids: &[u32], batch: usize, time: usize, bos_id: u32) -> Result<Vec<u32>> {
    if ids.len() != batch * time {
        return Err(invalid("starling reset mask needs batch * time ids"));
    }
    Ok(ids.iter().map(|&id| u32::from(id == bos_id)).collect())
}

/// Device scalars the credit backward records for reporting: the rank hinge,
/// the address margin and the active-pair count of one layer's backward.
#[derive(Clone, Debug)]
pub struct CreditReport {
    pub rank_hinge: Tensor,
    pub address_margin: Tensor,
    pub active_pairs: Tensor,
}

/// Where a model collects [`CreditReport`]s (optional; reporting only).
pub type CreditSink = Arc<Mutex<Vec<CreditReport>>>;

/// What the credit backward needs from the forward, all detached.
#[derive(Clone)]
pub(crate) struct CreditState {
    pub credit: Credit,
    /// Gathered slot values `[rows, time, S, dv]`.
    pub values: Tensor,
    /// 1 for a filled slot, `[rows, time, S]` F32.
    pub live: Tensor,
    /// Slot positions as F32 (ties: the later position), `[rows, time, S]`.
    pub position: Tensor,
    /// 1 where the slot's key cell differs from the query's home cell,
    /// `[rows, time, S]` F32.
    pub off_home: Tensor,
    pub sink: Option<CreditSink>,
}

/// The identity on the read whose backward injects the declared credit into
/// the score and address-dot tensors (A1 section C: one capture point, so no
/// gradient is lost or counted twice).
struct CreditCapture(CreditState);

impl CustomOp3 for CreditCapture {
    fn name(&self) -> &'static str {
        "starling-native-credit"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("starling credit capture is GPU-only; no host fallback (D24)")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        read: &candle_core::CudaStorage,
        layout: &Layout,
        _: &candle_core::CudaStorage,
        _: &Layout,
        _: &candle_core::CudaStorage,
        _: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        use candle_core::backend::BackendStorage;
        if !layout.is_contiguous() || layout.start_offset() != 0 {
            candle_core::bail!("starling credit capture needs a contiguous read");
        }
        Ok((read.try_clone(layout)?, layout.shape().clone()))
    }
    fn bwd(
        &self,
        _read: &Tensor,
        scores: &Tensor,
        dots: &Tensor,
        _res: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        let (score_grad, dot_grad) = credit_backward(&self.0, scores, dots, grad)
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        Ok((Some(grad.clone()), Some(score_grad), Some(dot_grad)))
    }
}

/// The credit gradients for the candidate scores `[rows, time, S + 1]` (the
/// null last) and the address dots `[rows, time, S + 1]` (the home vertex
/// last), from the read's upstream gradient `grad` (`[.., time, dv]`, any
/// leading shape with `rows * time` rows). Device tensor arithmetic only.
pub(crate) fn credit_backward(
    state: &CreditState,
    scores: &Tensor,
    dots: &Tensor,
    grad: &Tensor,
) -> Result<(Tensor, Tensor)> {
    require_cuda(grad, "starling credit")?;
    let (rows, time, slots, width) = state.values.dims4()?;
    let candidates = slots + 1;
    if scores.dims() != [rows, time, candidates] || dots.dims() != [rows, time, candidates] {
        return Err(invalid(
            "starling credit tensors disagree with the slot layout",
        ));
    }
    let device = grad.device();
    let credit = state.credit;
    let queries = (rows * time) as f64;
    let g = grad.to_dtype(DType::F32)?.reshape((rows, time, 1, width))?;
    // Utility of each real slot, u = -<g, v>; dead slots are masked below.
    let utility = state
        .values
        .broadcast_mul(&g)?
        .sum(3)?
        .neg()?
        .mul(&state.live)?;
    let zero_column = Tensor::zeros((rows, time, 1), DType::F32, device)?;
    let one_column = Tensor::ones((rows, time, 1), DType::F32, device)?;
    let u = Tensor::cat(&[&utility, &zero_column], 2)?; // the null's utility is 0
    let live = Tensor::cat(&[&state.live, &one_column], 2)?;
    let s = scores.detach().to_dtype(DType::F32)?;

    // Pairs (a, b): both live and u_a - u_b > delta (strictly).
    let ua = u.unsqueeze(3)?; // [rows, time, C, 1]
    let ub = u.unsqueeze(2)?; // [rows, time, 1, C]
    let gap = ua.broadcast_sub(&ub)?;
    let both = live.unsqueeze(3)?.broadcast_mul(&live.unsqueeze(2)?)?;
    let pair = gap
        .gt(credit.delta as f64)?
        .to_dtype(DType::F32)?
        .mul(&both)?;
    // Hinge max(0, mu - (s_a - s_b)) and its active indicator (> 0 strictly).
    let ds = s.unsqueeze(3)?.broadcast_sub(&s.unsqueeze(2)?)?;
    let raw = ds.neg()?.affine(1.0, credit.margin as f64)?;
    let active = raw.gt(0.0)?.to_dtype(DType::F32)?.mul(&pair)?;
    let hinge = raw.relu()?.mul(&pair)?;
    let pairs = pair.sum((2, 3))?; // [rows, time]
    let normalizer = pairs
        .maximum(&Tensor::ones((rows, time), DType::F32, device)?)?
        .recip()?;
    // d hinge / d s_a = -1 and d / d s_b = +1 on every active pair.
    let per_query = normalizer
        .affine(credit.rank as f64 / queries, 0.0)?
        .unsqueeze(2)?;
    let score_grad = active
        .sum(3)?
        .neg()?
        .add(&active.sum(2)?)?
        .broadcast_mul(&per_query)?;

    // Surrogate B: the best-utility live real slot (ties: the later position).
    let neg = Tensor::full(-3.0e38f32, (rows, time, slots), device)?;
    let live_mask = state.live.gt(0.0)?;
    let masked_u = live_mask.where_cond(&utility, &neg)?;
    let best_u = masked_u.max_keepdim(2)?;
    let tied = masked_u.broadcast_eq(&best_u)?.mul(&live_mask)?;
    let tied_position = tied.where_cond(
        &state.position,
        &Tensor::full(-1.0f32, (rows, time, slots), device)?,
    )?;
    let best_position = tied_position.max_keepdim(2)?;
    let chosen = tied_position
        .broadcast_eq(&best_position)?
        .mul(&tied)?
        .to_dtype(DType::F32)?; // one-hot over slots (positions are distinct)
    let helpful = best_u.squeeze(2)?.gt(0.0)?.to_dtype(DType::F32)?;
    let off_home = chosen.mul(&state.off_home)?.sum(2)?;
    let real_dots = dots.detach().to_dtype(DType::F32)?.narrow(2, 0, slots)?;
    let home_dot = dots
        .detach()
        .to_dtype(DType::F32)?
        .narrow(2, slots, 1)?
        .squeeze(2)?;
    let best_dot = chosen.mul(&real_dots)?.sum(2)?;
    let address_raw = best_dot
        .sub(&home_dot)?
        .neg()?
        .affine(1.0, credit.address_margin as f64)?;
    let address_on = helpful.mul(&off_home)?;
    let address_active = address_raw
        .gt(0.0)?
        .to_dtype(DType::F32)?
        .mul(&address_on)?;
    let address_scale = address_active.affine(credit.address as f64 / queries, 0.0)?;
    // d / d dot_best = -1, d / d dot_home = +1.
    let dot_grad = Tensor::cat(
        &[
            &chosen.broadcast_mul(&address_scale.unsqueeze(2)?)?.neg()?,
            &address_scale.unsqueeze(2)?,
        ],
        2,
    )?;

    if let Some(sink) = &state.sink {
        let report = CreditReport {
            rank_hinge: hinge
                .sum((2, 3))?
                .mul(&normalizer)?
                .sum_all()?
                .affine(1.0 / queries, 0.0)?,
            address_margin: address_raw
                .relu()?
                .mul(&address_on)?
                .sum_all()?
                .affine(1.0 / queries, 0.0)?,
            active_pairs: pairs.sum_all()?,
        };
        sink.lock()
            .map_err(|_| invalid("starling credit sink poisoned"))?
            .push(report);
    }
    Ok((score_grad, dot_grad))
}

/// Inputs of one native read, `B` batch, `H` heads, `T` time.
pub struct ReadInputs<'a> {
    /// Content queries and keys `[B, H, T, d]`.
    pub query: &'a Tensor,
    pub key: &'a Tensor,
    /// Values `[B, H, T, dv]`.
    pub value: &'a Tensor,
    /// Unit query addresses `[B, H, T, 4]` (the address credit's target).
    pub query_address: &'a Tensor,
    /// K1's output `[B * H, T, S + 2]`.
    pub selection: &'a Tensor,
    /// Rank table `[H, K]` and null score `[H]`.
    pub rank_table: &'a Tensor,
    pub null_score: &'a Tensor,
}

/// The native read `[B, H, T, dv]`: hard rank read over the kept slots with
/// the learned null, plus (when `credit` is on) the credit capture.
pub fn rank_read(
    spec: &NativeStarling,
    inputs: &ReadInputs<'_>,
    credit: bool,
    sink: Option<CreditSink>,
) -> Result<Tensor> {
    spec.validate()?;
    require_cuda(inputs.value, "the starling read")?;
    let (batch, heads, time, d) = inputs.query.dims4()?;
    let dv = inputs.value.dim(3)?;
    let (k, s) = (spec.kept, spec.slots());
    let rows = batch * heads;
    if inputs.key.dims() != inputs.query.dims()
        || inputs.value.dims() != [batch, heads, time, dv]
        || inputs.query_address.dims() != [batch, heads, time, 4]
        || inputs.selection.dims() != [rows, time, s + 2]
        || inputs.rank_table.dims() != [heads, k]
        || inputs.null_score.dims() != [heads]
    {
        return Err(invalid(
            "starling read tensors disagree with the declared route",
        ));
    }
    let device = inputs.value.device();
    let selection = inputs.selection;
    let index = selection.narrow(2, 0, s)?.contiguous()?; // [rows, time, S]
    let home = selection.narrow(2, s, 1)?.squeeze(2)?.contiguous()?; // [rows, time]
    let key_cell = selection.narrow(2, s + 1, 1)?.squeeze(2)?.contiguous()?;
    let absent = Tensor::full(ABSENT, (rows, time, s), device)?;
    let live_u8 = index.ne(&absent)?;
    let live = live_u8.to_dtype(DType::F32)?;
    let safe = live_u8.where_cond(&index, &Tensor::zeros((rows, time, s), DType::U32, device)?)?;
    // Flat row index of every slot: row * time + position (index metadata).
    let base = Tensor::from_vec(
        (0..rows).map(|r| (r * time) as u32).collect::<Vec<_>>(),
        (rows, 1, 1),
        device,
    )?;
    let flat = safe.broadcast_add(&base)?.flatten_all()?;
    let gather = |x: &Tensor, width: usize| -> Result<Tensor> {
        Ok(x.contiguous()?
            .reshape((rows * time, width))?
            .index_select(&flat, 0)?
            .reshape((rows, time, s, width))?)
    };
    let query = inputs
        .query
        .to_dtype(DType::F32)?
        .reshape((rows, time, 1, d))?;
    let keys = gather(&inputs.key.to_dtype(DType::F32)?, d)?;
    let values = gather(&inputs.value.to_dtype(DType::F32)?, dv)?;
    // Content scores s = <q, k> / sqrt(d), [rows, time, S].
    let scores = keys
        .broadcast_mul(&query)?
        .sum(3)?
        .affine(1.0 / (d as f64).sqrt(), 0.0)?;

    // Hard rank among the kept real slots (ties: the later position).
    let kept_live = live.narrow(2, 0, k)?;
    let kept_scores = scores.detach().narrow(2, 0, k)?;
    let position = safe.to_dtype(DType::F32)?;
    let kept_position = position.narrow(2, 0, k)?;
    // The head of each row, row % heads (index metadata).
    let head_index = Tensor::from_vec(
        (0..rows).map(|r| (r % heads) as u32).collect::<Vec<_>>(),
        rows,
        device,
    )?;
    let null_rows = inputs
        .null_score
        .to_dtype(DType::F32)?
        .index_select(&head_index, 0)?
        .reshape((rows, 1, 1))?; // [rows, 1, 1]
    let si = kept_scores.unsqueeze(3)?; // slot i
    let sj = kept_scores.unsqueeze(2)?; // slot j
    let pi = kept_position.unsqueeze(3)?;
    let pj = kept_position.unsqueeze(2)?;
    let above = sj
        .broadcast_gt(&si)?
        .to_dtype(DType::F32)?
        .add(
            &sj.broadcast_eq(&si)?
                .to_dtype(DType::F32)?
                .mul(&pj.broadcast_gt(&pi)?.to_dtype(DType::F32)?)?,
        )?
        .broadcast_mul(&kept_live.unsqueeze(2)?)?; // only live rivals rank above
    let rank = above.sum(3)?; // [rows, time, K]
                              // A real slot survives when it ranks at or above the null (ties: real).
    let survive = kept_scores
        .broadcast_ge(&null_rows.detach())?
        .to_dtype(DType::F32)?
        .mul(&kept_live)?;
    let table_index = rank
        .broadcast_add(
            &head_index
                .to_dtype(DType::F32)?
                .affine(k as f64, 0.0)?
                .reshape((rows, 1, 1))?,
        )?
        .to_dtype(DType::U32)?
        .flatten_all()?;
    let table = inputs
        .rank_table
        .to_dtype(DType::F32)?
        .flatten_all()?
        .index_select(&table_index, 0)?
        .reshape((rows, time, k))?;
    let weights = table.mul(&survive)?; // [rows, time, K]
    let read = weights
        .unsqueeze(3)?
        .broadcast_mul(&values.narrow(2, 0, k)?)?
        .sum(2)?; // [rows, time, dv]
    let read = if credit {
        // Address dots <a, rho_cell> of every slot's key cell, the home last.
        let (roots, _) = geometry()?;
        let roots = Tensor::from_vec(roots, (CELLS, 4), device)?;
        let address = inputs
            .query_address
            .to_dtype(DType::F32)?
            .reshape((rows, time, 1, 4))?;
        let cells = Tensor::cat(
            &[
                &key_cell
                    .flatten_all()?
                    .index_select(&flat, 0)?
                    .reshape((rows, time, s))?,
                &home.unsqueeze(2)?,
            ],
            2,
        )?;
        let cell_roots =
            roots
                .index_select(&cells.flatten_all()?, 0)?
                .reshape((rows, time, s + 1, 4))?;
        let dots = cell_roots.broadcast_mul(&address)?.sum(3)?;
        let off_home = cells
            .narrow(2, 0, s)?
            .broadcast_ne(&home.unsqueeze(2)?)?
            .to_dtype(DType::F32)?;
        let null_column = null_rows.broadcast_as((rows, time, 1))?;
        let candidate_scores = Tensor::cat(&[&scores, &null_column.contiguous()?], 2)?;
        let state = CreditState {
            credit: spec.credit,
            values: values.detach(),
            live: live.clone(),
            position,
            off_home,
            sink,
        };
        read.contiguous()?.apply_op3(
            &candidate_scores.contiguous()?,
            &dots.contiguous()?,
            CreditCapture(state),
        )?
    } else {
        read
    };
    Ok(read
        .reshape((batch, heads, time, dv))?
        .to_dtype(inputs.value.dtype())?)
}

#[cfg(test)]
mod spec_tests {
    use super::*;

    #[test]
    fn route_parses_and_refuses_bad_shapes() -> Result<()> {
        let spec = NativeStarling::parse("starling_native:8:7:2", 0, Credit::FIXTURE)?;
        assert_eq!((spec.depth, spec.kept, spec.explore), (8, 7, 2));
        assert_eq!(spec.name(), "starling_native:8:7:2");
        assert_eq!(spec.slots(), 9);
        for bad in [
            "starling:8:7",
            "starling_native:8:7",
            "starling_native:0:7:2",
            "starling_native:8:0:2",
            "starling_native:8:12:5",
            "starling_native:8:x:2",
        ] {
            assert!(
                NativeStarling::parse(bad, 0, Credit::FIXTURE).is_err(),
                "{bad}"
            );
        }
        let mut credit = Credit::FIXTURE;
        credit.delta = -0.1;
        assert!(NativeStarling::parse("starling_native:8:7:2", 0, credit).is_err());
        credit.delta = f32::NAN;
        assert!(NativeStarling::parse("starling_native:8:7:2", 0, credit).is_err());
        Ok(())
    }

    #[test]
    fn reset_mask_marks_document_starts_only() -> Result<()> {
        assert_eq!(
            reset_mask(&[0, 5, 6, 0, 7, 0], 2, 3, 0)?,
            vec![1, 0, 0, 1, 0, 1]
        );
        assert!(reset_mask(&[0, 5], 2, 3, 0).is_err());
        Ok(())
    }

    #[test]
    fn codebook_has_120_vertices_with_12_neighbours() -> Result<()> {
        let (roots, adjacency) = geometry()?;
        assert_eq!(roots.len(), CELLS * 4);
        assert_eq!(adjacency.len(), CELLS * 12);
        // Neighbours sit at 36 degrees: dot = cos 36 = phi / 2.
        let phi_half = (1.0 + 5f32.sqrt()) / 4.0;
        for c in 0..CELLS {
            for n in 0..12 {
                let o = adjacency[c * 12 + n] as usize;
                let dot: f32 = (0..4).map(|i| roots[4 * c + i] * roots[4 * o + i]).sum();
                assert!((dot - phi_half).abs() < 1e-5, "{c} -> {o}: {dot}");
            }
        }
        Ok(())
    }

    /// The interlock for this read's own sources: K1 carries no exponential,
    /// logarithm, normalised exponential or squashing function, and nothing here invokes a
    /// runtime CUDA compiler (owner, 13 October 2026). Tokens are assembled so
    /// this test does not match its own text.
    #[test]
    fn kernel_and_module_carry_no_prohibited_operation_or_runtime_compiler() {
        let kernel = include_str!("starling_native.cu").to_lowercase();
        for token in [
            ["ex", "p("].concat(),
            ["ex", "pf"].concat(),
            ["__ex", "p"].concat(),
            ["lo", "g("].concat(),
            ["lo", "gf"].concat(),
            ["soft", "max"].concat(),
            ["sig", "moid"].concat(),
            ["ta", "nh"].concat(),
            ["soft", "plus"].concat(),
            ["er", "f("].concat(),
            ["po", "w("].concat(),
        ] {
            assert!(
                !kernel.contains(&token),
                "starling_native.cu contains {token}"
            );
        }
        let module = include_str!("starling_native.rs").to_lowercase();
        for token in [
            ["nv", "rtc"].concat(),
            ["compile", "_ptx"].concat(),
            ["soft", "max("].concat(),
        ] {
            assert!(
                !module.contains(&token),
                "starling_native.rs contains {token}"
            );
        }
    }

    #[test]
    fn cpu_entry_is_refused_before_any_model_arithmetic() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let spec = NativeStarling::parse("starling_native:4:3:1", 0, Credit::FIXTURE)?;
        let address = Tensor::zeros((2, 3, 4), DType::F32, &device)?;
        let reset = Tensor::zeros((1, 3), DType::U32, &device)?;
        let error = select(&spec, 2, &address, &address, &reset)
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(error.contains("GPU-only"), "{error}");
        Ok(())
    }
}

/// Finite CUDA controls of the card's section 4 (A1): fixed expected values,
/// no CPU model reference, no optimizer step, no corpus.
#[cfg(all(test, feature = "cuda"))]
mod cuda_tests {
    use super::*;
    use candle_core::{Device, Var};

    fn cuda() -> Result<Device> {
        Ok(Device::new_cuda(0)?)
    }

    fn root(roots: &[f32], cell: usize) -> [f32; 4] {
        [
            roots[4 * cell],
            roots[4 * cell + 1],
            roots[4 * cell + 2],
            roots[4 * cell + 3],
        ]
    }

    fn rows_of(selection: &Tensor) -> Result<Vec<Vec<Vec<u32>>>> {
        Ok(selection.to_vec3::<u32>()?)
    }

    /// Ring eviction at depth M, ties to the later position, the 12-neighbour
    /// union (a neighbour key ranks after the home keys at cos 36 degrees),
    /// a far key excluded, and the home/key cell columns.
    #[test]
    fn k1_ring_ties_neighbours_and_far_cells() -> Result<()> {
        let device = cuda()?;
        let (roots, adjacency) = geometry()?;
        let c = 7usize;
        let n = adjacency[c * 12] as usize;
        let home = root(&roots, c);
        let near = root(&roots, n);
        let far = home.map(|v| -v); // the antipode: not c and not adjacent
        let keys = [home, home, home, near, far];
        let query: Vec<f32> = (0..5).flat_map(|_| home).collect();
        let key: Vec<f32> = keys.iter().flatten().copied().collect();
        let spec = NativeStarling::parse("starling_native:2:2:1", 0, Credit::FIXTURE)?;
        let selection = select(
            &spec,
            1,
            &Tensor::from_vec(query, (1, 5, 4), &device)?,
            &Tensor::from_vec(key, (1, 5, 4), &device)?,
            &Tensor::zeros((1, 5), DType::U32, &device)?,
        )?;
        let got = rows_of(&selection)?;
        let a = ABSENT;
        let (c, n) = (c as u32, n as u32);
        let far_cell = got[0][4][4];
        assert_ne!(far_cell, c);
        assert!(!adjacency[7 * 12..7 * 12 + 12].contains(&far_cell));
        assert_eq!(got[0][0], vec![0, a, a, c, c]);
        assert_eq!(got[0][1], vec![1, 0, a, c, c]); // tie: later first
        assert_eq!(got[0][2], vec![2, 1, a, c, c]); // 0 evicted at depth 2
        assert_eq!(got[0][3], vec![2, 1, 3, c, n]); // neighbour third
        assert_eq!(got[0][4], vec![2, 1, 3, c, far_cell]); // far key unseen
        Ok(())
    }

    /// A BOS clears every ring before its own key enters: no source crosses a
    /// document start.
    #[test]
    fn k1_document_reset_isolates_documents() -> Result<()> {
        let device = cuda()?;
        let (roots, _) = geometry()?;
        let home = root(&roots, 3);
        let stream: Vec<f32> = (0..4).flat_map(|_| home).collect();
        let spec = NativeStarling::parse("starling_native:4:3:0", 0, Credit::FIXTURE)?;
        let reset = Tensor::from_vec(vec![1u32, 0, 1, 0], (1, 4), &device)?;
        let selection = select(
            &spec,
            1,
            &Tensor::from_vec(stream.clone(), (1, 4, 4), &device)?,
            &Tensor::from_vec(stream, (1, 4, 4), &device)?,
            &reset,
        )?;
        let got = rows_of(&selection)?;
        let a = ABSENT;
        assert_eq!(&got[0][1][..3], &[1, 0, a]);
        assert_eq!(&got[0][2][..3], &[2, a, a]); // the reset at t = 2
        assert_eq!(&got[0][3][..3], &[3, 2, a]);
        Ok(())
    }

    /// Rows are independent: each (batch, head) row of a joint call equals the
    /// same row selected alone, and the reset of a batch item reaches all its
    /// heads and no other item.
    #[test]
    fn k1_batch_and_head_isolation() -> Result<()> {
        let device = cuda()?;
        let (roots, _) = geometry()?;
        let (batch, heads, time) = (2usize, 2usize, 6usize);
        let mut query = Vec::new();
        let mut key = Vec::new();
        for row in 0..batch * heads {
            for t in 0..time {
                query.extend(root(&roots, (row * 11 + t * 5) % CELLS));
                key.extend(root(&roots, (row * 7 + t * 3) % CELLS));
            }
        }
        let reset = vec![1u32, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0];
        let spec = NativeStarling::parse("starling_native:3:3:1", 0, Credit::FIXTURE)?;
        let joint = rows_of(&select(
            &spec,
            heads,
            &Tensor::from_vec(query.clone(), (batch * heads, time, 4), &device)?,
            &Tensor::from_vec(key.clone(), (batch * heads, time, 4), &device)?,
            &Tensor::from_vec(reset.clone(), (batch, time), &device)?,
        )?)?;
        for row in 0..batch * heads {
            let b = row / heads;
            let span = row * time * 4..(row + 1) * time * 4;
            let alone = rows_of(&select(
                &spec,
                1,
                &Tensor::from_vec(query[span.clone()].to_vec(), (1, time, 4), &device)?,
                &Tensor::from_vec(key[span].to_vec(), (1, time, 4), &device)?,
                &Tensor::from_vec(reset[b * time..(b + 1) * time].to_vec(), (1, time), &device)?,
            )?)?;
            assert_eq!(joint[row], alone[0], "row {row}");
        }
        Ok(())
    }

    #[test]
    fn k1_refuses_non_finite_addresses() -> Result<()> {
        let device = cuda()?;
        let spec = NativeStarling::parse("starling_native:2:2:0", 0, Credit::FIXTURE)?;
        let mut values = vec![0.5f32; 8];
        values[5] = f32::NAN;
        let bad = Tensor::from_vec(values, (1, 2, 4), &device)?;
        let good = Tensor::from_vec(vec![0.5f32; 8], (1, 2, 4), &device)?;
        let reset = Tensor::zeros((1, 2), DType::U32, &device)?;
        assert!(select(&spec, 1, &good, &bad, &reset).is_err());
        assert!(select(&spec, 1, &bad, &good, &reset).is_err());
        Ok(())
    }

    fn state(
        device: &Device,
        values: Vec<f32>,
        position: Vec<f32>,
        off_home: Vec<f32>,
        sink: Option<CreditSink>,
    ) -> Result<CreditState> {
        let s = values.len();
        Ok(CreditState {
            credit: Credit::FIXTURE,
            values: Tensor::from_vec(values, (1, 1, s, 1), device)?,
            live: Tensor::ones((1, 1, s), DType::F32, device)?,
            position: Tensor::from_vec(position, (1, 1, s), device)?,
            off_home: Tensor::from_vec(off_home, (1, 1, s), device)?,
            sink,
        })
    }

    fn close(got: &[f32], want: &[f32]) {
        assert_eq!(got.len(), want.len());
        for (g, w) in got.iter().zip(want) {
            assert!((g - w).abs() <= 1e-6, "{got:?} != {want:?}");
        }
    }

    /// K3a, as corrected by the FIT review (6148141029): delta = 0.125 and
    /// v_C = -0.625, so u_C - u_B = delta exactly and (C, B) is not a pair;
    /// (C, A) has gap 0.425. Five pairs, loss 1.12, the chain rule into q
    /// (-0.06), k and the null score (+0.6), and the read's gradient passed
    /// through bit-exactly.
    #[test]
    fn k3a_active_hinge_known_answers() -> Result<()> {
        let device = cuda()?;
        let sink: CreditSink = Arc::new(Mutex::new(Vec::new()));
        // v_A, v_B, v_C = -0.2, -0.5, -0.625 with g = 1: u = 0.2, 0.5, 0.625.
        let state = state(
            &device,
            vec![-0.2, -0.5, -0.625],
            vec![0.0, 1.0, 2.0],
            vec![0.0; 3],
            Some(Arc::clone(&sink)),
        )?;
        let q = Var::from_vec(vec![1.0f32], (1, 1, 1), &device)?;
        let k = Var::from_vec(vec![0.9f32, 0.5, 0.1], (1, 1, 3), &device)?;
        let b = Var::from_vec(vec![0.3f32], (1, 1, 1), &device)?;
        let read = Var::from_vec(vec![0.25f32], (1, 1, 1), &device)?;
        let scores = Tensor::cat(
            &[&k.as_tensor().broadcast_mul(q.as_tensor())?, b.as_tensor()],
            2,
        )?;
        let dots = Tensor::zeros((1, 1, 4), DType::F32, &device)?;
        let out = read
            .as_tensor()
            .apply_op3(&scores.contiguous()?, &dots, CreditCapture(state))?;
        assert_eq!(out.flatten_all()?.to_vec1::<f32>()?, vec![0.25]); // identity
        let grads = out.sum_all()?.backward()?; // g = 1
        let get = |v: &Var| -> Result<Vec<f32>> {
            Ok(grads
                .get(v.as_tensor())
                .ok_or_else(|| invalid("missing credit gradient"))?
                .flatten_all()?
                .to_vec1::<f32>()?)
        };
        close(&get(&k)?, &[0.2, -0.4, -0.4]);
        close(&get(&b)?, &[0.6]);
        close(&get(&q)?, &[-0.06]);
        close(&get(&read)?, &[1.0]); // passed through unchanged
        let reports = sink.lock().map_err(|_| invalid("sink"))?;
        close(&[reports[0].rank_hinge.to_scalar::<f32>()?], &[1.12]);
        close(&[reports[0].active_pairs.to_scalar::<f32>()?], &[5.0]);
        close(&[reports[0].address_margin.to_scalar::<f32>()?], &[0.0]);
        Ok(())
    }

    /// The capture returns the read's incoming gradient bit-exactly, for an
    /// arbitrary g: credit can add nothing to the value or rank-table
    /// gradients that flow through the read.
    #[test]
    fn credit_capture_passes_the_read_gradient_bit_exactly() -> Result<()> {
        let device = cuda()?;
        let state = state(
            &device,
            vec![-0.3, 0.7, -1.1],
            vec![0.0, 1.0, 2.0],
            vec![0.0; 3],
            None,
        )?;
        let read = Var::from_vec(vec![0.0f32], (1, 1, 1), &device)?;
        let scores = Tensor::from_vec(vec![0.2f32, -0.4, 0.9, 0.1], (1, 1, 4), &device)?;
        let dots = Tensor::zeros((1, 1, 4), DType::F32, &device)?;
        let out = read
            .as_tensor()
            .apply_op3(&scores, &dots, CreditCapture(state))?;
        let weight = Tensor::from_vec(vec![0.123_456_79f32], (1, 1, 1), &device)?;
        let grads = out.mul(&weight)?.sum_all()?.backward()?;
        let got = grads
            .get(read.as_tensor())
            .ok_or_else(|| invalid("missing read gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(got[0].to_bits(), 0.123_456_79f32.to_bits());
        Ok(())
    }

    /// K3b (corrected by the FIT review): both real utilities 0, equal to the
    /// null's: no real/real or real/null pair, exactly zero loss and credit.
    #[test]
    fn k3b_tie_and_empty_pair_set_give_zero() -> Result<()> {
        let device = cuda()?;
        let state = state(&device, vec![0.0, 0.0], vec![0.0, 1.0], vec![0.0; 2], None)?;
        let scores = Tensor::from_vec(vec![0.4f32, 0.2, 0.3], (1, 1, 3), &device)?;
        let dots = Tensor::zeros((1, 1, 3), DType::F32, &device)?;
        let g = Tensor::ones((1, 1, 1), DType::F32, &device)?;
        let (sg, dg) = credit_backward(&state, &scores, &dots, &g)?;
        assert_eq!(sg.flatten_all()?.to_vec1::<f32>()?, vec![0.0; 3]);
        assert_eq!(dg.flatten_all()?.to_vec1::<f32>()?, vec![0.0; 3]);
        Ok(())
    }

    /// K3c: the helpful best candidate off the home vertex. With a = rho_home
    /// and the candidate in an adjacent cell, the margin is
    /// 0.1 + (1 - phi/2) = 0.2909830, and d/da = -(rho_adj - rho_home).
    #[test]
    fn k3c_address_credit_off_home() -> Result<()> {
        let device = cuda()?;
        let (roots, adjacency) = geometry()?;
        let home = root(&roots, 0);
        let adjacent = root(&roots, adjacency[0] as usize);
        let sink: CreditSink = Arc::new(Mutex::new(Vec::new()));
        let state = state(
            &device,
            vec![-0.5],
            vec![0.0],
            vec![1.0],
            Some(Arc::clone(&sink)),
        )?;
        let a = Var::from_vec(home.to_vec(), (1, 1, 1, 4), &device)?;
        let rho = Tensor::from_vec(
            adjacent
                .iter()
                .chain(home.iter())
                .copied()
                .collect::<Vec<_>>(),
            (1, 1, 2, 4),
            &device,
        )?;
        let dots = rho.broadcast_mul(a.as_tensor())?.sum(3)?; // [1, 1, 2]
        let scores = Tensor::from_vec(vec![0.4f32, 0.3], (1, 1, 2), &device)?;
        let read = Var::from_vec(vec![0.0f32], (1, 1, 1), &device)?;
        let out = read
            .as_tensor()
            .apply_op3(&scores, &dots.contiguous()?, CreditCapture(state))?;
        let grads = out.sum_all()?.backward()?;
        let da = grads
            .get(a.as_tensor())
            .ok_or_else(|| invalid("missing address credit"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let want: Vec<f32> = (0..4).map(|i| -(adjacent[i] - home[i])).collect();
        close(&da, &want);
        let reports = sink.lock().map_err(|_| invalid("sink"))?;
        let phi_half = (1.0 + 5f32.sqrt()) / 4.0;
        close(
            &[reports[0].address_margin.to_scalar::<f32>()?],
            &[0.1 + (1.0 - phi_half)],
        );
        Ok(())
    }

    /// K3c composed (binding correction): from a raw, non-unit address
    /// a = 2 rho_home through [`unit_address`], the gradient is
    /// (I - a^ a^T)(rho_home - rho_adj) / |a| = ((phi/2) rho_home - rho_adj) / 2.
    #[test]
    fn k3c_composed_normalisation_gradient() -> Result<()> {
        let device = cuda()?;
        let (roots, adjacency) = geometry()?;
        let home = root(&roots, 0);
        let adjacent = root(&roots, adjacency[0] as usize);
        let state = state(&device, vec![-0.5], vec![0.0], vec![1.0], None)?;
        let raw = Var::from_vec(
            home.iter().map(|v| 2.0 * v).collect::<Vec<_>>(),
            (1, 1, 1, 4),
            &device,
        )?;
        let unit = unit_address(raw.as_tensor())?;
        let rho = Tensor::from_vec(
            adjacent
                .iter()
                .chain(home.iter())
                .copied()
                .collect::<Vec<_>>(),
            (1, 1, 2, 4),
            &device,
        )?;
        let dots = rho.broadcast_mul(&unit)?.sum(3)?;
        let scores = Tensor::from_vec(vec![0.4f32, 0.3], (1, 1, 2), &device)?;
        let read = Var::from_vec(vec![0.0f32], (1, 1, 1), &device)?;
        let out = read
            .as_tensor()
            .apply_op3(&scores, &dots.contiguous()?, CreditCapture(state))?;
        let grads = out.sum_all()?.backward()?;
        let got = grads
            .get(raw.as_tensor())
            .ok_or_else(|| invalid("missing raw address credit"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let phi_half = (1.0 + 5f32.sqrt()) / 4.0;
        let want: Vec<f32> = (0..4)
            .map(|i| (phi_half * home[i] - adjacent[i]) / 2.0)
            .collect();
        close(&got, &want);
        Ok(())
    }

    /// K3d: a best candidate in the home cell, or no helpful candidate, gives
    /// no address credit.
    #[test]
    fn k3d_home_or_unhelpful_gives_no_address_credit() -> Result<()> {
        let device = cuda()?;
        let dots = Tensor::from_vec(vec![0.2f32, 1.0], (1, 1, 2), &device)?;
        let scores = Tensor::from_vec(vec![0.4f32, 0.3], (1, 1, 2), &device)?;
        let g = Tensor::ones((1, 1, 1), DType::F32, &device)?;
        for (value, off_home) in [(-0.5f32, 0.0f32), (0.5, 1.0)] {
            let state = state(&device, vec![value], vec![0.0], vec![off_home], None)?;
            let (_, dg) = credit_backward(&state, &scores, &dots, &g)?;
            assert_eq!(dg.flatten_all()?.to_vec1::<f32>()?, vec![0.0, 0.0]);
        }
        Ok(())
    }
}
