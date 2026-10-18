//! Host bookkeeping for the opt-in BVG-K block proposal (Stage C2, handoff
//! H2b). Everything here works on small device readbacks: block admission,
//! window reconstruction for the scratch re-carry, refusal messages and the
//! per-batch record. The device chooses, decides and commits; nothing here
//! scores, ranks or selects. It cross-checks that the device's decision words
//! are consistent with the rule, and refuses when they are not.
use crate::{Error, Identity, Parameter, Result};

/// Words per coordinate record written by `discrete_bvgk_choose`.
pub const RECORD_WORDS: usize = 8;

/// One batch's BVG-K decision, read back after `discrete_bvgk_decide`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockRecord {
    /// Coordinates in the block.
    pub k: u32,
    /// `|S_t|`: eligible coordinates whose chosen move strictly improves.
    pub s_len: u32,
    /// True iff the whole `S_t` (`|S_t| >= 2`) is committed.
    pub joint: bool,
    /// Current loss on the pre-commit state (lane 0 of a scored coordinate).
    pub l_before: u64,
    /// `L(S_t)` on a scratch state; present iff `|S_t| >= 2`.
    pub l_joint: Option<u64>,
    /// `L({best})`; equals `l_before` when `S_t` is empty (no move).
    pub l_best: u64,
    /// Loss of the committed set: `l_joint`, `l_best`, or `l_before` if empty.
    pub committed_loss: u64,
    /// Block ordinals of the committed coordinates, ascending.
    pub commit_ordinals: Vec<u32>,
}

/// Outcome of one block commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockCommit {
    pub moves: u32,
    pub joint: bool,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// Shape refusals for a block before any launch: one eligibility flag per
/// coordinate, `1..=max` coordinates and no repeated parameter (two members
/// sharing one index would not commit disjoint storage). The one-kind rule is
/// refused by the K-coordinate scorer.
pub fn admit_block(block: &[Parameter], eligible: &[bool], max: usize) -> Result<()> {
    if block.is_empty() || block.len() > max {
        return Err(invalid(format!(
            "coordinate block must hold 1..={max} parameters"
        )));
    }
    if eligible.len() != block.len() {
        return Err(invalid(
            "a block needs exactly one eligibility flag per coordinate",
        ));
    }
    for (c, parameter) in block.iter().enumerate() {
        if block[..c].contains(parameter) {
            return Err(invalid("a coordinate block must not repeat a parameter"));
        }
    }
    Ok(())
}

/// The causal windows a carried batch was built from, in row order. Both
/// `carry` (full-context rows) and `carry_windows` keep the unpadded
/// identities and one valid length per row, so this is exact.
pub fn split_windows(identities: &[Identity], lengths: &[u32]) -> Result<Vec<Vec<Identity>>> {
    let mut at = 0usize;
    let mut windows = Vec::with_capacity(lengths.len());
    for &length in lengths {
        let end = at
            .checked_add(length as usize)
            .ok_or_else(|| invalid("carried window lengths overflow"))?;
        let window = identities
            .get(at..end)
            .ok_or_else(|| invalid("carried window lengths exceed the carried identities"))?;
        windows.push(window.to_vec());
        at = end;
    }
    if at != identities.len() || windows.is_empty() {
        return Err(invalid(
            "carried window lengths do not cover the carried identities",
        ));
    }
    Ok(windows)
}

/// Meaning of a BVG-K kernel status word (`bvgk_source.rs`).
pub fn status_meaning(code: u32) -> &'static str {
    match code {
        3 => "a member's proposal record is stale at the current epoch",
        4 => "a lane, kind, index or decision summary is out of range",
        5 => "a member does not strictly improve, or the epoch is at its maximum",
        9 => "the parameter kind does not match the read mode (gated mismatch)",
        14 => "a member's recorded kind differs from the launch kind",
        15 => "two members share one index",
        16 => "the commit set is empty (the host skips the commit for a no-op batch)",
        17 => "|S_t| >= 2 but the joint loss is absent",
        _ => "unknown BVG-K status",
    }
}

