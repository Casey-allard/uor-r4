/// Code Pick over one causal window `w` (`t = w.len() - 1`): the latest `p`
/// in `1..t` minimizing `d(p) = sum_b popcount(codes[w[p-1]][b] ^
/// codes[w[t-1]][b]) + popcount(codes[w[p]][b] ^ codes[w[t]][b])`. Returns
/// `(p, d(p), ties)`, `ties` being the number of `p` at the minimum, as
/// `pick_read` counts it; `None` when `t < 2`. The copied token is `w[p + 1]`.
/// Every id in `w` must be below `codes.len()`.
pub fn code_pick<const B: usize>(codes: &[[u16; B]], w: &[u16]) -> Option<(usize, u32, u32)> {
    if w.len() < 3 {
        return None;
    }
    let t = w.len() - 1;
    let q1 = &codes[usize::from(w[t - 1])];
    let q0 = &codes[usize::from(w[t])];
    let mut pick = 1usize;
    let mut best = u32::MAX;
    let mut ties = 0u32;
    for p in 1..t {
        let c1 = &codes[usize::from(w[p - 1])];
        let c0 = &codes[usize::from(w[p])];
        let mut d = 0u32;
        for b in 0..B {
            d += (c1[b] ^ q1[b]).count_ones();
            d += (c0[b] ^ q0[b]).count_ones();
        }
        if d < best {
            ties = 0;
        }
        if d <= best {
            pick = p;
            best = d;
            ties += 1;
        }
    }
    Some((pick, best, ties))
}
