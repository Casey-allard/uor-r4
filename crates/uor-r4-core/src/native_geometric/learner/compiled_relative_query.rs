//! Constant-size compiled relative transport, preserving checked path semantics.
#![forbid(unsafe_code)]
use super::{ht, RelativeActionModel};

/// A learned relation path reduced to a signed permutation and its overflow domain.
///
/// The mask is essential: two cancelling negations still reject i32::MIN at the
/// first negation. This is a transient execution plan, not a new learned model or
/// serialized format. Compiling and executing a valid plan allocate no heap data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRelativePath {
    mapping: [i8; 4],
    reject_min_mask: u8,
}

impl CompiledRelativePath {
    /// Resolve learned relation identities once, in their original path order.
    pub fn compile(model: &RelativeActionModel, path: &[usize]) -> Result<Self, String> {
        let mut markers = [1, 2, 3, 4];
        let mut reject_min_mask = 0;
        for &relation in path {
            let action = *model.actions().get(relation).ok_or("unknown relation")?;
            markers = ht::apply(action, markers)?;
            for marker in markers {
                if marker < 0 {
                    reject_min_mask |= 1 << (marker.unsigned_abs() - 1);
                }
            }
        }
        Ok(Self {
            mapping: markers.map(|m| m as i8),
            reject_min_mask,
        })
    }

    fn validate_key(&self, key: &[i32; 4]) -> Result<(), String> {
        for (i, &value) in key.iter().enumerate() {
            if value == i32::MIN && self.reject_min_mask & (1 << i) != 0 {
                return Err("signed transport overflow".into());
            }
        }
        Ok(())
    }

    /// Execute the folded action, including rejection caused by any earlier prefix.
    pub fn act(&self, key: [i32; 4]) -> Result<[i32; 4], String> {
        self.validate_key(&key)?;
        let mut out = [0; 4];
        for (j, &marker) in self.mapping.iter().enumerate() {
            let value = key[usize::from(marker.unsigned_abs() - 1)];
            out[j] = if marker < 0 {
                value.checked_neg().ok_or("signed transport overflow")?
            } else {
                value
            };
        }
        Ok(out)
    }

    /// Move the query to the keys' frame, then scan unchanged candidate identities.
    ///
    /// For a signed permutation R, ||q - Rk||_1 = ||R^-1 q - k||_1. The query
    /// inverse is widened to i64 before sign changes: unlike transported keys,
    /// queries may legitimately contain i32::MIN on any coordinate.
    /// All candidates are validated, even after a zero-distance winner is found.
    pub fn select(&self, query: [i32; 4], keys: &[[i32; 4]]) -> Result<usize, String> {
        if keys.is_empty() || keys.len() > 1024 {
            return Err("invalid candidate count".into());
        }
        let mut inverse_query = [0i64; 4];
        for (j, &marker) in self.mapping.iter().enumerate() {
            let value = i64::from(query[j]);
            inverse_query[usize::from(marker.unsigned_abs() - 1)] =
                if marker < 0 { -value } else { value };
        }
        let mut best = (u64::MAX, 0);
        for (i, key) in keys.iter().enumerate() {
            self.validate_key(key)?;
            // Each difference is at most 2^32, and their sum fits in u64.
            let d: u64 = inverse_query
                .iter()
                .zip(key)
                .map(|(&q, &k)| (q - i64::from(k)).unsigned_abs())
                .sum();
            if d < best.0 {
                best = (d, i);
            }
        }
        Ok(best.1)
    }
}
