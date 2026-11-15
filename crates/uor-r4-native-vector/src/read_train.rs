//! Native Q31 pick-margin update for the first-pick read's gates.
//!
//! The loss of a row with a target pick is the L1 margin hinge
//! `max(0, d(q, k*) - d(q, k_r) + m)`, where `d` is the read's own address
//! distance, `k*` the target row's key and `k_r` the nearest other earlier key
//! (a tie goes to the latest row, as in the read; `m` is at least 1 when the
//! rival is later than the target). The subgradient of each distance term is
//! a sign per address word. Each gate's credit is that sign residual times the
//! conjugate block, the rule of [`crate::q31_hamilton_train_step`] at `2^30`
//! per residual unit, computed exactly in `i128` and summed there; the
//! candidate gate is `sat(gate - (credit >> (shift + 1)))` and is kept only
//! when the batch loss strictly drops. The weights and the gain are not
//! changed. No float, no squashing and no dense product.

use crate::read::{read_distance, ADDRESS_LANES};
use crate::{
    q31_read_address, q31_saturate, Error, Q31FirstPickRead, Q31Quaternion, Result,
    MAX_Q31_STEP_SHIFT,
};

/// The largest shifted credit applied to one gate word, before saturation.
const CREDIT_CLAMP: i128 = 1 << 32;

fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}

/// One update step of the read's gates over one batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q31ReadTrainStep {
    /// The read after the step: the candidate when accepted, else unchanged.
    pub read: Q31FirstPickRead,
    /// Whether the candidate lowered the batch loss and was kept.
    pub accepted: bool,
    /// Batch hinge loss of the read before the step.
    pub loss_before: u128,
    /// Batch hinge loss of the candidate.
    pub loss_after: u128,
    /// The read's picks before the step, from [`Q31FirstPickRead::apply`].
    pub picks_before: Vec<u32>,
    /// The kept read's picks, from [`Q31FirstPickRead::apply`].
    pub picks_after: Vec<u32>,
    /// `rivals[row]` is `r + 1` for the rival row `r` before the step, or `0`
    /// when the row has no target or no other earlier row.
    pub rivals: Vec<u32>,
}

fn sign(value: i64) -> i64 {
    value.signum()
}

type Address = [i64; ADDRESS_LANES];

/// Per-row query and key addresses of one batch under one read.
fn addresses(read: &Q31FirstPickRead, stream: &[i64]) -> Result<(Vec<Address>, Vec<Address>)> {
    let width = read.width();
    let mut queries = Vec::with_capacity(stream.len() / width);
    let mut keys = Vec::with_capacity(stream.len() / width);
    for row in stream.chunks_exact(width) {
        queries.push(q31_read_address(read.query_gates(), row)?);
        keys.push(q31_read_address(read.key_gates(), row)?);
    }
    Ok((queries, keys))
}

/// The nearest earlier row other than `target`; a tie goes to the latest row.
fn rival(
    query: &[i64; ADDRESS_LANES],
    keys: &[[i64; ADDRESS_LANES]],
    target: usize,
) -> Option<usize> {
    let mut best = None;
    let mut best_distance = u64::MAX;
    for (s, key) in keys.iter().enumerate() {
        if s == target {
            continue;
        }
        let distance = read_distance(query, key);
        if distance <= best_distance {
            best_distance = distance;
            best = Some(s);
        }
    }
    best
}

/// One row's hinge: `max(0, d(q, k*) + m - d(q, k_r))`.
fn hinge(
    query: &[i64; ADDRESS_LANES],
    target: &[i64; ADDRESS_LANES],
    rival: &[i64; ADDRESS_LANES],
    margin: u64,
) -> u128 {
    let gap = i128::from(read_distance(query, target)) + i128::from(margin)
        - i128::from(read_distance(query, rival));
    // `gap > 0` here, so it fits `u128`.
    if gap > 0 {
        gap as u128
    } else {
        0
    }
}

