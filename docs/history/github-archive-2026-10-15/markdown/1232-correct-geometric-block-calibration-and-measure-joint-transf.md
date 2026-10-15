# #1232 — Correct geometric block calibration and measure joint transfer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T00:07:33Z
- **Updated:** 2026-10-09T13:17:26Z
- **Closed:** 2026-09-13T00:09:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1232
- **Merged:** 2026-09-13T00:09:10Z
- **Merge commit:** 386a078ae147f6143366b2741b67805ca2b7544c
- **Base / Head:** main / UOR-Foundation:codex/geometric-loss-frontier
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The shared-core emitter's three single-coordinate calibration passes missed a lower-loss angular-cap setting. An exhaustive numerical loss/error search finds 211/660 ASCII errors at NLL sum 406.900191, versus 220 errors at 414.318812, with the original gate unchanged. Existing finite paired-update source and primary min/max/discrete-learning research informed a complete block-calibration correction.

The new Rust offline learner minimizes each occupied node's complete angular-cap block by likelihood alone, using a compact survival-count identity. It preserves the serving equation, canonical geometry and recurrent parameters during calibration; source/config/data/parent provenance and old schema-2 byte compatibility remain explicit. Deadlines return no partial artifact. The serving integer-kernel source is byte-identical to the parent commit.

Actual calibration: **PASS_BLOCK_CALIBRATION**. One sweep of 94 occupied nodes/327,571,200 settings takes 15.198 seconds; construction accuracy 65→80/672, root 12/672 and ASCII 211/660 satisfy the unchanged conditions. Preserve block artifact 4d83b1bb and previous 968cf841.

The admitted single joint fit nevertheless returns **FAIL_CONTEXTUAL_TRANSFER_SMOKE**. Candidate ade9a1cf exhausts 100,000 proposals in 31.444 seconds. Fresh NLL improves only 1.19185% against block parent (required >2%); accuracy 29→28/381. Context removal raises loss 1.10791% (passes its scoped criterion), prior-root removal only 0.27630% (fails). All four Full candidate continuations remain incoherent and hit 96 bytes without EOS. No model promotion or issue completion is claimed; retain 15baec48.

Validation: exhaustive saved-state audit and direct arithmetic fixture; 18 focused shared-core tests; final release build; actual model fitting, 48 generated cases and actual-candidate 512-step zero-allocation census; formatting, claim wording and diff checks. The focused tests precede two reviewed final deadline guards; final executable/model/allocation runs use corrected source. All seven sealed report file sets/hashes, final source freeze, binaries and retained artifact hash verify. Queue acknowledgements are not test results.

Resources: 331,322/450,000 model/build/test ms; 17,362,944 additional storage bytes within 256 MiB; all cumulative charges preserved, no model/storage extension or paid compute. Current-state, research rationale, exact evidence and project-local restart are updated. Next is a bounded hard-forward categorical credit-assignment check for the existing recurrent/read parameters, with surrogate directions compared against actual finite loss changes before a new training campaign. Capacity/curriculum remain unresolved; no extra serving mechanism is adopted.

References #973 and #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

