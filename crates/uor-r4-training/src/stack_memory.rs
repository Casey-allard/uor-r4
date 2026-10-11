//! Sparse product-key memory for the geometric stack (*Literature*: Lample et
//! al. 2019, "Large Memory Layers with Product Keys", arXiv 1907.05242).
//!
//! A memory layer replaces a layer's MLP. Per head, the query splits into two
//! halves; each half scores its own set of `sub_keys` sub-keys, the top `top_k`
//! of each side form `top_k^2` candidate slots scored by the sum of their two
//! halves, and the best `top_k` slots are read. Their softmax weights mix rows of
//! one value table of `sub_keys^2` slots shared by all heads, and the heads'
//! reads are summed. Each token reads `heads * top_k` value rows of a table
//! that can be much larger than the model's dense layers: sparse parameter
//! access (D5's end state), with the sub-key scores as the index.
//!
//! The sub-key score is the Dot product or the Lorentz score `-beta
//! d(q, k)`, the hyperboloid distance of the lifted points `(sqrt(1 + |x|^2),
//! x)` with a learned scale per head. The selection is discrete: its gradient
//! is taken with the selected slots held fixed, as in the original, and ties
//! break toward the lower index so the backward pass reselects exactly.
//!
//! Sub-keys are learned, or fixed to a geometric codebook ([`Codebook`]): the
//! 120 unit icosians (the vertices of the 600-cell, H4) for quaternion halves,
//! or the 240 roots of E8 for 8-dimensional halves. A fixed codebook addresses
//! the memory by fixed geometric codes; only the query map and the values learn.

use candle_core::{CpuStorage, CustomOp3, Layout, Shape, Tensor};
#[cfg(feature = "cuda")]
use candle_core::CudaStorage;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{invalid, Result};

/// Lower bound on the Lorentz excess `z - 1`; below it the distance is clamped
/// and carries no gradient (the same floor as the stack's reads).
const MIN_EXCESS: f64 = 1e-7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScore {
    Dot,
    Lorentz,
}

/// A fixed set of unit sub-keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Codebook {
    /// The 120 unit icosians, the vertices of the 600-cell (H4), in 4 dimensions.
    H4,
    /// The 240 roots of E8, scaled to unit norm, in 8 dimensions.
    E8,
}

impl Codebook {
    pub fn dim(self) -> usize {
        match self {
            Self::H4 => 4,
            Self::E8 => 8,
        }
    }

    pub fn size(self) -> usize {
        match self {
            Self::H4 => 120,
            Self::E8 => 240,
        }
    }

    /// The codebook's unit vectors, in a fixed order.
    pub fn vectors(self) -> Vec<Vec<f64>> {
        let mut out = Vec::with_capacity(self.size());
        match self {
            Self::H4 => {
                let phi = (1.0 + 5f64.sqrt()) / 2.0;
                for axis in 0..4 {
                    for sign in [1.0, -1.0] {
                        let mut v = vec![0.0; 4];
                        v[axis] = sign;
                        out.push(v);
                    }
                }
                for signs in 0..16u32 {
                    out.push(
                        (0..4)
                            .map(|i| if signs >> i & 1 == 1 { -0.5 } else { 0.5 })
                            .collect(),
                    );
                }
                // Even permutations of (phi, 1, 1/phi, 0) / 2 with every sign.
                let base = [phi / 2.0, 0.5, 0.5 / phi, 0.0];
                for permutation in even_permutations() {
                    for signs in 0..8u32 {
                        let mut v = vec![0.0; 4];
                        for (source, &target) in permutation.iter().enumerate() {
                            let sign = if source < 3 && signs >> source & 1 == 1 {
                                -1.0
                            } else {
                                1.0
                            };
                            v[target] = sign * base[source];
                        }
                        out.push(v);
                    }
                }
            }
            Self::E8 => {
                let scale = 1.0 / 2f64.sqrt();
                for i in 0..8 {
                    for j in i + 1..8 {
                        for (a, b) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                            let mut v = vec![0.0; 8];
                            v[i] = a * scale;
                            v[j] = b * scale;
                            out.push(v);
                        }
                    }
                }
                for signs in 0..256u32 {
                    if signs.count_ones() % 2 == 0 {
                        out.push(
                            (0..8)
                                .map(|i| if signs >> i & 1 == 1 { -0.5 } else { 0.5 } * scale)
                                .collect(),
                        );
                    }
                }
            }
        }
        out
    }
}

