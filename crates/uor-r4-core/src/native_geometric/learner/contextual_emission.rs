//! A learned shared low-bit emission residual for one read -> geometric update -> emit primitive.
//!
//! The frozen reader can only boost the selected payload logit, so it cannot favour a token absent
//! from every admitted payload. This module adds a bounded shared residual
//!
//! ```text
//! q0 = bounded causal query state (frozen)
//! q1 = update algebra applied to (q0, relation, selected payload)
//! z  = z_local + ( W . (R[q1] - R[q0]) ) << shift
//! NoRead / UpdateDisabled: q1 = q0, residual exactly zero
//! ```
//!
//! `R` is a small seeded nonconstant ternary state embedding (`120 x CE_WIDTH`); `W` is a learned
//! ternary output map (`vocab x CE_WIDTH`). All served coefficient values are ternary (stored as one byte each); the only scale is a
//! declared integer shift, and the residual is a sum of additions/shifts/table reads.
#![forbid(unsafe_code)]

use super::group_table::GROUP_ORDER;
use super::read_conditioned::ReadConditionedParams;

/// Residual feature width.
pub const CE_WIDTH: usize = 16;

/// One development example: everything the residual and optimiser need, with no target inside the
/// prediction path. `z_local` already contains the query row `u(q0)`.
#[derive(Clone, Debug)]
pub struct CeExample {
    pub z_local: Vec<i32>,
    pub q0: usize,
    pub r: usize,
    pub payload: u32,
    pub read: bool,
    pub target: u32,
}

impl CeExample {
    /// The update's end state under one algebra (`cyclic = false` is exact signed-H4).
    pub fn q1(&self, params: &ReadConditionedParams, cyclic: bool) -> usize {
        if !self.read {
            return self.q0;
        }
        if cyclic {
            params.update_cyclic(self.q0, self.r, self.payload)
        } else {
            params.update(self.q0, self.r, self.payload)
        }
    }
}

/// The served residual model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmissionResidual {
    /// Seeded nonconstant ternary state embedding, `GROUP_ORDER x CE_WIDTH`.
    pub r_embed: Vec<[i8; CE_WIDTH]>,
    /// Learned ternary output map, `vocab x CE_WIDTH`. Zero rows are legal.
    pub w: Vec<[i8; CE_WIDTH]>,
    /// Declared integer scale applied to the summed residual.
    pub shift: u32,
}

impl EmissionResidual {
    /// Nonconstant ternary state embedding with a zero output map: the exact-baseline start that
    /// still admits a learning signal once the state changes.
    pub fn seeded(vocab: usize, seed: u64) -> Self {
        let mut st = seed | 1;
        let mut next = |m: u32| -> i8 {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            match st % (m as u64) {
                0 => -1,
                1 => 0,
                _ => 1,
            }
        };
        let r_embed = (0..GROUP_ORDER)
            .map(|_| std::array::from_fn(|_| next(3)))
            .collect();
        EmissionResidual {
            r_embed,
            w: vec![[0i8; CE_WIDTH]; vocab],
            shift: 0,
        }
    }

    /// `R[q1] - R[q0]` in integer units (zero when the state is unchanged).
    pub fn delta(&self, q0: usize, q1: usize) -> [i32; CE_WIDTH] {
        if q0 == q1 {
            return [0i32; CE_WIDTH];
        }
        let a = &self.r_embed[q1.min(GROUP_ORDER - 1)];
        let b = &self.r_embed[q0.min(GROUP_ORDER - 1)];
        std::array::from_fn(|j| i32::from(a[j]) - i32::from(b[j]))
    }

    /// One ternary row, evaluated as signed additions and a declared shift. The same arithmetic
    /// is used by fitting diagnostics and serving; artifact validation bounds coefficients/shift.
    fn row_residual(&self, row: &[i8; CE_WIDTH], d: &[i32; CE_WIDTH]) -> i32 {
        let mut acc = 0i64;
        for j in 0..CE_WIDTH {
            match row[j] {
                -1 => acc -= i64::from(d[j]),
                0 => {}
                1 => acc += i64::from(d[j]),
                _ => unreachable!("validated ternary coefficient"),
            }
        }
        (acc << self.shift.min(20)).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    }

