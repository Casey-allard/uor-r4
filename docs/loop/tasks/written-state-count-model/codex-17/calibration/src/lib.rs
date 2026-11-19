#![forbid(unsafe_code)]

//! Synthetic calibration of custody and frozen-writer boundaries.

use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    UnexpectedDigest,
    StateBytesChanged,
}

/// Compare the bytes at the fit freeze with the exact bytes loaded at serving.
/// The expected digest comes from an independently pinned freeze receipt.
pub fn check_frozen_snapshot(
    fit_freeze: &[u8],
    served_state: &[u8],
    frozen_digest: [u8; 32],
) -> Result<(), SnapshotError> {
    let fit_digest: [u8; 32] = Sha256::digest(fit_freeze).into();
    let served_digest: [u8; 32] = Sha256::digest(served_state).into();
    if fit_digest != frozen_digest {
        return Err(SnapshotError::UnexpectedDigest);
    }
    if fit_freeze != served_state || served_digest != frozen_digest {
        return Err(SnapshotError::StateBytesChanged);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_all_starts_finite::{Candidates, Evidence, Nomination};
    use uor_r4_integer::{
        assertion_value::AssertionValue,
        discrete_d11::{ExactCopyState, ROLE_COPY_STOPS},
        paired_role::ROLE_VOCAB,
        span_selector::{CONT, START},
        value_selector::Decisions,
    };

    const PROMPT: [u32; 12] = [0, 55, 42, 10, 11, 12, 13, 14, 15, 16, 42, 201];
    const FREE_IDS: [u32; 4] = [301, 302, 303, 1];
    const MISSING: u32 = u32::MAX;

    fn capture() -> Candidates<'static> {
        let mut classes = [0; ROLE_VOCAB];
        for id in [10, 11, 13, 15, 42] {
            classes[id] = START;
        }
        for id in [12, 14] {
            classes[id] = CONT;
        }
        let assertion = AssertionValue {
            fence: Some(9),
            value: Decisions {
                eligible_mask: 7,
                word_start_mask: 5,
                ..Decisions::default()
            },
            ..AssertionValue::default()
        };
        Candidates::capture(&PROMPT, &classes, 2, 10, assertion).unwrap()
    }

    fn score_words(scores: [Option<i32>; 3]) -> [Evidence; 3] {
        std::array::from_fn(|i| Evidence {
            source: [4, 6, 8][i],
            score: scores[i],
        })
    }

    fn writer(source: Option<u32>) -> Vec<[u32; 8]> {
        let mut state = ExactCopyState::new(source, PROMPT.len() as u32);
        let mut prefix = PROMPT.to_vec();
        let mut recorded = Vec::new();
        for ordinary in FREE_IDS {
            let step = state
                .write(&prefix, ordinary, ROLE_COPY_STOPS)
                .unwrap()
                .finish_with_eos(true, 1);
            recorded.push(step.words());
            if step.emitted == 1 {
                break;
            }
            prefix.push(step.emitted);
            state = step.state;
        }
        recorded
    }

    #[test]
    fn identical_prompt_and_candidates_preserve_source_keys_when_scores_swap() {
        let candidates = capture();
        let original = candidates
            .nominate(&score_words([Some(2), Some(7), Some(3)]))
            .unwrap();
        let swapped = candidates
            .nominate(&score_words([Some(2), Some(3), Some(7)]))
            .unwrap();
        assert_eq!(original, Nomination::Source(6));
        assert_eq!(swapped, Nomination::Source(8));

        // These words are the authored boundary oracle. The probe uses the
        // normal frozen writer as the observed output, including every cursor
        // offset, exact source ID, copy flag and delimiter release.
        assert_eq!(
            writer(Some(6)),
            [
                [1, 6, 7, 12, 13, 13, 1, 0],
                [1, 6, 8, 12, 14, 14, 1, 0],
                [1, 6, 9, 12, 15, 15, 1, 0],
                [0, 6, 9, 12, 1, MISSING, 0, 1],
            ]
        );
        assert_eq!(
            writer(Some(8)),
            [[1, 8, 9, 12, 15, 15, 1, 0], [0, 8, 9, 12, 1, MISSING, 0, 1],]
        );
    }

    #[test]
    fn tied_or_unseen_hold_a_disabled_cursor_despite_reachable_sources() {
        let candidates = capture();
        assert_eq!(
            candidates
                .reachable()
                .iter()
                .map(|c| c.source)
                .collect::<Vec<_>>(),
            [4, 6, 8]
        );
        for (scores, expected) in [
            ([Some(9), Some(9), Some(3)], Nomination::Tie),
            ([Some(9), None, Some(3)], Nomination::Unseen),
        ] {
            let nomination = candidates.nominate(&score_words(scores)).unwrap();
            assert_eq!(nomination, expected);
            let source = match nomination {
                Nomination::Source(source) => Some(source),
                _ => None,
            };
            let expected = FREE_IDS.map(|id| [0, MISSING, MISSING, 12, id, MISSING, 0, 0]);
            assert_eq!(writer(source), expected);
            assert_eq!(writer(source), writer(None));
        }
    }

    #[test]
    fn snapshot_custody_rejects_changed_or_forged_frozen_state() {
        // Published SHA-256 test vector. This is a custody fixture, so the
        // three bytes require no unregistered checkpoint format.
        let a = *b"abc";
        let b = *b"abc";
        let pinned = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(check_frozen_snapshot(&a, &b, pinned), Ok(()));
        assert_eq!(
            check_frozen_snapshot(&a, b"acb", pinned),
            Err(SnapshotError::StateBytesChanged)
        );
        assert_eq!(
            check_frozen_snapshot(&a, b"abc\0", pinned),
            Err(SnapshotError::StateBytesChanged)
        );
        assert_eq!(
            check_frozen_snapshot(b"acb", b"acb", pinned),
            Err(SnapshotError::UnexpectedDigest)
        );
    }
}

pub mod fixtures;
pub mod oracle;
