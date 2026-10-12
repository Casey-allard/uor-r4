//! E8 orbit memory: an addressed memory that can replace a layer's MLP, built
//! on the SpiralCore spin-group structure of the E8 roots (#2032).
//!
//! Per head the query has two 8-dimensional halves. Each half is decoded to
//! its nearest E8 root **exactly in O(8)** (the two largest coordinates for
//! the integer roots, the sign pattern with a parity fix for the half-integer
//! roots; no scan of the 240 roots). The 64-element Cl(0,6) bivector group of
//! [`uor_r4_core::spiralcore_operator::cl06_finite_group`] preserves the 240
//! roots and splits them into 15 orbits of 16 ([`e8_tables`]). A half's
//! candidates are the 16 roots of its decoded root's orbit (its expert); the
//! top `k` of each half form `k^2` slot pairs `(a, b)` of a value table of
//! `240^2` rows, and the best `k` pairs by summed score are read.
//!
//! The weight of the kept slot at rank `r` is `R[h][r] * sigmoid(beta_h *
//! s + gamma_h)` with `s` the summed sub-key score: a plain learned rank table
//! and a gate, with no softmax and no normalisation. Selection is detached;
//! queries, values, the rank table and the gate get exact gradients.
//!
//! Per token and head the address costs two O(8) decodes and 2 x 16 dot
//! products of 8 numbers; the read touches `k` value rows. A dense MLP of
//! width `d` and hidden `m` touches `3 d m` weights.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use candle_core::{DType, Tensor};
use rayon::prelude::*;

use crate::stack_memory::Codebook;
use crate::{invalid, Result};

pub const E8_ROOTS: usize = 240;
pub const E8_ORBITS: usize = 15;
pub const E8_ORBIT_SIZE: usize = 16;
/// Slots of the value table: one per ordered pair of roots.
pub const E8_SLOTS: usize = E8_ROOTS * E8_ROOTS;

/// The E8 roots (unit norm, [`Codebook::E8`] order) with their exact
/// SpiralCore orbit structure.
pub struct E8Tables {
    exact_keys: Vec<[i8; 8]>,
    pub roots: Vec<[f32; 8]>,
    index: HashMap<[i8; 8], usize>,
    /// The orbit (0..15) of each root.
    pub orbit_of: Vec<u8>,
    /// The 16 roots of each orbit, ascending.
    pub members: Vec<[u16; E8_ORBIT_SIZE]>,
    /// Offline-compiled SpiralCore signed actions; numerical route transport is
    /// a bounded integer lookup, with no float root decoding in that operation.
    action_images: Vec<[u16; E8_ROOTS]>,
}

/// A root's exact integer key: coordinates times `2 sqrt 2` (D8 roots give
/// +-2, half-integer roots +-1).
fn root_key(v: &[f64]) -> [i8; 8] {
    let scale = 2.0 * 2f64.sqrt();
    let mut key = [0i8; 8];
    for (k, x) in key.iter_mut().zip(v) {
        *k = (x * scale).round() as i8;
    }
    key
}

pub fn e8_tables() -> Result<&'static E8Tables> {
    static TABLES: OnceLock<std::result::Result<E8Tables, String>> = OnceLock::new();
    TABLES
        .get_or_init(|| build_tables().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|message| invalid(message.clone()))
}

