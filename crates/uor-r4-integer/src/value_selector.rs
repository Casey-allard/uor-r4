//! Source choices inside a frozen pair of entity spans.
use crate::{
    content_selector::{BOUNDARY, WIDTH},
    discrete_d11::spiralcore_orbit_roots,
    invalid,
    span_selector::{Bank, SpanChoice, CAP, START},
    Result,
};
use serde::{Deserialize, Serialize};

pub type Connector = [u32; 4];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExactBank {
    /// Connector length followed by its original token IDs. Remaining slots hold BOUNDARY.
    pub connectors: Vec<Connector>,
}
impl ExactBank {
    pub fn validate(&self) -> Result<()> {
        if self.connectors.is_empty() || self.connectors.len() > CAP {
            return Err(invalid("invalid exact connector count"));
        }
        for c in &self.connectors {
            let len = c[0] as usize;
            if !(1..=3).contains(&len)
                || c[1..=len].iter().any(|&id| id >= 4096)
                || c[1 + len..].iter().any(|&id| id != BOUNDARY)
            {
                return Err(invalid("invalid exact connector IDs"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Decisions {
    pub current: Option<u32>,
    pub word_start: Option<u32>,
    pub ordered: Option<u32>,
    pub both: Option<u32>,
    /// Bit n-1 denotes a length-n candidate strictly before the query span.
    pub eligible_mask: u32,
    pub word_start_mask: u32,
    pub ordered_mask: u32,
}
impl Decisions {
    pub fn words(self) -> [u32; 7] {
        [
            self.current.unwrap_or(BOUNDARY),
            self.word_start.unwrap_or(BOUNDARY),
            self.ordered.unwrap_or(BOUNDARY),
            self.both.unwrap_or(BOUNDARY),
            self.eligible_mask,
            self.word_start_mask,
            self.ordered_mask,
        ]
    }
}

pub fn decide(
    ids: &[u32],
    roots: &[u32],
    span_bank: &Bank,
    exact: &ExactBank,
    frozen: SpanChoice,
) -> Result<Decisions> {
    let Some(frozen_source) = frozen.source else {
        validate_input(ids, roots, span_bank, exact)?;
        return Ok(Decisions::default());
    };
    let end = frozen.fact_end.ok_or(invalid("missing fact endpoint"))? as usize;
    let limit = frozen
        .query_start
        .ok_or(invalid("missing query endpoint"))? as usize;
    let out = decide_before(ids, roots, span_bank, exact, end, limit)?;
    if out.current != Some(frozen_source) {
        return Err(invalid("frozen span source differed"));
    }
    Ok(out)
}

pub(crate) fn validate_input(
    ids: &[u32],
    roots: &[u32],
    span_bank: &Bank,
    exact: &ExactBank,
) -> Result<()> {
    span_bank.validate()?;
    exact.validate()?;
    if ids.is_empty()
        || ids.len() > WIDTH
        || roots.len() != 4096
        || ids
            .iter()
            .any(|&id| id >= 4096 || roots[id as usize] > 65535)
    {
        return Err(invalid("invalid value selector prefix or roots"));
    }
    Ok(())
}

pub(crate) fn decide_before(
    ids: &[u32],
    roots: &[u32],
    span_bank: &Bank,
    exact: &ExactBank,
    end: usize,
    limit: usize,
) -> Result<Decisions> {
    validate_input(ids, roots, span_bank, exact)?;
    if end >= limit || limit >= ids.len() {
        return Err(invalid("invalid value search bounds"));
    }
    let orbit = spiralcore_orbit_roots();
    let distance = |a: u32, b: u32| {
        (0..4)
            .map(|k| {
                let x = &orbit[((a >> (4 * k)) & 15) as usize];
                let y = &orbit[((b >> (4 * k)) & 15) as usize];
                x.iter()
                    .zip(y)
                    .map(|(&x, &y)| u32::from(x.abs_diff(y)))
                    .sum::<u32>()
            })
            .sum::<u32>()
    };
    let mut out = Decisions::default();
    let mut winners = [None; 2];
    for r in &span_bank.relations {
        let len = r[0];
        let source = end + 1 + len as usize;
        if source >= limit {
            continue;
        }
        let bit = 1 << (len - 1);
        out.eligible_mask |= bit;
        let start = span_bank.classes[ids[source] as usize] == START;
        if start {
            out.word_start_mask |= bit;
        }
        let mut cost = len * distance(roots[ids[source] as usize], r[4]);
        for j in 0..len as usize {
            cost += 4 * distance(roots[ids[end + 1 + j] as usize], r[1 + j]);
        }
        for best in winners.iter_mut().take(if start { 2 } else { 1 }) {
            if best.is_none_or(|(prev, n)| {
                cost * n < prev * len || (cost * n == prev * len && len > n)
            }) {
                *best = Some((cost, len));
            }
        }
    }
    let address = |len: u32| end as u32 + 1 + len;
    out.current = winners[0].map(|(_, len)| address(len));
    out.word_start = winners[1].map(|(_, len)| address(len));
    for c in &exact.connectors {
        let len = c[0] as usize;
        let bit = 1 << (len - 1);
        if out.eligible_mask & bit != 0 && ids[end + 1..end + 1 + len] == c[1..=len] {
            out.ordered_mask |= bit;
        }
    }
    let longest = |mask: u32| {
        (1..=3)
            .rev()
            .find(|&n| mask & (1 << (n - 1)) != 0)
            .map(address)
    };
    out.ordered = longest(out.ordered_mask);
    out.both = longest(out.ordered_mask & out.word_start_mask);
    Ok(out)
}
