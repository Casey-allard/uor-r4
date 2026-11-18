//! Resolve the complete written occurrence around a frozen content anchor, then its value.
use crate::{
    content_selector::{Choice, BOUNDARY, WIDTH},
    discrete_d11::spiralcore_orbit_roots,
    invalid, Result,
};
use serde::{Deserialize, Serialize};

pub const CAP: usize = 4096;
pub const NONWORD: u32 = 0;
pub const START: u32 = 1;
pub const CONT: u32 = 2;

// Length, up to three ordered packed connector roots, then the first value root.
pub type Relation = [u32; 5];

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub classes: Vec<u32>,
    pub relations: Vec<Relation>,
}
impl Bank {
    pub fn validate(&self) -> Result<()> {
        if self.classes.len() != 4096
            || self.classes.iter().any(|&v| v > CONT)
            || self.relations.is_empty()
            || self.relations.len() > CAP
        {
            return Err(invalid("invalid entity span bank"));
        }
        for r in &self.relations {
            let n = r[0] as usize;
            if !(1..=3).contains(&n)
                || r[1..=n].iter().any(|&v| v > 65535)
                || r[n + 1..4].iter().any(|&v| v != BOUNDARY)
                || r[4] > 65535
            {
                return Err(invalid("invalid connector prototype"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanChoice {
    pub query: Option<u32>,
    pub fact: Option<u32>,
    pub query_start: Option<u32>,
    pub query_end: Option<u32>,
    pub fact_start: Option<u32>,
    pub fact_end: Option<u32>,
    pub source: Option<u32>,
}
impl SpanChoice {
    pub fn words(self) -> [u32; 7] {
        [
            self.query,
            self.fact,
            self.query_start,
            self.query_end,
            self.fact_start,
            self.fact_end,
            self.source,
        ]
        .map(|p| p.unwrap_or(BOUNDARY))
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arms {
    pub span: SpanChoice,
    pub endpoint_only: SpanChoice,
    pub connector_only: SpanChoice,
}

fn bounds(ids: &[u32], classes: &[u32], at: usize) -> (usize, usize) {
    let class = |i: usize| classes[ids[i] as usize];
    let (mut l, mut r) = (at, at);
    if class(at) != NONWORD {
        while l > 0 && class(l) == CONT && class(l - 1) != NONWORD {
            l -= 1;
        }
        while r + 1 < ids.len() && class(r + 1) == CONT {
            r += 1;
        }
    }
    (l, r)
}

pub fn resolve(
    ids: &[u32],
    roots: &[u32],
    bank: &Bank,
    legacy_root: u32,
    anchor: Choice,
) -> Result<Arms> {
    bank.validate()?;
    if ids.is_empty()
        || ids.len() > WIDTH
        || roots.len() != 4096
        || legacy_root > 65535
        || ids
            .iter()
            .any(|&id| id >= 4096 || roots[id as usize] > 65535)
    {
        return Err(invalid("invalid span prompt or root table"));
    }
    let (q, f) = match (anchor.query, anchor.fact) {
        (None, None) => return Ok(Arms::default()),
        (Some(q), Some(f)) => (q as usize, f as usize),
        _ => return Err(invalid("partial span anchor")),
    };
    if q >= ids.len() || q.saturating_sub(f) < 2 || ids[f] != ids[q] {
        return Err(invalid("invalid frozen span anchor"));
    }
    let orbit = spiralcore_orbit_roots();
    let distance = |a: u32, b: u32| -> u32 {
        (0..4)
            .map(|k| {
                let x = &orbit[((a >> (4 * k)) & 15) as usize];
                let y = &orbit[((b >> (4 * k)) & 15) as usize];
                x.iter()
                    .zip(y)
                    .map(|(&i, &j)| u32::from(i.abs_diff(j)))
                    .sum::<u32>()
            })
            .sum()
    };
    let root = |i: usize| roots[ids[i] as usize];
    let choose = |end: usize, limit: usize| {
        let mut best: Option<(u32, u32)> = None;
        for r in &bank.relations {
            let n = r[0];
            let len = n as usize;
            let source = end + 1 + len;
            if source >= limit {
                continue;
            }
            let bridge = (0..len)
                .map(|j| distance(root(end + j + 1), r[j + 1]))
                .sum::<u32>();
            let cost = 4 * bridge + n * distance(root(source), r[4]);
            if best.is_none_or(|(prev, len)| {
                cost * len < prev * n || (cost * len == prev * n && n > len)
            }) {
                best = Some((cost, n));
            }
        }
        best.map(|(_, n)| end as u32 + 1 + n)
    };
    let make = |(ql, qr): (usize, usize), (fl, fr): (usize, usize), source| SpanChoice {
        query: anchor.query,
        fact: anchor.fact,
        query_start: Some(ql as u32),
        query_end: Some(qr as u32),
        fact_start: Some(fl as u32),
        fact_end: Some(fr as u32),
        source,
    };
    let (ql, qr) = bounds(ids, &bank.classes, q);
    let (fl, fr) = bounds(ids, &bank.classes, f);
    let matched = fr < ql && f - fl == q - ql && ids[fl..=fr] == ids[ql..=qr];
    let endpoint_source = if matched {
        // Frozen one-token relation: gap zero costs 64, gap one costs the root
        // distance, and every larger gap adds 64 per word to that latter cost.
        (fr + 1..ql)
            .take(2)
            .min_by_key(|&s| {
                if s == fr + 1 {
                    64
                } else {
                    distance(root(fr + 1), legacy_root)
                }
            })
            .map(|s| s as u32)
    } else {
        None
    };
    Ok(Arms {
        span: make(
            (ql, qr),
            (fl, fr),
            if matched { choose(fr, ql) } else { None },
        ),
        endpoint_only: make((ql, qr), (fl, fr), endpoint_source),
        connector_only: make((q, q), (f, f), choose(f, q)),
    })
}
