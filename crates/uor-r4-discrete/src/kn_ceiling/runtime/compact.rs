const MAGIC: &[u8; 8] = b"UORKNC02";
const KEY_MASK: u128 = (1 << 120) - 1;

fn key_width(order: u32) -> usize {
    (order as usize * 12).div_ceil(8)
}

fn key_body(word: u128, order: u32) -> [u8; 16] {
    let shift = 120 - order * 12;
    ((word & KEY_MASK) >> shift).to_be_bytes()
}

fn follower_width(shared: bool) -> usize {
    if shared {
        6
    } else {
        10
    }
}

fn take<'a>(remaining: &mut &'a [u8], size: usize) -> Result<&'a [u8]> {
    let (head, tail) = remaining
        .split_at_checked(size)
        .ok_or_else(|| invalid("truncated compact count lookup"))?;
    *remaining = tail;
    Ok(head)
}

fn word<const N: usize>(remaining: &mut &[u8]) -> Result<[u8; N]> {
    take(remaining, N)?
        .try_into()
        .map_err(|_| invalid("invalid compact count word"))
}

struct CompactOrder<'a> {
    contexts: &'a [u8],
    followers: &'a [u8],
    key_width: usize,
    shared: bool,
    discounts: KnDiscounts,
}

impl CompactOrder<'_> {
    fn stride(&self) -> usize {
        self.key_width + 4
    }

    fn context_key(&self, index: usize) -> &[u8] {
        let start = index * self.stride();
        &self.contexts[start..start + self.key_width]
    }

    fn context_offset(&self, index: usize) -> usize {
        let end = (index + 1) * self.stride();
        if end > self.contexts.len() {
            return self.followers.len() / follower_width(self.shared);
        }
        let offset = &self.contexts[end - 4..end];
        u32::from_le_bytes([offset[0], offset[1], offset[2], offset[3]]) as usize
    }

    fn follower_rows(&self, context: usize) -> Option<&[u8]> {
        let width = follower_width(self.shared);
        let start = self.context_offset(context);
        let end = self.context_offset(context + 1);
        // Every valid context has at least one and at most 4,095 successors.
        if end <= start || end - start >= NGRAM_VOCABULARY as usize {
            return None;
        }
        self.followers
            .get(start.checked_mul(width)?..end.checked_mul(width)?)
    }

    fn read_follower(&self, row: &[u8]) -> (u16, u32, u32) {
        let next = u16::from_le_bytes([row[0], row[1]]);
        let raw = u32::from_le_bytes([row[2], row[3], row[4], row[5]]);
        let kn = if self.shared {
            raw
        } else {
            u32::from_le_bytes([row[6], row[7], row[8], row[9]])
        };
        (next, raw, kn)
    }

    fn validate(&self, order: u32, raw: [u64; 4], kn: [u64; 4]) -> Result<()> {
        if self.context_offset(0) != 0 {
            return Err(invalid("compact lookup does not begin at row zero"));
        }
        let mut spectra = [[0u64; 4]; 2];
        let mut previous = None;
        for i in 0..self.contexts.len() / self.stride() {
            let key = self.context_key(i);
            if (order % 2 == 1 && key[0] > 0x0f) || previous.is_some_and(|prior| prior >= key) {
                return Err(invalid("invalid or unordered compact context keys"));
            }
            previous = Some(key);
            let rows = self
                .follower_rows(i)
                .ok_or_else(|| invalid("invalid compact follower span"))?;
            let mut previous_next = NGRAM_BOS;
            for row in rows.chunks_exact(follower_width(self.shared)) {
                let (next, raw, kn) = self.read_follower(row);
                if next <= previous_next
                    || u32::from(next) >= NGRAM_VOCABULARY
                    || raw == 0
                    || kn == 0
                {
                    return Err(invalid("invalid or unordered compact follower"));
                }
                previous_next = next;
                for (spectrum, count) in spectra.iter_mut().zip([raw, kn]) {
                    if count <= 4 {
                        spectrum[count as usize - 1] += 1;
                    }
                }
            }
        }
        if spectra != [raw, kn] {
            return Err(invalid("compact count spectrum does not match followers"));
        }
        Ok(())
    }

    fn matched_rows(&self, search_key: &[u8]) -> Option<&[u8]> {
        let n = self.contexts.len() / self.stride();
        let (mut left, mut right) = (0, n);
        while left < right {
            let middle = left + (right - left) / 2;
            if self.context_key(middle) < search_key {
                left = middle + 1;
            } else {
                right = middle;
            }
        }
        if left == n || self.context_key(left) != search_key {
            return None;
        }
        self.follower_rows(left)
    }

    fn lookup(&self, search_key: &[u8], target: u16) -> Option<(u32, u64, u32, NgramContext)> {
        let mut raw_total = 0;
        let mut raw_target = 0;
        let mut kn_target = 0;
        let mut context = NgramContext {
            key: 0,
            total: 0,
            followers: 0,
            n1: 0,
            n2: 0,
            n3_plus: 0,
        };
        for row in self
            .matched_rows(search_key)?
            .chunks_exact(follower_width(self.shared))
        {
            let (next, raw, kn) = self.read_follower(row);
            raw_total += u64::from(raw);
            context.total += u64::from(kn);
            context.followers += 1;
            match kn {
                1 => context.n1 += 1,
                2 => context.n2 += 1,
                _ => context.n3_plus += 1,
            }
            if next == target {
                raw_target = raw;
                kn_target = kn;
            }
        }
        Some((raw_target, raw_total, kn_target, context))
    }
}

