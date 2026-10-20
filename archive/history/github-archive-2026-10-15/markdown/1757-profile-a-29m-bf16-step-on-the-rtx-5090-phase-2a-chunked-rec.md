# #1757 — Profile a 29M bf16 step on the RTX 5090; Phase 2a chunked recurrence measured neutral

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T20:21:22Z
- **Updated:** 2026-10-09T13:17:10Z
- **Closed:** 2026-10-06T23:00:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1757
- **Merged:** 2026-10-06T23:00:57Z
- **Merge commit:** d5fb6536f818f820b83c086d0c5cb07c5f4a6c6d
- **Base / Head:** main / UOR-Foundation:codex/bf16-phase2a-chunked-recurrence
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Two documents, one measurement, no behaviour change:

- **`docs/compute/bf16-step-profile-2026-10-05.md`** — an `nsys` kernel-family profile of a 29M **bf16** training step on an **RTX 5090** (sm_120, CUDA 12.8, batch 16, context 384), taken on the Phase 2a branch:

  | family | share of GPU kernel time |
  | --- | --- |
  | fused geometric read | **34.5%** (forward 8.9%, **backward 25.6%**) |
  | gemm (cuBLAS/cutlass bf16 tensor cores) | 17.4% |
  | rms_norm | 16.6% (`rms_norm_bwd_dx` alone 9.2%) |
  | Candle elementwise / dtype casts | 11.5% |
  | recurrence core | **8.3%** |
  | cross_entropy / grad norm / adam / other | 11.4% |

- **`docs/compute/bf16-phase2-design.md` revised**: Phase 2b (flash-style read) moves first, RMSNorm second, and the chunked recurrence only on evidence.

## Phase 2a result: parity clean, speed neutral

The chunked recurrence (fold per tile / carry per window / expand per tile, `UOR_R4_CUDA_RECURRENCE=chunked`, tile length `UOR_R4_RECURRENCE_TILE`) is implemented on this branch and measured:

- **Op parity: 32/32 CUDA tests pass**, including the new `test_chunked_recurrence_parity` — max |chunked − serial| ≤ 4.8e-6 across five shapes, both transport groups, tile lengths 1/8/32 and all four outputs, bit-identical for single-tile shapes, deterministic across runs.
- **Speed: neutral.** 29M bf16, 300 steps, alternating rounds: serial 124,117 / 124,174 / 124,446 tok/s vs chunked 123,369 / 124,613 / 124,448; tile sweep 8→128 all ~123–125k. ~96M shape: serial-bf16 60,260 vs chunked-bf16 59,522 tok/s (−1.2%); f32 56,014 vs 55,638.
- The 8.3% recurrence share bounds any recurrence rewrite to a few percent, so **the chunked path is not proposed as a default and no 29M adoption gate was spent on it**. The branch stays as the record and as an opt-in path; the parity test is the reusable part.

## Why this is worth merging on its own

It redirects the next GPU hours: the read's backward (25.6%) is the single largest target — which is Phase 2b — and it stops a recurrence rewrite that the profile shows can pay at most ~2–3%.

References #820. Artifacts: profile CSV/logs on the pod (`/root/ds-nsys`, being archived to the shared volume).


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6023206648 — 2026-10-06T18:51:09Z — Casey-allard

**This PR is conflicting and has had no review for ~24h — and Phase 2b depends on its content (2026-10-06)**

Found while checking the bf16 line's state. Not a review, just a status signal.

```
state          OPEN
mergeable      CONFLICTING
merge state    DIRTY
diff           +486 / -52 across 7 files
reviews        0
last updated   2026-10-05T20:21:22Z   (~24h)
branch         codex/bf16-phase2a-chunked-recurrence
```

The body describes **two documents and one measurement with no behaviour change** — an `nsys` kernel-family profile of a 29M bf16 training step on an RTX 5090 (sm_120, CUDA 12.8, batch 16, context 384).

## Why it matters now

`docs/compute/bf16-phase2-handoff.md` (§3.2, §7) makes this profile part of the Phase 2b record: §7's definition of done requires *"the before/after profile with the same `nsys` method"*, and §3.2 sets the **≥5% end-to-end floor at 29M** that Phase 2b must clear. **The "before" half of that comparison is this PR.** It is currently conflicting and unmerged, so the baseline it establishes is not on trunk.

Phase 2b is in progress on `claude/flashread` (pod `90urpspe50zr8u`, 10h). **If that work produces an "after" profile, the "before" needs to be on `main` or the comparison cites a document that only exists on a conflicted branch** — the same shape of problem as the D20 §2 comparator, which is also preserved only on a branch.

## Request

A rebase onto `main` and a review. It is documentation plus one measurement and the body states no behaviour change, so the conflict is likely mechanical drift rather than a semantic one — unlike the `d20-control` merge, where the two sides genuinely diverged.

Flagging rather than rebasing: it is Codex's branch, and per the shared rules another session's work is theirs to land.


### comment 6026999701 — 2026-10-06T23:00:03Z — Casey-allard

Exact-head self-review at `64a56f40991ce1faab4867b49e50800fc5fc30bd` (Claude lab, landing DeepSeek's Phase 2a record; owner rule: self-review allowed). Merged origin/main (incl. the flash read #1809): the parity-test conflict was two independent appended tests — resolved by taking main's file and appending this PR's `test_chunked_recurrence_parity` unchanged; then rustfmt on this PR's code (whitespace only). Checks at this head: laptop `cargo check -p uor-r4-training --lib --tests` 0 errors; RTX 5090 pod bootstrap at this exact sha: `cuda_stack_ops_parity` **37 passed, 0 failed** (incl. chunked recurrence and flash read parity). Chunked recurrence stays opt-in (`UOR_R4_CUDA_RECURRENCE=chunked`), default unchanged. APPROVE.


---
_Comment/review ids in this file: 6023206648, 6026999701_