    /// Add the sparse shared residual in place. NoRead / UpdateDisabled uses q1 == q0.
    pub fn add_to_logits(&self, q0: usize, q1: usize, rows: &[usize], logits: &mut [i32]) {
        let d = self.delta(q0, q1);
        if d.iter().all(|v| *v == 0) {
            return;
        }
        for &o in rows {
            logits[o] = logits[o].saturating_add(self.row_residual(&self.w[o], &d));
        }
    }

    /// Derive the served sparse row set from the artifact, rather than a training-only side input.
    pub fn active_rows(&self) -> Vec<usize> {
        self.w
            .iter()
            .enumerate()
            .filter_map(|(o, row)| row.iter().any(|v| *v != 0).then_some(o))
            .collect()
    }

    /// The residual added to every vocabulary logit. Exactly zero when q0 == q1.
    pub fn residual_i32(&self, q0: usize, q1: usize) -> Vec<i32> {
        let d = self.delta(q0, q1);
        self.w
            .iter()
            .map(|row| self.row_residual(row, &d))
            .collect()
    }

    /// The largest |residual| any state change can produce, in integer logit units:
    /// `CE_WIDTH * max|W| * max|delta_R| * 2^shift`.
    pub fn range_bound(&self) -> i64 {
        let maxw = self
            .w
            .iter()
            .flatten()
            .map(|v| (*v as i64).abs())
            .max()
            .unwrap_or(0);
        let maxr = self
            .r_embed
            .iter()
            .flatten()
            .map(|v| (*v as i64).abs())
            .max()
            .unwrap_or(0);
        (CE_WIDTH as i64) * maxw * 2 * maxr * (1i64 << self.shift.min(20))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLCE");
        o.extend_from_slice(&1u32.to_le_bytes());
        o.extend_from_slice(&(CE_WIDTH as u32).to_le_bytes());
        o.extend_from_slice(&self.shift.to_le_bytes());
        o.extend_from_slice(&(self.r_embed.len() as u32).to_le_bytes());
        for row in self.r_embed.iter() {
            for v in row.iter() {
                o.push(*v as u8);
            }
        }
        o.extend_from_slice(&(self.w.len() as u32).to_le_bytes());
        for row in self.w.iter() {
            for v in row.iter() {
                o.push(*v as u8);
            }
        }
        o
    }

    /// Independent load with bounded allocations and strict ternary/range checks.
    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated emission artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLCE" {
            return Err("bad emission artifact magic".into());
        }
        let ver = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        if ver != 1 {
            return Err("unsupported emission artifact version".into());
        }
        let width = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if width != CE_WIDTH {
            return Err("emission artifact width mismatch".into());
        }
        let shift = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        if shift > 20 {
            return Err("emission artifact shift out of range".into());
        }
        let n_r = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n_r != GROUP_ORDER {
            return Err("emission state embedding has the wrong row count".into());
        }
        let raw_r = take(&mut c, n_r * CE_WIDTH)?;
        let r_embed: Vec<[i8; CE_WIDTH]> = (0..n_r)
            .map(|i| std::array::from_fn(|j| raw_r[i * CE_WIDTH + j] as i8))
            .collect();
        let n_w = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n_w == 0 || n_w > max_vocab {
            return Err("emission output map has an out-of-range row count".into());
        }
        let raw_w = take(&mut c, n_w * CE_WIDTH)?;
        let w: Vec<[i8; CE_WIDTH]> = (0..n_w)
            .map(|i| std::array::from_fn(|j| raw_w[i * CE_WIDTH + j] as i8))
            .collect();
        if c != bytes.len() {
            return Err("trailing bytes in the emission artifact".into());
        }
        if r_embed
            .iter()
            .chain(w.iter())
            .flatten()
            .any(|v| !(-1..=1).contains(v))
        {
            return Err("emission coefficients outside the declared ternary range".into());
        }
        Ok(EmissionResidual { r_embed, w, shift })
    }
}

