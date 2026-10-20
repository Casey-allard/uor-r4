# #1450 — feat(integer): serve width-576 dialogue models in uor-chat with 100% turn parity

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T04:40:50Z
- **Updated:** 2026-09-28T05:31:38Z
- **Closed:** 2026-09-28T05:31:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1450
- **Merged:** 2026-09-28T05:31:38Z
- **Merge commit:** 000b41c6fb01dff7ac291ad1a33f04de04658254
- **Base / Head:** main / UOR-Foundation:lab/anti-gravity/dialogue576-chat-serving
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview (Lab 3: Runtime Lab)

This PR delivers **Queue Item 2** under the Standing Charter: **Serve dialogue models in `uor-chat` at width 576 (Dot)**, fully integrated with persistent dialogue streaming and certified under transitive call-graph zero-matmul audit.

Previously, `SessionState` and `step_conversational_into` accepted only width-256 Dot shapes, preventing the `uor-chat` conversational front-end from serving width-576 dialogue artifacts trained by Codex (`dialogue-child-bundle-1` from #1431/#1432).

This pull request extends conversational serving to width 576 with zero transformers, zero matrix multiplication, zero hardware multipliers/dividers, and zero floating-point arithmetic across 100% of compiled serving symbols.

## Key Changes
1. **Width 576 Conversational Serving**:
   - Extended `SessionState` and `IntegerModel::new_conversational_session` to support width 576.
   - Sized stack buffers to 1728 (`token_affine`, `recurrent`, `fused`) and 1152 (`concat_buf`).
   - Integrated zero-allocation `normalize_state_576_into`.
   - Updated `step_conversational_into` to accumulate and store full 576-wide values.
2. **PR #1434 Streaming Integration & Front-End Hardening**:
   - Merged `origin/codex/dialogue-conversation-streaming-20260927` into this branch.
   - **Seed Tracking**: Added `starting_selection: Selection` to `DialogueConversationStream` so `into_turn()` records the true pre-turn seed/selection rather than post-generation state. Added `assert_eq!(actual.selection, expected.selection)` in reference tests.
   - **Context Capacity Guard**: Implemented prompt-aware auto-capping against `context_cap - prompt_total` to prevent runtime budget overflow aborts.
   - **Transactional Settings**: `/persona` and `/read-mode` only commit settings after `create_conversation` successfully returns.
   - **Persistence Notices**: `/save` and `/load` return plain unsupported notices.
   - **Poisoning**: Immediate error poisoning on text session initialization failures.
   - **Syntax Safety**: Replaced `_ => unreachable!()` with explicit error propagation.
3. **Opcode Violations Eliminated (0 Multipliers, 0 Dividers, 0 Floats)**:
   - **Address Multipliers (`madd`)**: Padded `SessionValues::Dialogue576` with `DialogueSlotValue576` (4096 B = 2^12 stride) to eliminate LLVM stride `madd` instructions.
   - **Float Negation (`fneg.2d`) in `extract_salient_tokens`**: Replaced `[(u32, i32); 224]` initialized to `i32::MIN` with separate flat arrays (`candidate_tokens` and `candidate_scores`), eliminating IEEE 754 `-0.0` bitmask synthesis.
   - **Float Negation (`fneg.2d`) in `T8ZetaState::step`**: Extracted `step_zeta_phase_scalar` with `#[inline(never)]`, preventing SIMD vectorizer from synthesizing float bitmasks.
   - **Quicksort Multipliers (`madd`) in `Sampler::select`**: Replaced stdlib `pdqsort` with `heapsort_ranked`, computing parent and child indices using pure bit-shifts (`>> 1`, `<< 1`) and additions with zero multiplies.
   - **SWAR Multiplier (`mul`) in Byte Matching**: Replaced `slice.contains(&b'.')` with `bytes_contain_byte`, eliminating stdlib `memchr_aligned` SWAR multiplier.
   - **Duration Multiplier (`mul` / `umulh`) in Telemetry**: Replaced `Duration::as_nanos` with `duration_nanos_exact` using bit-for-bit shift-add over the 13 set bits of 10^9.
4. **Auditor Upgrade (`scripts/audit_zero_matmul_serving.py`)**:
   - Added failing sentinels covering `mul`, `madd`, `smull`, `umulh`, `sdot`, `udot`, `smlal`, `sdiv`, `udiv`, `fmul`, `fadd`, `fneg`, `fneg.2d`, `fneg.4s`, `fabs.2d`, and `scvtf`.
   - Implemented transitive call-graph reachability traversal (`--call-graph`) traversing from serving roots (`step_conversational`, `IntegerModel4step`, `DialogueConversation7respond`, `DialogueConversationStream`, `Sampler6select`) down to all reachable callees.
   - Added `heapsort_ranked` and `step_zeta_phase_scalar` to mandatory serving symbols.

## Verification & Forensic Receipts
- **Frozen Binary Identity**:
  - Path: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-bins/698bdda481de57c2257b158bb53f7942568a3be7/uor-chat`
  - SHA-256: `4a4253a174d5af324fde647324f7830210bcb7bb0f6d5f2bdd94dcafe0c68aa4`
  - Commit: `698bdda481de57c2257b158bb53f7942568a3be7`
- **Transitive Call-Graph Disassembly Audit (`scripts/audit_zero_matmul_serving.py`)**:
  - Matched Symbol Ranges Checked: **52 symbols (0 Class I, 0 Class II, 0 Class III)**
  - Reachable Call-Graph Functions Visited: **112 functions (0 Class I, 0 Class II, 0 Class III)**
  - Missing Mandatory Symbols: **0**
  - Result: **FULL PASS** (100% clean under strict ARM64 rules)
- **TAP Version 13 Results**:
  - `ok 1 - [T1_F05_TC01] [Tier 1] Disassembly tool availability (objdump, otool)`
  - `ok 2 - [T1_F05_TC02] [Tier 1] Zero floating-point instructions in serving symbols (Class III clean)`
  - `ok 3 - [T1_F05_TC03] [Tier 1] Zero hardware multipliers in numerical serving symbols (Class I clean)`
  - `ok 4 - [T1_F05_TC04] [Tier 1] Zero hardware dividers in numerical serving symbols (Class II clean)`
  - `ok 5 - [T1_F05_TC05] [Tier 1] Strict audit exit code integrity for declared symbol coverage (0 violations)`
- **58-Turn Parity Suite (`tests/dialogue576_turn_parity.rs`)**:
  - 38 requests, 58 turns, 2,526 step calls replayed against `responses-integer.json` from #1432.
  - **Bit-for-Bit Exact Parity**: 100% exact match across all 1,433 generated tokens (0 departures).
  - Peak RSS: **30.00 MB** (< 35.0 MB gate — PASS).
  - Step Latency: **3.683 ms/step** (< 4.0 ms gate — PASS).
- **Code Hygiene**:
  - `cargo fmt --check -p uor-r4-integer`: clean.
  - `python3 scripts/check_claim_wording.py`: clean.

References #964, #962, #820, #963, #973.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5864067472 — 2026-09-28T05:30:45Z — Casey-allard

**Director: merging under the owner's instruction to clear the PR backlog.** This PR integrates #1434, #1436 and #1446; their heads are ancestors of `698bdda4`.

**Independent matcher check.** The strict pattern in `scripts/audit_zero_matmul_serving.py` still misses `fmla`, `fmax` and `fmin`. I re-ran the auditor with those added (plus the `fmls`, pairwise, reduction, compare, estimate and bf16 families, and any float vector arrangement) on the frozen binary `4a4253a1…`. Result: **0 forbidden instructions** across 52 symbol ranges and 112 reachable functions. The FULL PASS holds for this binary. A director follow-up adds these forms to the pattern, with failing sentinels. References #820.


---
_Comment/review ids in this file: 5864067472_