fn build_tables() -> Result<E8Tables> {
    let vectors = Codebook::E8.vectors();
    if vectors.len() != E8_ROOTS {
        return Err(invalid("the E8 codebook does not have 240 roots"));
    }
    let keys: Vec<[i8; 8]> = vectors.iter().map(|v| root_key(v)).collect();
    let index: HashMap<[i8; 8], usize> = keys.iter().enumerate().map(|(i, k)| (*k, i)).collect();
    if index.len() != E8_ROOTS {
        return Err(invalid("E8 root keys are not distinct"));
    }
    let group = uor_r4_core::spiralcore_operator::cl06_finite_group()
        .map_err(|e| invalid(format!("SpiralCore group: {e:?}")))?;
    let mut action_images = Vec::with_capacity(group.len());
    for element in &group {
        let mut images = [0; E8_ROOTS];
        for (root, key) in keys.iter().enumerate() {
            let mut image = [0i8; 8];
            for (r, out) in image.iter_mut().enumerate() {
                *out = (0..8).map(|c| element.entries()[r][c] * key[c]).sum();
            }
            images[root] = *index
                .get(&image)
                .ok_or_else(|| invalid("SpiralCore action leaves E8 root set"))?
                as u16;
        }
        action_images.push(images);
    }
    let mut orbit_of = vec![u8::MAX; E8_ROOTS];
    let mut members = Vec::new();
    for start in 0..E8_ROOTS {
        if orbit_of[start] != u8::MAX {
            continue;
        }
        let mut orbit = Vec::new();
        for element in &group {
            let m = element.entries();
            let mut image = [0i8; 8];
            for (r, out) in image.iter_mut().enumerate() {
                *out = (0..8).map(|c| m[r][c] * keys[start][c]).sum();
            }
            let j = *index
                .get(&image)
                .ok_or_else(|| invalid("the SpiralCore group does not preserve the E8 roots"))?;
            if !orbit.contains(&j) {
                orbit.push(j);
            }
        }
        orbit.sort_unstable();
        let id = members.len() as u8;
        for &j in &orbit {
            orbit_of[j] = id;
        }
        let orbit: [u16; E8_ORBIT_SIZE] = orbit
            .iter()
            .map(|&j| j as u16)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|o: Vec<u16>| invalid(format!("an E8 orbit has {} roots, not 16", o.len())))?;
        members.push(orbit);
    }
    if members.len() != E8_ORBITS {
        return Err(invalid(format!("{} E8 orbits, not 15", members.len())));
    }
    Ok(E8Tables {
        exact_keys: keys,
        roots: vectors
            .iter()
            .map(|v| [0, 1, 2, 3, 4, 5, 6, 7].map(|i| v[i] as f32))
            .collect(),
        index,
        orbit_of,
        members,
        action_images,
    })
}

impl E8Tables {
    pub fn exact_root_key(&self, root: u16) -> Result<[i8; 8]> {
        self.exact_keys
            .get(usize::from(root))
            .copied()
            .ok_or_else(|| invalid("E8 exact root outside240"))
    }
    /// Apply an existing SpiralCore action to an existing Claude E8 address.
    /// These are distinct typed domains: an action index is not a root index.
    pub fn transport_root(&self, root: u16, action: u8) -> Result<u16> {
        self.action_images
            .get(usize::from(action))
            .and_then(|r| r.get(usize::from(root)))
            .copied()
            .ok_or_else(|| invalid("E8 root/SpiralCore action outside bound domain"))
    }
    /// The root with the largest dot product with `x`, in O(8): the best D8
    /// root takes the two largest `|x_i|` with their signs; the best
    /// half-integer root takes the signs of `x`, flipping the smallest `|x_i|`
    /// when the minus count is odd. All roots have one norm, so the larger
    /// dot product of the two wins (ties to the D8 candidate).
    pub fn nearest(&self, x: &[f32; 8]) -> Result<usize> {
        let mut order = [0usize, 1, 2, 3, 4, 5, 6, 7];
        order.sort_by(|&a, &b| x[b].abs().total_cmp(&x[a].abs()).then(a.cmp(&b)));
        let mut d = [0i8; 8];
        for &i in &order[..2] {
            d[i] = if x[i] >= 0.0 { 2 } else { -2 };
        }
        let mut h = [0i8; 8];
        for i in 0..8 {
            h[i] = if x[i] >= 0.0 { 1 } else { -1 };
        }
        if h.iter().filter(|&&v| v < 0).count() % 2 == 1 {
            let smallest = order[7];
            h[smallest] = -h[smallest];
        }
        let dot = |k: &[i8; 8]| -> f32 { (0..8).map(|i| f32::from(k[i]) * x[i]).sum() };
        let key = if dot(&d) >= dot(&h) { d } else { h };
        self.index
            .get(&key)
            .copied()
            .ok_or_else(|| invalid("decoded vector is not an E8 root"))
    }
}