/// The margin one row needs: at least 1 when the rival is later than the
/// target, because serving sends an equal distance to the later row.
fn row_margin(margin: u64, target: usize, rival: usize) -> u64 {
    if rival > target {
        margin.max(1)
    } else {
        margin
    }
}

/// The batch hinge loss, and each row's rival as `r + 1` (or `0`).
fn batch_loss(
    queries: &[[i64; ADDRESS_LANES]],
    keys: &[[i64; ADDRESS_LANES]],
    targets: &[u32],
    batch: usize,
    time: usize,
    margin: u64,
) -> (u128, Vec<u32>) {
    let mut total = 0u128;
    let mut rivals = vec![0u32; targets.len()];
    for window in 0..batch {
        let first = window * time;
        for t in 1..time {
            let row = first + t;
            if targets[row] == 0 {
                continue;
            }
            let target = targets[row] as usize - 1;
            if let Some(r) = rival(&queries[row], &keys[first..row], target) {
                // `r < t < 2^30`, so `r + 1` fits `u32`.
                rivals[row] = (r + 1) as u32;
                total += hinge(
                    &queries[row],
                    &keys[first + target],
                    &keys[first + r],
                    row_margin(margin, target, r),
                );
            }
        }
    }
    (total, rivals)
}

/// The largest left shift that keeps the largest stream word at most `2^30`;
/// `0` when that word is already above `2^30` (up to the domain's `2^31`).
///
/// A gate change `delta` moves an address word by about `delta * |x| / 2^31`,
/// and the credit itself scales with `|x|`, so an unscaled step shrinks with
/// `|x|^2`. Shifting the rows into the top of the domain before the conjugate
/// product keeps the step's effect on the address independent of the stream's
/// scale; it is one exact power of two per batch.
pub fn q31_read_credit_shift(stream: &[i64]) -> u32 {
    let largest = stream
        .iter()
        .map(|word| word.unsigned_abs())
        .max()
        .unwrap_or(0);
    let mut shift = 0;
    while largest != 0 && shift < 31 && largest << (shift + 1) <= 1 << 30 {
        shift += 1;
    }
    shift
}

/// The exact Hamilton product `a (x) conj(b)` in `i128`.
///
/// The same component formula as [`crate::q31_hamilton_rotate`], without its
/// Q31 rounding and wrapping `u64` sum: a residual of two units on four
/// aligned lanes against a block at `2^30` reaches `2^63` there and wraps.
fn conjugate_product(a: [i128; 4], b: [i128; 4]) -> [i128; 4] {
    let c = [b[0], -b[1], -b[2], -b[3]];
    [
        a[0] * c[0] - a[1] * c[1] - a[2] * c[2] - a[3] * c[3],
        a[0] * c[1] + a[1] * c[0] + a[2] * c[3] - a[3] * c[2],
        a[0] * c[2] - a[1] * c[3] + a[2] * c[0] + a[3] * c[1],
        a[0] * c[3] + a[1] * c[2] - a[2] * c[1] + a[3] * c[0],
    ]
}

/// Add `residual (x) conj(block << shift)` per gate into `credits`, exactly.
///
/// The residual is in sign units (`-2..=2` per lane); the step later divides
/// the summed credit by `2^(step_shift + 1)`, the residual unit `2^30` over
/// the product's `2^31`.
fn add_credit(
    credits: &mut [[i128; 4]],
    row: &[i64],
    residual: &[i64; ADDRESS_LANES],
    shift: u32,
) -> Result<()> {
    for (j, (credit, block)) in credits.iter_mut().zip(row.chunks_exact(4)).enumerate() {
        let base = (4 * j) % ADDRESS_LANES;
        let lanes: [i128; 4] = std::array::from_fn(|k| i128::from(residual[base + k]));
        if lanes == [0; 4] {
            continue;
        }
        let v: [i128; 4] = std::array::from_fn(|k| i128::from(block[k]) << shift);
        for (sum, word) in credit.iter_mut().zip(conjugate_product(lanes, v)) {
            *sum += word;
        }
    }
    Ok(())
}

