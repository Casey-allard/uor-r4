//! Integer sampling of the next token from `v * 2^-16` logits.
//!
//! Temperature, top-k, top-p and a presence penalty use integer arithmetic
//! only: weights come from the artifact's sealed `exp` table, the temperature
//! is a Q8 constant applied by one division per candidate, and the random
//! numbers come from a seeded xorshift generator, so a seed reproduces a
//! conversation exactly. Temperature 0 is greedy decoding.

use std::collections::BTreeMap;

use crate::engine::CacheRead;
use crate::kernels::{argmax, exp_neg};
use crate::{invalid, Result, RESIDUAL_EXP};

/// Sampling settings, in fixed point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SamplingSettings {
    /// Temperature times 256; 0 means greedy.
    pub temperature_q8: u32,
    /// Keep only the `top_k` largest logits (0: no limit).
    pub top_k: usize,
    /// Keep the smallest set of candidates holding this share of the mass,
    /// times 65536 (65536: no limit).
    pub top_p_q16: u32,
    /// Subtracted from the logit of every token seen in the recent window,
    /// in nats times 256.
    pub presence_q8: u32,
    /// How many recent tokens the presence penalty covers.
    pub window: usize,
}

impl Default for SamplingSettings {
    fn default() -> Self {
        Self {
            temperature_q8: 0,
            top_k: 0,
            top_p_q16: 1 << 16,
            presence_q8: 0,
            window: 64,
        }
    }
}

impl SamplingSettings {
    /// From decimal values: temperature, top-p in `(0, 1]`, presence in nats.
    pub fn from_decimal(temperature: f64, top_k: usize, top_p: f64, presence: f64) -> Result<Self> {
        if !(0.0..=16.0).contains(&temperature)
            || !(top_p > 0.0 && top_p <= 1.0)
            || !(0.0..=16.0).contains(&presence)
        {
            return Err(invalid(
                "temperature and presence must be in [0, 16], top_p in (0, 1]",
            ));
        }
        Ok(Self {
            temperature_q8: (temperature * 256.0).round() as u32,
            top_k,
            top_p_q16: (top_p * 65536.0).round() as u32,
            presence_q8: (presence * 256.0).round() as u32,
            ..Self::default()
        })
    }
}

/// A seeded sampler with scratch space.
pub struct Sampler {
    settings: SamplingSettings,
    state: u64,
    adjusted: Vec<i32>,
    candidates: Vec<(u32, u64)>,
}

impl Sampler {
    pub fn new(settings: SamplingSettings, seed: u64) -> Self {
        Self {
            settings,
            state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            adjusted: Vec::new(),
            candidates: Vec::new(),
        }
    }

