//! A learned continuous cache over a frozen language model (lab roadmap M4a).
//!
//! Grave, Joulin and Usunier (2016, arXiv 1612.04426) mix a model's next-token
//! distribution with a cache that points at the tokens which followed similar
//! earlier states. Here the cache's query and key maps are learned, and the
//! score is one of three geometries with the same parameters, so that the
//! geometry is the only difference between arms:
//!
//! - `Dot`: `beta q.k`;
//! - `Euclid`: `-beta |q - k|^2`;
//! - `Lorentz`: `-beta d(q, k)^2`, with `q` and `k` lifted to the hyperboloid
//!   `x0 = sqrt(1 + |u|^2)` of curvature -1. The maps' scale decides how
//!   hyperbolic the embedding is: at small `|u|` the score tends to `Euclid`.
//!
//! Cache entries are the backbone's states at earlier positions, each pointing
//! at the token that followed it. A query at position `t` reads only entries at
//! positions `<= t - gap`, so with `gap` equal to the backbone's context the
//! cache holds only what the backbone cannot see. The mixture weight is a
//! learned gate of the query state:
//! `p(y_t) = (1 - g_t) p_model(y_t) + g_t sum_{i <= t - gap, y_i = y_t} a_{t,i}`.

use std::collections::BTreeMap;

use candle_core::{DType, Device, Tensor, Var};

use crate::kappa_llama::{arcosh1p_squared, KappaLlama, Site};
use crate::{invalid, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheGeometry {
    Dot,
    Euclid,
    Lorentz,
}

impl CacheGeometry {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "dot" => Ok(Self::Dot),
            "euclid" => Ok(Self::Euclid),
            "lorentz" => Ok(Self::Lorentz),
            other => Err(invalid(format!(
                "geometry must be dot, euclid or lorentz, not {other}"
            ))),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Dot => "dot",
            Self::Euclid => "euclid",
            Self::Lorentz => "lorentz",
        }
    }
}

/// Segments of backbone output: `states` `(batch, len, width)`, `logp`
/// `(batch, len)` the backbone's log-probability of each position's next token,
/// and `next[b][t]` that token.
pub struct CacheBatch<'a> {
    pub states: &'a Tensor,
    pub logp: &'a Tensor,
    pub next: &'a [Vec<u32>],
}

/// Per query position (`t >= gap`), shaped `(batch, len - gap)`, except
/// `attention` `(batch, len - gap, len - gap)` over entries `i = 0..len - gap`.
pub struct CacheOutput {
    /// `-ln p(y_t)` of the mixture.
    pub nll: Tensor,
    /// `-ln p_model(y_t)` of the backbone alone.
    pub backbone_nll: Tensor,
    /// Cache probability of the true next token.
    pub cache_probability: Tensor,
    /// Mixture weight of the cache.
    pub gate: Tensor,
    /// Read distribution over the readable entries (zero elsewhere).
    pub attention: Tensor,
    /// `|k|^2` of every entry's key.
    pub key_norm_sq: Tensor,
}

pub struct CacheMemory {
    pub geometry: CacheGeometry,
    pub gap: usize,
    variables: BTreeMap<String, Var>,
}

/// Deterministic standard normal samples (xorshift and Box-Muller).
fn normal(count: usize, seed: u64) -> Vec<f32> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut uniform = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        ((state >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    };
    (0..count)
        .map(|_| {
            let (a, b) = (uniform(), uniform());
            ((-2.0 * a.ln()).sqrt() * (2.0 * std::f64::consts::PI * b).cos()) as f32
        })
        .collect()
}