/// Borrow a validated complete count index in the serialized representation.
/// Construction validates every row; each lookup uses stack scalars only.
pub struct CompactCountLookup<'a> {
    orders: Vec<CompactOrder<'a>>,
}

impl<'a> CompactCountLookup<'a> {
    pub fn from_bytes(
        bytes: &'a [u8],
        expected_training: [u8; 32],
        expected_artifact: [u8; 32],
    ) -> Result<Self> {
        if Sha256::digest(bytes)[..] != expected_artifact {
            return Err(invalid("compact count artifact digest mismatch"));
        }
        let mut remaining = bytes;
        if &word::<8>(&mut remaining)? != MAGIC {
            return Err(invalid("wrong compact count lookup format"));
        }
        let highest = u32::from_le_bytes(word(&mut remaining)?);
        if highest > NGRAM_MAX_ORDER || word::<32>(&mut remaining)? != expected_training {
            return Err(invalid("invalid order or compact count training binding"));
        }
        let mut orders = Vec::with_capacity(highest as usize + 1);
        for order in 0..=highest {
            let followers = u32::from_le_bytes(word(&mut remaining)?) as usize;
            let contexts = u32::from_le_bytes(word(&mut remaining)?) as usize;
            let mut spectra = [[0u64; 4]; 2];
            for count in spectra.iter_mut().flatten() {
                *count = u64::from_le_bytes(word(&mut remaining)?);
            }
            let key_width = key_width(order);
            let shared = order == highest;
            let size = |count: usize, width| {
                count
                    .checked_mul(width)
                    .ok_or_else(|| invalid("compact count table length overflow"))
            };
            let (discounts, _) = KnDiscounts::with_fallback(spectra[1])?;
            let table = CompactOrder {
                contexts: take(&mut remaining, size(contexts, key_width + 4)?)?,
                followers: take(&mut remaining, size(followers, follower_width(shared))?)?,
                key_width,
                shared,
                discounts,
            };
            table.validate(order, spectra[0], spectra[1])?;
            orders.push(table);
        }
        if !remaining.is_empty() {
            return Err(invalid("trailing bytes in compact count lookup"));
        }
        Ok(Self { orders })
    }

    /// Visit the raw and adjusted count words in canonical (order, key, next)
    /// order, using the same validated tables and row decoder as prediction.
    pub fn visit_counts(
        &self,
        mut visit: impl FnMut(u32, u128, u16, u32, u32),
    ) -> Result<()> {
        for (k, order) in self.orders.iter().enumerate() {
            let k = k as u32;
            let width = follower_width(order.shared);
            for context in 0..order.contexts.len() / order.stride() {
                let mut body = [0u8; 16];
                body[16 - order.key_width..].copy_from_slice(order.context_key(context));
                let key = (u128::from(k) << 120) | (u128::from_be_bytes(body) << (120 - 12 * k));
                let followers = order
                    .follower_rows(context)
                    .ok_or_else(|| invalid("invalid compact follower span during readback"))?;
                for row in followers.chunks_exact(width) {
                    let (next, raw, adjusted) = order.read_follower(row);
                    visit(k, key, next, raw, adjusted);
                }
            }
        }
        Ok(())
    }

    /// Number of stored order sections, including order zero.
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }

    /// Fill per-order raw count and total words and return the original KN Q48
    /// mass, using the caller's causal history and target ID at `at`.
    pub fn lookup(
        &self,
        tokens: &[u16],
        at: usize,
        counts: &mut [u32],
        totals: &mut [u64],
    ) -> Result<u64> {
        let target = *tokens
            .get(at)
            .ok_or_else(|| invalid("missing compact lookup target"))?;
        if target == NGRAM_BOS || u32::from(target) >= NGRAM_VOCABULARY {
            return Err(invalid(
                "compact lookup target is not a valid non-BOS token",
            ));
        }
        if counts.len() != self.orders.len() || totals.len() != self.orders.len() {
            return Err(invalid("wrong compact lookup output width"));
        }
        counts.fill(0);
        totals.fill(0);
        let mut mass = u128::from(KN_BASE_MASS);
        for (k, order) in self.orders.iter().take(at + 1).enumerate() {
            let key = PrimeRouteKey::from_context(&tokens[at - k..at])?;
            let body = key_body(key.word(), k as u32);
            if let Some((count, total, adjusted, context)) =
                order.lookup(&body[16 - order.key_width..], target)
            {
                counts[k] = count;
                totals[k] = total;
                mass = interpolate_order(mass, adjusted, context, order.discounts);
            }
        }
        u64::try_from(mass.max(1)).map_err(|_| invalid("compact KN mass exceeded u64"))
    }
}
