/// Q32 unit of a mixing weight.
pub const WEIGHT_ONE: u64 = 1 << 32;
/// Buckets per seen order: min(15, floor(log2 total)).
pub const BUCKETS_PER_ORDER: usize = 16;

/// Experts per target: KN, ML_0..=ML_highest, U.
pub fn expert_count(highest: u32) -> usize {
    highest as usize + 3
}

/// Buckets: 16 per highest seen order.
pub fn bucket_count(highest: u32) -> usize {
    (highest as usize + 1) * BUCKETS_PER_ORDER
}

/// The highest seen order h of one target's totals row and its bucket
/// h*16 + min(15, floor(log2 total_h)). Refuses a target with no seen order.
pub fn ceiling_bucket(totals: &[u64]) -> Result<(usize, u16)> {
    let h = totals
        .iter()
        .rposition(|&total| total > 0)
        .ok_or_else(|| invalid("a target has no seen order"))?;
    let log2 = 63 - totals[h].leading_zeros() as usize;
    let bucket = h * BUCKETS_PER_ORDER + log2.min(BUCKETS_PER_ORDER - 1);
    let bucket = u16::try_from(bucket).map_err(|_| invalid("a bucket exceeded u16"))?;
    Ok((h, bucket))
}

/// Convert one raw lookup row into its bucket and Q48 expert masses.
/// The caller supplies KN, one count/total pair per order, and a row with
/// one slot per expert. Orders beyond the last seen context are zero.
pub fn ceiling_expert_row(
    kn: u64,
    counts: &[u32],
    totals: &[u64],
    masses: &mut [u64],
) -> Result<u16> {
    if counts.len() != totals.len() || masses.len() != totals.len() + 2 {
        return Err(invalid("invalid ceiling expert row shape"));
    }
    let (_, bucket) = ceiling_bucket(totals)?;
    masses[0] = kn;
    for (i, (&count, &total)) in counts.iter().zip(totals).enumerate() {
        if total != 0 && u64::from(count) > total {
            return Err(invalid(format!(
                "order {i} count {count} exceeds its total {total}"
            )));
        }
        masses[i + 1] = if total == 0 {
            0
        } else {
            // count <= total, so this quotient is at most 2^48.
            ((u128::from(count) << 48) / u128::from(total)) as u64
        };
    }
    masses[totals.len() + 1] = KN_BASE_MASS;
    Ok(bucket)
}

/// Whether expert e is present in bucket b: KN and U always, ML_k iff k <= h.
fn present(highest: u32, bucket: usize, e: usize) -> bool {
    let h = bucket / BUCKETS_PER_ORDER;
    e == 0 || e == expert_count(highest) - 1 || e - 1 <= h
}

/// claude-439: the Pick class of a target that did not fire.
pub const PICK_NOT_FIRED: u8 = 0;
/// claude-439: Pick fired at pair distance 0.
pub const PICK_FIRED_EXACT: u8 = 1;
/// claude-439: Pick fired at pair distance above 0.
pub const PICK_FIRED_NEAR: u8 = 2;
/// claude-439: Pick classes; each holds one copy of the base buckets.
pub const PICK_CLASSES: u8 = 3;

/// claude-439: one target's Pick reading: its class and whether the copied
/// token is the target. `hit` needs a fired class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickTarget {
    pub class: u8,
    pub hit: bool,
}

fn ceiling_pick_row(
    highest: u32,
    kn: u64,
    counts: &[u32],
    totals: &[u64],
    pick: PickTarget,
    row: &mut [u64],
) -> Result<u16> {
    if pick.class >= PICK_CLASSES {
        return Err(invalid(format!(
            "Pick class {} is above {}",
            pick.class,
            PICK_CLASSES - 1
        )));
    }
    if pick.hit && pick.class == PICK_NOT_FIRED {
        return Err(invalid("a Pick hit that did not fire"));
    }
    let base = expert_count(highest);
    let plain = ceiling_expert_row(kn, counts, totals, &mut row[..base])?;
    row[base] = if pick.hit { 1 << 48 } else { 0 };
    let bucket = usize::from(pick.class) * bucket_count(highest) + usize::from(plain);
    u16::try_from(bucket).map_err(|_| invalid("a bucket exceeded u16"))
}

fn mix_sum(weights: &[u64], masses: &[u64]) -> u128 {
    weights
        .iter()
        .zip(masses)
        .map(|(&weight, &mass)| u128::from(weight) * u128::from(mass))
        .sum()
}

fn mixed_q48(sum: u128) -> u64 {
    u64::try_from(sum >> 32).unwrap_or(u64::MAX).max(1)
}
