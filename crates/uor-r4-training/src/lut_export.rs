//! Offline export of a Llama checkpoint to the integer serving artifact of
//! `uor-r4-lut` (owner decision D10).
//!
//! Floating point is used here only, once, offline: RMSNorm gains are folded
//! into the following weight maps, every weight matrix is quantized to 4 bits
//! in groups of `GROUP` with the scale of each group chosen from the grid
//! `(16 + m) 2^(e - 4)` by least squares, and the exp, SiLU and RoPE tables are
//! sealed into the artifact. The artifact is then served with integers only.
//!
//! Quantization is round-to-nearest, or, given a [`Calibration`] of input
//! moments, GPTQ (Frantar et al. 2022, arXiv 2210.17323): the columns of each
//! matrix are rounded in order and every rounding error is spread over the
//! columns not yet rounded, in proportion to the inverse input moments, so the
//! quantized map reproduces the float map on the calibration inputs rather
//! than the float weights. The embedding (a lookup) stays round-to-nearest.

use std::collections::BTreeMap;

use candle_core::DType;
use rayon::prelude::*;
use serde_json::{json, Value};
use uor_r4_lut::format::{ArtifactBuilder, CacheSpec, Fixed, Numerics, Shape, TableValues};
use uor_r4_lut::GROUP;

use crate::kappa_llama::{Checkpoint, KappaLlama, Site};
use crate::{invalid, Result};

/// Exponent step of the exp table (`2^-8` nats).
pub const EXP_STEP_LOG2: i32 = -8;
/// The exp table covers `d` in `[0, EXP_RANGE]` nats.
pub const EXP_RANGE: usize = 32;
/// SiLU table step (`2^-8`) and half range (`2^4 = 16`).
pub const SILU_STEP_LOG2: i32 = -8;
pub const SILU_RANGE_LOG2: i32 = 4;
/// RoPE tables are stored in Q14.
pub const ROPE_Q: u32 = 14;

/// A 4-bit matrix in the artifact layout, with its relative RMS error.
pub struct Packed {
    pub nibbles: Vec<u8>,
    pub scales: Vec<u8>,
    pub exp_base: i32,
    pub relative_rms_error: f64,
}

fn grid_scale(m: u8, e: i32) -> f64 {
    (16.0 + f64::from(m)) * 2f64.powi(e - 4)
}

fn group_error(w: &[f64], s: f64) -> f64 {
    w.iter()
        .map(|v| {
            let q = (v / s).round().clamp(-8.0, 7.0);
            (v - q * s).powi(2)
        })
        .sum()
}

/// The grid scale `(m, e)` with the least squared rounding error for one group
/// (searched over `[a / 8.5, a / 5.5]`, `a` the largest magnitude), or `None`
/// for an all-zero group.
fn best_scale(w: &[f64]) -> Option<(u8, i32)> {
    let a = w.iter().map(|v| v.abs()).fold(0.0, f64::max);
    if a == 0.0 {
        return None;
    }
    let (lo, hi) = (a / 8.5, a / 5.5);
    let mut best: Option<(f64, u8, i32)> = None;
    let e_lo = (lo / 32.0).log2().floor() as i32 + 4;
    let e_hi = (hi / 16.0).log2().ceil() as i32 + 4;
    for e in e_lo..=e_hi {
        for m in 0..16u8 {
            let s = grid_scale(m, e);
            if s < lo || s > hi {
                continue;
            }
            let err = group_error(w, s);
            if best.is_none_or(|b| err < b.0) {
                best = Some((err, m, e));
            }
        }
    }
    Some(match best {
        Some((_, m, e)) => (m, e),
        // Smallest grid scale >= a / 7.
        None => (0, (a / 7.0 / 16.0).log2().ceil() as i32 + 4),
    })
}

/// Pack nibble `q + 8` of column `c` into a row's nibble bytes.
fn put_nibble(row: &mut [u8], c: usize, q: f64) {
    let nibble = (q as i32 + 8) as u8;
    if c.is_multiple_of(2) {
        row[c / 2] |= nibble;
    } else {
        row[c / 2] |= nibble << 4;
    }
}

