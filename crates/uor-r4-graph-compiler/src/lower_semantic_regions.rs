//! Lowering Reference Semantic Regions into Boolean, Mask, Popcount, and Fixed-Point Programs
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§5, 7, 17;
//! `docs/formal_vocabulary.md` §6; GitHub Issue #130.
//!
//! This module implements the lowering boundary from the reference floating-point graph
//! to the normative transformerless runtime representation:
//! - Lowering region predicates into Boolean bitmasks, bit signatures, and Hamming/popcount thresholds.
//! - Quantizing transition scores into fixed-width ScoreQ (Q8.8 i16) tables with explicit saturation.
//! - Emitting traceable `LoweringWitness` linking runtime binary layout to reference IR CIDs.
//! - Enforcing runtime transformerless invariants (XOR/AND/OR/shift/rotate/popcount/add/sub only).

/// Lowered Boolean region predicate evaluated via bitwise XOR and POPCOUNT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredBooleanRegion {
    pub region_id: String,
    pub bitmask: u64,
    pub expected_signature: u64,
    pub max_hamming_distance: u32,
    pub reference_contribution_id: u32,
}

impl LoweredBooleanRegion {
    /// Runtime integer-only predicate evaluation (XOR + POPCOUNT).
    /// Returns true if Hamming distance between input signature and region signature <= max threshold.
    #[inline]
    pub fn evaluate_runtime_integer(&self, input_signature: u64) -> bool {
        let diff = (input_signature & self.bitmask) ^ (self.expected_signature & self.bitmask);
        let hamming = diff.count_ones();
        hamming <= self.max_hamming_distance
    }
}

/// Quantized Q8.8 fixed-point ScoreQ program entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoweredFixedPointScore {
    /// Q8.8 fixed-point representation (scaled by 256.0).
    pub q88_value: i16,
    pub saturated: bool,
}

impl LoweredFixedPointScore {
    /// Quantize f32 float into Q8.8 i16 with explicit saturation. Values beyond
    /// the Q8.8 range saturate (they are representable, flagged `saturated`);
    /// only a NaN has no fixed-point image, so it yields `None` (R5 — the
    /// absence of a product rather than a raised error).
    pub fn quantize_q88(val: f32) -> Option<Self> {
        if val.is_nan() {
            return None;
        }
        let scaled = (val * 256.0).round();
        if scaled > i16::MAX as f32 {
            Some(Self {
                q88_value: i16::MAX,
                saturated: true,
            })
        } else if scaled < i16::MIN as f32 {
            Some(Self {
                q88_value: i16::MIN,
                saturated: true,
            })
        } else {
            Some(Self {
                q88_value: scaled as i16,
                saturated: false,
            })
        }
    }

    /// Dequantize back to f32 float for reference comparison testing.
    pub fn dequantize(&self) -> f32 {
        self.q88_value as f32 / 256.0
    }
}

/// Traceable witness linking runtime integer table indices back to reference IR CIDs.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweringWitnessEntry {
    pub runtime_table_index: u32,
    pub reference_cid: String,
    pub lowered_type: String,
    pub quantization_delta: f32,
}

/// Complete Lowering Witness emitted alongside binary packed artifacts.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweringWitness {
    pub compiler_version: String,
    pub total_lowered_regions: usize,
    pub total_lowered_scores: usize,
    pub entries: Vec<LoweringWitnessEntry>,
}

/// Lowering Compiler engine.
pub struct BooleanLoweringCompiler;