/// One deterministic projection for training forwards and export: keep at most `max_rows`
/// largest L1 latent rows, break norm ties by token id, then take coefficient signs.
/// Row admission is learned from development gradients; inference consumes only the projected map.
pub fn project_ternary_support(w: &[[f32; CE_WIDTH]], max_rows: usize) -> Vec<[i8; CE_WIDTH]> {
    let mut norms: Vec<(f64, usize)> = w
        .iter()
        .enumerate()
        .map(|(o, row)| (row.iter().map(|v| f64::from(*v).abs()).sum::<f64>(), o))
        .collect();
    norms.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut projected = vec![[0; CE_WIDTH]; w.len()];
    for (norm, o) in norms.into_iter().filter(|(n, _)| *n > 0.0).take(max_rows) {
        let _ = norm;
        projected[o] = std::array::from_fn(|j| {
            if w[o][j] > 0.0 {
                1
            } else if w[o][j] < 0.0 {
                -1
            } else {
                0
            }
        });
    }
    projected
}

/// Full-batch surrogate-gradient fitting with the exact sparse ternary serving forward.
/// The surrogate differentiates nats through signs and row admission as identity; reported losses
/// are bits. It is not a derivative of the discontinuous projector. The supplied output map is the
/// incumbent, and only strictly better exact served CE checkpoints can replace it. The returned
/// latent map must be exported through `project_ternary_support` with the same row budget.
#[allow(clippy::too_many_arguments)]
pub fn train_output_map(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    epochs: usize,
    lr: f64,
    f_bits: u32,
    max_rows: usize,
) -> (Vec<[f32; CE_WIDTH]>, f64, f64) {
    let vocab = res.w.len();
    let mut w: Vec<[f32; CE_WIDTH]> = res
        .w
        .iter()
        .map(|row| std::array::from_fn(|j| f32::from(row[j])))
        .collect();
    if examples.is_empty() || vocab == 0 {
        return (w, f64::NAN, f64::NAN);
    }
    let logit_unit = (-(f_bits as f64)).exp2();
    let scale = (1i64 << res.shift.min(20)) as f64 * logit_unit;
    let feats: Vec<[f32; CE_WIDTH]> = examples
        .iter()
        .map(|ex| {
            let d = res.delta(ex.q0, ex.q1(params, cyclic));
            std::array::from_fn(|j| d[j] as f32 * scale as f32)
        })
        .collect();
    let fscale = (feats
        .iter()
        .map(|f| f.iter().map(|v| f64::from(*v).abs()).sum::<f64>())
        .sum::<f64>()
        / examples.len() as f64)
        .max(1.0);
    let step = (lr / (examples.len() as f64 * fscale)) as f32;
    let mut served = res.clone();
    served.w = project_ternary_support(&w, max_rows);
    let mut rows = served.active_rows();
    let initial = served_nll_bits(examples, params, &served, cyclic, &rows, f_bits);
    let mut best = initial;
    let mut best_w = w.clone();
    for _ in 0..epochs {
        let mut grad = vec![[0.0f32; CE_WIDTH]; vocab];
        for (ex, f) in examples.iter().zip(feats.iter()) {
            let t = (ex.target as usize).min(vocab - 1);
            let mut hard_logits = ex.z_local.clone();
            served.add_to_logits(ex.q0, ex.q1(params, cyclic), &rows, &mut hard_logits);
            let logits: Vec<f64> = hard_logits
                .iter()
                .map(|z| f64::from(*z) * logit_unit)
                .collect();
            let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let sum: f64 = logits.iter().map(|v| (v - max).exp()).sum();
            for o in 0..vocab {
                let p = (logits[o] - max).exp() / sum;
                let g = (p - if o == t { 1.0 } else { 0.0 }) as f32;
                for j in 0..CE_WIDTH {
                    grad[o][j] += g * f[j];
                }
            }
        }
        for o in 0..vocab {
            for j in 0..CE_WIDTH {
                w[o][j] -= step * grad[o][j];
            }
        }
        served.w = project_ternary_support(&w, max_rows);
        rows = served.active_rows();
        let candidate = served_nll_bits(examples, params, &served, cyclic, &rows, f_bits);
        if candidate < best - 1e-12 {
            best_w = w.clone();
            best = candidate;
        }
    }
    (best_w, initial, best)
}