/// Quantize a row-major `rows x cols` matrix (`cols` a multiple of `GROUP`).
pub fn quantize_matrix(values: &[f32], rows: usize, cols: usize) -> Result<Packed> {
    if values.len() != rows * cols
        || !cols.is_multiple_of(GROUP)
        || values.iter().any(|v| !v.is_finite())
    {
        return Err(invalid(
            "matrix to quantize has the wrong size or a nonfinite value",
        ));
    }
    let groups = cols / GROUP;
    // First pass: best (m, e) per group, None for an all-zero group.
    let mut chosen: Vec<Option<(u8, i32)>> = Vec::with_capacity(rows * groups);
    let mut group = [0f64; GROUP];
    for w in values.chunks_exact(GROUP) {
        for (g, v) in group.iter_mut().zip(w) {
            *g = f64::from(*v);
        }
        chosen.push(best_scale(&group));
    }
    let e_max = chosen.iter().flatten().map(|(_, e)| *e).max().unwrap_or(0);
    let exp_base = e_max - 15;
    let mut nibbles = vec![0u8; rows * cols / 2];
    let mut scales = vec![0u8; rows * groups];
    let (mut error, mut energy) = (0f64, 0f64);
    for r in 0..rows {
        for g in 0..groups {
            let (m, e) = match chosen[r * groups + g] {
                Some((m, e)) if e >= exp_base => (m, e),
                _ => (0, exp_base),
            };
            let s = grid_scale(m, e);
            scales[r * groups + g] = m | (((e - exp_base) as u8) << 4);
            for c in g * GROUP..(g + 1) * GROUP {
                let v = f64::from(values[r * cols + c]);
                let q = (v / s).round().clamp(-8.0, 7.0);
                error += (v - q * s).powi(2);
                energy += v * v;
                let nibble = (q as i32 + 8) as u8;
                let index = (r * cols + c) / 2;
                if c % 2 == 0 {
                    nibbles[index] |= nibble;
                } else {
                    nibbles[index] |= nibble << 4;
                }
            }
        }
    }
    Ok(Packed {
        nibbles,
        scales,
        exp_base,
        relative_rms_error: if energy > 0.0 {
            (error / energy).sqrt()
        } else {
            0.0
        },
    })
}

/// Second moments `sum_t x_t x_t^T` of every weight map's input over
/// calibration text, as seen by the folded export (see
/// [`KappaLlama::forward_with_capture`]).
pub struct Calibration {
    /// Per site: the input width and the row-major moment matrix.
    moments: BTreeMap<Site, (usize, Vec<f64>)>,
    /// Calibration positions accumulated.
    pub positions: usize,
}

impl Calibration {
    /// Run `model` (float) over `windows` windows of `time` tokens spread
    /// evenly across `tokens` and accumulate the input moments.
    pub fn collect(
        model: &KappaLlama,
        tokens: &[u32],
        windows: usize,
        time: usize,
    ) -> Result<Self> {
        if windows == 0 || time == 0 || tokens.len() < time {
            return Err(invalid("calibration needs windows, time and enough tokens"));
        }
        let span = tokens.len() - time;
        let mut moments: BTreeMap<Site, (usize, Vec<f64>)> = BTreeMap::new();
        for w in 0..windows {
            let start = if windows == 1 {
                0
            } else {
                w * span / (windows - 1)
            };
            let ids = &tokens[start..start + time];
            model.forward_with_capture(ids, 1, time, &mut |site, x| {
                let x = x.to_dtype(DType::F32)?.contiguous()?;
                let cols = x.dim(1)?;
                let gram = x
                    .t()?
                    .contiguous()?
                    .matmul(&x)?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let entry = moments
                    .entry(site)
                    .or_insert_with(|| (cols, vec![0.0; cols * cols]));
                if entry.0 != cols {
                    return Err(invalid(format!("site {site:?} changed width")));
                }
                for (sum, value) in entry.1.iter_mut().zip(&gram) {
                    *sum += f64::from(*value);
                }
                Ok(())
            })?;
        }
        Ok(Self {
            moments,
            positions: windows * time,
        })
    }

    /// Moments collected elsewhere: per site, the input width and the
    /// row-major moment.
    pub(crate) fn from_moments(
        moments: BTreeMap<Site, (usize, Vec<f64>)>,
        positions: usize,
    ) -> Self {
        Self { moments, positions }
    }

    pub(crate) fn moment(&self, site: Site, cols: usize) -> Result<&[f64]> {
        match self.moments.get(&site) {
            Some((width, moment)) if *width == cols => Ok(moment),
            _ => Err(invalid(format!(
                "no {cols}-wide calibration moment for {site:?}"
            ))),
        }
    }
}

