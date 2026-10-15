# #1766 — Audit frozen geometric cue credit without further fitting

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T22:52:49Z
- **Updated:** 2026-10-09T13:17:22Z
- **Closed:** 2026-10-06T22:44:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1766
- **Merged:** not merged
- **Merge commit:** 27f902e84fe2ca77b6b18c1ea408efc3ad74866a
- **Base / Head:** codex/geometric-cue-composition-20261005 / UOR-Foundation:codex/geometric-cue-credit-audit-20261005
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The prospective cue fit changed evaluation margins without source-decision crossings. This adds and executes a development-only audit at the frozen selected native checkpoint to distinguish conflicting shared-address credit from narrow update coverage.

The Rust driver processes all 512 development rows through the unchanged native answer/EOS objective, with all Source candidates admitted and integer trace parity at every token. Each row loss matches the sealed selected canonical trace. Saved first-read metadata is checked against the independently compiled selected cue. It records signed and absolute credit, cancellation and concentration across all 960 cue coefficients. No optimization, proposals, generation or evaluation gradients occur; cue, source, prefix and endpoint remain frozen.

Measured result: all 512 rows and the full-panel gradient pass. Native CE is 0.98608077336532 for both saved reference and actual audit; maximum row-average/full-batch gradient error is 3.430614e-9. 721 coordinates have nonzero credit, 455 receive both signs, and magnitude-weighted cancellation is 57.99%. None of the 721 negative-gradient quarter directions is blocked by Q4 saturation. Development-only top-eight support covers 204 rows with nonzero full-answer credit versus 36 for top one. These findings motivate a separately declared same-checkpoint native one-quarter supports {1,8} discriminator; they do not establish representational impossibility or finite-step benefit.

Validation at exact archived head ba49e76e (numerical source aa6c679d; subsequent 69e095b7 changes only documentation): 42 fitter and 49 observation release tests, rustfmt and release build pass; all commands exit 0. Actual Linux x86 audit completes in 352.171s, CPU16, sampled owned RSS 1,553,018,880 bytes. Wrong-reference rejection and frozen input/parameter checks pass. Independent source review and saved-file analysis pass. No Metal/CUDA model behavior was tested. This is native gradient attribution on a pod CPU backend, not D19 session/reply grading.

Durable evidence: canonical EUR-NO volume lmd1pfah3y, /workspace/uor-r4/codex/cue-credit-audit/{checks-1,audit-1,analysis-1}. Audit manifest SHA256: 183465cef3e80167448f6085295f4e88341bf4222a52d92b8de8568a277fe4f6. Executable SHA256: 6b0f988bd60cebea2f4f5327f057f3608b3a23fd1e0f5adacfe821848abc45c2.

DeepSeek owns softmax/tensor-core work; the loss here stays fixed and that seam is untouched. Full-answer gradients can include prefix/emission/normalization effects; first-read occurrence heterogeneity is not typed-record contrast. No replicated transfer or general attention/chat qualification.

Stacked on PR1762. References #820 and #1552. Complete 60-minute work envelope was precharged while preserving prior cumulative ledger charges; limits unchanged. Protected delivery remains pending.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6004878661 — 2026-10-05T22:55:54Z — Casey-allard

Independent exact numerical source review PASS aa6c679d9084923f566d9114c194b4ef0f9b5d43; shared prospective loader, honest occurrence-candidate incidence scope and saved/actual cue metadata linkage repaired. ba49e76e adds current-state only; numerical sources unchanged. Rustfmt/diff and claim-wording pass. Actual Cargo and512row audit NOT_RUN while the single bounded uor-pod stockwait runs (session cue-credit-audit, deadline23:04Z). No modelquality conclusion or hardargmax derivative claim follows from source review.

### comment 6005246601 — 2026-10-05T23:18:43Z — Casey-allard

Exact-head final review: 69e095b7 is a documentation-only delta from executed ba49e76e; numerical source remains independently reviewed aa6c679d. I verified measured counters, source/executable/config/checkpoint identities, zero-update/frozen boundaries and saved-analysis interpretation against actual pod outputs; claim-wording and diff checks pass. Root independently reconstructed all512 sparse row-gradient signed sums with zero error. Source critic passed aa6; saved-analysis reviewer independently checked512 metadata objects and all public-axis partitions. A new critic turn was unavailable due agent-thread limit; owner-authorized self-review covers the documentation-only final head. Actual release checks and measured audit are recorded in the PR body; compatibility CI is not numerical validation. No general attention, impossibility or fresh transfer verdict.

### comment 6026815719 — 2026-10-06T22:44:46Z — Casey-allard

RECORD: frozen512-row credit audit reproduced CE0.98608077336532; 57.99% magnitude-weighted cancellation, no fitting or finite-step benefit measured. Retained result/artifacts: https://github.com/UOR-Foundation/uor-r4/pull/1766. Branch preserved.


---
_Comment/review ids in this file: 6004878661, 6005246601, 6026815719_

