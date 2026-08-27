# Normative Fixed-Point Scoring Semantics Specification

> **Preserved R4G1 runtime contract.** These scoring semantics remain scoped to
> the historical graph runtime. They are not the energy function for the active
> recursive route hierarchy and do not establish source-free intelligence. See
> the [documentation map](README.md).

**Version:** 1.0.0  
**Date:** 2026-07-24  
**Source Baseline:** `docs/hologram_formal_analysis_direction.md` §§7, 12, 13; `docs/formal_vocabulary.md`; `docs/inference_contract.md`; GitHub Issue #158.

---

## 1. Overview & Scope

This specification defines the normative fixed-point scoring semantics for the production R⁴/R4G1 transformerless inference runtime.

### Core Scoring Invariants
- **Fully Quantized Residual Emits Only:** All scores entering the runtime are pre-quantized residuals emitted by the compiler.
- **Integer Operations Only:** Runtime score calculation consists strictly of integer saturating accumulation and compare operations over pre-signed residuals. No floating-point operations, no multiplication, no division, no runtime rescaling, no runtime cosine or dot-product similarity, and no softmax are executed on the deployed hot path.
- **Fixed-Capacity & Zero-Allocation:** All scoring operations are `no_std`, `alloc`-free, and operate over fixed-capacity stack/inline storage.

---

## 2. Score Width, Signedness & Representation

- **Canonical Score Type (`ScoreQ`):** Signed 32-bit fixed-point integer (`i32`, Q16.16 representation).
- **Per-Section Storage Descriptor (`StorageDescriptor`):** Emitted as `{ width, shift, zero_point }` where decoding to canonical `ScoreQ` is achieved via shift-and-add:
  $$\text{ScoreQ} = (\text{raw\_value} \ll \text{shift}) + \text{zero\_point}$$
- **Sentinel Encodings:**
  - `ScoreQ::MIN` (`i32::MIN` = `-2147483648`): Saturated low / minus infinity / uninitialized.
  - `ScoreQ::MAX` (`i32::MAX` = `2147483647`): Saturated high / maximum priority.

---

## 3. Residual Taxonomy & Contribution Kinds

Score composition incorporates seven distinct, typed residual contribution kinds:

1. **`RootPrior` (`B(v)`):** Base prior score assigned to root nodes.
2. **`ChildCorrection`:** Hierarchical residual correction for refined child nodes.
3. **`InteractionResidual`:** Non-linear interaction residual between co-occurring concepts.
4. **`GoalReward`:** Positive score contribution for goal state satisfaction.
5. **`ConstraintPenalty`:** Negative score contribution for hazard/constraint proximity.
6. **`UncertaintyPenalty`:** Variance/entropy penalty.
7. **`TokenEmission`:** Token log-probability emission residual.

---

## 4. Overlap Residualization & No-Double-Counting Rule

When evaluating overlapping memberships across multiresolution semantic regions:
- Every evidence contribution carries a unique 32-bit `contribution_id`.
- The canonical `ScoreAccumulator` maintains a tracked evidence set during candidate evaluation.
- **No-Double-Counting Invariant:** If a `contribution_id` has already been incorporated into the running score for a candidate, subsequent occurrences within the same candidate evaluation step are ignored.

---

## 5. Deterministic Accumulation Order & Saturation

Scores are accumulated in the strict canonical order:
$$\text{ScoreQ}_{\text{final}} = \text{RootPrior} \oplus \text{ChildCorrection} \oplus \text{InteractionResidual} \oplus \text{GoalReward} \oplus \text{ConstraintPenalty} \oplus \text{UncertaintyPenalty} \oplus \text{TokenEmission}$$

Every emitted residual is already signed in canonical `ScoreQ`; `ConstraintPenalty` and `UncertaintyPenalty` **MUST** therefore be emitted as non-positive values. The deployed runtime applies every contribution with **saturating integer addition** (`i32::saturating_add`), and overflow/underflow clamp to `i32::MAX` and `i32::MIN` respectively without panicking.

---

## 6. Deterministic Tie-Breaking Protocol

When selecting or ranking candidates (frontier eviction, shortlist selection, top-$k$ decoding):
1. **Primary Key:** `ScoreQ` descending (higher score wins).
2. **Secondary Key:** Candidate `TokenId` or `NodeId` ascending (lower integer ID wins).

This guarantees an unambiguous, 100% deterministic total order across x86_64, AArch64, and portable scalar platforms.

---

## 7. Quantization Loss & Empirical Loss Certification

Quantization error for each contribution kind is measured exclusively during offline compilation and recorded in an **empirical quantization-loss certificate** (`D(·,·) \le \epsilon`). The runtime performs 0 floating-point error computations.