/// Upper-triangular `U` with `H^-1 = U^T U`, for symmetric positive definite
/// `H` (`n x n`, row-major): the factor GPTQ spreads rounding errors with.
/// `H = R R^T` with `R` upper triangular (Cholesky with rows and columns in
/// reverse order), and `U = R^-1`.
fn inverse_cholesky_upper(h: &[f64], n: usize) -> Result<Vec<f64>> {
    let mut r = vec![0f64; n * n];
    for j in (0..n).rev() {
        let tail = |row: &[f64]| -> f64 {
            row[j + 1..n]
                .iter()
                .zip(&r[j * n + j + 1..(j + 1) * n])
                .map(|(a, b)| a * b)
                .sum()
        };
        let pivot = h[j * n + j] - tail(&r[j * n..(j + 1) * n]);
        if !pivot.is_finite() || pivot <= 0.0 {
            return Err(invalid("calibration moment is not positive definite"));
        }
        let rjj = pivot.sqrt();
        let column: Vec<f64> = (0..j)
            .into_par_iter()
            .map(|i| (h[i * n + j] - tail(&r[i * n..(i + 1) * n])) / rjj)
            .collect();
        r[j * n + j] = rjj;
        for (i, value) in column.into_iter().enumerate() {
            r[i * n + j] = value;
        }
    }
    // Row j of U^T (column j of U), from R U = I: for i = j down to 0,
    // U[i][j] = (delta_ij - sum_{k=i+1..=j} R[i][k] U[k][j]) / R[i][i].
    let transposed: Vec<Vec<f64>> = (0..n)
        .into_par_iter()
        .map(|j| {
            let mut column = vec![0f64; j + 1];
            for i in (0..=j).rev() {
                let sum: f64 = r[i * n + i + 1..i * n + j + 1]
                    .iter()
                    .zip(&column[i + 1..=j])
                    .map(|(a, b)| a * b)
                    .sum();
                let delta = if i == j { 1.0 } else { 0.0 };
                column[i] = (delta - sum) / r[i * n + i];
            }
            column
        })
        .collect();
    let mut u = vec![0f64; n * n];
    for (j, column) in transposed.iter().enumerate() {
        for (i, value) in column.iter().enumerate() {
            u[i * n + j] = *value;
        }
    }
    Ok(u)
}

/// `sqrt(sum_r d_r^T H d_r / sum_r w_r^T H w_r)` for `d = quantized - w`: the
/// relative error of the map's outputs on the calibration inputs.
fn relative_output_error(w: &[f32], quantized: &[f32], moment: &[f64], cols: usize) -> f64 {
    let (error, energy) = w
        .par_chunks(cols)
        .zip(quantized.par_chunks(cols))
        .map(|(row, qrow)| {
            let d: Vec<f64> = row
                .iter()
                .zip(qrow)
                .map(|(a, b)| f64::from(*b) - f64::from(*a))
                .collect();
            let x: Vec<f64> = row.iter().map(|a| f64::from(*a)).collect();
            let quad = |v: &[f64]| -> f64 {
                moment
                    .chunks_exact(cols)
                    .zip(v)
                    .map(|(hrow, vi)| vi * hrow.iter().zip(v).map(|(h, vj)| h * vj).sum::<f64>())
                    .sum()
            };
            (quad(&d), quad(&x))
        })
        .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
    if energy > 0.0 {
        (error / energy).sqrt()
    } else {
        0.0
    }
}