/// The actual bounded output-learning block used both before and after update-map search.
/// Refitting starts from the supplied incumbent and returns the model that serving will consume.
#[allow(clippy::too_many_arguments)]
pub fn fit_output_block(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    incumbent: &EmissionResidual,
    cyclic: bool,
    epochs: usize,
    lr: f64,
    f_bits: u32,
    max_rows: usize,
    refinement_passes: usize,
) -> (EmissionResidual, f64, f64, usize) {
    let (latent, before, _) = train_output_map(
        examples, params, incumbent, cyclic, epochs, lr, f_bits, max_rows,
    );
    let mut fitted = incumbent.clone();
    fitted.w = project_ternary_support(&latent, max_rows);
    let rows = fitted.active_rows();
    let flips = refine_ternary_map(
        examples,
        params,
        &mut fitted,
        cyclic,
        &rows,
        refinement_passes,
        f_bits,
    );
    let after = served_nll_bits(
        examples,
        params,
        &fitted,
        cyclic,
        &fitted.active_rows(),
        f_bits,
    );
    (fitted, before, after, flips)
}

/// Teacher-forced NLL in bits of the **served** (quantized, sparse) residual.
pub fn served_nll_bits(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    rows: &[usize],
    f_bits: u32,
) -> f64 {
    if examples.is_empty() || res.w.is_empty() {
        return f64::NAN;
    }
    let vocab = res.w.len();
    let logit_unit = (-(f_bits as f64)).exp2();
    let mut total = 0.0f64;
    for ex in examples.iter() {
        let q1 = ex.q1(params, cyclic);
        let mut logits = ex.z_local.clone();
        res.add_to_logits(ex.q0, q1, rows, &mut logits);
        let t = (ex.target as usize).min(vocab - 1);
        let max = logits.iter().copied().max().unwrap_or(0) as f64 * logit_unit;
        let sum: f64 = logits
            .iter()
            .map(|v| (f64::from(*v) * logit_unit - max).exp())
            .sum();
        total += (max + sum.ln() - f64::from(logits[t]) * logit_unit) / std::f64::consts::LN_2;
    }
    total / examples.len() as f64
}

/// Bounded coordinate refinement using the same calibrated served CE as gradient checkpoint
/// selection and map search. Retain the incumbent on ties and recompute each committed score.
pub fn refine_ternary_map(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &mut EmissionResidual,
    cyclic: bool,
    rows: &[usize],
    passes: usize,
    f_bits: u32,
) -> usize {
    let score =
        |res: &EmissionResidual| served_nll_bits(examples, params, res, cyclic, rows, f_bits);
    let mut best = score(res);
    let mut accepted = 0usize;
    for _ in 0..passes {
        let mut improved = false;
        for &o in rows.iter() {
            for j in 0..CE_WIDTH {
                let incumbent = res.w[o][j];
                let mut selected = incumbent;
                let mut selected_score = best;
                for cand in [-1i8, 0, 1] {
                    if cand == incumbent {
                        continue;
                    }
                    res.w[o][j] = cand;
                    let m = score(res);
                    if m < selected_score - 1e-12 {
                        selected_score = m;
                        selected = cand;
                    }
                }
                res.w[o][j] = selected;
                if selected_score < best - 1e-12 {
                    best = score(res);
                    accepted += 1;
                    improved = true;
                }
            }
        }
        if !improved {
            break;
        }
    }
    accepted
}

