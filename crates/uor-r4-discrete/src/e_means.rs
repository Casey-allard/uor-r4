//! e-means: mini-batch k-means with exponential-moving-average centroids, in integers.
//!
//! Bessa, Mazaré, Szilvasy, Touvron, Picard, Douze and Jégou, "e-means:
//! outperforming k-means (in one-tenth the epochs)" (2026). Each cluster keeps
//! a count `N_j` and a sum `S_j`, updated as `N_j <- gamma N_j + (1-gamma) n_j`
//! and `S_j <- gamma S_j + (1-gamma) s_j`; the centroid is always read as
//! `S_j / N_j`, never stored separately. The step `alpha = 1 - gamma` is
//! cosine-annealed, a cluster whose count falls below a tenth of the mean is
//! revived from the farthest point, and the last epoch's exact per-cluster
//! means are the returned centroids.
//!
//! The solver is integer-only (D32): counts and sums are Q16, the cosine is a
//! 33-entry Q16 table with linear interpolation, and every distance is an
//! `i128` square. It changes the solver, not the table format, and the paper's
//! scaling laws are not adopted at this scale.
//!
//! Deterministic for a seed: the initialization, the shuffle and the batches
//! all draw from the module's xorshift.

/// Q16 fixed-point one.
pub const SCALE: i64 = 1 << 16;
/// Epochs a fit runs: four is the safe end of the paper's 1.4--2.6 range at
/// these sample sizes.
pub const EPOCHS: usize = 4;
/// Mini-batches an epoch. This is a fixed choice, not the paper's batch law.
pub const BATCHES: usize = 32;
/// The cosine schedule's peak step (`alpha_0`), in Q16.
pub const ALPHA0_Q16: i64 = 19661;

/// `cos(pi * i / 32)` in Q16, for the annealed step.
const COSINE_Q16: [i64; 33] = [
    65536, 65220, 64277, 62714, 60547, 57798, 54491, 50660, 46341, 41576, 36410, 30893, 25080,
    19024, 12785, 6424, 0, -6424, -12785, -19024, -25080, -30893, -36410, -41576, -46341, -50660,
    -54491, -57798, -60547, -62714, -64277, -65220, -65536,
];

fn next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// The annealed step at `step` of `total`, in Q16: `0.3 (0.1 + 0.45 (1 + cos))`.
fn alpha_q16(step: usize, total: usize) -> i64 {
    let total = total.max(1);
    let scaled = step.saturating_mul(32);
    let index = (scaled / total).min(32);
    let remainder = scaled % total;
    let low = COSINE_Q16[index];
    let high = COSINE_Q16[(index + 1).min(32)];
    let cosine = low + (high - low) * remainder as i64 / total as i64;
    // 0.1 + 0.45 (1 + cos) in Q16, then 0.3 of it.
    let inner = SCALE / 10 + 9 * (SCALE + cosine) / 20;
    (ALPHA0_Q16 * inner / SCALE).clamp(0, SCALE)
}

/// Squared Euclidean distance, exact in `i128`.
pub(crate) fn distance2(a: &[i64], b: &[i64]) -> u128 {
    a.iter()
        .zip(b)
        .map(|(x, y)| {
            let delta = (*x as i128 - *y as i128).unsigned_abs();
            delta * delta
        })
        .sum()
}

/// The index of the nearest centroid to `point`.
pub(crate) fn nearest(centroids: &[i64], dim: usize, point: &[i64]) -> usize {
    let mut best = 0usize;
    let mut best_distance = u128::MAX;
    for (code, centroid) in centroids.chunks_exact(dim).enumerate() {
        let distance = distance2(centroid, point);
        if distance < best_distance {
            best_distance = distance;
            best = code;
        }
    }
    best
}

/// Seeded k-means++ initialization: the first centroid is a uniform row and
/// each next one is drawn with probability proportional to its squared
/// distance from the nearest chosen centroid. `k` centroids of `dim` values.
pub fn init_centroids(points: &[i64], n: usize, dim: usize, k: usize, seed: u64) -> Vec<i64> {
    let mut state = seed | 1;
    let mut centroids = Vec::with_capacity(k * dim);
    let first = (next(&mut state) as usize) % n;
    centroids.extend_from_slice(&points[first * dim..(first + 1) * dim]);
    let mut closest = vec![u128::MAX; n];
    for _ in 1..k {
        let start = centroids.len() - dim;
        let chosen = &centroids[start..];
        let mut total = 0u128;
        for index in 0..n {
            let point = &points[index * dim..(index + 1) * dim];
            let distance = distance2(point, chosen).min(closest[index]);
            closest[index] = distance;
            total = total.saturating_add(distance);
        }
        // Uniform in [0, total) from the top 64 bits of the RNG draw.
        let target = if total == 0 {
            0
        } else {
            ((next(&mut state) as u128 * total) >> 64).min(total - 1)
        };
        let mut cumulative = 0u128;
        let mut index = n - 1;
        for (row, &weight) in closest.iter().enumerate() {
            cumulative = cumulative.saturating_add(weight);
            if cumulative > target {
                index = row;
                break;
            }
        }
        centroids.extend_from_slice(&points[index * dim..(index + 1) * dim]);
    }
    centroids
}