/// GPTQ quantization of a row-major `rows x cols` matrix given the moment
/// `H = sum x x^T` of its inputs; `damp` (relative to the mean diagonal) is
/// added to `H` and raised tenfold, twice, if `H` is not positive definite.
/// Exponents stay within the 16 steps above the round-to-nearest base, so the
/// artifact layout is unchanged. Returns the packing and the relative output
/// errors of GPTQ and of round-to-nearest.
pub fn quantize_matrix_gptq(
    values: &[f32],
    rows: usize,
    cols: usize,
    moment: &[f64],
    damp: f64,
) -> Result<(Packed, f64, f64)> {
    let nearest = quantize_matrix(values, rows, cols)?;
    if moment.len() != cols * cols
        || moment.iter().any(|v| !v.is_finite())
        || !damp.is_finite()
        || damp < 0.0
    {
        return Err(invalid(
            "GPTQ moment has the wrong size or a nonfinite value",
        ));
    }
    let exp_base = nearest.exp_base;
    let mut h = moment.to_vec();
    let dead: Vec<bool> = (0..cols).map(|j| h[j * cols + j] <= 0.0).collect();
    for (j, dead) in dead.iter().enumerate() {
        if *dead {
            h[j * cols + j] = 1.0;
        }
    }
    let mean_diag = (0..cols).map(|j| h[j * cols + j]).sum::<f64>() / cols as f64;
    let mut added = 0.0;
    let mut u = None;
    for factor in [1.0, 10.0, 100.0] {
        let target = damp * factor * mean_diag;
        for j in 0..cols {
            h[j * cols + j] += target - added;
        }
        added = target;
        if let Ok(found) = inverse_cholesky_upper(&h, cols) {
            u = Some(found);
            break;
        }
    }
    let u = u.ok_or_else(|| invalid("GPTQ: moment not positive definite even when damped"))?;
    let groups = cols / GROUP;
    let rows_out: Vec<(Vec<u8>, Vec<u8>)> = values
        .par_chunks(cols)
        .map(|row| {
            let mut w: Vec<f64> = row
                .iter()
                .zip(&dead)
                .map(|(v, dead)| if *dead { 0.0 } else { f64::from(*v) })
                .collect();
            let mut nibbles = vec![0u8; cols / 2];
            let mut scales = vec![0u8; groups];
            for g in 0..groups {
                let (m, e) = match best_scale(&w[g * GROUP..(g + 1) * GROUP]) {
                    Some((_, e)) if e < exp_base => (0, exp_base),
                    Some((_, e)) if e > exp_base + 15 => (15, exp_base + 15),
                    Some(chosen) => chosen,
                    None => (0, exp_base),
                };
                let s = grid_scale(m, e);
                scales[g] = m | (((e - exp_base) as u8) << 4);
                for c in g * GROUP..(g + 1) * GROUP {
                    let q = (w[c] / s).round().clamp(-8.0, 7.0);
                    put_nibble(&mut nibbles, c, q);
                    let err = (w[c] - q * s) / u[c * cols + c];
                    for (wk, uk) in w[c + 1..]
                        .iter_mut()
                        .zip(&u[c * cols + c + 1..(c + 1) * cols])
                    {
                        *wk -= err * uk;
                    }
                }
            }
            (nibbles, scales)
        })
        .collect();
    let mut nibbles = Vec::with_capacity(rows * cols / 2);
    let mut scales = Vec::with_capacity(rows * groups);
    for (n, sc) in rows_out {
        nibbles.extend(n);
        scales.extend(sc);
    }
    let dequantized = dequantize_matrix(rows, cols, exp_base, &nibbles, &scales)?;
    let nearest_values =
        dequantize_matrix(rows, cols, exp_base, &nearest.nibbles, &nearest.scales)?;
    let (mut error, mut energy) = (0f64, 0f64);
    for (v, q) in values.iter().zip(&dequantized) {
        error += (f64::from(*q) - f64::from(*v)).powi(2);
        energy += f64::from(*v).powi(2);
    }
    let gptq_output = relative_output_error(values, &dequantized, moment, cols);
    let nearest_output = relative_output_error(values, &nearest_values, moment, cols);
    Ok((
        Packed {
            nibbles,
            scales,
            exp_base,
            relative_rms_error: if energy > 0.0 {
                (error / energy).sqrt()
            } else {
                0.0
            },
        },
        gptq_output,
        nearest_output,
    ))
}

fn tensor(checkpoint: &Checkpoint, name: &str) -> Result<Vec<f32>> {
    Ok(checkpoint
        .tensors
        .get(name)
        .ok_or_else(|| invalid(format!("missing tensor {name}")))?
        .flatten_all()?
        .to_vec1::<f32>()?)
}

/// Multiply column `c` of a row-major matrix by `gain[c]` (RMSNorm folding).
fn fold_columns(values: &mut [f32], cols: usize, gain: &[f32]) {
    for row in values.chunks_exact_mut(cols) {
        for (v, g) in row.iter_mut().zip(gain) {
            *v *= g;
        }
    }
}

/// A trained cache memory (the `save=true` output of the `cache-memory`
/// example) to export beside the backbone (lab M4b).
pub struct CacheWeights {
    pub geometry: String,
    pub gap: usize,
    pub dim: usize,
    /// `width x dim` maps, row-major.
    pub query: Vec<f32>,
    pub key: Vec<f32>,
    pub log_beta: f32,
    pub gate_weight: Vec<f32>,
    pub gate_bias: f32,
}