/// Which layers' MLPs are E8 orbit memories, with `heads` query heads that
/// each keep `k` slots ([`crate::geometric_stack::StackModel::set_e8_memory`]).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct E8MemoryConfig {
    pub layers: Vec<usize>,
    pub heads: usize,
    pub k: usize,
}

impl E8MemoryConfig {
    /// Parse `e8_memory=L1,L2:HEADS:K`, or `none`.
    pub fn parse(text: &str) -> Result<Option<Self>> {
        if text == "none" {
            return Ok(None);
        }
        let bad = || invalid(format!("invalid e8_memory={text} (none or LAYERS:HEADS:K)"));
        let parts: Vec<&str> = text.split(':').collect();
        let [layers, heads, k] = parts.as_slice() else {
            return Err(bad());
        };
        let layers = layers
            .split(',')
            .map(|l| l.parse::<usize>().map_err(|_| bad()))
            .collect::<Result<Vec<_>>>()?;
        let heads: usize = heads.parse().map_err(|_| bad())?;
        let k: usize = k.parse().map_err(|_| bad())?;
        let config = Self { layers, heads, k };
        if config.heads == 0
            || config.k == 0
            || config.k > E8_ORBIT_SIZE
            || config.layers.is_empty()
        {
            return Err(bad());
        }
        Ok(Some(config))
    }

    pub fn name(&self) -> String {
        let layers: Vec<String> = self.layers.iter().map(usize::to_string).collect();
        format!("{}:{}:{}", layers.join(","), self.heads, self.k)
    }
}

/// Evaluation-time routing record ([`e8_usage_start`], [`e8_usage_take`]):
/// per (layer, head), how often each of the 15 orbits was chosen for each
/// query half and which value slots were read. Off unless started.
type UsageRecord = BTreeMap<(usize, usize), ([u64; E8_ORBITS], [u64; E8_ORBITS], HashSet<u32>)>;
static USAGE: Mutex<Option<UsageRecord>> = Mutex::new(None);

/// Start recording E8 memory routing (clears any previous record).
pub fn e8_usage_start() {
    if let Ok(mut usage) = USAGE.lock() {
        *usage = Some(BTreeMap::new());
    }
}

/// Stop recording and summarise: per layer and head, the orbit entropy of
/// each query half in bits (log2 15 = 3.91 is perfectly balanced, 0 is
/// collapse onto one orbit) and the share of the 240^2 slots read.
pub fn e8_usage_take() -> serde_json::Value {
    let record = USAGE
        .lock()
        .ok()
        .and_then(|mut usage| usage.take())
        .unwrap_or_default();
    let entropy = |counts: &[u64; E8_ORBITS]| -> f64 {
        let total: u64 = counts.iter().sum();
        if total == 0 {
            return 0.0;
        }
        counts
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f64 / total as f64;
                -p * p.log2()
            })
            .sum()
    };
    let heads: Vec<serde_json::Value> = record
        .iter()
        .map(|((layer, head), (a, b, slots))| {
            serde_json::json!({
                "layer": layer, "head": head,
                "orbit_entropy_bits_a": entropy(a), "orbit_entropy_bits_b": entropy(b),
                "slots_read": slots.len(), "slot_share": slots.len() as f64 / E8_SLOTS as f64,
            })
        })
        .collect();
    serde_json::json!({"max_orbit_entropy_bits": (E8_ORBITS as f64).log2(), "heads": heads})
}

fn dot8(a: &[f32], b: &[f32; 8]) -> f32 {
    (0..8).map(|i| a[i] * b[i]).sum()
}

