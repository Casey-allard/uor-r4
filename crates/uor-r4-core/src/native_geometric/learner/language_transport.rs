//! Language-conditioned finite relative addressing. Exact ownership is never inferred from a vector distance.
use super::hamilton_transport as ht;
use std::collections::BTreeSet;
#[derive(Clone)]
pub struct QueryExample {
    pub tokens: Vec<u32>,
    pub query: [i32; 4],
    pub keys: Vec<[i32; 4]>,
    pub target: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageTransport {
    vocab: usize,
    tokenizer: [u8; 32],
    bias: [i8; 8],
    features: Vec<(u64, [i8; 8])>,
}
#[derive(Clone, Copy, Debug)]
pub struct OwnedKey {
    pub owner: u64,
    pub occurrence: u64,
    pub vector: [i32; 4],
}
fn visit(tokens: &[u32], mut f: impl FnMut(u64)) {
    for &t in tokens {
        f(u64::from(t));
    }
    for pair in tokens.windows(2) {
        f((1u64 << 63) | (u64::from(pair[0]) << 32) | u64::from(pair[1]));
    }
}
fn distance(a: [i32; 4], b: [i32; 4]) -> u64 {
    a.iter()
        .zip(b)
        .map(|(&a, b)| (i64::from(a) - i64::from(b)).unsigned_abs())
        .sum()
}
fn best(scores: &[i32; 8]) -> usize {
    let mut b = 0;
    for i in 1..8 {
        if scores[i] > scores[b] {
            b = i
        }
    }
    b
}
impl LanguageTransport {
    pub fn fit(
        vocab: usize,
        tokenizer: [u8; 32],
        examples: &[QueryExample],
        epochs: usize,
    ) -> Result<Self, String> {
        if vocab < 2
            || vocab > 65536
            || examples.is_empty()
            || examples.len() > 65536
            || epochs == 0
            || epochs > 512
        {
            return Err("invalid language-action fitting bounds".into());
        }
        let mut features = BTreeSet::new();
        let mut labels = Vec::new();
        for ex in examples {
            if ex.tokens.is_empty()
                || ex.tokens.len() > 256
                || ex.tokens.iter().any(|&t| t as usize >= vocab)
                || ex.keys.len() < 2
                || ex.keys.len() > 1024
                || ex.target >= ex.keys.len()
            {
                return Err("invalid query supervision".into());
            }
            visit(&ex.tokens, |k| {
                features.insert(k);
            });
            let mut label = 0;
            let mut score = u64::MAX;
            for a in 0..8 {
                let d = distance(ex.query, ht::apply(a, ex.keys[ex.target])?);
                if d < score {
                    score = d;
                    label = a as usize;
                }
            }
            labels.push(label);
        }
        if features.len() > 65536 {
            return Err("language feature bound exceeded".into());
        }
        let mut m = Self {
            vocab,
            tokenizer,
            bias: [0; 8],
            features: features.into_iter().map(|x| (x, [0; 8])).collect(),
        };
        for _ in 0..epochs {
            let mut mistakes = 0;
            for (ex, &label) in examples.iter().zip(&labels) {
                let predicted = best(&m.scores(&ex.tokens)?);
                if predicted != label {
                    mistakes += 1;
                    m.bias[label] = (m.bias[label] + 1).min(7);
                    m.bias[predicted] = (m.bias[predicted] - 1).max(-7);
                    visit(&ex.tokens, |k| {
                        if let Ok(i) = m.features.binary_search_by_key(&k, |x| x.0) {
                            m.features[i].1[label] = (m.features[i].1[label] + 1).min(7);
                            m.features[i].1[predicted] = (m.features[i].1[predicted] - 1).max(-7);
                        }
                    });
                }
            }
            if mistakes == 0 {
                break;
            }
        }
        Ok(m)
    }
    pub fn scores(&self, tokens: &[u32]) -> Result<[i32; 8], String> {
        if tokens.is_empty()
            || tokens.len() > 256
            || tokens.iter().any(|&t| t as usize >= self.vocab)
        {
            return Err("query token bounds".into());
        }
        let mut scores = self.bias.map(i32::from);
        let mut seen = false;
        visit(tokens, |k| {
            if let Ok(i) = self.features.binary_search_by_key(&k, |x| x.0) {
                seen = true;
                for (a, s) in scores.iter_mut().enumerate() {
                    *s += i32::from(self.features[i].1[a]);
                }
            }
        });
        if !seen {
            return Err("query has no fitted features".into());
        }
        Ok(scores)
    }
    pub fn action(&self, tokens: &[u32]) -> Result<u8, String> {
        Ok(best(&self.scores(tokens)?) as u8)
    }
    pub fn select_owned(
        &self,
        tokens: &[u32],
        query: [i32; 4],
        owner: u64,
        keys: &[OwnedKey],
    ) -> Result<usize, String> {
        let action = self.action(tokens)?;
        Self::select_action(action, query, owner, keys)
    }
    pub fn select_action(
        action: u8,
        query: [i32; 4],
        owner: u64,
        keys: &[OwnedKey],
    ) -> Result<usize, String> {
        if action >= 8 || keys.is_empty() || keys.len() > 1024 {
            return Err("candidate bounds".into());
        }
        let mut choice = None;
        let mut nearest = u64::MAX;
        let mut tie = false;
        for (i, key) in keys.iter().enumerate() {
            if key.owner != owner {
                continue;
            }
            let d = distance(query, ht::apply(action, key.vector)?);
            if d < nearest {
                nearest = d;
                choice = Some(i);
                tie = false;
            } else if d == nearest {
                tie = true;
            }
        }
        if tie {
            return Err("ambiguous owned source".into());
        }
        if choice.is_some() && nearest != 0 {
            return Err("requested owned relation absent".into());
        }
        choice.ok_or_else(|| "requested owner absent".into())
    }
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = b"LQT1".to_vec();
        out.extend_from_slice(&(self.vocab as u32).to_le_bytes());
        out.extend_from_slice(&(self.features.len() as u32).to_le_bytes());
        out.extend(self.tokenizer);
        out.extend(self.bias.map(|x| x as u8));
        for (k, w) in &self.features {
            out.extend(k.to_le_bytes());
            out.extend(w.map(|x| x as u8));
        }
        out
    }
    pub fn from_bytes(b: &[u8], tokenizer: [u8; 32]) -> Result<Self, String> {
        if b.len() < 52 || b.len() > 2_000_000 || &b[..4] != b"LQT1" {
            return Err("invalid language-action envelope".into());
        }
        let read = |i| -> Result<u32, String> {
            Ok(u32::from_le_bytes(
                b[i..i + 4].try_into().map_err(|_| "header")?,
            ))
        };
        let vocab = read(4)? as usize;
        let n = read(8)? as usize;
        if vocab < 2
            || vocab > 65536
            || n == 0
            || n > 65536
            || b.len() != 52 + 16 * n
            || b[12..44] != tokenizer
        {
            return Err("language-action identity/dimensions".into());
        }
        let bias: [i8; 8] = std::array::from_fn(|i| b[44 + i] as i8);
        let mut features = Vec::with_capacity(n);
        for c in b[52..].chunks_exact(16) {
            let k = u64::from_le_bytes(c[..8].try_into().map_err(|_| "feature key")?);
            let weights: [i8; 8] = std::array::from_fn(|i| c[8 + i] as i8);
            if weights.iter().any(|x| !(-7..=7).contains(x))
                || features.last().is_some_and(|(prev, _)| *prev >= k)
            {
                return Err("invalid language feature order/weights".into());
            }
            let lo = k as u32 as usize;
            let hi = ((k & !(1u64 << 63)) >> 32) as usize;
            if lo >= vocab || (k >> 63 == 0 && hi != 0) || (k >> 63 != 0 && hi >= vocab) {
                return Err("invalid language feature token".into());
            }
            features.push((k, weights));
        }
        if bias.iter().any(|x| !(-7..=7).contains(x)) {
            return Err("invalid language bias".into());
        }
        Ok(Self {
            vocab,
            tokenizer,
            bias,
            features,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn model() -> LanguageTransport {
        let q = [1, 2, 4, 8];
        let keys = (0..8).map(|a| ht::apply(a, q).unwrap()).collect::<Vec<_>>();
        let ex = (0..8)
            .map(|a| QueryExample {
                tokens: vec![100, a + 1, 101],
                query: ht::apply(a as u8, q).unwrap(),
                keys: keys.clone(),
                target: 0,
            })
            .collect::<Vec<_>>();
        LanguageTransport::fit(128, [2; 32], &ex, 32).unwrap()
    }
    #[test]
    fn language_is_learned_from_selected_key_not_given_relation_id() {
        let m = model();
        for a in 0..8 {
            assert_eq!(m.action(&[a + 1]).unwrap(), a as u8);
        }
    }
    #[test]
    fn ownership_absence_and_ambiguity_do_not_fall_back() {
        let m = model();
        let k = OwnedKey {
            owner: 7,
            occurrence: 3,
            vector: [1, 2, 4, 8],
        };
        assert_eq!(m.select_owned(&[1], k.vector, 7, &[k]).unwrap(), 0);
        assert!(m.select_owned(&[1], k.vector, 8, &[k]).is_err());
        assert!(m.select_owned(&[1], k.vector, 7, &[k, k]).is_err());
        let mut foreign = k;
        foreign.owner = 8;
        foreign.vector = [0; 4];
        assert_eq!(m.select_owned(&[1], k.vector, 7, &[foreign, k]).unwrap(), 1);
    }
    #[test]
    fn model_roundtrip_rejects_foreign_and_malformed_data() {
        let m = model();
        let b = m.to_bytes();
        assert_eq!(LanguageTransport::from_bytes(&b, [2; 32]).unwrap(), m);
        assert!(LanguageTransport::from_bytes(&b, [3; 32]).is_err());
        assert!(LanguageTransport::from_bytes(&b[..b.len() - 1], [2; 32]).is_err());
        let mut bad = b.clone();
        bad[44] = 127;
        assert!(LanguageTransport::from_bytes(&bad, [2; 32]).is_err());
        assert!(m.action(&[]).is_err());
        assert!(m.action(&[128]).is_err());
        assert!(m.action(&[110]).is_err());
    }
}

#[cfg(test)]
#[test]
fn missing_owned_relation_does_not_select_an_approximate_neighbour() {
    let key = OwnedKey {
        owner: 1,
        occurrence: 9,
        vector: [1, 2, 4, 8],
    };
    let query = ht::apply(2, key.vector).unwrap();
    assert!(LanguageTransport::select_action(0, query, 1, &[key]).is_err());
}