    pub fn settings(&self) -> SamplingSettings {
        self.settings
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// The next token given the logits (`v * 2^-16` nats) and the tokens
    /// generated or read so far. `exp_table` and `exp_step_log2` are the
    /// artifact's sealed exp table.
    pub fn sample(
        &mut self,
        logits: &[i32],
        recent: &[u32],
        exp_table: &[u32],
        exp_step_log2: i32,
    ) -> Result<u32> {
        if logits.is_empty() {
            return Err(invalid("no logits to sample from"));
        }
        self.adjust(logits, recent)?;
        if self.settings.temperature_q8 == 0 {
            return Ok(argmax(&self.adjusted) as u32);
        }
        self.build_candidates(exp_table, exp_step_log2);
        let total: u64 = self.candidates.iter().map(|c| c.1).sum();
        if total == 0 {
            return Ok(self.candidates[0].0);
        }
        let mut draw = self.next_u64() % total.max(1);
        for c in &self.candidates {
            if draw < c.1 {
                return Ok(c.0);
            }
            draw -= c.1;
        }
        Ok(self.candidates[self.candidates.len() - 1].0)
    }

    /// The presence-adjusted, tempered and truncated candidates of `logits`
    /// in `self.candidates` (sampling with a positive temperature).
    fn sample_candidates(
        &mut self,
        logits: &[i32],
        recent: &[u32],
        exp_table: &[u32],
        exp_step_log2: i32,
    ) -> Result<()> {
        if logits.is_empty() {
            return Err(invalid("no logits to sample from"));
        }
        self.adjust(logits, recent)?;
        self.build_candidates(exp_table, exp_step_log2);
        Ok(())
    }

    fn adjust(&mut self, logits: &[i32], recent: &[u32]) -> Result<()> {
        let s = self.settings;
        self.adjusted.clear();
        self.adjusted.extend_from_slice(logits);
        if s.presence_q8 > 0 {
            let penalty = i32::try_from(s.presence_q8)
                .map_err(|_| invalid("presence penalty out of range"))?
                << (-RESIDUAL_EXP - 8);
            let start = recent.len().saturating_sub(s.window);
            let mut seen = recent[start..].to_vec();
            seen.sort_unstable();
            seen.dedup();
            for token in seen {
                if let Some(v) = self.adjusted.get_mut(token as usize) {
                    *v = v.saturating_sub(penalty);
                }
            }
        }
        Ok(())
    }

    fn build_candidates(&mut self, exp_table: &[u32], exp_step_log2: i32) {
        let s = self.settings;
        let max = self.adjusted.iter().copied().max().unwrap_or(0);
        self.candidates.clear();
        self.candidates.extend(
            self.adjusted
                .iter()
                .enumerate()
                .map(|(token, &v)| (token as u32, (i64::from(max) - i64::from(v)) as u64)),
        );
        // Top-k by smallest gap to the maximum (ties by token id).
        if s.top_k > 0 && s.top_k < self.candidates.len() {
            self.candidates
                .select_nth_unstable_by_key(s.top_k - 1, |&(token, gap)| (gap, token));
            self.candidates.truncate(s.top_k);
        }
        // Weights exp(-gap / T): the gap is in 2^-16 nats, T = temperature_q8 / 256.
        let temperature = u64::from(s.temperature_q8);
        for candidate in self.candidates.iter_mut() {
            let scaled = ((candidate.1 << 8) / temperature) as i64;
            candidate.1 = exp_neg(scaled, RESIDUAL_EXP, exp_table, exp_step_log2);
        }
        self.candidates
            .sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let total: u64 = self.candidates.iter().map(|c| c.1).sum();
        if total > 0 && s.top_p_q16 < 1 << 16 {
            let target = (u128::from(total) * u128::from(s.top_p_q16)) >> 16;
            let mut running = 0u128;
            let mut keep = self.candidates.len();
            for (i, c) in self.candidates.iter().enumerate() {
                running += u128::from(c.1);
                if running >= target {
                    keep = i + 1;
                    break;
                }
            }
            self.candidates.truncate(keep.max(1));
        }
    }
}

impl Sampler {
    /// [`Self::sample`] from the mixture `(1 - g) p_model + g p_cache` when a
    /// cache read is given. Greedy decoding takes the most probable token of
    /// the mixture (cross-multiplied, so no division); sampling mixes the
    /// tempered and truncated model candidates with the cache distribution.
    pub fn sample_with_cache(
        &mut self,
        logits: &[i32],
        recent: &[u32],
        exp_table: &[u32],
        exp_step_log2: i32,
        cache: Option<&CacheRead<'_>>,
    ) -> Result<u32> {
        let Some(cache) = cache.filter(|c| c.gate_q31 > 0 && c.total > 0) else {
            return self.sample(logits, recent, exp_table, exp_step_log2);
        };
        let gate = u128::from(cache.gate_q31.min(1 << 31));
        let keep = (1u128 << 31) - gate;
        let mut cached: BTreeMap<u32, u128> = BTreeMap::new();
        for (token, weight) in cache.tokens.iter().zip(cache.weights) {
            *cached.entry(*token).or_default() += u128::from(*weight);
        }
        let cache_total = u128::from(cache.total);
        if self.settings.temperature_q8 == 0 {
            // p_model(w) = e_w / z with e_w = exp(l_w - max) in Q31, from the
            // presence-adjusted logits.
            self.adjust(logits, recent)?;
            let adjusted = &self.adjusted;
            let max = adjusted.iter().copied().max().unwrap_or(0);
            let weight = |v: i32| {
                u128::from(exp_neg(
                    i64::from(max) - i64::from(v),
                    RESIDUAL_EXP,
                    exp_table,
                    exp_step_log2,
                ))
            };
            let z: u128 = adjusted.iter().map(|&v| weight(v)).sum::<u128>().max(1);
            // Compare (1 - g) e_w C + g c_w z (the mixture times 2^31 z C).
            let best_model = argmax(adjusted) as u32;
            let score = |token: u32| -> u128 {
                let e = adjusted.get(token as usize).map_or(0, |&v| weight(v));
                keep * e * cache_total + gate * cached.get(&token).copied().unwrap_or(0) * z
            };
            let mut best = (score(best_model), best_model);
            for &token in cached.keys() {
                let s = score(token);
                if s > best.0 || (s == best.0 && token < best.1) {
                    best = (s, token);
                }
            }
            return Ok(best.1);
        }
        // Tempered, truncated model candidates, then the mixture.
        self.sample_candidates(logits, recent, exp_table, exp_step_log2)?;
        let model_total: u128 = self
            .candidates
            .iter()
            .map(|c| u128::from(c.1))
            .sum::<u128>()
            .max(1);
        let mut mixture: BTreeMap<u32, u128> = BTreeMap::new();
        for &(token, w) in &self.candidates {
            *mixture.entry(token).or_default() += keep * u128::from(w) * cache_total;
        }
        for (token, c) in &cached {
            *mixture.entry(*token).or_default() += gate * c * model_total;
        }
        let total: u128 = mixture.values().sum();
        let mut draw =
            (u128::from(self.next_u64()) << 64 | u128::from(self.next_u64())) % total.max(1);
        for (token, w) in &mixture {
            if draw < *w {
                return Ok(*token);
            }
            draw -= w;
        }
        Ok(*mixture.keys().next_back().unwrap_or(&0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp_table() -> Vec<u32> {
        (0..32 * 256 + 2)
            .map(|i| (2f64.powi(31) * (-(i as f64) / 256.0).exp()).round() as u32)
            .collect()
    }

    fn logits(nats: &[f64]) -> Vec<i32> {
        nats.iter().map(|v| (v * 65536.0).round() as i32).collect()
    }

    #[test]
    fn zero_temperature_is_greedy() -> Result<()> {
        let table = exp_table();
        let mut sampler = Sampler::new(SamplingSettings::default(), 1);
        let l = logits(&[0.5, 2.0, 1.9, -1.0]);
        for _ in 0..10 {
            assert_eq!(sampler.sample(&l, &[], &table, -8)?, 1);
        }
        Ok(())
    }

    #[test]
    fn frequencies_follow_the_softmax() -> Result<()> {
        let table = exp_table();
        let settings = SamplingSettings::from_decimal(1.0, 0, 1.0, 0.0)?;
        let mut sampler = Sampler::new(settings, 7);
        let nats = [0.0, 1.0, -0.5, 0.3];
        let l = logits(&nats);
        let draws = 40_000;
        let mut counts = [0usize; 4];
        for _ in 0..draws {
            counts[sampler.sample(&l, &[], &table, -8)? as usize] += 1;
        }
        let z: f64 = nats.iter().map(|v| v.exp()).sum();
        for (i, v) in nats.iter().enumerate() {
            let p = v.exp() / z;
            let f = counts[i] as f64 / draws as f64;
            assert!((f - p).abs() < 0.01, "token {i}: {f} vs {p}");
        }
        Ok(())
    }

    #[test]
    fn top_k_top_p_and_presence_restrict_the_support() -> Result<()> {
        let table = exp_table();
        let l = logits(&[3.0, 2.9, 0.0, -1.0, 2.95]);
        let mut top_k = Sampler::new(SamplingSettings::from_decimal(1.0, 2, 1.0, 0.0)?, 3);
        let mut top_p = Sampler::new(SamplingSettings::from_decimal(1.0, 0, 0.5, 0.0)?, 3);
        for _ in 0..2000 {
            let t = top_k.sample(&l, &[], &table, -8)?;
            assert!(t == 0 || t == 4, "top-k drew {t}");
            // 0 holds about 34% of the mass and 4 about 33%: top-p 0.5 keeps both.
            let t = top_p.sample(&l, &[], &table, -8)?;
            assert!(t == 0 || t == 4, "top-p drew {t}");
        }
        let settings = SamplingSettings::from_decimal(0.0, 0, 1.0, 1.0)?;
        let mut greedy = Sampler::new(settings, 1);
        // Token 0 was seen recently: a 1-nat penalty makes 4 the greedy choice.
        assert_eq!(greedy.sample(&l, &[0, 0, 3], &table, -8)?, 4);
        Ok(())
    }

    #[test]
    fn the_cache_mixture_moves_greedy_and_sampled_choices() -> Result<()> {
        let table = exp_table();
        // The model prefers token 0 (p about 0.66); the cache points at token 2.
        let l = logits(&[1.0, 0.0, -1.0]);
        let tokens = [2u32, 2, 1];
        let weights = [1u64 << 20, 1 << 20, 1 << 10];
        let read = |gate: f64| CacheRead {
            gate_q31: (gate * 2f64.powi(31)) as u64,
            tokens: &tokens,
            weights: &weights,
            total: weights.iter().sum(),
        };
        let mut greedy = Sampler::new(SamplingSettings::default(), 1);
        assert_eq!(
            greedy.sample_with_cache(&l, &[], &table, -8, Some(&read(0.1)))?,
            0
        );
        // At gate 0.8: 0.2 * 0.66 < 0.8 * 0.9995 for token 2.
        assert_eq!(
            greedy.sample_with_cache(&l, &[], &table, -8, Some(&read(0.8)))?,
            2
        );
        assert_eq!(greedy.sample_with_cache(&l, &[], &table, -8, None)?, 0);
        let mut sampler = Sampler::new(SamplingSettings::from_decimal(1.0, 0, 1.0, 0.0)?, 4);
        let draws = 20_000;
        let mut count = 0usize;
        for _ in 0..draws {
            if sampler.sample_with_cache(&l, &[], &table, -8, Some(&read(0.5)))? == 2 {
                count += 1;
            }
        }
        // p(2) = 0.5 * 0.0900 + 0.5 * 0.9995 = 0.545.
        let f = count as f64 / draws as f64;
        assert!((f - 0.545).abs() < 0.015, "token 2 drawn {f}");
        Ok(())
    }

    #[test]
    fn a_seed_reproduces_the_draws() -> Result<()> {
        let table = exp_table();
        let settings = SamplingSettings::from_decimal(0.8, 3, 0.9, 0.2)?;
        let l = logits(&[0.1, 0.2, 0.3, 0.05, 0.25]);
        let draw = |seed| -> Result<Vec<u32>> {
            let mut sampler = Sampler::new(settings, seed);
            (0..50)
                .map(|_| sampler.sample(&l, &[2], &table, -8))
                .collect()
        };
        assert_eq!(draw(9)?, draw(9)?);
        assert_ne!(draw(9)?, draw(10)?);
        Ok(())
    }
}