/// The twelve even permutations of four positions: `p[source] = target`.
fn even_permutations() -> Vec<[usize; 4]> {
    let mut out = Vec::with_capacity(12);
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

/// Where the memory layers are and how large they are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Layers whose MLP is replaced by a memory.
    pub layers: Vec<usize>,
    /// Sub-keys per half; a memory has `sub_keys^2` slots.
    pub sub_keys: usize,
    /// Slots read per head.
    pub top_k: usize,
    pub heads: usize,
    /// Query width per head; each half has `key_dim / 2` coordinates.
    pub key_dim: usize,
    pub score: MemoryScore,
    /// Fixed sub-keys; absent means learned sub-keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codebook: Option<Codebook>,
}

impl MemoryConfig {
    pub fn validate(&self, layers: usize) -> Result<()> {
        if self.layers.is_empty()
            || self.layers.iter().any(|&l| l >= layers)
            || (1..self.layers.len()).any(|i| self.layers[..i].contains(&self.layers[i]))
            || self.sub_keys < 2
            || self.top_k == 0
            || self.top_k > self.sub_keys
            || self.heads == 0
            || self.key_dim < 2
            || !self.key_dim.is_multiple_of(2)
            || self.sub_keys > 1 << 12
        {
            return Err(invalid(
                "memory needs distinct layers of the stack, 2 <= top_k <= sub_keys <= 4096 and an even key width",
            ));
        }
        if let Some(codebook) = self.codebook {
            if self.key_dim / 2 != codebook.dim()
                || self.sub_keys != codebook.size()
                || self.score != MemoryScore::Dot
            {
                return Err(invalid(format!(
                    "a {codebook:?} codebook needs key_dim {}, sub_keys {} and the Dot score",
                    2 * codebook.dim(),
                    codebook.size()
                )));
            }
        }
        Ok(())
    }

    pub fn slots(&self) -> usize {
        self.sub_keys * self.sub_keys
    }

    /// Parameters of one memory layer that a token does not read: the value
    /// rows outside its `heads * top_k` selections.
    pub fn idle_parameters(&self, width: usize) -> usize {
        self.slots().saturating_sub(self.heads * self.top_k) * width
    }
}

/// The per-head scale `beta` of the Lorentz score travels after the sub-keys.
pub fn keys_aux_len(config: &MemoryConfig) -> usize {
    config.heads * 2 * config.sub_keys * (config.key_dim / 2)
        + if config.score == MemoryScore::Lorentz {
            config.heads
        } else {
            0
        }
}

/// Memory output [rows, width] for queries [rows, heads * key_dim], the
/// sub-keys (and Lorentz scales) packed as [`keys_aux_len`] values, and the
/// value table [sub_keys^2, width].
pub fn product_key_memory(
    query: &Tensor,
    keys_aux: &Tensor,
    values: &Tensor,
    config: &MemoryConfig,
) -> Result<Tensor> {
    let (rows, query_width) = query.dims2()?;
    let (slots, width) = values.dims2()?;
    if query_width != config.heads * config.key_dim
        || slots != config.slots()
        || keys_aux.elem_count() != keys_aux_len(config)
    {
        return Err(invalid(
            "product-key memory inputs do not match its configuration",
        ));
    }
    Ok(query.contiguous()?.apply_op3(
        &keys_aux.contiguous()?,
        &values.contiguous()?,
        ProductKeyMemory {
            rows,
            heads: config.heads,
            half: config.key_dim / 2,
            sub_keys: config.sub_keys,
            top_k: config.top_k,
            width,
            score: config.score,
        },
    )?)
}

#[derive(Clone, Copy, Debug)]
struct ProductKeyMemory {
    rows: usize,
    heads: usize,
    half: usize,
    sub_keys: usize,
    top_k: usize,
    width: usize,
    score: MemoryScore,
}

