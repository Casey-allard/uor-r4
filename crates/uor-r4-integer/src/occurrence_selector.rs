//! Integer occurrence selector. The fitted rule consists of two unsigned offsets.
//! A candidate p is scored using only the orbit roots of its historical anchor
//! token p-d and the query anchor at n-1-q. Smaller source index breaks ties.
#![forbid(unsafe_code)]
use crate::{discrete_d11::spiralcore_orbit_roots, invalid, Result};

pub fn select(
    prefix: &[u32],
    packed_roots: &[u32],
    lookback: u8,
    anchor_back: u8,
) -> Result<Option<u32>> {
    if !(1..=6).contains(&lookback) || anchor_back >= 32 || prefix.len() > 192 {
        return Err(invalid(
            "occurrence selector offset or context exceeds its fitted domain",
        ));
    }
    if packed_roots.len() != 4096
        || prefix
            .iter()
            .any(|&id| id >= 4096 || packed_roots[id as usize] > 65535)
    {
        return Err(invalid(
            "occurrence selector needs four packed roots per token",
        ));
    }
    let Some(anchor) = prefix.len().checked_sub(usize::from(anchor_back) + 1) else {
        return Ok(None);
    };
    let query = packed_roots[prefix[anchor] as usize];
    let roots = spiralcore_orbit_roots();
    let d = usize::from(lookback);
    let mut best = None;
    let mut min = u32::MAX;
    for p in d..anchor {
        let key = packed_roots[prefix[p - d] as usize];
        let mut distance = 0;
        for b in 0..4 {
            let a = &roots[((key >> (4 * b)) & 15) as usize];
            let z = &roots[((query >> (4 * b)) & 15) as usize];
            distance += a
                .iter()
                .zip(z)
                .map(|(&x, &y)| u32::from(x.abs_diff(y)))
                .sum::<u32>();
        }
        if distance < min {
            min = distance;
            best = Some(p as u32);
        }
    }
    Ok(best)
}