impl CacheMemory {
    /// Query and key maps `width -> dim` initialized with standard deviation
    /// `1 / sqrt(width dim)`, so that RMS-normalized states (`|h|^2 = width`)
    /// map to `|u|^2` near 1; `beta = 1`; gate bias -2 (weight 0.12).
    pub fn new(
        geometry: CacheGeometry,
        width: usize,
        dim: usize,
        gap: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        if width == 0 || dim == 0 || gap == 0 {
            return Err(invalid("width, dim and gap must be positive"));
        }
        let scale = 1.0 / ((width * dim) as f32).sqrt();
        let map = |salt: u64| -> Result<Var> {
            let values: Vec<f32> = normal(width * dim, seed.wrapping_add(salt))
                .into_iter()
                .map(|v| v * scale)
                .collect();
            Ok(Var::from_tensor(&Tensor::from_vec(
                values,
                (width, dim),
                device,
            )?)?)
        };
        let mut variables = BTreeMap::new();
        variables.insert("query".to_owned(), map(1)?);
        variables.insert("key".to_owned(), map(2)?);
        variables.insert(
            "log_beta".to_owned(),
            Var::from_tensor(&Tensor::zeros(1, DType::F32, device)?)?,
        );
        variables.insert(
            "gate_weight".to_owned(),
            Var::from_tensor(&Tensor::zeros((width, 1), DType::F32, device)?)?,
        );
        variables.insert(
            "gate_bias".to_owned(),
            Var::from_tensor(&Tensor::full(-2f32, 1, device)?)?,
        );
        Ok(Self {
            geometry,
            gap,
            variables,
        })
    }

    pub fn variables(&self) -> &BTreeMap<String, Var> {
        &self.variables
    }

    /// A trained cache saved by the `cache-memory` example (`PATH.safetensors`
    /// with its `PATH.json`).
    pub fn load(path: &std::path::Path, device: &Device) -> Result<Self> {
        let meta: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path.with_extension("json"))?)?;
        let number = |key: &str| -> Result<usize> {
            meta[key]
                .as_u64()
                .map(|v| v as usize)
                .ok_or_else(|| invalid(format!("cache {key} missing")))
        };
        let geometry = CacheGeometry::parse(meta["geometry"].as_str().unwrap_or(""))?;
        let memory = Self::new(
            geometry,
            number("width")?,
            number("dim")?,
            number("gap")?,
            0,
            device,
        )?;
        let saved = candle_core::safetensors::load(path, device)?;
        for (name, var) in memory.variables() {
            let tensor = saved
                .get(name)
                .ok_or_else(|| invalid(format!("saved cache lacks {name}")))?;
            if tensor.dims() != var.dims() {
                return Err(invalid(format!("saved cache {name} has the wrong shape")));
            }
            var.set(tensor)?;
        }
        Ok(memory)
    }

    fn var(&self, name: &str) -> Result<&Tensor> {
        self.variables
            .get(name)
            .map(Var::as_tensor)
            .ok_or_else(|| invalid(format!("missing cache variable {name}")))
    }

    pub fn beta(&self) -> Result<f32> {
        Ok(self.var("log_beta")?.exp()?.to_vec1::<f32>()?[0])
    }

    /// Scores `(batch, n, n)` between queries `q` and keys `k` `(batch, n, dim)`.
    fn scores(&self, q: &Tensor, k: &Tensor) -> Result<Tensor> {
        let beta = self.var("log_beta")?.exp()?;
        let dot = q.matmul(&k.transpose(1, 2)?.contiguous()?)?;
        let raw = match self.geometry {
            CacheGeometry::Dot => dot,
            CacheGeometry::Euclid | CacheGeometry::Lorentz => {
                let qq = q.sqr()?.sum_keepdim(2)?;
                let kk = k.sqr()?.sum_keepdim(2)?.transpose(1, 2)?.contiguous()?;
                let euclid = qq
                    .broadcast_add(&kk)?
                    .sub(&dot.affine(2.0, 0.0)?)?
                    .clamp(0f32, f32::MAX)?;
                if self.geometry == CacheGeometry::Euclid {
                    euclid.neg()?
                } else {
                    // z - 1 = (|q - k|^2 - (q0 - k0)^2) / 2 with
                    // q0 - k0 = (|q|^2 - |k|^2) / (q0 + k0): no cancellation near 0.
                    let q0 = qq.affine(1.0, 1.0)?.sqrt()?;
                    let k0 = kk.affine(1.0, 1.0)?.sqrt()?;
                    let lift = qq.broadcast_sub(&kk)?.div(&q0.broadcast_add(&k0)?)?.sqr()?;
                    let z_minus_one = euclid.sub(&lift)?.affine(0.5, 0.0)?.clamp(0f32, f32::MAX)?;
                    arcosh1p_squared(&z_minus_one)?.neg()?
                }
            }
        };
        Ok(raw.broadcast_mul(&beta)?)
    }

    /// Mixture of the backbone and the cache for every position `t >= gap`.
    pub fn forward(&self, batch: &CacheBatch<'_>) -> Result<CacheOutput> {
        let (b, len, _) = batch.states.dims3()?;
        if len <= self.gap
            || batch.logp.dims() != [b, len]
            || batch.next.len() != b
            || batch.next.iter().any(|row| row.len() != len)
        {
            return Err(invalid(
                "cache batch shapes disagree or are shorter than the gap",
            ));
        }
        let n = len - self.gap;
        let device = batch.states.device();
        let query_states = batch.states.narrow(1, self.gap, n)?;
        let key_states = batch.states.narrow(1, 0, n)?;
        let q = query_states.broadcast_matmul(self.var("query")?)?;
        let k = key_states.broadcast_matmul(self.var("key")?)?;
        let key_norm_sq = k.sqr()?.sum(2)?;
        let scores = self.scores(&q, &k)?;
        // Query t' = t - gap reads entries i <= t'.
        let hidden: Vec<u8> = (0..n)
            .flat_map(|row| (0..n).map(move |column| u8::from(column > row)))
            .collect();
        let mask = Tensor::from_vec(hidden, (1, n, n), device)?.broadcast_as((b, n, n))?;
        let excluded = Tensor::full(f32::NEG_INFINITY, (b, n, n), device)?;
        let attention = candle_nn::ops::softmax(&mask.where_cond(&excluded, &scores)?, 2)?;
        let mut matches = vec![0f32; b * n * n];
        for (row_index, next) in batch.next.iter().enumerate() {
            for t in 0..n {
                let target = next[t + self.gap];
                let row = &mut matches[(row_index * n + t) * n..(row_index * n + t + 1) * n];
                for (i, slot) in row.iter_mut().enumerate().take(t + 1) {
                    if next[i] == target {
                        *slot = 1.0;
                    }
                }
            }
        }
        let matches = Tensor::from_vec(matches, (b, n, n), device)?;
        let cache_probability = attention.mul(&matches)?.sum(2)?;
        let gate = candle_nn::ops::sigmoid(
            &query_states
                .broadcast_matmul(self.var("gate_weight")?)?
                .squeeze(2)?
                .broadcast_add(self.var("gate_bias")?)?,
        )?;
        let logp = batch.logp.narrow(1, self.gap, n)?;
        let model = logp.exp()?;
        let mixture = model
            .mul(&gate.affine(-1.0, 1.0)?)?
            .add(&cache_probability.mul(&gate)?)?;
        Ok(CacheOutput {
            nll: mixture.log()?.neg()?,
            backbone_nll: logp.neg()?,
            cache_probability,
            gate,
            attention,
            key_norm_sq,
        })
    }
}