/// One head's read at one row.
struct Selection {
    /// Selected slots, their two sub-key indices and softmax weights.
    slots: Vec<usize>,
    first: Vec<usize>,
    second: Vec<usize>,
    weights: Vec<f32>,
}

fn contiguous<'a>(storage: &'a CpuStorage, layout: &Layout) -> candle_core::Result<&'a [f32]> {
    let values = storage.as_slice::<f32>()?;
    match layout.contiguous_offsets() {
        Some((start, end)) => Ok(&values[start..end]),
        None => candle_core::bail!("product-key memory needs contiguous inputs"),
    }
}

#[inline]
fn dot(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum()
}

#[inline]
fn lift(x: &[f32]) -> f64 {
    (1.0 + dot(x, x)).sqrt()
}

/// `arcosh(1 + e)` with the excess clamped at [`MIN_EXCESS`].
#[inline]
fn lorentz_distance(excess: f64) -> f64 {
    let e = excess.max(MIN_EXCESS);
    (e + (e * (e + 2.0)).sqrt()).ln_1p()
}

/// Indices of the `k` largest scores, best first; ties go to the lower index.
fn top(scores: &[f64], k: usize) -> Vec<usize> {
    let better = |a: &usize, b: &usize| scores[*b].total_cmp(&scores[*a]).then(a.cmp(b));
    let mut order: Vec<usize> = (0..scores.len()).collect();
    if k > 0 && k < order.len() {
        order.select_nth_unstable_by(k - 1, better);
        order.truncate(k);
    }
    order.sort_by(better);
    order
}

impl ProductKeyMemory {
    fn key_block(&self) -> usize {
        self.sub_keys * self.half
    }

    /// Sub-key `index` of side `side` (0 or 1) of head `head`.
    fn key<'a>(&self, keys: &'a [f32], head: usize, side: usize, index: usize) -> &'a [f32] {
        let start = ((head * 2 + side) * self.sub_keys + index) * self.half;
        &keys[start..start + self.half]
    }

    fn beta(&self, keys_aux: &[f32], head: usize) -> f64 {
        f64::from(keys_aux[self.heads * 2 * self.key_block() + head])
    }

    /// Every sub-key's score for one half-query.
    fn side_scores(&self, keys_aux: &[f32], head: usize, side: usize, query: &[f32]) -> Vec<f64> {
        let query_lift = lift(query);
        (0..self.sub_keys)
            .map(|index| {
                let key = self.key(keys_aux, head, side, index);
                match self.score {
                    MemoryScore::Dot => dot(query, key),
                    MemoryScore::Lorentz => {
                        let excess = query_lift * lift(key) - dot(query, key) - 1.0;
                        -self.beta(keys_aux, head) * lorentz_distance(excess)
                    }
                }
            })
            .collect()
    }

    fn select(&self, query: &[f32], keys_aux: &[f32], head: usize) -> Selection {
        let q = &query[head * 2 * self.half..(head + 1) * 2 * self.half];
        let first_scores = self.side_scores(keys_aux, head, 0, &q[..self.half]);
        let second_scores = self.side_scores(keys_aux, head, 1, &q[self.half..]);
        let (firsts, seconds) = (
            top(&first_scores, self.top_k),
            top(&second_scores, self.top_k),
        );
        let mut candidates = Vec::with_capacity(self.top_k * self.top_k);
        for &a in &firsts {
            for &b in &seconds {
                candidates.push((first_scores[a] + second_scores[b], a, b));
            }
        }
        let better = |x: &(f64, usize, usize), y: &(f64, usize, usize)| {
            y.0.total_cmp(&x.0)
                .then((x.1 * self.sub_keys + x.2).cmp(&(y.1 * self.sub_keys + y.2)))
        };
        if self.top_k < candidates.len() {
            candidates.select_nth_unstable_by(self.top_k - 1, better);
            candidates.truncate(self.top_k);
        }
        candidates.sort_by(better);
        let maximum = candidates[0].0;
        let exps: Vec<f64> = candidates.iter().map(|c| (c.0 - maximum).exp()).collect();
        let total: f64 = exps.iter().sum();
        Selection {
            slots: candidates
                .iter()
                .map(|c| c.1 * self.sub_keys + c.2)
                .collect(),
            first: candidates.iter().map(|c| c.1).collect(),
            second: candidates.iter().map(|c| c.2).collect(),
            weights: exps.iter().map(|e| (e / total) as f32).collect(),
        }
    }

    /// Gradient of one side's selected sub-key scores `(index, d_score)` into
    /// the half-query, its sub-keys and the head's `beta`.
    #[allow(clippy::too_many_arguments)]
    fn side_backward(
        &self,
        keys_aux: &[f32],
        head: usize,
        side: usize,
        query: &[f32],
        d_scores: &[(usize, f64)],
        d_query: &mut [f32],
        d_keys: &mut [f64],
        d_beta: &mut f64,
    ) {
        let query_lift = lift(query);
        for &(index, d_score) in d_scores {
            let key = self.key(keys_aux, head, side, index);
            let key_start = ((head * 2 + side) * self.sub_keys + index) * self.half;
            match self.score {
                MemoryScore::Dot => {
                    for i in 0..self.half {
                        d_query[i] += (d_score * f64::from(key[i])) as f32;
                        d_keys[key_start + i] += d_score * f64::from(query[i]);
                    }
                }
                MemoryScore::Lorentz => {
                    let beta = self.beta(keys_aux, head);
                    let key_lift = lift(key);
                    let excess = query_lift * key_lift - dot(query, key) - 1.0;
                    *d_beta -= d_score * lorentz_distance(excess);
                    if excess > MIN_EXCESS {
                        // score = -beta arcosh(1 + e), e = lift_q lift_k - <q, k> - 1.
                        let d_excess = -beta * d_score / (excess * (excess + 2.0)).sqrt();
                        for i in 0..self.half {
                            let (q, k) = (f64::from(query[i]), f64::from(key[i]));
                            d_query[i] += (d_excess * (key_lift / query_lift * q - k)) as f32;
                            d_keys[key_start + i] += d_excess * (query_lift / key_lift * k - q);
                        }
                    }
                }
            }
        }
    }
}