/// Ternary quantization of a trained float map: the sign of each coefficient.
pub fn quantize_ternary(w_f: &[[f32; CE_WIDTH]]) -> Vec<[i8; CE_WIDTH]> {
    w_f.iter()
        .map(|row| {
            std::array::from_fn(|j| {
                if row[j] > 0.0 {
                    1
                } else if row[j] < 0.0 {
                    -1
                } else {
                    0
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_training_forward_and_export_agree_above_the_row_budget() {
        let mut params = ReadConditionedParams::identity();
        params.transport[0] = 1;
        params.value_domain = vec![42];
        params.value_code = vec![0];
        let mut res = EmissionResidual::seeded(70, 3);
        res.shift = 10;
        res.r_embed = vec![[0; CE_WIDTH]; GROUP_ORDER];
        res.r_embed[1][0] = 1;
        // Seventy tied nonzero latent rows would exceed the serving budget. Token id breaks ties.
        for row in &mut res.w {
            row[0] = 1;
        }
        let ex = CeExample {
            z_local: vec![0; 70],
            q0: 0,
            r: 0,
            payload: 42,
            read: true,
            target: 69,
        };
        let (latent, initial, final_loss) = train_output_map(
            std::slice::from_ref(&ex),
            &params,
            &res,
            true,
            0,
            0.5,
            10,
            64,
        );
        res.w = project_ternary_support(&latent, 64);
        assert_eq!(res.active_rows(), (0..64).collect::<Vec<_>>());
        let loaded = EmissionResidual::from_bytes(&res.to_bytes(), 70).unwrap();
        let delta = loaded.residual_i32(0, 1);
        assert_eq!(&delta[..64], &[1024; 64]);
        assert_eq!(&delta[64..], &[0; 6]);
        let expected = (64.0 * std::f64::consts::E + 6.0).ln() / std::f64::consts::LN_2;
        let served = served_nll_bits(&[ex], &params, &loaded, true, &loaded.active_rows(), 10);
        assert!((initial - expected).abs() < 1e-12);
        assert!((final_loss - served).abs() < 1e-12);
    }

    #[test]
    fn zero_map_can_admit_a_target_row_and_retains_best_served_ce() {
        let mut params = ReadConditionedParams::identity();
        params.transport[0] = 1;
        params.value_domain = vec![42];
        params.value_code = vec![0];
        let mut res = EmissionResidual::seeded(70, 3);
        res.r_embed = vec![[0; CE_WIDTH]; GROUP_ORDER];
        res.r_embed[1][0] = 1;
        let ex = CeExample {
            z_local: vec![0; 70],
            q0: 0,
            r: 0,
            payload: 42,
            read: true,
            target: 69,
        };
        let (fitted, before, after, _) =
            fit_output_block(&[ex], &params, &res, true, 4, 0.5, 0, 64, 0);
        assert!(fitted.active_rows().contains(&69));
        assert!(fitted.active_rows().len() <= 64);
        assert!(after < before);
    }

    #[test]
    fn post_map_output_refit_is_consumed_and_keeps_the_committed_incumbent() {
        let mut params = ReadConditionedParams::identity();
        params.transport[0] = 1;
        params.value_domain = vec![42];
        params.value_code = vec![0];
        let mut res = EmissionResidual::seeded(2, 3);
        res.r_embed = vec![[0; CE_WIDTH]; GROUP_ORDER];
        res.r_embed[1][0] = 1;
        res.r_embed[2][0] = -1;
        res.w[0][0] = 1;
        let ex = CeExample {
            z_local: vec![0; 2],
            q0: 0,
            r: 0,
            payload: 42,
            read: true,
            target: 0,
        };
        let (same, _, _, _) = fit_output_block(
            std::slice::from_ref(&ex),
            &params,
            &res,
            true,
            0,
            0.5,
            0,
            2,
            0,
        );
        assert_eq!(
            same, res,
            "a no-work refit must retain the nonzero incumbent"
        );
        params.transport[0] = 2; // Changed map reverses the feature of the actual update.
        let (fitted, before, after, flips) = fit_output_block(
            std::slice::from_ref(&ex),
            &params,
            &res,
            true,
            0,
            0.5,
            0,
            2,
            1,
        );
        assert_eq!(flips, 1);
        assert!(after < before);
        assert_eq!(fitted.w[0][0], -1);
        let loaded = EmissionResidual::from_bytes(&fitted.to_bytes(), 2).unwrap();
        let q1 = ex.q1(&params, true);
        assert_eq!(q1, 2);
        assert_eq!(res.residual_i32(ex.q0, q1), vec![-1, 0]);
        assert_eq!(loaded.residual_i32(ex.q0, q1), vec![1, 0]);
        assert!(
            (served_nll_bits(&[ex], &params, &loaded, true, &loaded.active_rows(), 0) - after)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn disabled_state_gives_an_exactly_zero_residual_and_round_trips() {
        let mut res = EmissionResidual::seeded(64, 0x1234);
        let q0 = 7usize;
        let z = res.residual_i32(q0, q0);
        assert!(z.iter().all(|v| *v == 0), "q1 == q0 must be exactly zero");
        res.w[3] = [1, -1, 0, 1, 0, -1, 1, 0, 0, 1, -1, 0, 1, 0, -1, 1];
        assert!(res.residual_i32(q0, q0).iter().all(|v| *v == 0));
        let bytes = res.to_bytes();
        let back = EmissionResidual::from_bytes(&bytes, 64).unwrap();
        assert_eq!(back, res);
        assert!(
            EmissionResidual::from_bytes(&bytes, 4).is_err(),
            "bounded vocab"
        );
        let mut bad = bytes.clone();
        bad.truncate(bad.len() - 1);
        assert!(EmissionResidual::from_bytes(&bad, 64).is_err());
    }

    #[test]
    fn training_reduces_nll_and_the_update_can_change_the_residual() {
        // Two examples with different features and different targets.
        let params = ReadConditionedParams::identity();
        let res = EmissionResidual::seeded(8, 9);
        let mk = |target: u32, payload: u32, r: usize| CeExample {
            z_local: vec![0; 8],
            q0: 1,
            r,
            payload,
            read: true,
            target,
        };
        let examples = vec![mk(2, 11, 3), mk(5, 12, 4)];
        let mut p = params.clone();
        p.value_domain = vec![11, 12];
        p.value_code = vec![1, 2];
        let (w_f, before, after) = train_output_map(&examples, &p, &res, false, 40, 0.5, 0, 8);
        assert!(after <= before + 1e-9, "training must not increase NLL");
        assert!(
            after < before - 1e-6,
            "training must reduce NLL: {before} -> {after}"
        );
        let _ = quantize_ternary(&w_f);
    }
    #[test]
    fn ternary_refinement_retains_an_earlier_accepted_coordinate() {
        let mut params = ReadConditionedParams::identity();
        params.transport[0] = 1;
        params.value_domain = vec![42];
        params.value_code = vec![0];
        let ex = CeExample {
            z_local: vec![0, 0],
            q0: 0,
            r: 0,
            payload: 42,
            read: true,
            target: 0,
        };
        let mut res = EmissionResidual::seeded(2, 1);
        res.r_embed = vec![[0; CE_WIDTH]; GROUP_ORDER];
        res.r_embed[0][0] = 1;
        res.w[0][0] = 1;
        assert_eq!(res.residual_i32(0, 1)[0], -1);
        assert_eq!(
            refine_ternary_map(&[ex], &params, &mut res, true, &[0], 2, 0),
            1
        );
        assert_eq!(res.w[0][0], -1);
        assert_eq!(res.residual_i32(0, 1)[0], 1);
    }

    #[test]
    fn fixed_point_loss_matches_the_parent_units_and_training_start() {
        let params = ReadConditionedParams::identity();
        let mut res = EmissionResidual::seeded(2, 1);
        res.shift = 10;
        let ex = CeExample {
            z_local: vec![1024, 0],
            q0: 0,
            r: 0,
            payload: 42,
            read: false,
            target: 0,
        };
        let expected = (1.0 + (-1.0f64).exp()).ln() / std::f64::consts::LN_2;
        let observed = served_nll_bits(std::slice::from_ref(&ex), &params, &res, false, &[], 10);
        let (_, initial, final_loss) = train_output_map(&[ex], &params, &res, false, 1, 0.5, 10, 2);
        assert!((observed - expected).abs() < 1e-12);
        assert!((initial - expected).abs() < 1e-12);
        assert!((final_loss - expected).abs() < 1e-12);
    }

    #[test]
    fn sparse_and_full_residuals_agree_after_independent_loading() {
        let mut res = EmissionResidual::seeded(3, 2);
        res.shift = 20;
        res.w[1] = [-1; CE_WIDTH];
        res.w[2] = [1; CE_WIDTH];
        let loaded = EmissionResidual::from_bytes(&res.to_bytes(), 3).unwrap();
        let mut logits = vec![0; 3];
        loaded.add_to_logits(0, 1, &loaded.active_rows(), &mut logits);
        assert_eq!(logits, res.residual_i32(0, 1));
        let original = logits.clone();
        loaded.add_to_logits(0, 0, &loaded.active_rows(), &mut logits);
        assert_eq!(logits, original);
    }
}
