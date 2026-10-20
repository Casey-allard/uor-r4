# #2478 — Assess SpiralCore transport and attention proposal with reproducible evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T15:24:48Z
- **Updated:** 2026-10-14T15:36:00Z
- **Closed:** 2026-10-14T15:34:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2478
- **Merged:** 2026-10-14T15:34:01Z
- **Merge commit:** 67c57781e1f5953a42b537152f3604e0395cbe1e
- **Base / Head:** main / UOR-Foundation:codex/spiralcore-attention-research-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232. Owner-requested research assessment, no model trial; existing research-line counts are unchanged (R14).

## What and why

Evaluates six owner-supplied SpiralCore/Cantor–Abraxas transport, attention and provenance artifacts against the current UOR-R4 discrete path. Three parallel native reviewers cover mathematics, byte transport and live source, followed by reconciled exact-source review. The attention note is evaluated as a proposal, as requested. This delivers the research report and reproducible byte-codec evidence, with the canonical current-state link.

## Result

Decision: NOT YET PROMOTED for the attention proposal. Keep the demonstrated finite geometric actions and stateful byte transport at their scoped guarantees. A small frontier does not bound retained history or establish semantic correctness. The current arm is Route; Flock and the new separate prime-zeta constructor retain their existing scope. No model replacement, model computation, model-code/plan change, pod or owner release occurs.

All supplied byte-codec suites passed (6 core, 4 v1.1, 10 showcase groups). Independent byte/tree checks passed: 1,000,000 carry-accounting steps, 8,192 Tier-3 round trips and four exact reconstruction cases. Independent 128 KiB cold/warm/overwrite/prefix-insertion exchanges: 230,705 / 367 / 1,083 / 230,768 bytes. 1.59 s wall, 131,923,968 B maximum RSS, Node v26.4.0. Those are byte-codec observations, not model or physical-network performance. Source identities and original receipts are retained; user attachments are not republished wholesale.

## Exact-head review and checks

Reviewed head: 6338816d1a593dd61047565e4d2a8604cedb4ef6.

- Three independent draft reviews reconciled against source; corrected Route self-leaf eligibility, icosian-to-E8 wording, exact cold-round count and extraction layout. Final exact-head self-review verifies those corrections, preserved peer current-state entries and documentation/evidence-only diff against main. Updated main was merged normally with peer entries/receipts retained. A final source refresh incorporates #2477 finite CUDA receipt status, its wrapper checksum failure and unchanged D5 adoption boundary; an independent source reviewer checked this addition.
- `python3 scripts/check_claim_wording.py`: PASS.
- `uor-harness ratchet d23`: PASS, 1641 remaining / baseline 1641.
- `uor-harness ratchet d25`: PASS, 93 remaining / baseline 93.
- `uor-harness shard-check`: PASS after rebase, all peer receipts preserved. Checks against newer main, including the earlier CI run, correctly flagged newer Antigravity receipts absent from the older base; the normal merge retains all 90 Antigravity receipt lines.
- `git diff --check origin/main..HEAD`: PASS.
- Evidence `shasum -a 256 -c SHA256SUMS.txt`: all seven files PASS; source copied unchanged from executed audit.
- `node --check .../independent-codec.cjs`: PASS. The retained byte-codec run is identified separately from current documentation checks.
- Pre-merge harness inventory at e3db7ce86, `uor-harness check`: exit 0; 50 rules, 0 errors, 15 warnings (plus one informational finding). Reports peer stale claims/presence, legacy unknown receipt, pending sheets/panels, and this now-released delivery branch; own heartbeat refreshed afterward. No peer state is edited to clear warnings.
- `uor-harness lint check --body-file ...`: the heuristic `prlint.prereg_before_compute` misclassifies the prose review plus byte-codec numbers as model compute. Its source explicitly documents this run-word-plus-digit false-positive class (`prgates.rs:909-918`). This report executes no model, so no compute FIT is applicable or fabricated. This is a scoped manual reconciliation, not a green lint claim.
- [x] Checks run at this exact head; commands and source-bound outcomes are listed above.

No Rust/model source changed, so no model tests or compute were run for delivery. Queue compatibility statuses are not tests.

## Delivery scope

- Canonical dated record `docs/labs/spiralcore-attention-assessment-2026-10-14/README.md`, evidence and current-state entry included.
- Model milestone/STATUS/ROADMAP/#2028 headline unchanged; no capability promotion or README capability change.
- Existing stale initializer implementation wording is identified with evidence; peer-owned code is untouched.
- Training remains HELD under the existing per-seed owner release rules. This report supplies no FIT or release.
- Verify protected merge on fresh main, remove only this owned branch/worktree after preserving local review material, then run normal hygiene.

References #2029
References #2032


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