impl CustomOp3 for ProductKeyMemory {
    fn name(&self) -> &'static str {
        "geometric-stack-product-key-memory"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
        s3: &CpuStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (query, keys_aux, values) = (
            contiguous(s1, l1)?,
            contiguous(s2, l2)?,
            contiguous(s3, l3)?,
        );
        let (width, query_width) = (self.width, self.heads * 2 * self.half);
        let mut out = vec![0f32; self.rows * width];
        out.par_chunks_mut(width)
            .enumerate()
            .for_each(|(row, out)| {
                let q = &query[row * query_width..(row + 1) * query_width];
                for head in 0..self.heads {
                    let selection = self.select(q, keys_aux, head);
                    for (&slot, &w) in selection.slots.iter().zip(&selection.weights) {
                        for (o, v) in out
                            .iter_mut()
                            .zip(&values[slot * width..(slot + 1) * width])
                        {
                            *o += w * v;
                        }
                    }
                }
            });
        Ok((CpuStorage::F32(out), Shape::from((self.rows, width))))
    }

    /// The memory has no CUDA kernel yet, so its CUDA forward is the exact CPU
    /// forward on host copies of the three inputs, uploaded back to the device
    /// ([`crate::geometric_stack::cuda_ops::via_host3`]) — the same rule the
    /// stack's other kernel-less configurations follow, so CUDA training
    /// computes what the CPU computes. The backward is already device-generic
    /// (`bwd` reads host copies and rebuilds its gradients on the input's
    /// device). Costs the three transfers per call; measured in the record
    /// that enabled it.
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CudaStorage, Shape)> {
        crate::geometric_stack::cuda_ops::via_host3(self, [(s1, l1), (s2, l2), (s3, l3)])
    }

    fn bwd(
        &self,
        query: &Tensor,
        keys_aux: &Tensor,
        values: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        let q_values = query.flatten_all()?.to_vec1::<f32>()?;
        let aux = keys_aux.flatten_all()?.to_vec1::<f32>()?;
        let table = values.flatten_all()?.to_vec1::<f32>()?;
        let d_out = grad.flatten_all()?.to_vec1::<f32>()?;
        let (width, query_width) = (self.width, self.heads * 2 * self.half);
        let key_len = self.heads * 2 * self.key_block();
        let mut d_query = vec![0f32; q_values.len()];
        // Per row: the query gradient, and each head's selection for the value
        // scatter; sub-key and beta gradients reduce over rows.
        let (partial_keys, partial_beta, selections) = d_query
            .par_chunks_mut(query_width)
            .enumerate()
            .map(|(row, d_query)| {
                let q = &q_values[row * query_width..(row + 1) * query_width];
                let d_row = &d_out[row * width..(row + 1) * width];
                let mut d_keys = vec![0f64; key_len];
                let mut d_beta = vec![0f64; self.heads];
                let mut picked = Vec::with_capacity(self.heads);
                for head in 0..self.heads {
                    let selection = self.select(q, &aux, head);
                    let reads: Vec<f64> = selection
                        .slots
                        .iter()
                        .map(|&slot| dot(&table[slot * width..(slot + 1) * width], d_row))
                        .collect();
                    let mean: f64 = selection
                        .weights
                        .iter()
                        .zip(&reads)
                        .map(|(&w, &r)| f64::from(w) * r)
                        .sum();
                    let mut d_first: Vec<(usize, f64)> = Vec::new();
                    let mut d_second: Vec<(usize, f64)> = Vec::new();
                    for (m, (&w, &read)) in selection.weights.iter().zip(&reads).enumerate() {
                        let d_score = f64::from(w) * (read - mean);
                        for (list, index) in [
                            (&mut d_first, selection.first[m]),
                            (&mut d_second, selection.second[m]),
                        ] {
                            match list.iter_mut().find(|(i, _)| *i == index) {
                                Some(entry) => entry.1 += d_score,
                                None => list.push((index, d_score)),
                            }
                        }
                    }
                    let q_head = &q[head * 2 * self.half..(head + 1) * 2 * self.half];
                    let (d_first_half, d_second_half) = d_query
                        [head * 2 * self.half..(head + 1) * 2 * self.half]
                        .split_at_mut(self.half);
                    self.side_backward(
                        &aux,
                        head,
                        0,
                        &q_head[..self.half],
                        &d_first,
                        d_first_half,
                        &mut d_keys,
                        &mut d_beta[head],
                    );
                    self.side_backward(
                        &aux,
                        head,
                        1,
                        &q_head[self.half..],
                        &d_second,
                        d_second_half,
                        &mut d_keys,
                        &mut d_beta[head],
                    );
                    picked.push((selection.slots, selection.weights));
                }
                (d_keys, d_beta, vec![(row, picked)])
            })
            .reduce(
                || (vec![0f64; key_len], vec![0f64; self.heads], Vec::new()),
                |mut a, b| {
                    for (x, y) in a.0.iter_mut().zip(&b.0) {
                        *x += y;
                    }
                    for (x, y) in a.1.iter_mut().zip(&b.1) {
                        *x += y;
                    }
                    a.2.extend(b.2);
                    a
                },
            );
        // Value rows: each read adds its weight times the row's output gradient.
        let mut d_values = vec![0f32; table.len()];
        for (row, picked) in &selections {
            let d_row = &d_out[row * width..(row + 1) * width];
            for (slots, weights) in picked {
                for (&slot, &w) in slots.iter().zip(weights) {
                    for (d, g) in d_values[slot * width..(slot + 1) * width]
                        .iter_mut()
                        .zip(d_row)
                    {
                        *d += w * g;
                    }
                }
            }
        }
        let mut d_aux: Vec<f32> = partial_keys.iter().map(|&v| v as f32).collect();
        if self.score == MemoryScore::Lorentz {
            d_aux.extend(partial_beta.iter().map(|&v| v as f32));
        }
        let device = query.device();
        Ok((
            Some(Tensor::from_vec(d_query, query.shape(), device)?),
            Some(Tensor::from_vec(d_aux, keys_aux.shape(), device)?),
            Some(Tensor::from_vec(d_values, values.shape(), device)?),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Var};

    fn config(score: MemoryScore) -> MemoryConfig {
        MemoryConfig {
            layers: vec![0],
            sub_keys: 6,
            top_k: 3,
            heads: 2,
            key_dim: 6,
            score,
            codebook: None,
        }
    }

    #[test]
    fn codebooks_are_the_600_cell_and_the_e8_roots() {
        let phi = (1.0 + 5f64.sqrt()) / 2.0;
        // Inner products of distinct unit icosians, and of distinct unit E8 roots.
        let h4 = [
            -1.0,
            -phi / 2.0,
            -0.5,
            -0.5 / phi,
            0.0,
            0.5 / phi,
            0.5,
            phi / 2.0,
        ];
        let e8 = [-0.5, 0.0, 0.5, -1.0];
        for (codebook, allowed) in [(Codebook::H4, &h4[..]), (Codebook::E8, &e8[..])] {
            let vectors = codebook.vectors();
            assert_eq!(vectors.len(), codebook.size());
            for (i, a) in vectors.iter().enumerate() {
                assert_eq!(a.len(), codebook.dim());
                let norm: f64 = a.iter().map(|x| x * x).sum();
                assert!(
                    (norm - 1.0).abs() < 1e-12,
                    "{codebook:?} vector {i} has norm {norm}"
                );
                for b in &vectors[i + 1..] {
                    let inner: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
                    assert!(
                        allowed.iter().any(|v| (inner - v).abs() < 1e-12),
                        "{codebook:?}: inner product {inner}"
                    );
                }
            }
        }
    }

    fn uniform(seed: &mut u64) -> f32 {
        *seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (((*seed >> 11) as f64) / ((1u64 << 53) as f64) - 0.5) as f32
    }

    fn inputs(
        config: &MemoryConfig,
        rows: usize,
        width: usize,
        seed: u64,
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let mut s = seed;
        let query: Vec<f32> = (0..rows * config.heads * config.key_dim)
            .map(|_| 2.0 * uniform(&mut s))
            .collect();
        let mut keys: Vec<f32> = (0..keys_aux_len(config))
            .map(|_| 2.0 * uniform(&mut s))
            .collect();
        if config.score == MemoryScore::Lorentz {
            let n = keys.len();
            for (h, beta) in keys[n - config.heads..].iter_mut().enumerate() {
                *beta = 0.7 + 0.3 * h as f32;
            }
        }
        let values: Vec<f32> = (0..config.slots() * width)
            .map(|_| uniform(&mut s))
            .collect();
        (query, keys, values)
    }

    /// A direct reference: every slot's score by brute force, the top `top_k`
    /// slots, softmax and the weighted sum.
    fn brute_force(
        config: &MemoryConfig,
        query: &[f32],
        keys: &[f32],
        values: &[f32],
        width: usize,
    ) -> Vec<f64> {
        let half = config.key_dim / 2;
        let op = ProductKeyMemory {
            rows: 1,
            heads: config.heads,
            half,
            sub_keys: config.sub_keys,
            top_k: config.top_k,
            width,
            score: config.score,
        };
        let mut out = vec![0f64; width];
        for head in 0..config.heads {
            let q = &query[head * 2 * half..(head + 1) * 2 * half];
            let first = op.side_scores(keys, head, 0, &q[..half]);
            let second = op.side_scores(keys, head, 1, &q[half..]);
            let all: Vec<f64> = (0..config.slots())
                .map(|slot| first[slot / config.sub_keys] + second[slot % config.sub_keys])
                .collect();
            let chosen = top(&all, config.top_k);
            let maximum = all[chosen[0]];
            let total: f64 = chosen.iter().map(|&s| (all[s] - maximum).exp()).sum();
            for &slot in &chosen {
                let w = (all[slot] - maximum).exp() / total;
                for c in 0..width {
                    out[c] += w * f64::from(values[slot * width + c]);
                }
            }
        }
        out
    }

    #[test]
    fn product_keys_find_the_brute_force_top_slots() -> Result<()> {
        for score in [MemoryScore::Dot, MemoryScore::Lorentz] {
            let config = config(score);
            let (rows, width) = (5, 4);
            let (query, keys, values) = inputs(&config, rows, width, 17);
            let device = Device::Cpu;
            let out = product_key_memory(
                &Tensor::from_vec(
                    query.clone(),
                    (rows, config.heads * config.key_dim),
                    &device,
                )?,
                &Tensor::from_vec(keys.clone(), keys.len(), &device)?,
                &Tensor::from_vec(values.clone(), (config.slots(), width), &device)?,
                &config,
            )?
            .to_vec2::<f32>()?;
            let qw = config.heads * config.key_dim;
            for row in 0..rows {
                let want = brute_force(
                    &config,
                    &query[row * qw..(row + 1) * qw],
                    &keys,
                    &values,
                    width,
                );
                for c in 0..width {
                    assert!(
                        (f64::from(out[row][c]) - want[c]).abs() < 1e-5,
                        "{score:?} row {row}"
                    );
                }
            }
        }
        Ok(())
    }

    /// Weighted sum of the outputs, so every output coordinate is exercised.
    fn loss(
        config: &MemoryConfig,
        query: &[f32],
        keys: &[f32],
        values: &[f32],
        rows: usize,
        width: usize,
    ) -> f64 {
        let qw = config.heads * config.key_dim;
        (0..rows)
            .map(|row| {
                brute_force(
                    config,
                    &query[row * qw..(row + 1) * qw],
                    keys,
                    values,
                    width,
                )
                .iter()
                .enumerate()
                .map(|(c, v)| v * (1.0 + 0.37 * (c + row) as f64).sin())
                .sum::<f64>()
            })
            .sum()
    }

    #[test]
    fn product_key_memory_gradients_match_finite_differences() -> Result<()> {
        for score in [MemoryScore::Dot, MemoryScore::Lorentz] {
            let config = config(score);
            let (rows, width) = (4, 3);
            let (query, keys, values) = inputs(&config, rows, width, 29);
            let device = Device::Cpu;
            let q_var = Var::from_vec(
                query.clone(),
                (rows, config.heads * config.key_dim),
                &device,
            )?;
            let k_var = Var::from_vec(keys.clone(), keys.len(), &device)?;
            let v_var = Var::from_vec(values.clone(), (config.slots(), width), &device)?;
            let out = product_key_memory(
                q_var.as_tensor(),
                k_var.as_tensor(),
                v_var.as_tensor(),
                &config,
            )?;
            let weights: Vec<f32> = (0..rows * width)
                .map(|i| (1.0 + 0.37 * ((i % width) + i / width) as f64).sin() as f32)
                .collect();
            let objective = out
                .mul(&Tensor::from_vec(weights, (rows, width), &device)?)?
                .sum_all()?;
            let grads = objective.backward()?;
            let analytic = |var: &Var| -> Vec<f32> {
                grads
                    .get(var.as_tensor())
                    .unwrap()
                    .flatten_all()
                    .unwrap()
                    .to_vec1::<f32>()
                    .unwrap()
            };
            let (g_query, g_keys, g_values) =
                (analytic(&q_var), analytic(&k_var), analytic(&v_var));
            let eps = 1e-3f32;
            let check = |which: usize, index: usize, analytic: f32| {
                let mut plus = (query.clone(), keys.clone(), values.clone());
                let mut minus = (query.clone(), keys.clone(), values.clone());
                let (p, m) = match which {
                    0 => (&mut plus.0, &mut minus.0),
                    1 => (&mut plus.1, &mut minus.1),
                    _ => (&mut plus.2, &mut minus.2),
                };
                p[index] += eps;
                m[index] -= eps;
                let numeric = (loss(&config, &plus.0, &plus.1, &plus.2, rows, width)
                    - loss(&config, &minus.0, &minus.1, &minus.2, rows, width))
                    / (2.0 * f64::from(eps));
                (numeric - f64::from(analytic)).abs() <= 2e-3 * (1.0 + numeric.abs())
            };
            let mut failures = Vec::new();
            for (which, grads) in [(0usize, &g_query), (1, &g_keys), (2, &g_values)] {
                for (index, &g) in grads.iter().enumerate() {
                    if !check(which, index, g) {
                        failures.push((which, index, g));
                    }
                }
            }
            // These inputs have no near-tied selection, so every coordinate agrees.
            assert!(
                failures.is_empty(),
                "{score:?}: {} coordinates disagree: {:?}",
                failures.len(),
                &failures[..failures.len().min(8)]
            );
        }
        Ok(())
    }
}