/// Fit `k` centroids of `dim` coordinates on `n` flat integer points, returning
/// `k * dim` coordinates in row-major order.
pub fn fit(
    points: &[i64],
    n: usize,
    dim: usize,
    k: usize,
    epochs: usize,
    seed: u64,
) -> Result<Vec<i64>, String> {
    if n == 0 || dim == 0 || k == 0 {
        return Err("e-means needs points, dimensions and clusters".to_owned());
    }
    if points.len() != n * dim {
        return Err("e-means points are n * dim".to_owned());
    }
    let mut state = seed | 1;
    let mut centroids = init_centroids(points, n, dim, k, seed);
    let mut counts = vec![SCALE; k];
    let mut sums = centroids
        .iter()
        .map(|&value| value.saturating_mul(SCALE))
        .collect::<Vec<i64>>();
    let mut order: Vec<usize> = (0..n).collect();
    let batches = BATCHES.min(n).max(1);
    let total = epochs.saturating_mul(batches).max(1);
    let mut step = 0usize;
    for _ in 0..epochs {
        for i in (1..n).rev() {
            let j = (next(&mut state) as usize) % (i + 1);
            order.swap(i, j);
        }
        for batch in 0..batches {
            let start = batch * n / batches;
            let end = (batch + 1) * n / batches;
            let alpha = alpha_q16(step, total) as i128;
            let gamma = SCALE as i128 - alpha;
            let mut batch_sums = vec![0i128; k * dim];
            let mut batch_counts = vec![0i64; k];
            for &index in &order[start..end] {
                let point = &points[index * dim..(index + 1) * dim];
                let code = nearest(&centroids, dim, point);
                batch_counts[code] += 1;
                for j in 0..dim {
                    batch_sums[code * dim + j] += point[j] as i128;
                }
            }
            // Every cluster updates, including one the batch did not touch: its
            // count decays, which is what makes the revival test reachable.
            for code in 0..k {
                counts[code] = ((gamma * counts[code] as i128
                    + alpha * batch_counts[code] as i128 * SCALE as i128)
                    >> 16) as i64;
                for j in 0..dim {
                    sums[code * dim + j] = ((gamma * sums[code * dim + j] as i128
                        + alpha * batch_sums[code * dim + j] * SCALE as i128)
                        >> 16) as i64;
                }
                if counts[code] > 0 {
                    for j in 0..dim {
                        centroids[code * dim + j] = sums[code * dim + j] / counts[code];
                    }
                }
            }
            step += 1;
            if step.is_multiple_of(5) {
                revive(&mut centroids, &mut counts, &mut sums, points, n, dim, k);
            }
        }
    }
    // Last-epoch mean: the exact mean of every point's final assignment.
    let mut final_sums = vec![0i128; k * dim];
    let mut final_counts = vec![0i64; k];
    for index in 0..n {
        let point = &points[index * dim..(index + 1) * dim];
        let code = nearest(&centroids, dim, point);
        final_counts[code] += 1;
        for j in 0..dim {
            final_sums[code * dim + j] += point[j] as i128;
        }
    }
    for code in 0..k {
        if final_counts[code] == 0 {
            continue;
        }
        let count = final_counts[code] as i128;
        for j in 0..dim {
            centroids[code * dim + j] = (final_sums[code * dim + j] / count) as i64;
        }
    }
    Ok(centroids)
}

/// The total squared distance from every point to its nearest centroid.
pub fn squared_error(points: &[i64], n: usize, dim: usize, centroids: &[i64]) -> u128 {
    (0..n)
        .map(|index| {
            let point = &points[index * dim..(index + 1) * dim];
            centroids
                .chunks_exact(dim)
                .map(|centroid| distance2(centroid, point))
                .min()
                .unwrap_or(0)
        })
        .sum()
}

