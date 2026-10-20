# #1349 — Learn grounded lexical realization in the retained scoped session

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-22T18:31:46Z
- **Updated:** 2026-10-09T13:17:34Z
- **Closed:** 2026-09-22T18:37:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1349
- **Merged:** 2026-09-22T18:37:10Z
- **Merge commit:** 27fdc1798f0a8536238a72062ca269f976085e73
- **Base / Head:** main / UOR-Foundation:codex/grounded-lexical-realization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executed on base `origin/main` `535b3ecc5cca1703070dfdae7cf53bc00cb342ab` (PR #1348, verified merged and equal to `origin/main`). Worktree `.worktrees/grounded-lexical-realization`. Retained attempt roots `grounded-lexical-realization-{1..9}` in `realtext-prior-2026-09-20/`; `-1` is a failed claim, `-2..-6` development, `-7..-8` reporting-identical replays, and `-9` the frozen-candidate evaluation reported here.

## What this delivers

The milestone's missing interaction: **one same-session Copy/Vocabulary/Stop policy plus a vocabulary effect conditioned on owned evidence or a derived result.** A compact finite context-indexed table changes an *uncopied* word from older owned evidence while the request and the generated prefix up to that decision are held fixed — the property a scalar copy boost provably cannot express.

Served execution is a keyed row lookup, an integer argmax and integer additions; the context key is multiply-free and there is no floating point on the served path. Fitting is integer counting over declared development response text and is explicitly offline.

## Learned realization, actual autoregressive output

| Question | Realized | Legacy | Exact span |
| --- | --- | --- | --- |
| Mara's office (one assertion) | `it is Harbor` | `Harbor` | ` Harbor` |
| Mara's office (superseded) | `it was Harbor` | `Harbor` | ` Harbor` |
| Mara's office (same-value reassertion) | `it is Harbor` | `Harbor` | ` Harbor` |
| Juno's office (unfamiliar strings) | `it is Larkspur` | `Larkspur` | ` Larkspur` |
| Juno's office (unfamiliar, superseded) | `it was Larkspur` | `Larkspur` | ` Larkspur` |
| Mara's office then i (consumed computation) | `it is now Tarn` | — | ` Tarn` |

## Reduced causal test (fixed prefix, same copied span)

| Pair | Fixed prefix | First differing decision |
| --- | --- | --- |
| direct vs superseded | 1 token (` it`) | `Insert(1)`=` is` vs `Insert(2)`=` was`, both `from_table` |
| fresh direct vs fresh superseded | 1 token (` it`) | the same two learned slots, both `from_table` |
| direct vs consumed result | 2 tokens (` it`, ` is`) | learned `Copy` vs `Insert(3)`=` now` |
| direct vs same-value reassertion | 6 tokens | none — emitted vectors identical |

## Controls

* Context-disabled (`MemoryControl::RealizationContextDisabled`, same artifact, flags held at zero): the pair **collapses** to `it is Harbor` — so the effect is not copy placement or a template shortcut.
* Source-disabled (`NoRead`): typed `NoRead`, zero emissions.
* Legacy contract on the same artifact: byte-exact owned payload in every case.

## Retained lifecycle, same artifact, same run

56/56 development, 24/24 exposed final, 32/32 fresh withheld; **38/38** prior primary/control outcomes matched with 0 missing and 0 different; the computation-forms-only ablation still fails to restore the prior lifecycle (intended falsifier); 392 rows; four roots sealed with 0 unlisted.

## Actual loaded multi-turn interaction and restart

One loaded realized runtime: observe -> ordinary `it is Harbor`; an answer started before a later correction and run after it -> `it is Harbor` (pinned view); fresh request after the correction -> `it was Quarry`; previous -> `it was Quarry`; initial -> `it is Harbor`; consumed computation -> `it is now Marsh`. Nine restart boundaries reproduce the complete frame in process, and a separate child process reloads `artifacts/realization.json` plus store/request from disk and reproduces the same emitted vector, terminal and frame.

## Independent evidence

`docs/evidence/grounded-lexical-realization-audit-2026-09-22.py` rebuilds every claim from the saved per-emission records (not the runner's booleans): each copied span and learned word from the recorded `Copy`/`Insert` actions, the emitted vector as prelude + span + suffix + terminator, the prelude length from the trace, and each causal pair's fixed prefix. Result: `all_reconstructed: true`, no rebuild problems. It cannot re-decode tokens, so decoded text is checked structurally; that limitation is recorded.

## Limits and negatives

* Four learned words; realized surfaces are *fragments*, not sentences.
* Development-exposed population; novelty is unfamiliar whole lexical strings and an unfamiliar superseded history inside authored familiar forms.
* The frozen `E/S` donor was deliberately **not** used (a one-token boost cannot change uncopied-token odds; adding it would have added a signature without capability).
* The bounded geometric evidence class is computed and reported but **not** a key field: an exact-key lookup cannot generalise to an unseen class. Recorded as a measured design negative; the class is `1` in every case here.
* Historical positions were absent from the first fit and fell back to the declared default; the multi-turn interaction found the gap and the fit was extended.
* Five library-test failures (`geometric_attention`, `lowbit_attention` x3, `lowbit_core`) were reproduced on the **clean base tree with these changes stashed** — pre-existing, not caused here. Not a full-suite pass. The touched modules pass (28 `scoped_memory`, 5 `lexical_realization`).
* Whole-path D0-b, broad prose, general reasoning/coding, geometric advantage and energy savings remain unqualified; energy `UNAVAILABLE`.

## Resources

The full projection was recorded before execution in `docs/evidence/grounded-lexical-realization-projection-2026-09-22.json` and the charge applied once in `docs/evidence/grounded-lexical-realization-charge-2026-09-22.json`. No second build tree was created (owner cache reused via `CARGO_TARGET_DIR`); the shared debug cache grew, leaving ~18 GiB free, far above the 128 MiB stop margin. No paid or external compute.

## Recommended next decision

Widen the learned lexical interface before adding representation: a source-separated text fit with a real learned vocabulary distribution and a variable-length learned prelude/suffix, keeping the same causal key, owned-span invariants and versioned contract. Do not restart arithmetic fitting or a scheduler redesign — the measured limit is support and vocabulary width, not representation.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

