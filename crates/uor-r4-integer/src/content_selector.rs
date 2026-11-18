//! Fitted question-role context and exact token identity with geometric relation replay.
use crate::{discrete_d11::spiralcore_orbit_roots, invalid, Result};
use serde::{Deserialize, Serialize};

pub const BOUNDARY: u32 = u32::MAX;
pub const WIDTH: usize = 192;
pub const BRIDGE: usize = 15;
const OFFSETS: [isize; 4] = [-2, -1, 1, 2];

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Prototypes {
    pub requested: Vec<[u32; 4]>,
    pub excluded: Vec<[u32; 4]>,
    pub fact: Vec<[u32; 4]>,
    /// Relation length followed by up to 15 root codes; unused words are BOUNDARY.
    pub relation: Vec<[u32; 16]>,
}

impl Prototypes {
    pub fn validate(&self) -> Result<()> {
        for ctx in [&self.requested, &self.excluded, &self.fact] {
            if ctx.is_empty()
                || ctx.len() > 288
                || ctx.iter().flatten().any(|&c| c != BOUNDARY && c > 65535)
            {
                return Err(invalid("invalid fitted entity context"));
            }
        }
        if self.relation.is_empty()
            || self.relation.len() > 288
            || self.relation.iter().any(|r| {
                let len = r[0] as usize;
                len > BRIDGE
                    || r[1..=len].iter().any(|&c| c > 65535)
                    || r[1 + len..].iter().any(|&c| c != BOUNDARY)
            })
        {
            return Err(invalid("invalid fitted subject-to-value relation"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Choice {
    pub query: Option<u32>,
    pub fact: Option<u32>,
    pub source: Option<u32>,
}

pub fn select(
    prefix: &[u32],
    roots: &[u32],
    model: &Prototypes,
    query_ablated: bool,
) -> Result<Choice> {
    Ok(rank(prefix, roots, model, query_ablated, None)?.0)
}

/// Rank exact-token pairs, optionally requiring the span resolver's identity
/// predicate before minimization. The score and its (query, fact) tie order
/// are invariant; the two site costs support a blind diagnostic trace.
pub fn rank(
    prefix: &[u32],
    roots: &[u32],
    model: &Prototypes,
    query_ablated: bool,
    classes: Option<&[u32]>,
) -> Result<(Choice, Vec<[u32; 2]>)> {
    use crate::span_selector::{bounds, matched, CONT};
    if classes.is_some_and(|c| c.len() != 4096 || c.iter().any(|&k| k > CONT)) {
        return Err(invalid("invalid entity classes"));
    }
    model.validate()?;
    if prefix.is_empty()
        || prefix.len() > WIDTH
        || roots.len() != 4096
        || prefix
            .iter()
            .any(|&id| id >= 4096 || roots[id as usize] > 65535)
    {
        return Err(invalid("invalid entity selector token or root"));
    }
    let codes: Vec<_> = prefix.iter().map(|&id| roots[id as usize]).collect();
    let orbit = spiralcore_orbit_roots();
    let mut lut = [[0u32; 16]; 16];
    for (i, a) in orbit.iter().enumerate() {
        for (j, b) in orbit.iter().enumerate() {
            lut[i][j] = a
                .iter()
                .zip(b)
                .map(|(&x, &y)| u32::from(x.abs_diff(y)))
                .sum();
        }
    }
    let distance = |a: u32, b: u32| -> u32 {
        if a == b {
            return 0;
        }
        if a == BOUNDARY || b == BOUNDARY {
            return 64;
        }
        (0..4)
            .map(|k| lut[((a >> (4 * k)) & 15) as usize][((b >> (4 * k)) & 15) as usize])
            .sum()
    };
    let mut scores = Vec::with_capacity(prefix.len());
    let mut best = None;
    for q in 0..prefix.len() {
        let site = OFFSETS.map(|offset| {
            q.checked_add_signed(offset)
                .and_then(|j| codes.get(j))
                .copied()
                .unwrap_or(BOUNDARY)
        });
        let near = |bank: &[[u32; 4]]| {
            bank.iter()
                .map(|p| {
                    site.iter()
                        .zip(p)
                        .map(|(&x, &y)| distance(x, y))
                        .sum::<u32>()
                })
                .min()
                .unwrap()
        };
        let fact_cost = near(&model.fact);
        let query_cost = if query_ablated {
            0
        } else {
            near(&model.requested) + 48u32.saturating_sub(near(&model.excluded))
        };
        scores.push([query_cost, fact_cost]);
        for f in 0..q.saturating_sub(1) {
            if prefix[f] == prefix[q] {
                if classes.is_some_and(|c| {
                    !matched(prefix, q, f, bounds(prefix, c, q), bounds(prefix, c, f))
                }) {
                    continue;
                }
                let candidate = (query_cost + scores[f][1], q, f);
                if best.is_none_or(|v| candidate < v) {
                    best = Some(candidate);
                }
            }
        }
    }
    let Some((_, q, f)) = best else {
        return Ok((Choice::default(), scores));
    };
    let source = (f + 1..q.min(f + BRIDGE + 2))
        .min_by_key(|&s| {
            let gap = s - f - 1;
            model
                .relation
                .iter()
                .map(|p| {
                    let len = p[0] as usize;
                    64 * gap.abs_diff(len) as u32
                        + (0..gap.min(len))
                            .map(|j| distance(codes[f + 1 + j], p[1 + j]))
                            .sum::<u32>()
                })
                .min()
                .unwrap()
        })
        .ok_or_else(|| invalid("empty relation domain"))?;
    Ok((
        Choice {
            query: Some(q as u32),
            fact: Some(f as u32),
            source: Some(source as u32),
        },
        scores,
    ))
}
