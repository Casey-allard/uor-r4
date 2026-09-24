//! Supervised finite relative-action learner. Exact payload identity is not a vector metric.
use super::hamilton_transport as ht;
#[derive(Clone, Debug)]
pub struct ActionExample {
    pub relation: usize,
    pub query: [i32; 4],
    pub keys: Vec<[i32; 4]>,
    pub target: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelativeActionModel {
    actions: Vec<u8>,
}
impl RelativeActionModel {
    pub fn fit(classes: usize, examples: &[ActionExample]) -> Result<Self, String> {
        if classes == 0 || classes > 256 || examples.is_empty() || examples.len() > 1_000_000 {
            return Err("invalid learning population".into());
        }
        let mut costs = vec![[0u64; 8]; classes];
        let mut observed = vec![false; classes];
        for ex in examples {
            if ex.relation >= classes
                || ex.keys.len() < 2
                || ex.keys.len() > 1024
                || ex.target >= ex.keys.len()
            {
                return Err("invalid supervised relative example".into());
            }
            observed[ex.relation] = true;
            for a in 0..8 {
                let correct = distance(ex.query, ht::apply(a as u8, ex.keys[ex.target])?);
                let mut other = u64::MAX;
                for (i, &k) in ex.keys.iter().enumerate() {
                    if i != ex.target {
                        other = other.min(distance(ex.query, ht::apply(a as u8, k)?));
                    }
                }
                let loss = (correct + 1).saturating_sub(other);
                costs[ex.relation][a] = costs[ex.relation][a]
                    .checked_add(loss)
                    .ok_or("relative objective overflow")?;
            }
        }
        if observed.iter().any(|x| !*x) {
            return Err("relation without training observations".into());
        }
        let actions = costs
            .iter()
            .map(|c| (0..8).min_by_key(|&a| (c[a], a)).unwrap_or(0) as u8)
            .collect();
        Ok(Self { actions })
    }
    pub fn act(&self, path: &[usize], key: [i32; 4]) -> Result<[i32; 4], String> {
        let mut v = key;
        for &r in path {
            v = ht::apply(*self.actions.get(r).ok_or("unknown relation")?, v)?;
        }
        Ok(v)
    }
    pub fn select(
        &self,
        path: &[usize],
        query: [i32; 4],
        keys: &[[i32; 4]],
    ) -> Result<usize, String> {
        if keys.is_empty() || keys.len() > 1024 {
            return Err("invalid candidate count".into());
        }
        let mut best = (u64::MAX, 0);
        for (i, &key) in keys.iter().enumerate() {
            let d = distance(query, self.act(path, key)?);
            if d < best.0 {
                best = (d, i);
            }
        }
        Ok(best.1)
    }
    pub fn actions(&self) -> &[u8] {
        &self.actions
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = b"Q8L1".to_vec();
        b.extend_from_slice(&(self.actions.len() as u16).to_le_bytes());
        b.extend(&self.actions);
        b
    }
    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() < 6 || &b[..4] != b"Q8L1" {
            return Err("invalid relative model".into());
        }
        let n = u16::from_le_bytes([b[4], b[5]]) as usize;
        if n == 0 || n > 256 || b.len() != 6 + n || b[6..].iter().any(|&a| a >= 8) {
            return Err("invalid relative actions".into());
        }
        Ok(Self {
            actions: b[6..].to_vec(),
        })
    }
}
fn distance(a: [i32; 4], b: [i32; 4]) -> u64 {
    a.iter()
        .zip(b)
        .map(|(&x, y)| (i64::from(x) - i64::from(y)).unsigned_abs())
        .sum()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn learns_relation_action_and_generalizes_composed_vectors() {
        let keys = vec![[1, 2, 4, 8], [-3, 1, 0, 2], [4, 8, 2, 1]];
        let ex: Vec<_> = (0..8)
            .flat_map(|r| {
                (0..3).map({
                    let keys = keys.clone();
                    move |t| ActionExample {
                        relation: r,
                        query: ht::apply(r as u8, keys[t]).unwrap(),
                        keys: keys.clone(),
                        target: t,
                    }
                })
            })
            .collect();
        let model = RelativeActionModel::fit(8, &ex).unwrap();
        let novel = vec![[21, 13, -5, 2], [8, -4, 15, -2], [-20, 1, 11, 3]];
        for a in 0..8 {
            for b in 0..8 {
                for t in 0..3 {
                    let query = ht::apply(b as u8, ht::apply(a as u8, novel[t]).unwrap()).unwrap();
                    assert_eq!(model.select(&[a, b], query, &novel).unwrap(), t);
                }
            }
        }
        assert_eq!(
            RelativeActionModel::from_bytes(&model.to_bytes()).unwrap(),
            model
        );
    }
    #[test]
    fn rejects_missing_classes_unknown_relation_and_bad_artifact() {
        assert!(RelativeActionModel::fit(1, &[]).is_err());
        assert!(RelativeActionModel::from_bytes(b"Q8L1\x01\x00\x09").is_err());
    }
}
