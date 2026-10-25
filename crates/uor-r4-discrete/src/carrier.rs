//! Carrier hard selection (`carrier-credit-hard-selection`, #2819, claude-103).
//!
//! Three coordinate schedules choose which vector block the exact counterfactual
//! move tries on a batch. CUDA computes every choice (`discrete_carrier_select`),
//! and the host receives only the two words of [`CarrierSelection`].
//!
//! - [`CarrierMode::CreditV1`]: the token carrying the most integer rank-margin
//!   hinge (token plus address) in the batch. A row credits its hinge to its
//!   target, and to its strongest admitted rival when top-1 is wrong.
//! - [`CarrierMode::UniformTargetV1`]: a model-blind draw over the batch's
//!   distinct targets.
//! - [`CarrierMode::UniformRowV1`]: a model-blind draw over the batch's rows.
//!
//! The draws are a fixed integer hash of the schedule key, not a sampled credit:
//! nothing reads a model score, and there is no probability.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CarrierMode {
    CreditV1,
    UniformTargetV1,
    UniformRowV1,
}

impl CarrierMode {
    /// The kernel's mode word.
    pub const fn word(self) -> u32 {
        match self {
            Self::CreditV1 => 0,
            Self::UniformTargetV1 => 1,
            Self::UniformRowV1 => 2,
        }
    }
}

/// The selector's device result. `value` is the winning credit for
/// `CreditV1`, the distinct-target count for `UniformTargetV1`, and the drawn
/// row for `UniformRowV1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarrierSelection {
    pub mode: CarrierMode,
    pub token: u32,
    pub value: u64,
}

/// The schedule key of batch `completed_batches` under `seed`.
pub const fn schedule_key(seed: u32, completed_batches: u64) -> u64 {
    ((seed as u64) << 32) ^ completed_batches
}

/// SplitMix64's finalizer, the same integer mix the kernel applies to the key.
/// The host copy exists only so tests can state expected draws.
pub const fn splitmix64(key: u64) -> u64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_words_and_names_are_fixed() {
        assert_eq!(CarrierMode::CreditV1.word(), 0);
        assert_eq!(CarrierMode::UniformTargetV1.word(), 1);
        assert_eq!(CarrierMode::UniformRowV1.word(), 2);
        assert_eq!(
            serde_json::to_string(&CarrierMode::UniformTargetV1).unwrap(),
            "\"uniform_target_v1\""
        );
    }

    #[test]
    fn splitmix64_matches_its_reference_values() {
        // The first outputs of SplitMix64 seeded at 0 (state advanced once
        // per call), as published with the generator.
        assert_eq!(splitmix64(0), 0xE220_A839_7B1D_CDAF);
        assert_eq!(splitmix64(0x9E37_79B9_7F4A_7C15), 0x6E78_9E6A_A1B9_65F4);
    }

    /// The body of the first `fn` with this signature prefix, up to the closing
    /// brace at the indentation where it starts.
    fn body<'a>(source: &'a str, signature: &str) -> &'a str {
        let start = source.find(signature).expect("selection function present");
        let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
        let indent = &source[line_start..start];
        let close = format!("\n{indent}}}\n");
        let end = source[start..]
            .find(&close)
            .expect("selection function closed");
        &source[start..start + end]
    }

    #[test]
    fn the_selection_downloads_only_its_two_words() {
        let device = include_str!("device.rs");
        let method = body(device, "pub fn carrier_select(");
        let selector = body(device, "fn select_carrier(");
        let evaluate = body(device, "fn launch_evaluate(");
        assert!(
            !method.contains("clone_dtoh"),
            "carrier_select must not download rows"
        );
        // The shared evaluate launch keeps its rows on the device too; its only
        // host read is the one-word status inside check_status.
        assert!(
            !evaluate.contains("clone_dtoh"),
            "launch_evaluate must not download rows"
        );
        assert_eq!(
            selector.matches("clone_dtoh(").count(),
            1,
            "select_carrier downloads exactly one buffer"
        );
        assert!(
            selector.contains("clone_dtoh(&out)"),
            "the one download is the two-word result"
        );
        assert!(method.contains("launch_evaluate") && method.contains("select_carrier"));
    }

    #[test]
    fn schedule_key_separates_seeds_and_batches() {
        assert_eq!(schedule_key(1001, 0), 1001u64 << 32);
        assert_ne!(schedule_key(1001, 7), schedule_key(2001, 7));
        assert_ne!(schedule_key(1001, 7), schedule_key(1001, 8));
    }
}
