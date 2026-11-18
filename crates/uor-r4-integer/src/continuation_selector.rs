//! Advance a frozen WordStart source only with evidence of connector continuation.
use crate::{
    content_selector::{BOUNDARY, WIDTH},
    discrete_d11::spiralcore_orbit_roots,
    invalid,
    span_selector::{Bank, SpanChoice, NONWORD, START},
    value_selector::Decisions,
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Continuation {
    pub source: Option<u32>,
    /// 0 retains WordStart; 1 advances on exact IDs; 2 advances on orbit distance.
    pub reason: u32,
    /// Bit n-1 classifies the alphabetic candidate at fact_end+1+n.
    pub connector_mask: u32,
    pub onset_mask: u32,
}

impl Continuation {
    pub fn words(self) -> [u32; 4] {
        [
            self.source.unwrap_or(BOUNDARY),
            self.reason,
            self.connector_mask,
            self.onset_mask,
        ]
    }
}

fn distance(a: u32, b: u32) -> u32 {
    let orbit = spiralcore_orbit_roots();
    (0..4)
        .map(|k| {
            let p = &orbit[((a >> (4 * k)) & 15) as usize];
            let q = &orbit[((b >> (4 * k)) & 15) as usize];
            p.iter()
                .zip(q)
                .map(|(&x, &y)| u32::from(x.abs_diff(y)))
                .sum::<u32>()
        })
        .sum()
}

/// The caller obtains baseline via value_selector::decide on this same prefix.
pub fn decide(
    ids: &[u32],
    roots: &[u32],
    bank: &Bank,
    frozen: SpanChoice,
    baseline: Decisions,
) -> Result<Continuation> {
    bank.validate()?;
    if ids.is_empty()
        || ids.len() > WIDTH
        || roots.len() != 4096
        || ids
            .iter()
            .any(|&id| id >= 4096 || roots[id as usize] > 65535)
    {
        return Err(invalid("invalid continuation tape or codebook"));
    }
    let Some(start) = baseline.word_start else {
        return Ok(Continuation::default());
    };
    let end = frozen.fact_end.ok_or(invalid("missing frozen fact end"))? as usize;
    let limit = frozen
        .query_start
        .ok_or(invalid("missing frozen query start"))? as usize;
    let start = start as usize;
    if end >= limit
        || limit >= ids.len()
        || start >= limit
        || !(2..=4).contains(&start.saturating_sub(end))
        || baseline.word_start_mask & (1 << (start - end - 2)) == 0
        || bank.classes[ids[start] as usize] != START
    {
        return Err(invalid("invalid continuation source bounds"));
    }
    let mut result = Continuation {
        source: Some(start as u32),
        ..Default::default()
    };
    let first = end + 2;
    let stop = limit.min(first + 3);
    let bit = |i: usize| 1u32 << (i - first);
    for (i, &id) in ids[first..stop].iter().enumerate() {
        if bank.classes[id as usize] == NONWORD {
            continue;
        }
        let root = roots[id as usize];
        let mut connector = u32::MAX;
        let mut onset = u32::MAX;
        for r in &bank.relations {
            onset = onset.min(distance(root, r[4]));
            for &piece in &r[1..=r[0] as usize] {
                connector = connector.min(distance(root, piece));
            }
        }
        if connector < onset {
            result.connector_mask |= 1 << i;
        } else {
            result.onset_mask |= 1 << i;
        }
    }
    let registered = baseline.ordered_mask & baseline.word_start_mask;
    for i in (start + 1..stop).rev() {
        if registered & bit(i) != 0
            && ids[start..i]
                .iter()
                .all(|&id| bank.classes[id as usize] != NONWORD)
        {
            result.source = Some(i as u32);
            result.reason = 1;
            return Ok(result);
        }
    }
    for i in start..stop {
        if result.connector_mask & bit(i) != 0 {
            continue;
        }
        if i > start && result.onset_mask & baseline.word_start_mask & bit(i) != 0 {
            result.source = Some(i as u32);
            result.reason = 2;
        }
        break;
    }
    Ok(result)
}