/// Rebuild every cluster whose EMA count fell below a tenth of the mean from a
/// farthest point. Each call scores every point once against its nearest centroid
/// and hands the farthest distinct points to the dead clusters, so the cost is one
/// pass plus a sort rather than one scan per dead cluster.
fn revive(
    centroids: &mut [i64],
    counts: &mut [i64],
    sums: &mut [i64],
    points: &[i64],
    n: usize,
    dim: usize,
    k: usize,
) {
    let mean = counts.iter().map(|&count| count as i128).sum::<i128>() / k as i128;
    let threshold = mean / 10;
    let dead: Vec<usize> = (0..k)
        .filter(|&code| (counts[code] as i128) < threshold)
        .collect();
    if dead.is_empty() {
        return;
    }
    let mut scored: Vec<(u128, usize)> = (0..n)
        .map(|index| {
            let point = &points[index * dim..(index + 1) * dim];
            let distance = centroids
                .chunks_exact(dim)
                .map(|centroid| distance2(centroid, point))
                .min()
                .unwrap_or(0);
            (distance, index)
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    for (slot, &code) in dead.iter().enumerate() {
        let (_, index) = scored[slot % n];
        let source = index * dim;
        centroids[code * dim..(code + 1) * dim].copy_from_slice(&points[source..source + dim]);
        counts[code] = SCALE;
        for j in 0..dim {
            sums[code * dim + j] = centroids[code * dim + j].saturating_mul(SCALE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain integer Lloyd reference on the same initialization.
    fn lloyd(points: &[i64], n: usize, dim: usize, k: usize, iters: usize, seed: u64) -> Vec<i64> {
        let mut centroids = init_centroids(points, n, dim, k, seed);
        for _ in 0..iters {
            let mut sums = vec![0i128; k * dim];
            let mut counts = vec![0i64; k];
            for index in 0..n {
                let point = &points[index * dim..(index + 1) * dim];
                let code = nearest(&centroids, dim, point);
                counts[code] += 1;
                for j in 0..dim {
                    sums[code * dim + j] += point[j] as i128;
                }
            }
            for code in 0..k {
                if counts[code] == 0 {
                    continue;
                }
                let count = counts[code] as i128;
                for j in 0..dim {
                    centroids[code * dim + j] = (sums[code * dim + j] / count) as i64;
                }
            }
        }
        centroids
    }

    fn synthetic(k: usize, n: usize, dim: usize, seed: u64) -> Vec<i64> {
        let mut state = seed | 1;
        let centers: Vec<i64> = (0..k * dim)
            .map(|_| (next(&mut state) % 10_001) as i64 - 5_000)
            .collect();
        let mut points = Vec::with_capacity(n * dim);
        for index in 0..n {
            let center = &centers[(index % k) * dim..(index % k + 1) * dim];
            for &coordinate in center {
                points.push(coordinate + (next(&mut state) % 2_001) as i64 - 1_000);
            }
        }
        points
    }

    fn permille(part: u128, whole: u128) -> String {
        let value = part.saturating_mul(1_000) / whole.max(1);
        format!("{}.{:03}x", value / 1_000, value % 1_000)
    }

    #[test]
    fn e_means_reaches_lloyd_at_a_quarter_of_the_passes() {
        let (k, n, dim) = (16usize, 2048usize, 8usize);
        let points = synthetic(k, n, dim, 0xE3A5_0000_0001);
        let e = fit(&points, n, dim, k, EPOCHS, 7).expect("fit");
        let l = lloyd(&points, n, dim, k, 16, 7);
        let same = lloyd(&points, n, dim, k, EPOCHS, 7);
        let (e_error, l_error, same_error) = (
            squared_error(&points, n, dim, &e),
            squared_error(&points, n, dim, &l),
            squared_error(&points, n, dim, &same),
        );
        eprintln!(
            "e-means {EPOCHS} epochs vs Lloyd 16 passes: total squared error {e_error} vs {l_error} ({})",
            permille(e_error, l_error)
        );
        eprintln!(
            "e-means {EPOCHS} epochs vs Lloyd {EPOCHS} passes: total squared error {e_error} vs {same_error} ({})",
            permille(e_error, same_error)
        );
        assert!(
            e_error.saturating_mul(10) <= l_error.saturating_mul(11),
            "e-means is more than 10% worse than Lloyd"
        );
        assert_eq!(
            fit(&points, n, dim, k, EPOCHS, 7).expect("fit again"),
            e,
            "e-means is not deterministic for a seed"
        );
    }

    #[test]
    fn revival_rebuilds_dead_clusters_from_distinct_farthest_points() {
        // Two dead clusters share the stale centroid (0,0); the points are 0, 10
        // and 40, so the farthest is 40. Scoring once and handing out distinct
        // points separates the two revivals; a scan per dead cluster would give
        // both the same point.
        let points = vec![0i64, 0, 10, 10, 40, 40];
        let mut centroids = vec![0i64, 0, 10, 10, 0, 0];
        let mut counts = vec![0, SCALE, 0];
        let mut sums: Vec<i64> = centroids.iter().map(|&value| value * SCALE).collect();
        revive(&mut centroids, &mut counts, &mut sums, &points, 3, 2, 3);
        assert_eq!(counts[0], SCALE);
        assert_eq!(counts[2], SCALE);
        assert_eq!(&centroids[0..2], &[40, 40]);
        for code in [0usize, 2] {
            assert_eq!(sums[code * 2], centroids[code * 2] * SCALE);
            assert_eq!(sums[code * 2 + 1], centroids[code * 2 + 1] * SCALE);
            assert!(points
                .chunks_exact(2)
                .any(|point| point == &centroids[code * 2..code * 2 + 2]));
        }
        assert_ne!(
            &centroids[0..2],
            &centroids[4..6],
            "dead clusters must revive to distinct points"
        );
    }

    #[test]
    fn a_bad_shape_is_refused() {
        assert!(fit(&[0; 8], 4, 2, 2, EPOCHS, 1).is_ok());
        assert!(fit(&[0; 7], 4, 2, 2, EPOCHS, 1).is_err());
        assert!(fit(&[], 0, 2, 2, EPOCHS, 1).is_err());
    }
}