impl CacheWeights {
    /// Read `PATH.safetensors` and the `PATH.json` written beside it.
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let meta: Value = serde_json::from_slice(&std::fs::read(path.with_extension("json"))?)?;
        if meta["schema"] != "uor-r4.cache-memory-model/1" {
            return Err(invalid("not a cache-memory model"));
        }
        let geometry = meta["geometry"]
            .as_str()
            .ok_or_else(|| invalid("cache geometry missing"))?
            .to_owned();
        let number = |key: &str| -> Result<usize> {
            meta[key]
                .as_u64()
                .map(|v| v as usize)
                .ok_or_else(|| invalid(format!("cache {key} missing")))
        };
        let (gap, dim, width) = (number("gap")?, number("dim")?, number("width")?);
        let tensors = candle_core::safetensors::load(path, &candle_core::Device::Cpu)?;
        let get = |name: &str, len: usize| -> Result<Vec<f32>> {
            let values = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("cache tensor {name} missing")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if values.len() != len || values.iter().any(|v| !v.is_finite()) {
                return Err(invalid(format!("cache tensor {name} has the wrong size")));
            }
            Ok(values)
        };
        Ok(Self {
            query: get("query", width * dim)?,
            key: get("key", width * dim)?,
            log_beta: get("log_beta", 1)?[0],
            gate_weight: get("gate_weight", width)?,
            gate_bias: get("gate_bias", 1)?[0],
            geometry,
            gap,
            dim,
        })
    }
}

/// `(m, e)` with `(16 + m) 2^(e - 4)` nearest to `value > 0`.
pub(crate) fn grid_nearest(value: f64) -> (u8, i32) {
    let e = value.log2().floor() as i32;
    let m = (value / 2f64.powi(e - 4)).round() as i32 - 16;
    if m >= 16 {
        (0, e + 1)
    } else {
        (m.clamp(0, 15) as u8, e)
    }
}

