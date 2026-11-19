//! Native serving reference. On a false action the caller passes None to the ordinary writer.
pub const NONE: u32 = u32::MAX;
pub type Key = [u32; 14];
pub type Context = [u32; 6];

pub struct Geometry<'a> {
    pub roots: &'a [u32],
    pub classes: &'a [u32],
    pub metric: [[u32; 16]; 16],
    pub requested: &'a [Context],
    pub excluded: &'a [Context],
}
impl Geometry<'_> {
    pub fn distance(&self, a: u32, b: u32) -> u32 {
        if a == b {
            0
        } else if a == NONE || b == NONE {
            64
        } else {
            (0..4)
                .map(|k| {
                    self.metric[((a >> (4 * k)) & 15) as usize][((b >> (4 * k)) & 15) as usize]
                })
                .sum()
        }
    }
    fn bounds(&self, ids: &[u32], at: usize) -> (usize, usize) {
        let (mut l, mut r) = (at, at);
        let class = |p: usize| self.classes[ids[p] as usize];
        if class(at) != 0 {
            while l > 0 && class(l) == 2 && class(l - 1) != 0 {
                l -= 1;
            }
            while r + 1 < ids.len() && class(r + 1) == 2 {
                r += 1;
            }
        }
        (l, r)
    }
    fn context(&self, ids: &[u32], (l, r): (usize, usize)) -> Context {
        std::array::from_fn(|i| {
            let at = if i < 3 {
                l.checked_sub(3 - i)
            } else {
                Some(r + 1 + i - 3)
            };
            at.and_then(|p| ids.get(p))
                .map(|&id| self.roots[id as usize])
                .unwrap_or(NONE)
        })
    }
    pub fn features(
        &self,
        ids: &[u32],
        query: Option<usize>,
        source: Option<usize>,
    ) -> Option<(Key, usize, usize)> {
        let q = query?;
        if source? >= ids.len()
            || q >= ids.len()
            || ids.iter().any(|&i| i >= 4096)
            || self.classes[ids[q] as usize] == 0
        {
            return None;
        }
        let chosen = self.bounds(ids, q);
        let near = |ctx: &Context, group: &[Context]| {
            group
                .iter()
                .map(|r| r.iter().zip(ctx).map(|(&a, &b)| self.distance(a, b)).sum())
                .min()
                .unwrap_or(NONE)
        };
        let words = (0..ids.len()).filter(|&i| self.classes[ids[i] as usize] == 1);
        let (_, nom) = words
            .clone()
            .map(|i| {
                let ctx = self.context(ids, self.bounds(ids, i));
                (
                    near(&ctx, self.requested) + 48u32.saturating_sub(near(&ctx, self.excluded)),
                    i,
                )
            })
            .min()?;
        let named = self.bounds(ids, nom);
        let mut k = [0; 14];
        k[..6].copy_from_slice(&self.context(ids, chosen));
        k[6..12].copy_from_slice(&self.context(ids, named));
        k[12] = words
            .take_while(|&i| i < nom)
            .filter(|&i| {
                let w = self.bounds(ids, i);
                w.1 < named.0 && ids[w.0..=w.1] == ids[named.0..=named.1]
            })
            .take(2)
            .count() as u32;
        k[13] = u32::from(chosen.0 == named.0);
        Some((k, nom, chosen.0))
    }
    pub fn nearest(&self, k: &Key, group: &[Key]) -> u32 {
        group
            .iter()
            .map(|p| {
                let context_cost: u32 = (0..12).map(|i| self.distance(k[i], p[i])).sum();
                context_cost + 64 * (k[12].abs_diff(p[12]) + k[13].abs_diff(p[13]))
            })
            .min()
            .unwrap_or(NONE)
    }
}
