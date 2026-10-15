# #291 — feat(kernel): shift-add dot assignment — #243 Phase B (TLA6, P-4 clean, beam metric-consistent)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T01:01:47Z
- **Updated:** 2026-07-30T02:07:14Z
- **Closed:** 2026-07-30T01:57:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/291
- **Merged:** 2026-07-30T01:57:27Z
- **Merge commit:** dcb1eb6582aaeccae941a60288be36e1884ecd3d
- **Base / Head:** main / UOR-Foundation:issue-243-dot-kernel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Individual PR (runtime-kernel change). Implements the #243 Phase B decision made by measurement across three certify rounds, then corrected and confirmed on true integer semantics.

## Measured (integer kernel semantics, D3 harness, corpus 150k/754 stories/30,036 held-out)

| Row | top1 | agree | WB | keys |
|---|---|---|---|---|
| OLD shipped (sign-Hamming + beam) | 28.2 | 31.9 | 9.7868 | 134,733 |
| **NEW shipped (shift-add dot + metric-consistent beam)** | **30.7** | **34.3** | **9.0242** | **47,476** |
| NEW primary-key only (A-single) | 30.6 | 34.2 | 9.6389 | 47,476 |
| A-f32 ceiling | 32.1 | 36.4 | 9.6982 | 92,903 |

**Empirical Criterion (≥30.1 / ≥34.1): PASS** on real kernel arithmetic. +2.5pp top1 / +2.4pp agreement over shipped at **2.8× fewer store keys**, WB best-in-matrix. Kernel==plain equality witnessed (bundle/code/beam, CERTIFY_EXIT:0); op census: **multiply 0** — adds, shifts (shl/shr), compares, table reads only.

## What this changes

- `Compiled.dot_cb`: per-stage K×D packed u16 tables — each centroid value as two signed powers of two — derived deterministically from ctx_cb at compile time. `dot(work, cent)` is a shift-add reduction; P-4 source scan passes (chunks_exact indexing, no value multiplication in the runtime module).
- **TLA6 container** = TLA5 payload + dot tables. **Emission is opt-in (`R4_TLESS_TLA6=1`)**: a TLA6 artifact has a different κ, so flipping the default is the Phase C maintainer re-pin (era note), not a code change. TLA3/4/5 loads keep dot_cb empty → sign-Hamming path bit-for-bit unchanged (serving bundles, score-time replay, Gate C trend, κ gate all untouched).
- Runtime: both forms — plain (`assign_for_bundle`/`assign_memberships_for_bundle`, metric-agnostic membership beam) and kernel (`code_from_bundle_dot`, every op counted; OpKernel gains `shr` under the shift class). Active iff the artifact carries dot tables.
- certify: witnesses + `eval_query_beam` run through the bundle-metric dispatch; the run prints the active assignment metric.

## The beam record (honest history, commits in order)

Runs 1-2 measured the beam row at 18.3/20.6 and I misdiagnosed it twice (cost scale; dot runner-up quality) — both hypotheses were patched into functions the measured row never called, and two certify runs were wasted confirming nothing. The actual mechanism: `eval_query_beam` derived **sign-metric** membership prefixes and probed a **dot-keyed** store — pure key-space mismatch. With metric-consistent memberships the beam **helps** (30.7/34.3 vs 30.6/34.2, WB 9.02 vs 9.64), so #244's beam decision shape stands. Casey's interim beam-off approval was made on the faulty diagnosis and is superseded by this measurement.

## ⚑ For review (Casey)

- **Op budget**: dot scan ≈ 612k adds + 612k shifts per token vs ~46k on the sign path (~13×, over the design doc's ≤2× flag; all single-cycle integer ops). Mitigations, in cost order: 1-term po2 tables (measured equal accuracy in f32 emulation, halves ops), per-stage early exit, #277 VP-tree over the dot metric (the structural fix). None block this PR; certify wall-clock is unchanged.
- **A-multi ablation row** now reads 17.2/19.2 — it still fans out writes via sign memberships against the dot store (metric-inconsistent by construction). It is a non-shipped ablation; flagging rather than fixing here.

Phase C (TLA6 default + κ re-pin + serving/score signature-path adoption + BASELINE table) follows per the design doc's phasing; it also closes #230 (fragmentation resolves at 47.5k keys with higher accuracy).

Logs: /tmp/i243-kernel{,2,3,4}-certify.log (Casey's Mac).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018cWTa3B7xnsk6wygnfgL7o

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