/// Export `checkpoint` for integer serving with sessions of up to
/// `max_positions` tokens, with GPTQ when `calibration` gives input moments
/// (and its damping), and a trained Lorentz cache when `cache` is given.
/// Returns the artifact bytes and a quantization report.
pub fn export_llama(
    checkpoint: &Checkpoint,
    max_positions: usize,
    source: Value,
    calibration: Option<(&Calibration, f64)>,
    cache: Option<&CacheWeights>,
) -> Result<(Vec<u8>, Value)> {
    let s = &checkpoint.shape;
    let shape = Shape {
        vocab: s.vocab,
        width: s.width,
        layers: s.layers,
        heads: s.heads,
        kv_heads: s.kv_heads,
        head_dim: s.head_dim,
        ffn: s.ffn,
        max_positions,
    };
    let numerics = Numerics {
        rms_eps: Fixed {
            mantissa: (s.rms_eps * 2f64.powi(48)).round() as i64,
            exp: -48,
        },
        score_scale_q30: (2f64.powi(30) / (s.head_dim as f64).sqrt()).round() as i64,
        exp_step_log2: EXP_STEP_LOG2,
        silu_step_log2: SILU_STEP_LOG2,
        silu_range_log2: SILU_RANGE_LOG2,
        rope_q: ROPE_Q,
    };
    let mut builder =
        ArtifactBuilder::new(shape, numerics, source).map_err(|e| invalid(e.to_string()))?;
    let mut errors = serde_json::Map::new();
    let mut output_errors = serde_json::Map::new();
    let mut add = |builder: &mut ArtifactBuilder,
                   name: &str,
                   values: &[f32],
                   rows: usize,
                   cols: usize,
                   site: Option<Site>|
     -> Result<()> {
        let packed = match (calibration, site) {
            (Some((calibration, damp)), Some(site)) => {
                let moment = calibration.moment(site, cols)?;
                let (packed, gptq, nearest) =
                    quantize_matrix_gptq(values, rows, cols, moment, damp)?;
                output_errors.insert(
                    name.to_owned(),
                    json!({"gptq": gptq, "round_to_nearest": nearest}),
                );
                packed
            }
            _ => quantize_matrix(values, rows, cols)?,
        };
        builder
            .add_matrix(
                name,
                rows,
                cols,
                packed.exp_base,
                &packed.nibbles,
                &packed.scales,
            )
            .map_err(|e| invalid(e.to_string()))?;
        errors.insert(name.to_owned(), json!(packed.relative_rms_error));
        Ok(())
    };
    let embed = tensor(checkpoint, "model.embed_tokens.weight")?;
    add(&mut builder, "embed", &embed, s.vocab, s.width, None)?;
    let kv_rows = s.kv_heads * s.head_dim;
    for l in 0..s.layers {
        let p = format!("model.layers.{l}");
        let input_gain = tensor(checkpoint, &format!("{p}.input_layernorm.weight"))?;
        let post_gain = tensor(checkpoint, &format!("{p}.post_attention_layernorm.weight"))?;
        for (name, rows, cols, gain, site) in [
            (
                "q_proj",
                s.width,
                s.width,
                Some(&input_gain),
                Site::Attention(l),
            ),
            (
                "k_proj",
                kv_rows,
                s.width,
                Some(&input_gain),
                Site::Attention(l),
            ),
            (
                "v_proj",
                kv_rows,
                s.width,
                Some(&input_gain),
                Site::Attention(l),
            ),
            ("o_proj", s.width, s.width, None, Site::Output(l)),
        ] {
            let mut w = tensor(checkpoint, &format!("{p}.self_attn.{name}.weight"))?;
            if let Some(gain) = gain {
                fold_columns(&mut w, cols, gain);
            }
            let short = &name[..1];
            add(
                &mut builder,
                &format!("l{l}.{short}"),
                &w,
                rows,
                cols,
                Some(site),
            )?;
        }
        for (name, short, rows, cols, gain, site) in [
            (
                "gate_proj",
                "gate",
                s.ffn,
                s.width,
                Some(&post_gain),
                Site::Mlp(l),
            ),
            (
                "up_proj",
                "up",
                s.ffn,
                s.width,
                Some(&post_gain),
                Site::Mlp(l),
            ),
            ("down_proj", "down", s.width, s.ffn, None, Site::Down(l)),
        ] {
            let mut w = tensor(checkpoint, &format!("{p}.mlp.{name}.weight"))?;
            if let Some(gain) = gain {
                fold_columns(&mut w, cols, gain);
            }
            add(
                &mut builder,
                &format!("l{l}.{short}"),
                &w,
                rows,
                cols,
                Some(site),
            )?;
        }
    }
    let mut head = if s.tied_embeddings {
        embed
    } else {
        tensor(checkpoint, "lm_head.weight")?
    };
    fold_columns(
        &mut head,
        s.width,
        &tensor(checkpoint, "model.norm.weight")?,
    );
    add(
        &mut builder,
        "head",
        &head,
        s.vocab,
        s.width,
        Some(Site::Head),
    )?;
    if let Some(cache) = cache {
        if cache.geometry != "lorentz" || cache.dim == 0 || cache.gap == 0 {
            return Err(invalid("only a Lorentz cache is served"));
        }
        if cache.query.len() != s.width * cache.dim || cache.gate_weight.len() != s.width {
            return Err(invalid("cache maps do not match the backbone width"));
        }
        // The maps act on the final normalized state: rows are the dim outputs.
        let transpose = |map: &[f32]| -> Vec<f32> {
            (0..cache.dim)
                .flat_map(|d| (0..s.width).map(move |w| map[w * cache.dim + d]))
                .collect()
        };
        for (name, map) in [("cache.query", &cache.query), ("cache.key", &cache.key)] {
            add(
                &mut builder,
                name,
                &transpose(map),
                cache.dim,
                s.width,
                Some(Site::Head),
            )?;
        }
        add(
            &mut builder,
            "cache.gate",
            &cache.gate_weight,
            1,
            s.width,
            Some(Site::Head),
        )?;
        let (beta_m, beta_e) = grid_nearest(f64::from(cache.log_beta).exp());
        builder.set_cache(CacheSpec {
            geometry: "lorentz".to_owned(),
            dim: cache.dim,
            gap: cache.gap,
            beta_m,
            beta_e,
            gate_bias: (f64::from(cache.gate_bias) * 65536.0).round() as i64,
        });
        builder
            .add_table("arcosh", TableValues::U32(&arcosh_table()))
            .map_err(|e| invalid(e.to_string()))?;
    }

    let exp: Vec<u32> = (0..EXP_RANGE * (1 << -EXP_STEP_LOG2) + 2)
        .map(|i| (2f64.powi(31) * (-(i as f64) * 2f64.powi(EXP_STEP_LOG2)).exp()).round() as u32)
        .collect();
    let half = 1i64 << (SILU_RANGE_LOG2 - SILU_STEP_LOG2);
    let silu: Vec<i32> = (0..=2 * half)
        .map(|i| {
            let x = (i - half) as f64 * 2f64.powi(SILU_STEP_LOG2);
            (2f64.powi(16) * x / (1.0 + (-x).exp())).round() as i32
        })
        .collect();
    let pairs = s.head_dim / 2;
    let mut cos = Vec::with_capacity(max_positions * pairs);
    let mut sin = Vec::with_capacity(max_positions * pairs);
    for position in 0..max_positions {
        for pair in 0..pairs {
            let inverse = 1.0 / s.rope_theta.powf((2 * pair) as f64 / s.head_dim as f64);
            let angle = position as f64 * inverse;
            let q = f64::from(1u32 << ROPE_Q);
            cos.push((q * angle.cos()).round() as i16);
            sin.push((q * angle.sin()).round() as i16);
        }
    }
    let table = |b: &mut ArtifactBuilder, name: &str, v: TableValues<'_>| -> Result<()> {
        b.add_table(name, v).map_err(|e| invalid(e.to_string()))
    };
    table(&mut builder, "exp", TableValues::U32(&exp))?;
    table(&mut builder, "silu", TableValues::I32(&silu))?;
    table(&mut builder, "rope_cos", TableValues::I16(&cos))?;
    table(&mut builder, "rope_sin", TableValues::I16(&sin))?;
    let bytes = builder.finish().map_err(|e| invalid(e.to_string()))?;
    let method = match calibration {
        Some((calibration, damp)) => json!({
            "quantizer": "gptq", "damp": damp,
            "calibration_positions": calibration.positions,
        }),
        None => json!({"quantizer": "round_to_nearest"}),
    };
    Ok((
        bytes,
        json!({
            "method": method,
            "relative_rms_error": errors,
            "relative_output_error": output_errors,
        }),
    ))
}

