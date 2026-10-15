# #1258 — Integrate recurrent geometric reads with multi-source text

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T21:27:30Z
- **Updated:** 2026-10-09T13:17:52Z
- **Closed:** 2026-09-13T21:28:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1258
- **Merged:** 2026-09-13T21:28:58Z
- **Merge commit:** 2c2884cb4f2334b58db87f5927589e22534cc8c5
- **Base / Head:** main / UOR-Foundation:codex/recurrent-geometric-text
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Result

The earlier text loop held its query fixed, while adaptive reads ran separately. This adds one recurrent runtime and learned action policy that accumulates actual emitted bytes into a pending query, commits that state for later geometric reads, emits variable-length text, and learns when to stop. Earlier byte-link tasks use the same transition path without historical dispatch.

Final gate: **PASS_RECURRENT_GROUNDED_TEXT**. Development64/64 exact answers and source paths,32/32 changed-source pairs,8/8 four-source cases absent from the new training curriculum. Every disabled/reversed intervention removes all32long-answer successes. The same runtime retains8,256one-read,2,176dependent,2,240adaptive and576text examples. All64newFull trajectories reload exactly.

Attempt1 passed the new panel but regressed8short punctuation/EOS examples. Removing a redundant current-span-length policy input shares learned stopping parameters and recovers all8 without changing data, acceptance or adding output overrides. The negative attempt/source/executable are retained. Training fits only the shared policy over frozen learned primitives, with one categorical update from successful output-constrained trajectories; repeated confidence-only epochs were removed before fitting.

## Scope and validation

- Supplied bindings/span boundaries and observable context-length structure remain. The XOR address accumulator is order-insensitive, not semantic geometric transport. Only the shared action policy is newly trained; all new development feature rows occurred in training. No general-prose, joint primitive learning or promotion claim. Retain15baec48.
- Four new focused tests plus21retainedtests pass; both actual fit/report attempts were separately executed and sealed. `cargo fmt --all --check`, claim wording and whitespace checks pass. Queue acknowledgements are not tests.
- All37priorsealedroots/1,080files, prior source/binary, original dirty checkouts and normal model preserved. Row comparison preserves13,240successful prior-candidate retained rows and recovers8; training/development data and acceptance are byte-identical between attempts.
- Model/build/preparation/fits/controls/checks397,229/1,800,000ms; shared125,384,987/132,950,000ms; parent5,634,414/7,710,000ms. No extension, paid compute, cleanup or V3–V7 replay.

Result, exact evidence, current state and navigation identify order-sensitive learned geometric state and a contextual relation task as the next H1 responsibility. The geometric language-model objective remains binding despite permission to relax obsolete workflow restrictions.

References #973. References #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