/// Backbone output over one segment of `len + 1` tokens: the states at `site`
/// `(len, width)`, the log-probability of every next token `(len)` and the
/// next tokens. Windows of `window` tokens run with stride `window / 2`
/// (`len` a multiple of it); the first window is kept whole and every later
/// one contributes its second half, so each position after the first window
/// is read with at least `window / 2` tokens of context.
pub fn segment_features(
    model: &KappaLlama,
    segment: &[u32],
    window: usize,
    site: Site,
) -> Result<(Tensor, Tensor, Vec<u32>)> {
    let len = segment
        .len()
        .checked_sub(1)
        .ok_or_else(|| invalid("empty segment"))?;
    let half = window / 2;
    if window < 2 || !window.is_multiple_of(2) || len < window || !len.is_multiple_of(half) {
        return Err(invalid(
            "segment length must be a multiple of window / 2 and at least window",
        ));
    }
    let starts: Vec<usize> = (0..=(len - window) / half).map(|w| w * half).collect();
    let ids: Vec<u32> = starts
        .iter()
        .flat_map(|&start| segment[start..start + window].iter().copied())
        .collect();
    let targets: Vec<u32> = starts
        .iter()
        .flat_map(|&start| segment[start + 1..start + window + 1].iter().copied())
        .collect();
    let mut captured = None;
    let logits = model.forward_with_capture(&ids, starts.len(), window, &mut |at, x| {
        if at == site {
            captured = Some(x.clone());
        }
        Ok(())
    })?;
    let captured = captured.ok_or_else(|| invalid("the chosen site was never captured"))?;
    let width = captured.dim(1)?;
    let captured = captured.reshape((starts.len(), window, width))?;
    let target = Tensor::from_vec(targets, (starts.len(), window, 1), logits.device())?;
    let logp = candle_nn::ops::log_softmax(&logits, candle_core::D::Minus1)?
        .gather(&target, 2)?
        .squeeze(2)?;
    let mut state_parts = Vec::with_capacity(starts.len());
    let mut logp_parts = Vec::with_capacity(starts.len());
    for w in 0..starts.len() {
        let (from, count) = if w == 0 { (0, window) } else { (half, half) };
        state_parts.push(captured.get(w)?.narrow(0, from, count)?);
        logp_parts.push(logp.get(w)?.narrow(0, from, count)?);
    }
    Ok((
        Tensor::cat(&state_parts, 0)?,
        Tensor::cat(&logp_parts, 0)?,
        segment[1..].to_vec(),
    ))
}