/// `Ok` for status 0, otherwise a focused refusal naming the operation.
pub fn status_result(operation: &str, code: u32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(invalid(format!(
            "BVG-K {operation} refused with status {code}: {}",
            status_meaning(code)
        )))
    }
}

/// Assemble the batch record from the choose records (`k*8` words), the
/// summary `[|S_t|, best, L(best)]`, the joint loss, and decide's commit mask
/// and count. Refuses any readback inconsistent with the decision rule.
pub fn assemble_record(
    records: &[u64],
    summary: [u64; 3],
    l_joint: Option<u64>,
    member: &[u32],
    commit: &[u32],
    commit_count: u64,
) -> Result<BlockRecord> {
    let k = commit.len();
    if k == 0 || member.len() != k || records.len() != k * RECORD_WORDS {
        return Err(invalid(
            "BVG-K records must hold eight words per coordinate",
        ));
    }
    let word = |c: usize, w: usize| records[c * RECORD_WORDS + w];
    let l_before = (0..k)
        .find(|&c| word(c, 7) == 0)
        .map(|c| word(c, 1))
        .ok_or_else(|| {
            invalid("every coordinate was refused by K-coordinate scoring; no current loss")
        })?;
    let mut commit_ordinals = Vec::new();
    for (c, &m) in commit.iter().enumerate() {
        match m {
            0 => {}
            1 => commit_ordinals.push(c as u32),
            _ => return Err(invalid("BVG-K commit mask must be boolean")),
        }
    }
    if commit_ordinals.len() as u64 != commit_count {
        return Err(invalid("BVG-K commit count differs from its commit mask"));
    }
    let s_len = summary[0];
    if s_len > k as u64 {
        return Err(invalid("BVG-K |S_t| exceeds the block"));
    }
    let mut member_ordinals = Vec::new();
    for (c, &m) in member.iter().enumerate() {
        match m {
            0 => {}
            1 => member_ordinals.push(c as u32),
            _ => return Err(invalid("BVG-K member mask must be boolean")),
        }
    }
    if member_ordinals.len() as u64 != s_len {
        return Err(invalid("BVG-K |S_t| differs from its member mask"));
    }
    let k = k as u32;
    let s_len = s_len as u32;
    if s_len == 0 {
        if !commit_ordinals.is_empty() || !member_ordinals.is_empty() || l_joint.is_some() {
            return Err(invalid(
                "an empty S_t commits nothing and has no joint loss",
            ));
        }
        return Ok(BlockRecord {
            k,
            s_len,
            joint: false,
            l_before,
            l_joint,
            l_best: l_before,
            committed_loss: l_before,
            commit_ordinals,
        });
    }
    let best = usize::try_from(summary[1])
        .ok()
        .filter(|&best| best < k as usize)
        .ok_or_else(|| invalid("BVG-K best ordinal is outside the block"))?;
    let l_best = summary[2];
    if word(best, 2) != l_best || l_best >= word(best, 1) {
        return Err(invalid(
            "BVG-K L(best) differs from its record or does not strictly improve",
        ));
    }
    let joint = match (s_len, l_joint) {
        (1, None) => false,
        (1, Some(_)) => return Err(invalid("|S_t| = 1 has no verify forward")),
        (_, None) => return Err(invalid("|S_t| >= 2 requires the joint loss")),
        (_, Some(l)) => l < l_best,
    };
    let best_ordinal = best as u32;
    let consistent = member_ordinals.contains(&best_ordinal)
        && if joint {
            commit_ordinals == member_ordinals
        } else {
            commit_ordinals == [best_ordinal]
        };
    if !consistent {
        return Err(invalid("BVG-K commit set differs from the decision rule"));
    }
    let committed_loss = match (joint, l_joint) {
        (true, Some(l)) => l,
        _ => l_best,
    };
    Ok(BlockRecord {
        k,
        s_len,
        joint,
        l_before,
        l_joint,
        l_best,
        committed_loss,
        commit_ordinals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Record words for one coordinate: lane, L(lane 0), L(lane), status.
    fn rec(lane: u64, before: u64, after: u64, status: u64) -> [u64; RECORD_WORDS] {
        [lane, before, after, 7, 1, 1, 0, status]
    }

    fn records(rows: &[[u64; RECORD_WORDS]]) -> Vec<u64> {
        rows.iter().flatten().copied().collect()
    }

    fn identity(token: u32) -> Identity {
        Identity {
            token,
            occurrence: token as u64,
            record: 0,
            version: 0,
        }
    }

    #[test]
    fn block_admission_refuses_shape_and_repeats() {
        let block = [Parameter::VectorBlock(0), Parameter::VectorBlock(1)];
        assert!(admit_block(&block, &[true, false], 64).is_ok());
        assert!(admit_block(&block, &[true], 64).is_err());
        assert!(admit_block(&[], &[], 64).is_err());
        assert!(admit_block(&block, &[true, true], 1).is_err());
        let repeated = [Parameter::Route(3), Parameter::Route(3)];
        assert!(admit_block(&repeated, &[true, true], 64).is_err());
    }

    #[test]
    fn windows_are_reconstructed_exactly() -> Result<()> {
        let ids: Vec<_> = (0..5).map(identity).collect();
        let windows = split_windows(&ids, &[2, 3])?;
        assert_eq!(windows, vec![ids[..2].to_vec(), ids[2..].to_vec()]);
        assert!(split_windows(&ids, &[2, 2]).is_err());
        assert!(split_windows(&ids, &[2, 4]).is_err());
        assert!(split_windows(&[], &[]).is_err());
        Ok(())
    }

    #[test]
    fn status_codes_map_to_focused_refusals() {
        assert!(status_result("apply", 0).is_ok());
        for code in [3, 4, 5, 9, 14, 15, 16, 17] {
            assert_ne!(status_meaning(code), status_meaning(0));
            let message = status_result("apply", code).unwrap_err().to_string();
            assert!(message.contains(&format!("status {code}")), "{message}");
            assert!(message.contains(status_meaning(code)), "{message}");
        }
    }

    #[test]
    fn empty_s_t_records_a_no_op_batch() -> Result<()> {
        let words = records(&[rec(0, 50, 50, 0), rec(0, 50, 50, 0)]);
        let record = assemble_record(&words, [0, !0, !0], None, &[0, 0], &[0, 0], 0)?;
        assert_eq!(record.s_len, 0);
        assert_eq!(
            (record.l_before, record.l_best, record.committed_loss),
            (50, 50, 50)
        );
        assert!(!record.joint && record.commit_ordinals.is_empty());
        assert!(assemble_record(&words, [0, !0, !0], Some(1), &[0, 0], &[0, 0], 0).is_err());
        assert!(assemble_record(&words, [0, !0, !0], None, &[0, 0], &[1, 0], 1).is_err());
        Ok(())
    }

    #[test]
    fn single_member_commits_best_without_a_verify_forward() -> Result<()> {
        // Coordinate 0 is refused by scoring; l_before comes from coordinate 1.
        let words = records(&[rec(0, 0, 0, 9), rec(4, 50, 40, 0)]);
        let record = assemble_record(&words, [1, 1, 40], None, &[0, 1], &[0, 1], 1)?;
        assert_eq!(record.l_before, 50);
        assert_eq!((record.l_best, record.committed_loss), (40, 40));
        assert_eq!(record.commit_ordinals, vec![1]);
        assert!(assemble_record(&words, [1, 1, 40], Some(30), &[0, 1], &[0, 1], 1).is_err());
        assert!(assemble_record(&words, [1, 1, 40], None, &[0, 1], &[1, 0], 1).is_err());
        Ok(())
    }

    #[test]
    fn joint_commit_requires_a_strictly_lower_joint_loss() -> Result<()> {
        let words = records(&[rec(2, 50, 45, 0), rec(4, 50, 40, 0), rec(0, 50, 50, 0)]);
        let joint = assemble_record(&words, [2, 1, 40], Some(35), &[1, 1, 0], &[1, 1, 0], 2)?;
        assert!(joint.joint);
        assert_eq!(joint.committed_loss, 35);
        assert_eq!(joint.commit_ordinals, vec![0, 1]);
        // L(S_t) == L(best) is not strictly lower: {best} is committed.
        let tie = assemble_record(&words, [2, 1, 40], Some(40), &[1, 1, 0], &[0, 1, 0], 1)?;
        assert!(!tie.joint);
        assert_eq!((tie.l_joint, tie.committed_loss), (Some(40), 40));
        // A mask that disagrees with the rule is refused either way.
        assert!(assemble_record(&words, [2, 1, 40], Some(40), &[1, 1, 0], &[1, 1, 0], 2).is_err());
        assert!(assemble_record(&words, [2, 1, 40], Some(35), &[1, 1, 0], &[0, 1, 0], 1).is_err());
        assert!(assemble_record(&words, [2, 1, 40], None, &[1, 1, 0], &[0, 1, 0], 1).is_err());
        Ok(())
    }

    #[test]
    fn commit_mask_must_match_the_scored_member_mask() -> Result<()> {
        let words = records(&[rec(2, 50, 45, 0), rec(4, 50, 40, 0), rec(0, 50, 50, 0)]);
        // Legal: joint commits exactly the members; single commits only best.
        let joint = assemble_record(&words, [2, 1, 40], Some(35), &[1, 1, 0], &[1, 1, 0], 2)?;
        assert_eq!(joint.commit_ordinals, vec![0, 1]);
        let single = assemble_record(&words, [2, 1, 40], Some(40), &[1, 1, 0], &[0, 1, 0], 1)?;
        assert_eq!(single.commit_ordinals, vec![1]);
        let empty = assemble_record(&words, [0, !0, !0], None, &[0, 0, 0], &[0, 0, 0], 0)?;
        assert!(empty.commit_ordinals.is_empty());
        // Joint commit swaps member 0 for non-member 2: equal count, includes best.
        assert!(assemble_record(&words, [2, 1, 40], Some(35), &[1, 1, 0], &[0, 1, 1], 2).is_err());
        // Single commit at a non-best ordinal.
        assert!(assemble_record(&words, [2, 1, 40], Some(40), &[1, 1, 0], &[1, 0, 0], 1).is_err());
        // Empty S_t with a stray commit bit.
        assert!(assemble_record(&words, [0, !0, !0], None, &[0, 0, 0], &[0, 0, 1], 1).is_err());
        // Member popcount must equal |S_t|.
        assert!(assemble_record(&words, [2, 1, 40], Some(35), &[1, 0, 0], &[1, 0, 0], 1).is_err());
        Ok(())
    }

    #[test]
    fn inconsistent_readbacks_are_refused() {
        let words = records(&[rec(4, 50, 40, 0)]);
        assert!(assemble_record(&words, [1, 0, 40], None, &[1], &[1], 2).is_err());
        assert!(assemble_record(&words, [1, 0, 41], None, &[1], &[1], 1).is_err());
        assert!(assemble_record(&words, [1, 3, 40], None, &[1], &[1], 1).is_err());
        assert!(assemble_record(&words, [2, 0, 40], Some(1), &[1], &[1], 1).is_err());
        assert!(assemble_record(&words, [1, 0, 40], None, &[1], &[2], 1).is_err());
        assert!(assemble_record(&words[..7], [1, 0, 40], None, &[1], &[1], 1).is_err());
        let refused = records(&[rec(0, 0, 0, 9)]);
        assert!(assemble_record(&refused, [0, !0, !0], None, &[0], &[0], 0).is_err());
    }
}
