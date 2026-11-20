/// `2^48`, the Q48 unit; equal to `lawful_likelihood::PROBABILITY_ONE`.
pub const PROBABILITY_ONE: u64 = 1 << 48;

/// The uniform base mass `P_{-1}`: 2^48 / 4096 = 2^36.
pub const KN_BASE_MASS: u64 = PROBABILITY_ONE / NGRAM_VOCABULARY as u64;

/// One order's three discounts `Dq_1, Dq_2, Dq_3+` in Q48.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnDiscounts {
    pub dq: [u64; 3],
}

impl KnDiscounts {
    /// KenLM's `--discount_fallback` default (1/2, 1, 3/2) in Q48: an order
    /// whose spectrum gives no in-range Chen–Goodman estimate takes these
    /// three discounts whole (claude-421; the order-0 continuation spectrum of
    /// the whole D87 TRAIN stream is n1..n4 = [7, 6, 11, 11], so D2 < 0).
    pub const FALLBACK: Self = Self {
        dq: [
            PROBABILITY_ONE / 2,
            PROBABILITY_ONE,
            PROBABILITY_ONE / 2 * 3,
        ],
    };

    /// Chen–Goodman's estimates from one order's count-of-counts n1..n4,
    /// floored once to Q48. Refuses a zero n_i and a discount outside
    /// `[0, i M]`.
    pub fn from_count_of_counts(count_of_counts: [u64; 4]) -> Result<Self> {
        Self::chen_goodman(count_of_counts)?
            .map(|dq| Self { dq })
            .map_err(invalid)
    }

    /// The order's discounts and whether they are `FALLBACK`: Chen–Goodman
    /// when every n_i > 0 and every estimate lies in `[0, i M]`, else the
    /// fallback for the whole order, as KenLM does. Overflow is refused, but
    /// only in the estimates computed before the first degenerate one (as in
    /// KenLM's order of checks); that needs counts near 2^60.
    pub fn with_fallback(count_of_counts: [u64; 4]) -> Result<(Self, bool)> {
        Ok(match Self::chen_goodman(count_of_counts)? {
            Ok(dq) => (Self { dq }, false),
            Err(_) => (Self::FALLBACK, true),
        })
    }

    /// The outer error is overflow; the inner one is a degenerate spectrum.
    fn chen_goodman(count_of_counts: [u64; 4]) -> Result<std::result::Result<[u64; 3], String>> {
        if count_of_counts.contains(&0) {
            return Ok(Err(format!(
                "modified-KN discounts need n1..n4 > 0, got {count_of_counts:?}"
            )));
        }
        let n: [i128; 4] = count_of_counts.map(i128::from);
        let overflow = || invalid("modified-KN discount arithmetic overflowed");
        let s = n[0]
            .checked_add(n[1].checked_mul(2).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        let m = i128::from(PROBABILITY_ONE);
        let mut dq = [0u64; 3];
        for i in 1..=3usize {
            let ii = i as i128;
            let kept = ii
                .checked_mul(n[i - 1])
                .and_then(|x| x.checked_mul(s))
                .ok_or_else(overflow)?;
            let taken = (ii + 1)
                .checked_mul(n[0])
                .and_then(|x| x.checked_mul(n[i]))
                .ok_or_else(overflow)?;
            let numerator = m.checked_mul(kept - taken).ok_or_else(overflow)?;
            let denominator = n[i - 1].checked_mul(s).ok_or_else(overflow)?;
            let discount = numerator.div_euclid(denominator);
            if discount < 0 || discount > ii * m {
                return Ok(Err(format!(
                    "modified-KN discount D{i} = {discount} lies outside [0, {i} * 2^48] for n1..n4 {count_of_counts:?}"
                )));
            }
            dq[i - 1] = discount as u64;
        }
        Ok(Ok(dq))
    }

    /// `D(a)`: 0 for an unseen gram, then `Dq_min(a, 3)`.
    pub fn discount(&self, a: u32) -> u64 {
        match a {
            0 => 0,
            1 => self.dq[0],
            2 => self.dq[1],
            _ => self.dq[2],
        }
    }
}

/// One integer interpolation step for every representation of the same order.
pub(crate) fn interpolate_order(
    lower_mass: u128,
    target_count: u32,
    context: NgramContext,
    discounts: KnDiscounts,
) -> u128 {
    let m = u128::from(PROBABILITY_ONE);
    let [d1, d2, d3] = discounts.dq.map(u128::from);
    let total = u128::from(context.total);
    let gamma = (d1 * u128::from(context.n1)
        + d2 * u128::from(context.n2)
        + d3 * u128::from(context.n3_plus))
        / total;
    let kept = u128::from(target_count) * m - u128::from(discounts.discount(target_count));
    kept / total + gamma * lower_mass / m
}