/// Read concentration of one attention row over its `valid` readable entries:
/// the fraction of entries needed to hold `mass` of the read, and the mass in
/// the top `k` entries for each `k` in `tops`.
pub fn concentration(row: &[f32], valid: usize, mass: f64, tops: &[usize]) -> (f64, Vec<f64>) {
    let mut values: Vec<f64> = row[..valid].iter().map(|v| f64::from(*v)).collect();
    values.sort_by(|a, b| b.total_cmp(a));
    let total: f64 = values.iter().sum();
    let mut running = 0.0;
    let mut needed = valid;
    for (i, v) in values.iter().enumerate() {
        running += v;
        if running >= mass * total {
            needed = i + 1;
            break;
        }
    }
    let top_mass = tops
        .iter()
        .map(|&k| values.iter().take(k).sum::<f64>() / total.max(f64::MIN_POSITIVE))
        .collect();
    (needed as f64 / valid.max(1) as f64, top_mass)
}

/// Mean over the last dimension, as `f64`, of a `(batch, n)` tensor.
pub fn mean(tensor: &Tensor) -> Result<f64> {
    Ok(f64::from(tensor.mean_all()?.to_scalar::<f32>()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn batch_parts(
        b: usize,
        len: usize,
        width: usize,
        seed: u64,
    ) -> (Tensor, Tensor, Vec<Vec<u32>>) {
        let device = Device::Cpu;
        let states = Tensor::from_vec(normal(b * len * width, seed), (b, len, width), &device)
            .expect("states");
        let logp = Tensor::full(-2f32, (b, len), &device).expect("logp");
        let next = (0..b)
            .map(|row| (0..len).map(|t| ((t * 7 + row) % 5) as u32).collect())
            .collect();
        (states, logp, next)
    }

    #[test]
    fn a_closed_gate_leaves_the_backbone_unchanged() {
        let (states, logp, next) = batch_parts(2, 12, 8, 1);
        let memory =
            CacheMemory::new(CacheGeometry::Lorentz, 8, 4, 4, 3, &Device::Cpu).expect("new");
        memory.variables()["gate_bias"]
            .set(&Tensor::full(-60f32, 1, &Device::Cpu).expect("bias"))
            .expect("set");
        let out = memory
            .forward(&CacheBatch {
                states: &states,
                logp: &logp,
                next: &next,
            })
            .expect("forward");
        let diff = out
            .nll
            .sub(&out.backbone_nll)
            .expect("sub")
            .abs()
            .expect("abs");
        assert!(mean(&diff).expect("mean") < 1e-6);
    }

    #[test]
    fn the_cache_reads_only_entries_outside_the_gap() {
        let (states, logp, _) = batch_parts(1, 10, 8, 2);
        let gap = 4;
        // Only position 8 is followed by token 9; the query at t = 8 needs i <= 4.
        let mut next = vec![1u32; 10];
        next[8] = 9;
        next[3] = 9;
        let memory = CacheMemory::new(CacheGeometry::Dot, 8, 4, gap, 5, &Device::Cpu).expect("new");
        let out = memory
            .forward(&CacheBatch {
                states: &states,
                logp: &logp,
                next: std::slice::from_ref(&next),
            })
            .expect("forward");
        let attention = out.attention.to_vec3::<f32>().expect("attention");
        for (row, values) in attention[0].iter().enumerate() {
            let total: f32 = values.iter().sum();
            assert!((total - 1.0).abs() < 1e-5, "row {row} sums to {total}");
            for (column, v) in values.iter().enumerate() {
                if column > row {
                    assert_eq!(*v, 0.0, "row {row} reads entry {column} inside the gap");
                }
            }
        }
        let cache = out.cache_probability.to_vec2::<f32>().expect("cache");
        // Query t = 8 (row 4): entry i = 3 (followed by 9) is readable.
        assert!(cache[0][4] > 0.0);
        // Query t = 7 (row 3): target 1; entries 0..=3 except 3 are followed by 1.
        let row3: f32 = attention[0][3][..3].iter().sum();
        assert!((cache[0][3] - row3).abs() < 1e-6);
    }

    #[test]
    fn lorentz_scores_are_hyperbolic_distances_and_flat_at_small_radius() {
        let device = Device::Cpu;
        let memory = CacheMemory::new(CacheGeometry::Lorentz, 4, 3, 1, 1, &device).expect("new");
        let q = Tensor::new(&[[[0.3f32, -0.2, 0.5]]], &device).expect("q");
        let k = Tensor::new(&[[[-0.4f32, 0.1, 0.2]]], &device).expect("k");
        let got = memory
            .scores(&q, &k)
            .expect("scores")
            .to_vec3::<f32>()
            .expect("v")[0][0][0];
        let lift = |u: [f64; 3]| {
            let n: f64 = u.iter().map(|v| v * v).sum();
            ((1.0 + n).sqrt(), u)
        };
        let (q0, qu) = lift([0.3, -0.2, 0.5]);
        let (k0, ku) = lift([-0.4, 0.1, 0.2]);
        let z = q0 * k0 - qu.iter().zip(&ku).map(|(a, b)| a * b).sum::<f64>();
        let want = -(z.acosh().powi(2));
        assert!((f64::from(got) - want).abs() < 1e-5, "{got} vs {want}");
        let tiny = |v: f32| Tensor::new(&[[[v, 0.0, 0.0]]], &device).expect("t");
        let flat = memory
            .scores(&tiny(1e-3), &tiny(-1e-3))
            .expect("s")
            .to_vec3::<f32>()
            .expect("v")[0][0][0];
        assert!((flat + 4e-6).abs() < 1e-9, "{flat}");
    }

    #[test]
    fn gradients_reach_every_cache_variable() {
        let (states, logp, next) = batch_parts(2, 12, 8, 4);
        for geometry in [
            CacheGeometry::Dot,
            CacheGeometry::Euclid,
            CacheGeometry::Lorentz,
        ] {
            let memory = CacheMemory::new(geometry, 8, 4, 4, 9, &Device::Cpu).expect("new");
            let out = memory
                .forward(&CacheBatch {
                    states: &states,
                    logp: &logp,
                    next: &next,
                })
                .expect("forward");
            let grads = out
                .nll
                .mean_all()
                .expect("mean")
                .backward()
                .expect("backward");
            for (name, var) in memory.variables() {
                assert!(
                    grads.get(var.as_tensor()).is_some(),
                    "{} has no gradient for {name}",
                    geometry.name()
                );
            }
        }
    }

    #[test]
    fn concentration_counts_the_entries_holding_the_mass() {
        let row = [0.5f32, 0.3, 0.15, 0.05, 0.0];
        let (fraction, tops) = concentration(&row, 4, 0.9, &[1, 2]);
        assert!((fraction - 0.75).abs() < 1e-12);
        assert!((tops[0] - 0.5).abs() < 1e-6 && (tops[1] - 0.8).abs() < 1e-6);
    }
}
