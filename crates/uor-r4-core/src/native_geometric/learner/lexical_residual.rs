//! Sparse local prior and conditional lexical residual. Fit/compilation may use floats;
//! the bounded numerical scoring path uses integer lookup/add/subtract/shift only.
#![forbid(unsafe_code)]
use std::collections::BTreeMap;
pub const FRAC: u32 = 12;
pub const ONE: i32 = 1 << FRAC;
pub const NEG: i32 = -(1 << 28);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogAdd {
    table: Vec<u16>,
}
impl LogAdd {
    pub fn compile() -> Self {
        Self {
            table: (0..=16 * ONE)
                .map(|d| {
                    ((1.0 + 2f64.powf(-(d as f64) / ONE as f64)).log2() * ONE as f64).round() as u16
                })
                .collect(),
        }
    }
    pub fn add(&self, a: i32, b: i32) -> i32 {
        if a == NEG {
            return b;
        }
        if b == NEG {
            return a;
        }
        let hi = a.max(b);
        let d = (i64::from(a) - i64::from(b)).unsigned_abs() as usize;
        hi + self.table.get(d).copied().unwrap_or(0) as i32
    }
    pub fn sum(&self, x: &[i32]) -> i32 {
        if x.is_empty() {
            return NEG;
        }
        let mut s = x.to_vec();
        let mut n = s.len();
        while n > 1 {
            for j in 0..n.div_ceil(2) {
                s[j] = if (j << 1) + 1 < n {
                    self.add(s[j << 1], s[(j << 1) + 1])
                } else {
                    s[j << 1]
                };
            }
            n = n.div_ceil(2);
        }
        s[0]
    }
}
fn qlog(x: f64) -> i32 {
    if x <= 0.0 {
        NEG
    } else {
        (x.log2() * ONE as f64).round() as i32
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    key: u32,
    values: Vec<(u16, i32)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerPrior {
    pub vocab: usize,
    pub binding: [u8; 32],
    u: Vec<i32>,
    one: Vec<Row>,
    two: Vec<Row>,
    mix: [i32; 4],
    pub math: LogAdd,
}
impl IntegerPrior {
    pub fn compile(
        vocab: usize,
        triples: &[(u32, u32, u32)],
        lambda: (f64, f64),
        binding: [u8; 32],
    ) -> Result<Self, String> {
        if vocab == 0
            || vocab > 4096
            || triples.is_empty()
            || ![lambda.0, lambda.1]
                .iter()
                .all(|x| x.is_finite() && (0.0..1.0).contains(x))
        {
            return Err("invalid prior dimensions or interpolation".into());
        }
        let mut u = vec![0u64; vocab];
        let mut one = BTreeMap::<u32, BTreeMap<u16, u64>>::new();
        let mut two = one.clone();
        for &(p, c, t) in triples {
            if [p, c, t].iter().any(|x| *x as usize >= vocab) {
                return Err("out-of-vocabulary fit record".into());
            }
            u[t as usize] += 1;
            *one.entry(c).or_default().entry(t as u16).or_default() += 1;
            *two.entry((p << 12) | c)
                .or_default()
                .entry(t as u16)
                .or_default() += 1;
        }
        let rows = |m: BTreeMap<u32, BTreeMap<u16, u64>>| {
            m.into_iter()
                .map(|(key, r)| {
                    let n = r.values().sum::<u64>() as f64;
                    Row {
                        key,
                        values: r
                            .into_iter()
                            .map(|(t, k)| (t, qlog(k as f64 / n)))
                            .collect(),
                    }
                })
                .collect()
        };
        let total = triples.len() as f64 + vocab as f64;
        Ok(Self {
            vocab,
            binding,
            u: u.into_iter()
                .map(|n| qlog((n as f64 + 1.0) / total))
                .collect(),
            one: rows(one),
            two: rows(two),
            mix: [
                qlog(lambda.0),
                qlog(1.0 - lambda.0),
                qlog(lambda.1),
                qlog(1.0 - lambda.1),
            ],
            math: LogAdd::compile(),
        })
    }
    pub fn unigram(&self) -> &[i32] {
        &self.u
    }
    pub fn count_entries(&self) -> usize {
        self.one
            .iter()
            .chain(&self.two)
            .map(|r| r.values.len())
            .sum()
    }
    pub fn score(&self, prev: u32, cur: u32) -> Result<Vec<i32>, String> {
        let mut out = vec![NEG; self.vocab];
        self.score_into(prev, cur, &mut out)?;
        Ok(out)
    }
    pub fn score_into(&self, prev: u32, cur: u32, out: &mut [i32]) -> Result<(), String> {
        if out.len() != self.vocab || prev as usize >= self.vocab || cur as usize >= self.vocab {
            return Err("invalid scoring context/shape".into());
        }

        let r1 = find_row(&self.one, cur);
        let r2 = find_row(&self.two, (prev << 12) | cur);
        let (mut i, mut j) = (0, 0);
        for (t, o) in out.iter_mut().enumerate() {
            let u = self.u[t];
            let lookup = |r: Option<&Row>, k: &mut usize| match r {
                None => u,
                Some(r) => {
                    if *k < r.values.len() && r.values[*k].0 as usize == t {
                        let v = r.values[*k].1;
                        *k += 1;
                        v
                    } else {
                        NEG
                    }
                }
            };
            let p1 = lookup(r1, &mut i);
            let p2 = lookup(r2, &mut j);
            let back = self
                .math
                .add(weighted(p1, self.mix[0]), weighted(u, self.mix[1]));
            *o = self
                .math
                .add(weighted(p2, self.mix[2]), weighted(back, self.mix[3]));
        }
        Ok(())
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = b"IPL1".to_vec();
        b.extend_from_slice(&(self.vocab as u32).to_le_bytes());
        b.extend_from_slice(&self.binding);
        for v in self.mix.iter().chain(&self.u) {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for v in &self.math.table {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for level in [&self.one, &self.two] {
            b.extend_from_slice(&(level.len() as u32).to_le_bytes());
            for r in level {
                b.extend_from_slice(&r.key.to_le_bytes());
                b.extend_from_slice(&(r.values.len() as u32).to_le_bytes());
                for (t, p) in &r.values {
                    b.extend_from_slice(&t.to_le_bytes());
                    b.extend_from_slice(&p.to_le_bytes());
                }
            }
        }
        b
    }
    pub fn from_bytes(b: &[u8], expected: [u8; 32]) -> Result<Self, String> {
        if b.len() > 64 * 1024 * 1024 || b.get(..4) != Some(b"IPL1") {
            return Err("invalid prior header/size".into());
        }
        let mut c = Cursor { b, p: 4 };
        let v = c.u32()? as usize;
        if v == 0 || v > 4096 {
            return Err("prior vocabulary outside bound".into());
        }
        let binding: c_binding::Binding = c.take(32)?.try_into().map_err(|_| "binding")?;
        if binding != expected {
            return Err("foreign prior binding".into());
        }
        let mut mix = [0; 4];
        for x in &mut mix {
            *x = c.log()?;
        }
        let mut u = Vec::with_capacity(v);
        for _ in 0..v {
            u.push(c.log()?);
        }
        let mut table = Vec::with_capacity(16 * ONE as usize + 1);
        for _ in 0..=16 * ONE {
            let x = c.u16()?;
            if x > ONE as u16 {
                return Err("invalid log table".into());
            }
            table.push(x);
        }
        if table[0] != ONE as u16
            || !table.windows(2).all(|x| x[0] >= x[1])
            || table.last() != Some(&0)
        {
            return Err("invalid monotone log table".into());
        }
        let mut levels = Vec::new();
        for depth in 0..2 {
            let n = c.u32()? as usize;
            if n > 1000000 || n > c.remaining() / 8 {
                return Err("oversized sparse level".into());
            }
            let mut rows = Vec::with_capacity(n);
            for _ in 0..n {
                let key = c.u32()?;
                if (depth == 0 && key as usize >= v)
                    || (depth == 1 && ((key >> 12) as usize >= v || (key & 4095) as usize >= v))
                {
                    return Err("invalid context key".into());
                }
                let len = c.u32()? as usize;
                if len == 0 || len > v || len > c.remaining() / 6 {
                    return Err("invalid sparse row length".into());
                }
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    let t = c.u16()?;
                    let p = c.log()?;
                    if t as usize >= v || p == NEG {
                        return Err("invalid sparse token/probability".into());
                    }
                    values.push((t, p));
                }
                if !values.windows(2).all(|x| x[0].0 < x[1].0) {
                    return Err("unsorted or duplicate token".into());
                }
                rows.push(Row { key, values });
            }
            if !rows.windows(2).all(|x| x[0].key < x[1].key) {
                return Err("unsorted contexts".into());
            }
            levels.push(rows);
        }
        if c.remaining() != 0 {
            return Err("trailing prior bytes".into());
        }
        let two = levels.pop().ok_or("missing pair level")?;
        let one = levels.pop().ok_or("missing single level")?;
        Ok(Self {
            vocab: v,
            binding,
            u,
            one,
            two,
            mix,
            math: LogAdd { table },
        })
    }
}
mod c_binding {
    pub type Binding = [u8; 32];
}
fn weighted(x: i32, w: i32) -> i32 {
    if x == NEG || w == NEG {
        NEG
    } else {
        x + w
    }
}
struct Cursor<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.p.checked_add(n).ok_or("prior size overflow")?;
        let s = self.b.get(self.p..end).ok_or("truncated prior")?;
        self.p = end;
        Ok(s)
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| "u16")?,
        ))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| "u32")?,
        ))
    }
    fn log(&mut self) -> Result<i32, String> {
        let x = self.u32()? as i32;
        if x != NEG && !(-256 * ONE..=0).contains(&x) {
            return Err("invalid fixed log code".into());
        }
        Ok(x)
    }
    fn remaining(&self) -> usize {
        self.b.len() - self.p
    }
}
fn scale(x: i32, n: u8) -> Result<i32, String> {
    let mut out = 0i64;
    let mut a = i64::from(x);
    let mut k = n;
    while k != 0 {
        if k & 1 != 0 {
            out += a;
        }
        a <<= 1;
        k >>= 1;
    }
    i32::try_from(out >> 5).map_err(|_| "coefficient overflow".into())
}
/// Restore the original Generate-group log mass; Copy and Stop rows are unchanged.
/// alpha and gamma are unsigned numerators with denominator 32, selected offline.
pub fn compose(
    native: &[i32],
    count: &[i32],
    score_shift: u32,
    alpha: u8,
    gamma: u8,
    math: &LogAdd,
) -> Result<Vec<i32>, String> {
    compose_in_frame(native, count, None, score_shift, alpha, gamma, math)
}
fn compose_in_frame(
    native: &[i32],
    count: &[i32],
    unigram: Option<&[i32]>,
    score_shift: u32,
    alpha: u8,
    gamma: u8,
    math: &LogAdd,
) -> Result<Vec<i32>, String> {
    let v = count.len();
    if v == 0 || v > 4096 || native.len() != v + 2 || score_shift > 20 || alpha > 64 || gamma > 32 {
        return Err("invalid composite dimensions/coefficients".into());
    }
    let mut z = Vec::with_capacity(native.len());
    for &x in native {
        let q = if score_shift <= FRAC {
            i64::from(x) << (FRAC - score_shift)
        } else {
            i64::from(x) >> (score_shift - FRAC)
        };
        if q.abs() > (1 << 25) {
            return Err("native score outside composite envelope".into());
        }
        z.push(q as i32);
    }
    if count.iter().any(|x| !(-256 * ONE..=ONE).contains(x)) {
        return Err("count score outside envelope".into());
    }
    if let Some(u) = unigram {
        if u.len() != v || u.iter().any(|x| !(-256 * ONE..=0).contains(x)) {
            return Err("invalid unigram correction shape or bounds".into());
        }
    }
    let original = math.sum(&z[..v]);
    let mut r = Vec::with_capacity(v);
    for i in 0..v {
        let correction = z[i] - unigram.map(|u| u[i]).unwrap_or(0);
        r.push(scale(count[i], alpha)? + scale(correction, gamma)?);
    }
    let total = math.sum(&r);
    for i in 0..v {
        z[i] = r[i] - total + original;
    }
    Ok(z)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_equal_mass_and_absent_mass() {
        let t = LogAdd::compile();
        assert_eq!(t.add(0, 0), ONE);
        assert_eq!(t.add(NEG, -3 * ONE), -3 * ONE);
        assert_eq!(t.sum(&vec![0; 4096]), 12 * ONE);
    }
    #[test]
    fn integer_logadd_agrees_with_wide_reference() {
        let t = LogAdd::compile();
        for d in (0..24 * ONE).step_by(19) {
            let exact =
                ((1.0 + 2f64.powf(-(d as f64) / ONE as f64)).log2() * ONE as f64).round() as i32;
            assert!((t.add(0, -d) - exact).abs() <= 1);
        }
    }
    #[test]
    fn sparse_prior_matches_independent_counts() {
        let p = IntegerPrior::compile(4, &[(0, 1, 2), (0, 1, 2), (3, 1, 0)], (0.7, 0.4), [9; 32])
            .unwrap();
        let s = p.score(0, 1).unwrap();
        for t in 0..4 {
            let u = [2., 1., 3., 1.][t] / 7.;
            let p1 = [1., 0., 2., 0.][t] / 3.;
            let p2 = if t == 2 { 1. } else { 0. };
            let exact: f64 = 0.4 * p2 + 0.6 * (0.7 * p1 + 0.3 * u);
            assert!((s[t] as f64 / ONE as f64 - exact.log2()).abs() < 0.001);
        }
    }
    #[test]
    fn serialization_rejects_foreign_truncated_and_trailing() {
        let p = IntegerPrior::compile(4, &[(0, 1, 2)], (0.7, 0.4), [9; 32]).unwrap();
        let b = p.to_bytes();
        assert_eq!(IntegerPrior::from_bytes(&b, [9; 32]).unwrap(), p);
        assert!(IntegerPrior::from_bytes(&b, [0; 32]).is_err());
        assert!(IntegerPrior::from_bytes(&b[..b.len() - 1], [9; 32]).is_err());
        let mut tail = b;
        tail.push(0);
        assert!(IntegerPrior::from_bytes(&tail, [9; 32]).is_err());
    }
    #[test]
    fn copied_and_stop_logits_and_generate_mass_are_preserved() {
        let t = LogAdd::compile();
        let n = [3, 7, 9, -13];
        let c = [qlog(0.2), qlog(0.8)];
        let y = compose(&n, &c, 4, 32, 5, &t).unwrap();
        assert_eq!(&y[2..], &[9 << 8, -13 << 8]);
        assert_eq!(t.sum(&y[..2]), t.sum(&[3 << 8, 7 << 8]));
        assert_eq!(compose(&n, &c, 4, 0, 32, &t).unwrap(), n.map(|x| x << 8));
    }
    #[test]
    fn invalid_context_and_shift_are_errors() {
        let p = IntegerPrior::compile(4, &[(0, 1, 2)], (0.7, 0.4), [9; 32]).unwrap();
        assert!(p.score(4, 0).is_err());
        assert!(compose(&[0; 6], &[0; 4], 31, 32, 5, &p.math).is_err());
        assert!(IntegerPrior::compile(4, &[(0, 1, 4)], (0.7, 0.4), [9; 32]).is_err());
    }
}