/// Selected slots for one query (one row, one head): the kept slot pairs in
/// rank order, each `(root_a, root_b)`.
pub fn e8_select(tables: &E8Tables, query: &[f32], k: usize) -> Result<Vec<(usize, usize)>> {
    if query.len() != 16 || k == 0 || k > E8_ORBIT_SIZE {
        return Err(invalid(
            "an E8 memory query is two 8-vectors and 1 <= k <= 16",
        ));
    }
    let half = |h: &[f32]| -> Result<Vec<(f32, usize)>> {
        let x: [f32; 8] = h
            .try_into()
            .map_err(|_| invalid("E8 half is not 8 numbers"))?;
        let orbit = tables.orbit_of[tables.nearest(&x)?] as usize;
        let mut scored: Vec<(f32, usize)> = tables.members[orbit]
            .iter()
            .map(|&j| (dot8(h, &tables.roots[j as usize]), j as usize))
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        scored.truncate(k);
        Ok(scored)
    };
    let (left, right) = (half(&query[..8])?, half(&query[8..])?);
    let mut pairs: Vec<(f32, usize, usize)> = left
        .iter()
        .flat_map(|&(sa, a)| right.iter().map(move |&(sb, b)| (sa + sb, a, b)))
        .collect();
    pairs.sort_by(|x, y| y.0.total_cmp(&x.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
    Ok(pairs.into_iter().take(k).map(|(_, a, b)| (a, b)).collect())
}

/// The E8 orbit memory read. `query` is `[rows, heads * 16]`, `values` is
/// `[E8_SLOTS, width]`, `rank` is `[heads, k]` (plain learned weights),
/// `beta`/`gamma` are `[heads]`. Returns `[rows, width]`: the sum over heads
/// of each head's gated, rank-weighted mix of its `k` kept value rows.
pub fn e8_memory_read(
    layer: usize,
    query: &Tensor,
    values: &Tensor,
    rank: &Tensor,
    beta: &Tensor,
    gamma: &Tensor,
    k: usize,
) -> Result<Tensor> {
    let tables = e8_tables()?;
    let (rows, q_width) = query.dims2()?;
    let heads = q_width / 16;
    if heads * 16 != q_width || rank.dims() != [heads, k] {
        return Err(invalid(
            "E8 memory shapes: query [rows, heads*16], rank [heads, k]",
        ));
    }
    let device = query.device();
    let query = query.to_dtype(DType::F32)?.contiguous()?;
    let host = query.flatten_all()?.to_vec1::<f32>()?;
    let mut slot = vec![0u32; rows * heads * k];
    let mut root_a = vec![0u32; rows * heads * k];
    let mut root_b = vec![0u32; rows * heads * k];
    let mut live = vec![0f32; rows * heads * k];
    let mut usage = USAGE.lock().ok();
    let recording = usage.as_ref().is_some_and(|u| u.is_some());
    // Every (row, head) selection is independent: run them in parallel, then
    // record and lay them out in order.
    let selections: Vec<Vec<(usize, usize)>> = (0..rows * heads)
        .into_par_iter()
        .map(|i| {
            let base = (i / heads) * q_width + (i % heads) * 16;
            e8_select(tables, &host[base..base + 16], k)
        })
        .collect::<Result<_>>()?;
    let mut selections = selections.into_iter();
    for row in 0..rows {
        for h in 0..heads {
            let base = row * q_width + h * 16;
            let kept = selections
                .next()
                .ok_or_else(|| invalid("E8 selection count"))?;
            if recording {
                if let Some(Some(record)) = usage.as_deref_mut() {
                    let entry = record.entry((layer, h)).or_insert((
                        [0; E8_ORBITS],
                        [0; E8_ORBITS],
                        HashSet::new(),
                    ));
                    let half = |q: &[f32]| -> Result<usize> {
                        let x: [f32; 8] = q.try_into().map_err(|_| invalid("E8 half"))?;
                        Ok(tables.orbit_of[tables.nearest(&x)?] as usize)
                    };
                    entry.0[half(&host[base..base + 8])?] += 1;
                    entry.1[half(&host[base + 8..base + 16])?] += 1;
                    for &(a, b) in &kept {
                        entry.2.insert((a * E8_ROOTS + b) as u32);
                    }
                }
            }
            for (r, (a, b)) in kept.into_iter().enumerate() {
                let i = (row * heads + h) * k + r;
                slot[i] = (a * E8_ROOTS + b) as u32;
                root_a[i] = a as u32;
                root_b[i] = b as u32;
                live[i] = 1.0;
            }
        }
    }
    let n = rows * heads * k;
    let roots: Vec<f32> = tables
        .roots
        .iter()
        .flat_map(|r| r.iter().copied())
        .collect();
    let roots = Tensor::from_vec(roots, (E8_ROOTS, 8), device)?;
    let ra = roots.index_select(&Tensor::from_vec(root_a, n, device)?, 0)?;
    let rb = roots.index_select(&Tensor::from_vec(root_b, n, device)?, 0)?;
    // Each (row, head) query half, repeated for its k kept slots.
    let halves = query.reshape((rows, heads, 2, 8))?;
    let qa = halves
        .narrow(2, 0, 1)?
        .reshape((rows, heads, 1, 8))?
        .broadcast_as((rows, heads, k, 8))?
        .reshape((n, 8))?;
    let qb = halves
        .narrow(2, 1, 1)?
        .reshape((rows, heads, 1, 8))?
        .broadcast_as((rows, heads, k, 8))?
        .reshape((n, 8))?;
    let score = qa
        .mul(&ra)?
        .sum(1)?
        .add(&qb.mul(&rb)?.sum(1)?)?
        .reshape((rows, heads, k))?;
    let gate = score
        .broadcast_mul(&beta.reshape((1, heads, 1))?)?
        .broadcast_add(&gamma.reshape((1, heads, 1))?)?
        .neg()?
        .exp()?
        .affine(1.0, 1.0)?
        .recip()?;
    let live = Tensor::from_vec(live, (rows, heads, k), device)?;
    let weights = gate
        .broadcast_mul(&rank.reshape((1, heads, k))?)?
        .mul(&live)?
        .reshape((rows, heads * k, 1))?;
    let width = values.dim(1)?;
    let read = values
        .index_select(&Tensor::from_vec(slot, n, device)?, 0)?
        .reshape((rows, heads * k, width))?;
    Ok(weights.broadcast_mul(&read)?.sum(1)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Var};

    fn rng(seed: &mut u64) -> f32 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        ((*seed >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0) as f32
    }

    #[test]
    fn spin_group_splits_the_e8_roots_into_fifteen_orbits_of_sixteen() -> Result<()> {
        let t = e8_tables()?;
        assert_eq!(t.roots.len(), 240);
        assert_eq!(t.members.len(), 15);
        let mut seen = vec![false; 240];
        for (o, m) in t.members.iter().enumerate() {
            for &j in m {
                assert!(!seen[j as usize]);
                seen[j as usize] = true;
                assert_eq!(t.orbit_of[j as usize] as usize, o);
            }
        }
        assert!(seen.iter().all(|&s| s));
        Ok(())
    }

    #[test]
    fn exact_spiral_e8_transport_preserves_orbits_and_ordered_composition() -> Result<()> {
        let t = e8_tables()?;
        let g = uor_r4_core::spiralcore_operator::cl06_finite_composition_table()
            .map_err(|e| invalid(e.to_string()))?;
        for root in 0..E8_ROOTS as u16 {
            assert_eq!(t.transport_root(root, g.identity_index())?, root);
            for a in 0..64u8 {
                let image = t.transport_root(root, a)?;
                assert_eq!(
                    t.orbit_of[usize::from(root)],
                    t.orbit_of[usize::from(image)]
                );
                assert_eq!(
                    t.transport_root(
                        image,
                        g.inverse_index(a)
                            .ok_or_else(|| invalid("inverse absent"))?
                    )?,
                    root
                );
                for b in 0..64u8 {
                    let product = g
                        .compose_index(a, b)
                        .ok_or_else(|| invalid("product absent"))?;
                    assert_eq!(
                        t.transport_root(t.transport_root(root, b)?, a)?,
                        t.transport_root(root, product)?
                    );
                }
            }
        }
        assert!(t.transport_root(240, 0).is_err());
        assert!(t.transport_root(0, 64).is_err());
        Ok(())
    }

    #[test]
    fn the_o8_decoder_equals_a_full_scan() -> Result<()> {
        let t = e8_tables()?;
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..20_000 {
            let x: [f32; 8] = std::array::from_fn(|_| rng(&mut seed));
            let fast = t.nearest(&x)?;
            let best = (0..240)
                .map(|j| dot8(&x, &t.roots[j]))
                .fold(f32::NEG_INFINITY, f32::max);
            assert!((dot8(&x, &t.roots[fast]) - best).abs() < 1e-5);
        }
        Ok(())
    }

    #[test]
    fn selection_matches_a_brute_force_reference() -> Result<()> {
        let t = e8_tables()?;
        let mut seed = 77u64;
        for _ in 0..200 {
            let q: Vec<f32> = (0..16).map(|_| rng(&mut seed)).collect();
            let k = 5;
            let kept = e8_select(t, &q, k)?;
            // Reference: each half's orbit by full scan, its top k, best k pairs.
            let half = |h: &[f32]| -> Vec<(f32, usize)> {
                let best = (0..240)
                    .max_by(|&a, &b| dot8(h, &t.roots[a]).total_cmp(&dot8(h, &t.roots[b])))
                    .unwrap_or(0);
                let mut s: Vec<(f32, usize)> = t.members[t.orbit_of[best] as usize]
                    .iter()
                    .map(|&j| (dot8(h, &t.roots[j as usize]), j as usize))
                    .collect();
                s.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
                s.truncate(k);
                s
            };
            let (l, r) = (half(&q[..8]), half(&q[8..]));
            let mut pairs: Vec<(f32, usize, usize)> = l
                .iter()
                .flat_map(|&(sa, a)| r.iter().map(move |&(sb, b)| (sa + sb, a, b)))
                .collect();
            pairs.sort_by(|x, y| y.0.total_cmp(&x.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
            let expected: Vec<(usize, usize)> =
                pairs.into_iter().take(k).map(|(_, a, b)| (a, b)).collect();
            assert_eq!(kept, expected);
        }
        Ok(())
    }

    #[test]
    fn the_read_gradient_matches_finite_differences() -> Result<()> {
        let dev = Device::Cpu;
        let (rows, heads, k, width) = (3, 2, 4, 5);
        let mut seed = 5u64;
        let mut var = |shape: &[usize], scale: f32| -> Result<Var> {
            let n: usize = shape.iter().product();
            let v: Vec<f32> = (0..n).map(|_| scale * rng(&mut seed)).collect();
            Ok(Var::from_tensor(&Tensor::from_vec(v, shape, &dev)?)?)
        };
        let query = var(&[rows, heads * 16], 1.0)?;
        let values = var(&[E8_SLOTS, width], 0.5)?;
        let rank = var(&[heads, k], 0.5)?;
        let beta = var(&[heads], 1.0)?;
        let gamma = var(&[heads], 0.2)?;
        let probe = Tensor::from_vec(
            (0..rows * width)
                .map(|i| (i as f32 * 0.37).sin())
                .collect::<Vec<_>>(),
            (rows, width),
            &dev,
        )?;
        let loss = || -> Result<Tensor> {
            Ok(e8_memory_read(
                0,
                query.as_tensor(),
                values.as_tensor(),
                rank.as_tensor(),
                beta.as_tensor(),
                gamma.as_tensor(),
                k,
            )?
            .mul(&probe)?
            .sum_all()?)
        };
        let grads = loss()?.backward()?;
        // Selection is fixed under small steps of rank, gate and query.
        for var in [&rank, &beta, &gamma, &query] {
            let g = grads
                .get(var.as_tensor())
                .ok_or_else(|| invalid("missing gradient"))?;
            let g = g.flatten_all()?.to_vec1::<f32>()?;
            let original = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            for i in [0, original.len() / 2, original.len() - 1] {
                let eps = 1e-3f32;
                let mut plus = original.clone();
                plus[i] += eps;
                var.set(&Tensor::from_vec(plus, var.dims(), &dev)?)?;
                let lp = loss()?.to_scalar::<f32>()?;
                let mut minus = original.clone();
                minus[i] -= eps;
                var.set(&Tensor::from_vec(minus, var.dims(), &dev)?)?;
                let lm = loss()?.to_scalar::<f32>()?;
                var.set(&Tensor::from_vec(original.clone(), var.dims(), &dev)?)?;
                let numeric = (lp - lm) / (2.0 * eps);
                let scale = g[i].abs().max(numeric.abs()).max(1e-3);
                assert!(
                    (g[i] - numeric).abs() / scale < 2e-2,
                    "index {i}: analytic {} numeric {numeric}",
                    g[i]
                );
            }
        }
        Ok(())
    }
}
