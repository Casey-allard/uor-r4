/// Token ids are 12-bit: V = 4,096.
pub const NGRAM_VOCABULARY: u32 = 4096;
/// Up to ten context tokens: 12 x 10 = 120 bits, the order in the top byte.
pub const NGRAM_MAX_ORDER: u32 = 10;

/// The ordered context key of the DECISION, section 3, part 2, in its packed-id
/// form. No prime is computed here: the key is an ordered 12-bit packing of
/// (order, t_{-1}..t_{-k}), a bijection on (order, ordered context), and so
/// carries exactly the information of the decision's ordered positional prime
/// key. Slot 1 (the latest token) occupies bits 108..120, slot s bits
/// 120-12s..132-12s, and the order k occupies bits 120..128. Unused slots are 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimeRouteKey(u128);

impl PrimeRouteKey {
    /// The key of `context`, given in stream order (oldest first, latest last).
    pub fn from_context(context: &[u16]) -> Result<Self> {
        let order = context.len() as u32;
        if order > NGRAM_MAX_ORDER {
            return Err(invalid(format!(
                "n-gram key holds at most {NGRAM_MAX_ORDER} context tokens, got {order}"
            )));
        }
        let mut word = (order as u128) << 120;
        for (s, &token) in context.iter().rev().enumerate() {
            if u32::from(token) >= NGRAM_VOCABULARY {
                return Err(invalid(format!(
                    "token id {token} is outside the 12-bit vocabulary"
                )));
            }
            word |= (token as u128) << (108 - 12 * s as u32);
        }
        Ok(Self(word))
    }

    /// A stored word, checked: a known order, 12-bit slots and zero unused slots.
    pub fn from_word(word: u128) -> Result<Self> {
        let order = (word >> 120) as u32;
        if order > NGRAM_MAX_ORDER {
            return Err(invalid(format!("n-gram key order {order} exceeds {NGRAM_MAX_ORDER}")));
        }
        let used_bits = 12 * order;
        let unused_mask = ((1u128 << 120) - 1) >> used_bits;
        if word & unused_mask != 0 {
            return Err(invalid("n-gram key has nonzero unused slots"));
        }
        Ok(Self(word))
    }

    pub fn word(self) -> u128 {
        self.0
    }

    pub fn order(self) -> u32 {
        (self.0 >> 120) as u32
    }

    /// Slot s (1 = latest); None unless 1 <= s <= order (and so s <= 10).
    pub fn slot(self, s: u32) -> Option<u16> {
        if s == 0 || s > self.order() || s > NGRAM_MAX_ORDER {
            return None;
        }
        Some(((self.0 >> (120 - 12 * s)) & 0xfff) as u16)
    }

    /// The context in stream order (oldest first).
    pub fn context(self) -> Vec<u16> {
        (1..=self.order()).rev().filter_map(|s| self.slot(s)).collect()
    }

    /// Back-off to order k-1: drop the oldest slot.
    pub fn back_off(self) -> Option<Self> {
        let order = self.order();
        if order == 0 {
            return None;
        }
        let kept = self.0 & (((1u128 << 120) - 1) & !(((1u128 << 120) - 1) >> (12 * (order - 1))));
        Some(Self(kept | (((order - 1) as u128) << 120)))
    }
}

/// One context record: total count c(h), N1+(h.) distinct followers, and the
/// followers seen exactly once, exactly twice and three or more times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NgramContext {
    pub key: u128,
    pub total: u64,
    pub followers: u32,
    pub n1: u32,
    pub n2: u32,
    pub n3_plus: u32,
}

/// The document-start token. Streams frame each document with BOS = 0 at its
/// start and EOS = 1 at its end.
pub const NGRAM_BOS: u16 = 0;