impl BooleanLoweringCompiler {
    /// Lower a set of boolean signatures and radiuses into `LoweredBooleanRegion` predicates.
    ///
    /// `None` when the region is unrepresentable in the integer runtime: a
    /// signature longer than the 64-bit mask, or a radius outside the Hamming
    /// bound `[0, signature length]`. Both are properties of the caller's
    /// requested region, so no lowered region is a product of them (R5 — the
    /// absence of a product rather than a raised error; the mixed integer/float
    /// conditions unify as `None` rather than a single sanctioned bound type).
    pub fn lower_region(
        region_id: impl Into<String>,
        signature_bits: &[bool],
        radius_float: f32,
        ref_cid: &str,
        reference_contribution_id: u32,
        runtime_table_index: u32,
    ) -> Option<(LoweredBooleanRegion, LoweringWitnessEntry)> {
        let r_id = region_id.into();
        if signature_bits.len() > 64 {
            return None;
        }
        let max_radius = signature_bits.len() as f32;
        if !(0.0..=max_radius).contains(&radius_float) {
            return None;
        }

        let mut sig_u64 = 0u64;
        let mut mask_u64 = 0u64;

        for (idx, &bit) in signature_bits.iter().enumerate() {
            mask_u64 |= 1u64 << idx;
            if bit {
                sig_u64 |= 1u64 << idx;
            }
        }

        let max_dist = radius_float.floor() as u32;

        let region = LoweredBooleanRegion {
            region_id: r_id.clone(),
            bitmask: mask_u64,
            expected_signature: sig_u64,
            max_hamming_distance: max_dist,
            reference_contribution_id,
        };

        let witness = LoweringWitnessEntry {
            runtime_table_index,
            reference_cid: ref_cid.to_string(),
            lowered_type: "LoweredBooleanRegion".to_string(),
            quantization_delta: (radius_float - max_dist as f32).abs(),
        };

        Some((region, witness))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowered_boolean_region_runtime_integer_evaluation() {
        let (region, witness) = BooleanLoweringCompiler::lower_region(
            "reg_1",
            &[true, false, true, true], // sig = 0b1101 = 13
            1.0,
            "cid_test_123",
            101,
            0,
        )
        .unwrap();

        assert_eq!(witness.reference_cid, "cid_test_123");

        // Exact match -> distance 0 <= 1 -> true
        assert!(region.evaluate_runtime_integer(0b1101));
        // 1 bit difference -> distance 1 <= 1 -> true
        assert!(region.evaluate_runtime_integer(0b1100));
        // 2 bits difference -> distance 2 > 1 -> false
        assert!(!region.evaluate_runtime_integer(0b0000));
    }

    #[test]
    fn test_q88_fixed_point_quantization_and_saturation() {
        let q_normal = LoweredFixedPointScore::quantize_q88(1.5).unwrap();
        assert_eq!(q_normal.q88_value, 384); // 1.5 * 256
        assert!(!q_normal.saturated);
        assert!((q_normal.dequantize() - 1.5).abs() < 1e-4);

        // Saturation test
        let q_max = LoweredFixedPointScore::quantize_q88(500.0).unwrap();
        assert_eq!(q_max.q88_value, i16::MAX);
        assert!(q_max.saturated);

        let q_min = LoweredFixedPointScore::quantize_q88(-500.0).unwrap();
        assert_eq!(q_min.q88_value, i16::MIN);
        assert!(q_min.saturated);
    }

    #[test]
    fn test_unrepresentable_region_rejection() {
        let long_sig = vec![true; 100];
        let result = BooleanLoweringCompiler::lower_region(
            "reg_overflow",
            &long_sig,
            1.0,
            "cid_err",
            101,
            0,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_radius_exceeding_signature_width_rejected() {
        let sig = vec![true, false, true, true]; // 4-bit signature
        let result =
            BooleanLoweringCompiler::lower_region("reg_wide_radius", &sig, 5.0, "cid_err", 101, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_saturation_boundary_not_flagged_when_exactly_representable() {
        // i16::MAX / 256.0 is exactly representable and should not be marked saturated.
        let q_exact_max = LoweredFixedPointScore::quantize_q88(i16::MAX as f32 / 256.0).unwrap();
        assert_eq!(q_exact_max.q88_value, i16::MAX);
        assert!(!q_exact_max.saturated);

        // i16::MIN / 256.0 is exactly representable and should not be marked saturated.
        let q_exact_min = LoweredFixedPointScore::quantize_q88(i16::MIN as f32 / 256.0).unwrap();
        assert_eq!(q_exact_min.q88_value, i16::MIN);
        assert!(!q_exact_min.saturated);
    }
}