fn find_row(level: &[Row], key: u32) -> Option<&Row> {
    level
        .binary_search_by_key(&key, |r| r.key)
        .ok()
        .map(|i| &level[i])
}

/// Marginal-relative correction, with native Generate mass retained independently.
pub fn compose_ratio(
    native: &[i32],
    count: &[i32],
    unigram: &[i32],
    score_shift: u32,
    alpha: u8,
    gamma: u8,
    math: &LogAdd,
) -> Result<Vec<i32>, String> {
    compose_in_frame(
        native,
        count,
        Some(unigram),
        score_shift,
        alpha,
        gamma,
        math,
    )
}
#[cfg(test)]
#[test]
fn marginal_relative_correction_does_not_repeat_unigram_and_preserves_action_mass() {
    let math = LogAdd::compile();
    let u = [-ONE, -3 * ONE];
    let c = [qlog(0.3), qlog(0.7)];
    let native = [u[0], u[1], -4 * ONE, -5 * ONE];
    let result = compose_ratio(&native, &c, &u, FRAC, 32, 8, &math).unwrap();
    let count_only = compose(&native, &c, FRAC, 32, 0, &math).unwrap();
    assert_eq!(result, count_only);
    assert_eq!(&result[2..], &native[2..]);
    assert_eq!(math.sum(&result[..2]), math.sum(&native[..2]));
    assert!(compose_ratio(&native, &c, &u[..1], FRAC, 32, 8, &math).is_err());
}