fn apply_credit(
    gates: &[Q31Quaternion],
    credits: &[[i128; 4]],
    step_shift: u32,
) -> Vec<Q31Quaternion> {
    gates
        .iter()
        .zip(credits)
        .map(|(gate, credit)| {
            std::array::from_fn(|k| {
                let step =
                    (credit[k] >> (step_shift + 1)).clamp(-CREDIT_CLAMP, CREDIT_CLAMP) as i64;
                q31_saturate(gate[k] - step)
            })
        })
        .collect()
}

impl Q31FirstPickRead {
    /// One pick-margin update of the query and key gates over `batch` windows.
    ///
    /// `stream`, `ids` and `table` are as in [`Self::apply`]. Each row's
    /// target, `targets[b * time + t]`, is `s + 1` for the target pick `s < t`,
    /// or `0` for a row with no target; row 0 of a window takes `0`. `margin`
    /// is in address words. Any refusal returns an error and no step.
    #[allow(clippy::too_many_arguments)]
    pub fn train_pick_step(
        &self,
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        batch: usize,
        time: usize,
        targets: &[u32],
        margin: u64,
        step_shift: u32,
    ) -> Result<Q31ReadTrainStep> {
        // `apply` refuses every shape and word that the read itself refuses.
        let picks_before = self.apply(stream, ids, table, batch, time)?.picks;
        if targets.len() != picks_before.len() {
            return Err(invalid(
                "native Q31 read training needs batch * time targets",
            ));
        }
        if step_shift > MAX_Q31_STEP_SHIFT {
            return Err(invalid(
                "native Q31 read training step shift must be in 0..=31",
            ));
        }
        for (row, &target) in targets.iter().enumerate() {
            if target as usize > row % time {
                return Err(invalid(
                    "native Q31 read training target must be an earlier row of its window",
                ));
            }
        }

        let width = self.width();
        let (queries, keys) = addresses(self, stream)?;
        let (loss_before, rivals) = batch_loss(&queries, &keys, targets, batch, time, margin);
        let credit_shift = q31_read_credit_shift(stream);

        let mut query_credits = vec![[0i128; 4]; width / 4];
        let mut key_credits = vec![[0i128; 4]; width / 4];
        for window in 0..batch {
            let first = window * time;
            for t in 1..time {
                let row = first + t;
                if rivals[row] == 0 {
                    continue;
                }
                let a = first + targets[row] as usize - 1;
                let b = first + rivals[row] as usize - 1;
                let m = row_margin(margin, a - first, b - first);
                if hinge(&queries[row], &keys[a], &keys[b], m) == 0 {
                    continue;
                }
                let mut on_query = [0i64; ADDRESS_LANES];
                let mut on_target = [0i64; ADDRESS_LANES];
                let mut on_rival = [0i64; ADDRESS_LANES];
                for l in 0..ADDRESS_LANES {
                    let to_target = sign(queries[row][l] - keys[a][l]);
                    let to_rival = sign(queries[row][l] - keys[b][l]);
                    on_query[l] = to_target - to_rival;
                    on_target[l] = -to_target;
                    on_rival[l] = to_rival;
                }
                let rows = |r: usize| &stream[r * width..(r + 1) * width];
                add_credit(&mut query_credits, rows(row), &on_query, credit_shift)?;
                add_credit(&mut key_credits, rows(a), &on_target, credit_shift)?;
                add_credit(&mut key_credits, rows(b), &on_rival, credit_shift)?;
            }
        }

        let candidate = Q31FirstPickRead::new(
            apply_credit(self.query_gates(), &query_credits, step_shift),
            apply_credit(self.key_gates(), &key_credits, step_shift),
            self.weights().to_vec(),
            self.gain_shift(),
        )?;
        let (candidate_queries, candidate_keys) = addresses(&candidate, stream)?;
        let (loss_after, _) = batch_loss(
            &candidate_queries,
            &candidate_keys,
            targets,
            batch,
            time,
            margin,
        );
        let accepted = loss_after < loss_before;
        let read = if accepted { candidate } else { self.clone() };
        let picks_after = read.apply(stream, ids, table, batch, time)?.picks;
        Ok(Q31ReadTrainStep {
            read,
            accepted,
            loss_before,
            loss_after,
            picks_before,
            picks_after,
            rivals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: i64 = 1 << 31;
    const IDENTITY: Q31Quaternion = [ONE, 0, 0, 0];
    /// The served residual's scale at the read (claude-390): about 2^18.
    const S: i64 = 1 << 18;

    fn identity_read() -> Q31FirstPickRead {
        Q31FirstPickRead::new(vec![IDENTITY; 4], vec![IDENTITY; 4], vec![0; 16], 4).unwrap()
    }

    fn row(lane0: i64, lane1: i64) -> Vec<i64> {
        let mut row = vec![0; 16];
        row[0] = lane0;
        row[1] = lane1;
        row
    }

    fn refusal<T: std::fmt::Debug>(result: Result<T>) -> String {
        match result {
            Err(Error::Invalid(message)) => message,
            Ok(value) => panic!("expected a refusal, got {value:?}"),
        }
    }

    /// Row 2's nearest earlier key is row 0's; its target is row 1.
    fn fixture(scale: i64) -> (Vec<i64>, Vec<u32>, Vec<i64>) {
        let stream = [row(scale, 0), row(0, scale), row(3 * scale / 4, scale / 2)].concat();
        (stream, vec![0, 1, 2], vec![0; 3 * 16])
    }

    /// Train until a step is rejected; returns the read and the accepted step count.
    fn train(
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        targets: &[u32],
        shift: u32,
    ) -> (Q31FirstPickRead, usize) {
        let time = ids.len();
        let mut read = identity_read();
        for steps in 0..64 {
            let step = read
                .train_pick_step(stream, ids, table, 1, time, targets, 0, shift)
                .unwrap();
            if !step.accepted {
                assert_eq!(step.read, read, "a rejected step changes nothing");
                return (read, steps);
            }
            assert!(step.loss_after < step.loss_before);
            read = step.read;
        }
        panic!("training did not settle in 64 steps");
    }

    #[test]
    fn a_wrong_pick_becomes_right_and_serving_replays_it() {
        let (stream, ids, table) = fixture(S);
        let targets = [0, 0, 2];
        let before = identity_read().apply(&stream, &ids, &table, 1, 3).unwrap();
        assert_eq!(before.picks, vec![0, 1, 1], "row 2 starts on the wrong row");

        let (read, steps) = train(&stream, &ids, &table, &targets, 4);
        assert_eq!(steps, 4);
        // Four more bits of step shift take about sixteen times the steps.
        assert_eq!(train(&stream, &ids, &table, &targets, 8).1, 49);
        assert_ne!(read.query_gates(), identity_read().query_gates());
        assert_eq!(read.weights(), identity_read().weights());
        assert_eq!(read.gain_shift(), 4);

        let last = read
            .train_pick_step(&stream, &ids, &table, 1, 3, &targets, 0, 4)
            .unwrap();
        assert_eq!(last.loss_before, 0);
        assert_eq!(last.picks_after, vec![0, 1, 2]);

        // Serving replays the trained picks: batch, incremental, and from bytes.
        assert_eq!(
            read.apply(&stream, &ids, &table, 1, 3).unwrap().picks,
            last.picks_after
        );
        let restored = Q31FirstPickRead::from_bytes(&read.to_bytes()).unwrap();
        assert_eq!(restored, read);
        let mut history = crate::Q31ReadHistory::new();
        let stepped: Vec<u32> = stream
            .chunks_exact(16)
            .zip(&ids)
            .map(|(x, &id)| restored.step(&mut history, x, id, &table).unwrap().0)
            .collect();
        assert_eq!(stepped, last.picks_after);
    }

    #[test]
    fn the_step_count_does_not_depend_on_the_stream_scale() {
        for scale in [1 << 10, S, 1 << 28] {
            let (stream, ids, table) = fixture(scale);
            let (read, steps) = train(&stream, &ids, &table, &[0, 0, 2], 4);
            assert_eq!(steps, 4, "scale {scale}");
            assert_eq!(
                read.apply(&stream, &ids, &table, 1, 3).unwrap().picks,
                vec![0, 1, 2]
            );
        }
    }

    #[test]
    fn the_credit_shift_lifts_the_largest_word_to_at_most_2_30() {
        assert_eq!(q31_read_credit_shift(&[]), 0);
        assert_eq!(q31_read_credit_shift(&[0, 0]), 0);
        assert_eq!(q31_read_credit_shift(&[S, -3]), 12);
        assert_eq!(q31_read_credit_shift(&[-(1 << 29)]), 1);
        assert_eq!(q31_read_credit_shift(&[1 << 30]), 0);
        assert_eq!(q31_read_credit_shift(&[ONE]), 0);
        assert_eq!(q31_read_credit_shift(&[1]), 30);
    }

    #[test]
    fn rows_without_a_target_add_nothing() {
        let (stream, ids, table) = fixture(S);
        let read = identity_read();
        let idle = read
            .train_pick_step(&stream, &ids, &table, 1, 3, &[0, 0, 0], 0, 4)
            .unwrap();
        assert_eq!((idle.loss_before, idle.accepted), (0, false));
        assert_eq!(idle.read, read);
        assert_eq!(idle.rivals, vec![0, 0, 0]);

        // A fourth row with no target leaves the step as it was on three rows.
        let four = [stream.clone(), row(0, S)].concat();
        let with_idle_row = read
            .train_pick_step(&four, &[0, 1, 2, 0], &table, 1, 4, &[0, 0, 2, 0], 0, 4)
            .unwrap();
        let three = read
            .train_pick_step(&stream, &ids, &table, 1, 3, &[0, 0, 2], 0, 4)
            .unwrap();
        assert!(three.accepted);
        assert_eq!(with_idle_row.read, three.read);
    }

    #[test]
    fn a_margin_widens_a_right_pick_that_sits_too_close() {
        // Row 2 already picks row 0 (distance S * 3/4 against S * 5/4).
        let (stream, ids, table) = fixture(S);
        let read = identity_read();
        let at_zero = read
            .train_pick_step(&stream, &ids, &table, 1, 3, &[0, 0, 1], 0, 4)
            .unwrap();
        assert_eq!((at_zero.loss_before, at_zero.accepted), (0, false));

        let margin = S as u64;
        let step = read
            .train_pick_step(&stream, &ids, &table, 1, 3, &[0, 0, 1], margin, 4)
            .unwrap();
        assert_eq!(step.loss_before, margin as u128 - S as u128 / 2);
        assert!(step.accepted && step.loss_after < step.loss_before);
        assert_eq!(step.picks_after, vec![0, 1, 1]);
    }

    #[test]
    fn four_aligned_credit_terms_keep_their_sign() {
        // Each block `[S; 4]` lifts to `2^30`; a residual of +2 on every lane
        // makes four aligned terms of the conjugate product, which must not
        // wrap: word 0 of every gate's credit is `4 * 2 * 2^30 = 2^33`.
        let block = vec![S; 16];
        let shift = q31_read_credit_shift(&block);
        assert_eq!(S << shift, 1 << 30);
        for (residual, expected) in [(2i64, 1i128 << 33), (-2, -(1i128 << 33))] {
            let mut credits = vec![[0i128; 4]; 4];
            add_credit(&mut credits, &block, &[residual; ADDRESS_LANES], shift).unwrap();
            for credit in &credits {
                assert_eq!(credit[0], expected, "residual {residual}: {credit:?}");
            }
        }
        // A word of 2^31 takes shift 0; the credit stays exact there too.
        let full = vec![ONE; 16];
        let mut credits = vec![[0i128; 4]; 4];
        add_credit(
            &mut credits,
            &full,
            &[2; ADDRESS_LANES],
            q31_read_credit_shift(&full),
        )
        .unwrap();
        assert_eq!(credits[0][0], 1i128 << 34);
    }

    #[test]
    fn a_margin_zero_tie_with_a_later_rival_is_not_a_zero_loss() {
        // Rows 0 and 1 are at equal distance from row 2; serving sends the tie
        // to row 1, so a target of row 0 is wrong until row 0 is strictly nearer.
        let stream = [row(S, 0), row(0, S), row(S / 2, S / 2)].concat();
        let (ids, table) = (vec![0, 1, 2], vec![0; 3 * 16]);
        let read = identity_read();
        assert_eq!(
            read.apply(&stream, &ids, &table, 1, 3).unwrap().picks,
            vec![0, 1, 2]
        );
        let step = read
            .train_pick_step(&stream, &ids, &table, 1, 3, &[0, 0, 1], 0, 4)
            .unwrap();
        assert_eq!(step.loss_before, 1);
        let (read, _) = train(&stream, &ids, &table, &[0, 0, 1], 4);
        assert_eq!(
            read.apply(&stream, &ids, &table, 1, 3).unwrap().picks,
            vec![0, 1, 1]
        );
    }

    #[test]
    fn a_tie_sends_the_rival_to_the_latest_row() {
        let stream = [row(S, 0), row(S, 0), row(0, S), row(3 * S / 4, S / 2)].concat();
        let step = identity_read()
            .train_pick_step(
                &stream,
                &[0, 1, 2, 3],
                &vec![0; 4 * 16],
                1,
                4,
                &[0, 0, 0, 3],
                0,
                4,
            )
            .unwrap();
        assert_eq!(step.rivals, vec![0, 0, 0, 2]);
    }

    #[test]
    fn an_overshooting_candidate_is_rejected() {
        // Two windows whose rows sit in swapped order; at shift 0 the summed
        // credit rotates past the target and the batch loss rises.
        let (window_a, _, table) = fixture(S);
        let window_b = [row(0, S), row(S, 0), row(S / 2, S / 2)].concat();
        let stream = [window_a, window_b].concat();
        let ids = [0, 1, 2, 0, 1, 2];
        let read = identity_read();
        let step = read
            .train_pick_step(&stream, &ids, &table, 2, 3, &[0, 0, 2, 0, 0, 2], 0, 0)
            .unwrap();
        assert_eq!((step.loss_before, step.loss_after), (131_072, 393_216));
        assert!(!step.accepted);
        assert_eq!(step.read, read);
        assert_eq!(step.picks_after, step.picks_before);
    }

    #[test]
    fn refusals_name_their_rule() {
        let (stream, ids, table) = fixture(S);
        let read = identity_read();
        let step = |targets: &[u32], shift| {
            read.train_pick_step(&stream, &ids, &table, 1, 3, targets, 0, shift)
        };
        let cases: [(&str, Result<Q31ReadTrainStep>, &str); 5] = [
            (
                "short targets",
                step(&[0, 0], 4),
                "needs batch * time targets",
            ),
            (
                "target on row 0",
                step(&[1, 0, 0], 4),
                "an earlier row of its window",
            ),
            (
                "target on itself",
                step(&[0, 0, 3], 4),
                "an earlier row of its window",
            ),
            (
                "shift 32",
                step(&[0, 0, 2], 32),
                "step shift must be in 0..=31",
            ),
            (
                "short stream",
                read.train_pick_step(&stream[16..], &ids, &table, 1, 3, &[0, 0, 2], 0, 4),
                "batch * time * width",
            ),
        ];
        for (name, result, rule) in cases {
            let message = refusal(result);
            assert!(message.contains(rule), "{name}: {message:?} lacks {rule:?}");
        }
    }
}