/// The sealed table of [`uor_r4_lut::kernels::arcosh1p_q24`]:
/// `round(2^24 arcosh(1 + g 2^-32))` at every grid point `g`, with
/// `arcosh(1 + u) = log1p(u + sqrt(u (u + 2)))` for precision near 0.
pub fn arcosh_table() -> Vec<u32> {
    use uor_r4_lut::kernels::{arcosh_grid, ARCOSH_FRACTION_BITS, ARCOSH_TABLE_LEN};
    (0..ARCOSH_TABLE_LEN)
        .map(|i| {
            let u = arcosh_grid(i) as f64 * 2f64.powi(-(ARCOSH_FRACTION_BITS as i32));
            ((u + (u * (u + 2.0)).sqrt()).ln_1p() * 2f64.powi(24)).round() as u32
        })
        .collect()
}

/// Float values of a packed 4-bit matrix (diagnostics: isolates weight
/// quantization from integer arithmetic).
pub fn dequantize_matrix(
    rows: usize,
    cols: usize,
    exp_base: i32,
    nibbles: &[u8],
    scales: &[u8],
) -> Result<Vec<f32>> {
    if nibbles.len() != rows * cols / 2 || scales.len() != rows * cols / GROUP {
        return Err(invalid("packed matrix has the wrong size"));
    }
    let mut out = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            let byte = nibbles[(r * cols + c) / 2];
            let q = i32::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
            let scale = scales[(r * cols + c) / GROUP];
            out.push(
                (f64::from(q) * grid_scale(scale & 15, exp_base + i32::from(scale >> 4))) as f32,
            );
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantization_error_is_small_and_packing_round_trips() {
        let (rows, cols) = (4, 64);
        let values: Vec<f32> = (0..rows * cols)
            .map(|i| ((i * 7919 % 1000) as f32 / 500.0 - 1.0) * if i % 5 == 0 { 3.0 } else { 0.4 })
            .collect();
        let packed = quantize_matrix(&values, rows, cols).expect("quantize");
        // One weight in five is 7.5x the rest, so the outliers set the step: uniform
        // quantization predicts about 0.15 relative error.
        assert!(
            packed.relative_rms_error < 0.15,
            "{}",
            packed.relative_rms_error
        );
        for r in 0..rows {
            for c in 0..cols {
                let byte = packed.nibbles[(r * cols + c) / 2];
                let q = i32::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
                let scale = packed.scales[r * cols / GROUP + c / GROUP];
                let s = grid_scale(scale & 15, packed.exp_base + i32::from(scale >> 4));
                let error = (f64::from(values[r * cols + c]) - f64::from(q) * s).abs();
                assert!(error <= s * 0.5 + 1e-9 || q == 7 || q == -8, "r{r} c{c}");
            }
        }
    }

    struct Rng(u64);

    impl Rng {
        fn uniform(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
        }
    }

    /// `sum_t x_t x_t^T` for `samples` inputs mixing `cols` channels through a
    /// random matrix (strongly correlated channels, as in real activations).
    fn correlated_moment(cols: usize, samples: usize, rng: &mut Rng) -> Vec<f64> {
        let mix: Vec<f64> = (0..cols * cols).map(|_| rng.uniform()).collect();
        let mut h = vec![0f64; cols * cols];
        for _ in 0..samples {
            let z: Vec<f64> = (0..cols).map(|_| rng.uniform()).collect();
            let x: Vec<f64> = (0..cols)
                .map(|i| (0..cols).map(|k| mix[i * cols + k] * z[k]).sum())
                .collect();
            for i in 0..cols {
                for j in 0..cols {
                    h[i * cols + j] += x[i] * x[j];
                }
            }
        }
        h
    }

    #[test]
    fn inverse_cholesky_factor_inverts_the_moment() {
        let n = 48;
        let mut rng = Rng(3);
        let mut h = correlated_moment(n, 200, &mut rng);
        for j in 0..n {
            h[j * n + j] += 1e-3;
        }
        let u = inverse_cholesky_upper(&h, n).expect("factor");
        for i in 0..n {
            for j in 0..i {
                assert_eq!(u[i * n + j], 0.0, "U is upper triangular");
            }
        }
        // (U^T U) H = I
        for i in 0..n {
            for j in 0..n {
                let mut value = 0.0;
                for k in 0..n {
                    let utu: f64 = (0..n).map(|l| u[l * n + i] * u[l * n + k]).sum();
                    value += utu * h[k * n + j];
                }
                let want = if i == j { 1.0 } else { 0.0 };
                assert!((value - want).abs() < 1e-6, "({i},{j}) = {value}");
            }
        }
    }

    #[test]
    fn gptq_with_uncorrelated_inputs_is_round_to_nearest() {
        let (rows, cols) = (8, 64);
        let mut rng = Rng(5);
        let values: Vec<f32> = (0..rows * cols).map(|_| rng.uniform() as f32).collect();
        let mut identity = vec![0f64; cols * cols];
        for j in 0..cols {
            identity[j * cols + j] = 1.0;
        }
        let nearest = quantize_matrix(&values, rows, cols).expect("rtn");
        let (gptq, _, _) =
            quantize_matrix_gptq(&values, rows, cols, &identity, 0.01).expect("gptq");
        assert_eq!(gptq.nibbles, nearest.nibbles);
        assert_eq!(gptq.scales, nearest.scales);
        assert_eq!(gptq.exp_base, nearest.exp_base);
    }

    #[test]
    fn gptq_reduces_the_output_error_on_correlated_inputs() {
        let (rows, cols) = (16, 64);
        let mut rng = Rng(11);
        let values: Vec<f32> = (0..rows * cols).map(|_| rng.uniform() as f32).collect();
        let h = correlated_moment(cols, 400, &mut rng);
        let (_, gptq, nearest) = quantize_matrix_gptq(&values, rows, cols, &h, 0.01).expect("gptq");
        assert!(
            gptq < 0.8 * nearest,
            "gptq {gptq} vs round-to-nearest {nearest}"
        );
    }

    #[test]
    fn integer_arcosh_matches_the_function_across_every_octave() {
        use uor_r4_lut::kernels::{arcosh1p_q24, arcosh_grid, ARCOSH_TABLE_LEN};
        let table = arcosh_table();
        assert_eq!(table.len(), ARCOSH_TABLE_LEN);
        assert!(table.windows(2).all(|w| w[0] <= w[1]) && table[0] == 0);
        assert_eq!(arcosh_grid(ARCOSH_TABLE_LEN - 1), 1u128 << 96);
        let mut rng = Rng(17);
        let mut worst = 0f64;
        for octave in 0..96u32 {
            for _ in 0..40 {
                let base = 1u128 << octave;
                let code = base + ((rng.uniform() + 0.5) * base as f64) as u128;
                let u = code as f64 * 2f64.powi(-32);
                let want = (u + (u * (u + 2.0)).sqrt()).ln_1p();
                let got = f64::from(arcosh1p_q24(code, &table)) * 2f64.powi(-24);
                worst = worst.max((got - want).abs());
            }
        }
        // Q24 resolution is 6e-8; interpolation adds at most about 1.2e-7.
        assert!(worst < 3e-7, "worst error {worst}");
        let top = f64::from(arcosh1p_q24(u128::MAX, &table)) * 2f64.powi(-24);
        assert!(
            (top - (2f64.powi(65)).ln()).abs() < 1e-6,
            "saturation {top}"
        );
    }

    #[test]
    fn near_gaussian_weights_quantize_to_under_ten_percent_error() {
        // Sum of four uniforms: close to Gaussian, like trained projection weights.
        let mut seed = 7u64;
        let mut uniform = || {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((seed >> 40) as f32) / ((1u64 << 24) as f32) - 0.5
        };
        let values: Vec<f32> = (0..64 * 256)
            .map(|_| (uniform() + uniform() + uniform() + uniform()) * 0.05)
            .collect();
        let packed = quantize_matrix(&values, 64, 256).expect("quantize");
        assert!(
            packed.relative_rms_error < 0.10,
            "{}",
            packed.relative_rms_error
        );
    }
}
