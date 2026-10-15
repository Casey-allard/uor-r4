# #2093 — Review octonion substrate draft and correct sign-equivalence claims

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T21:33:33Z
- **Updated:** 2026-10-09T21:36:56Z
- **Closed:** 2026-10-09T21:34:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2093
- **Merged:** 2026-10-09T21:34:37Z
- **Merge commit:** 621a07f2011353ef64c26148b22bafd30ce95f44
- **Base / Head:** main / UOR-Foundation:codex/octonion-substrate-review-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The supplied octonion substrate PDFs describe a completed formal layer, but their literal signed table fails alternativity and norm multiplicativity, their projected route loses a coordinate, and their serialization remains empty. This review records the exact material and counterexamples, distinguishes reusable contracts from a new runtime, and corrects geometry.md's equivalence claim while retaining the canonical Rust convention and M2's earliest-query causal task.

## Result

KEEP typed algebra/state/routing/identity contracts; REJECT direct adoption of the draft table and route; a new substrate is NOT YET PROMOTED. The standalone exact arithmetic panel gives 176/224 mixed-unit left-alternativity cases for the draft versus 224/224 for the table parsed from source; both pass 64/64 single-basis cases. The explicit norm witness gives 0 versus 4; the route maps e4 and zero to the same projection. No model result or runtime change; accepted 8/512 is unchanged.

## Exact-head checks

At `474658f53c111fed02ca6be08e779406c4158727`: standalone rustc 1.97.1 compile with warnings denied and execution PASS, output matches committed arithmetic.txt; rustfmt PASS; geometry verification 29/29 PASS; SVG generation and XML parse PASS with clean tree; canonical record relative links PASS; claim-wording and diff checks PASS. The checker independently evaluates the table parsed from source; it is not a runtime/model test or general formal proof. Lean compilation, model training/grading and CUDA NOT_RUN. Independent exact-head review PASS is recorded in the review comment; protected merge and cleanup verified in the closeout comment.

## Checklist

- [x] Checks run at this exact head (commands and outcomes above; full receipt retained)
- [x] Research record: docs/labs/octonion-substrate-review-2026-10-09/README.md
- [x] New current-state entry with **Next:**
- [x] Stale-document search: N3mesis/octonion, Octonian, same algebra as, alt_basis; corrected geometry reference and figure; existing credits remain accurate
- [x] STATUS.md / ROADMAP.md / #2028 table: state and accepted model result unchanged
- [x] README: capability, headline, command and mechanism unchanged; no log entry added
- [x] Claim-wording check passes
- [x] After merge: verify fresh main, preserve evidence to iCloud, delete branch/worktree, run scoped hygiene and report on M2

References #2030


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6089625690 — 2026-10-09T21:33:45Z — Casey-allard

Independent adversarial exact-head review: PASS at 474658f53c111fed02ca6be08e779406c4158727. No actionable blocker. Independently verified literal alternativity/norm/route counterexamples; inspected checker, parsed canonical table identity, census scope, geometry/figure corrections and next-action boundaries. Read exact-head execution receipts; did not rerun tests or models. Separate math and engineering specialists both read all 16 slides and 31 discussion pages at source a8e45fe357f56cdd528bf885240059bfbec85898 and concurred: reuse existing signed operator contracts; no new model conclusion.


### comment 6089667885 — 2026-10-09T21:36:56Z — Casey-allard

Codex result: merged #2093, verified on fresh main 621a07f2011353ef64c26148b22bafd30ce95f44 (all 7 delivered blobs match reviewed head 474658f53c111fed02ca6be08e779406c4158727). Full review of the 16-slide/31-page supplied octonion documents: standalone exact arithmetic gives literal draft left-alternativity 176/224 mixed cases versus canonical-source table 224/224, while both pass 64/64 single-basis cases; explicit norm discrepancy 0 vs 4 and route projection collision reproduced. KEEP typed contracts/reuse existing operators; REJECT direct draft-table/route adoption; new substrate NOT YET PROMOTED. These are draft arithmetic findings, not model evaluation; accepted 8/512 unchanged. Exact-head expert review PASS; geometry 29/29 PASS; wording/figure checks PASS. Next remains the already-declared earliest-query intervention.

Private source/extraction/reviews/checks/merge receipt preserved: icloud:UOR-R4/results/codex/codex-octonion-substrate-review-20261009.tar, 45,209,088 bytes, MD5 000c29128cf8b123fecb11e49276539a; fresh remote index and streamed archive verified. Remote/local branch and owned worktree removed; Casey checkout clean main; scoped hygiene complete, peer worktrees preserved. About 54 GiB free. Current charge 760,754 ms includes prior delivery cleanup tail and this review through main verification; no model/GPU work. Cold OCR cache briefly exceeded the 256-MiB temporary projection by 8,680,015 bytes, recorded and removed; floor remained above 53 GiB.



---
_Comment/review ids in this file: 6089625690, 6089667885_

