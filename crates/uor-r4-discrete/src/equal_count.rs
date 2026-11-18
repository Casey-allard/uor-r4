//! Fixed envelope for the finite equal-byte transport probe (#4063).
//! The implicit index is direct addressing modulo the number of rows.
//! Compact rows hold four (token, count) u32 pairs. Budget saved on a
//! control's key metadata buys additional rows, never padding.
use crate::{position_mechanism::compose_frames, trunk::build_trunk_step_table, Error, Result};

pub const VOCAB: usize = 4096;
pub const BUDGET: usize = 128 * 1024;
pub const ROW_BYTES: usize = 32;
pub const HEADER_BYTES: usize = 32;
pub const MAX_TOKENS: usize = 1_048_576;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Transport,
    Reseeded,
    Ordered(u8),
    SplitMix,
}

impl Key {
    pub fn words(self) -> Result<(u32, u32)> {
        match self {
            Self::Transport => Ok((0, 12)),
            Self::Reseeded => Ok((1, 12)),
            Self::Ordered(k @ 1..=12) => Ok((2, u32::from(k))),
            Self::Ordered(_) => Err(Error::Invalid("context order must be 1..=12".into())),
            Self::SplitMix => Ok((3, 12)),
        }
    }

    pub fn metadata(self, roots: &[u32]) -> Result<Vec<u8>> {
        self.words()?;
        if roots.len() != VOCAB || roots.iter().any(|&v| v > 65535) {
            return Err(Error::Invalid("expected V4096 four-nibble codebook".into()));
        }
        if !matches!(self, Self::Transport | Self::Reseeded) {
            return Ok(Vec::new());
        }
        let mut meta: Vec<u8> = roots.iter().map(|&r| r as u8).collect();
        for a in 0..64 {
            for b in 0..64 {
                meta.push(compose_frames(a, b)?);
            }
        }
        let steps = build_trunk_step_table(4, 0)?;
        for block in 0..2 {
            let mut actions = [0; 16];
            for (r, action) in actions.iter_mut().enumerate() {
                *action = steps.step_action(block, r as u8)?;
            }
            if self == Self::Reseeded {
                // Permute assignments within block, retaining the complete multiset
                // of group elements, codebook, group law and history length.
                let mut s = 0x6a09e667f3bcc909_u64 + block as u64;
                for i in (1..16).rev() {
                    s ^= s << 13;
                    s ^= s >> 7;
                    s ^= s << 17;
                    actions.swap(i, (s % (i as u64 + 1)) as usize);
                }
            }
            meta.extend(actions);
        }
        Ok(meta)
    }
}

pub fn row_count(metadata_len: usize) -> Result<usize> {
    let rows_bytes = BUDGET
        .checked_sub(HEADER_BYTES + VOCAB * 4)
        .and_then(|n| n.checked_sub(metadata_len))
        .ok_or_else(|| Error::Invalid("key metadata exceeds table budget".into()))?;
    if !rows_bytes.is_multiple_of(ROW_BYTES) || rows_bytes == 0 {
        return Err(Error::Invalid(
            "equal-byte key leaves a partial or empty table".into(),
        ));
    }
    Ok(rows_bytes / ROW_BYTES)
}

/// Canonical little-endian image. Score launches read back these exact words.
/// Header words: magic, format version, arm, order, number of rows, key bytes,
/// vocabulary, and total training predictions.
pub fn image(key: Key, meta: &[u8], uni: &[u32], table: &[u32], targets: u32) -> Result<Vec<u8>> {
    let (arm, k) = key.words()?;
    let rows = row_count(meta.len())?;
    let expected_meta = if arm < 2 { 8224 } else { 0 };
    if meta.len() != expected_meta
        || uni.len() != VOCAB
        || table.len() != rows * 8
        || uni.iter().map(|&n| u64::from(n)).sum::<u64>() != u64::from(targets)
    {
        return Err(Error::Invalid(
            "invalid frozen count image shape or total".into(),
        ));
    }
    let mut out = Vec::with_capacity(BUDGET);
    for word in [
        u32::from_le_bytes(*b"U4EC"),
        1,
        arm,
        k,
        rows as u32,
        meta.len() as u32,
        VOCAB as u32,
        targets,
    ] {
        out.extend(word.to_le_bytes());
    }
    out.extend_from_slice(meta);
    for &word in uni.iter().chain(table) {
        out.extend(word.to_le_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn budget_includes_key_and_index_storage() -> Result<()> {
        let roots = vec![0; VOCAB];
        for key in [
            Key::Transport,
            Key::Reseeded,
            Key::Ordered(12),
            Key::SplitMix,
        ] {
            let meta = key.metadata(&roots)?;
            let rows = row_count(meta.len())?;
            let bytes = image(key, &meta, &vec![0; VOCAB], &vec![0; rows * 8], 0)?;
            assert_eq!(bytes.len(), BUDGET);
            assert_eq!(rows, if meta.is_empty() { 3583 } else { 3326 });
        }
        assert!(image(Key::Transport, &[], &vec![0; VOCAB], &vec![0; 3583 * 8], 0).is_err());
        assert!(Key::Ordered(0).words().is_err());
        assert!(row_count(1).is_err());
        Ok(())
    }
}
