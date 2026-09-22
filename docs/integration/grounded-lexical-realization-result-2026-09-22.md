# Learned grounded lexical realization in the retained scoped session

September 22, 2026. Executed on the base `origin/main` `535b3ecc5cca1703070dfdae7cf53bc00cb342ab` (PR #1348,
verified merged and equal to `origin/main`). Worktree `.worktrees/grounded-lexical-realization`, branch
`codex/grounded-lexical-realization`. Retained attempt roots `grounded-lexical-realization-{1..9}`; `-1` is a
failed claim, `-2..-6` are development executions, `-7..-8` are reporting-identical replays, and `-9` is the
frozen-candidate evaluation reported here. The [independent audit](../evidence/grounded-lexical-realization-audit-2026-09-22.json)
and its [read-only reconstruction script](../evidence/grounded-lexical-realization-audit-2026-09-22.py) rebuild every
claim below from the saved per-emission records.

## Decision

Retain the learned realization policy and the versioned output contract. A compact finite context-indexed table,
indexed on causal session state only, makes an **uncopied** lexical choice depend on **older owned evidence** while
the request and the generated prefix up to that decision are held fixed. This is the interaction the milestone
required and that a scalar copy boost provably cannot express; it is not a repetition of the frozen E/S donor, and
it does not replace the retained exact copy/computation lifecycle.

The milestone is **partially** met. Short realized responses are produced autoregressively by learned state and
carry exact owned spans. They are response *fragments* over a four-word learned vocabulary, fitted on a declared
authored development corpus, evaluated on a development-exposed population. General prose, semantic generalization
of arbitrary values and any geometric advantage remain unestablished.

## What was implemented

| Item | Change |
| --- | --- |
| Realization policy | new `crates/uor-r4-core/src/native_geometric/learner/lexical_realization.rs`: `RealizationModel`, `RealizationContext`, `RealizationAction`, `fit_realization` |
| Session integration | `scoped_memory.rs`: `OutputContract` (`LegacyWords` \| `RealizedV1`), `load_grounded`, `realized_emit`, `realization_context`, new session fields, new binding identity, session version 4 -> 5, `MemoryControl::RealizationContextDisabled` |
| Evaluation | `competitive-reader.rs`: realization fitted from declared development response text, `artifacts/realization.json` written and reload-verified, realized cases, reduced causal tests, context-disabled and source-disabled controls, multi-turn realized interaction, in-process and fresh-process restart |

Served execution is a keyed row lookup, an integer argmax and integer additions. The context key is
multiplication-free (shifts, rotations, xors, adds), the residual path has no multiplier instruction, and no
floating point runs on the served path. Fitting is integer counting over observed text and is explicitly offline.

The policy is a **hard action policy** over `{Copy, Insert(slot), Stop}`. Owned structural invariants keep the exact
span contiguous and complete: a learned word may never interrupt the payload and the payload may never be
truncated, so the learned choice is the words that surround the span and where to end. Every overridden action is
reported with `from_table = false`, so no learned credit is claimed for the owned guarantee.

## Retained lifecycle (same artifact, same run)

| Arm | Result |
| --- | --- |
| Development / exposed final / fresh withheld panels | 56/56, 24/24, 32/32 |
| Retained primary/control lifecycle | 38/38 matched, 0 missing, 0 different |
| Primary/control subset checks | `NoRead`, `UpdateDisabled`, `Unpinned`, `Unscoped`, `ParseScoreAuthority`, capacity arms all at their original counts |
| Computation-only support ablation | still does not restore the prior lifecycle (the intended falsifier) |
| Rows / seals | 392 rows; four attempt roots sealed with 0 unlisted files |

So the new lexical interface did **not** cost the retained memory/computation behavior. Byte-for-byte retention is
explicit: under the legacy contract the same artifact still answers exactly the owned payload (`Harbor`,
`Larkspur`, ...).

## Learned realization, actual autoregressive output

Artifact: 4 learned slots (tokens 357 ` it`, 314 ` is`, 436 ` was`, 1209 ` now`), 64 fitted rows from 736
(example, action) pairs derived by tokenizing declared development responses and locating the exact owned payload
inside them.

| Question | Realized answer | Legacy answer | Exact copied span |
| --- | --- | --- | --- |
| What is Mara's office? (one assertion) | `it is Harbor` | `Harbor` | ` Harbor` |
| What is Mara's office? (superseded) | `it was Harbor` | `Harbor` | ` Harbor` |
| What is Mara's office? (same-value reassertion) | `it is Harbor` | `Harbor` | ` Harbor` |
| What is Juno's office? (unfamiliar strings) | `it is Larkspur` | `Larkspur` | ` Larkspur` |
| What is Juno's office? (unfamiliar, superseded) | `it was Larkspur` | `Larkspur` | ` Larkspur` |
| Mara's office then i (consumed computation) | `it is now Tarn` | - | ` Tarn` |

## The reduced causal test

The strict pair holds the request **and the generated prefix** fixed and changes only older owned evidence:

| Pair | Fixed prefix | First differing decision | Same copied span |
| --- | --- | --- | --- |
| direct vs superseded (develop) | 1 token (` it`) | `Insert(1)`=` is` vs `Insert(2)`=` was`, both `from_table` | `[407,280,3259]` identical |
| fresh direct vs fresh superseded | 1 token (` it`) | same two learned slots, both `from_table` | identical |
| direct vs consumed result | 2 tokens (` it`, ` is`) | learned `Copy` vs `Insert(3)`=` now` | span differs by design (different read) |
| direct vs same-value reassertion | 6 tokens | none: the emitted vectors are identical | identical |

Why the requested change should affect the output: `prior_differs` is a property of the owned committed history for
that exact address, and `derived` is the consumed computed result; both are computed from the store at serve time.
Neither is a request feature, an evaluator label, a correctness signal or a target token.

## Targeted controls

| Control | Observed |
| --- | --- |
| Context-disabled (`MemoryControl::RealizationContextDisabled`, same artifact, flags held at zero) | the pair **collapses**: `superseded` becomes `it is Harbor`, identical to `direct` |
| Source access disabled (`MemoryControl::NoRead`) | typed `NoRead`, zero emissions |
| Legacy contract on the same artifact | byte-exact owned payload in every case |
| `E/S` donor | **not used**; see the negative below |

The context-disabled arm is what separates the vocabulary effect from copy placement: with the causal flags zeroed
the difference disappears, so the observed difference is not the pointer/copy path and not a question-template
shortcut.

## Actual loaded multi-turn interaction

One loaded realized runtime, turns in order: observe `Harbor` -> ordinary `it is Harbor`; a second answer started
and then run only **after** a later correction commits -> `it is Harbor` (pinned view retained); fresh request after
that correction -> `it was Quarry`; previous -> `it was Quarry`; initial -> `it is Harbor`; consumed computation ->
`it is now Marsh` with its exact span; mid-copy/mid-word restart at 8 in-process boundaries, all reproducing the
complete final frame. `mixed_interaction.ok` is true and is part of the aggregated checks.

## Restart

Nine boundaries (before reading, after capture, mid-vocabulary, first copy, mid-copy, payload complete, before the
learned stop, after stop, after the terminator) all restore to a byte-identical final frame in process, and a
separate child process reloads `artifacts/realization.json` plus the store and request from disk and reproduces the
same emitted vector, terminal and complete frame. A session snapshot from the previous format returns the typed
unsupported-version error rather than being reinterpreted.

## Independent reconstruction

`docs/evidence/grounded-lexical-realization-audit-2026-09-22.py` reloads the sealed attempt and rebuilds the claims
from the per-emission records alone: it reconstructs each copied span and learned word from the recorded
`Copy`/`Insert` actions, rebuilds the emitted vector as prelude + span + suffix + terminator, recomputes the
prelude length from the trace rather than trusting the counter, and re-derives each causal pair's fixed prefix and
differing decision. It checks the payload surface is inside the realized answer and that the legacy answer is the
exact payload surface. Result: `all_reconstructed: true`, every claim `true`, no rebuild problems. It cannot
re-decode tokens (no tokenizer is loaded), so decoded text is checked structurally; that limitation is recorded.

## What is learned, exact, and supplied

* **Learned:** which shared vocabulary word follows the owned history, and where to stop, as a finite table fitted
  by counting observed development response text.
* **Exact/owned infrastructure:** the copied span, the value bytes, version authority, the copy cursor, the
  structural invariants, the terminator and the store.
* **Supplied:** the declared development corpus (short response strings over declared worlds), the four learned
  words, the slot ordering and the declared development values. Nothing evaluator-supplied reaches the policy.

## Negative and unresolved

* The bounded **geometric evidence class is computed and reported but deliberately not a key field.** An exact-key
  lookup cannot generalize to a class it never observed, so a class term would have replaced a causal effect with a
  memorized per-value mapping. Recorded as a measured design negative: the class is `1` in every case here, so no
  claim is made about it.
* The **frozen E/S donor was not used.** The prior reviews already establish that a one-token boost cannot change
  the odds between two uncopied tokens, and this milestone's whole point is that interaction; adding the donor
  would have added a signature without adding capability. The donor remains the retained comparator for a future
  arm that needs its local-prefix mass.
* The realized surfaces are fragments (`it is Harbor`), not sentences. Multi-word grammatical realization is not
  claimed.
* The realized population is **development-exposed**. There is no fresh withheld realized population; novelty is
  unfamiliar whole lexical strings and an unfamiliar superseded history inside authored familiar forms.
* Historical positions were **not** in the first fit and fell back to the declared default (exact span only). The
  mixed interaction found this; the fit was extended to every learned history position and the gap closed.
* Five library-test failures (`geometric_attention`, `lowbit_attention` x3, `lowbit_core`) were reproduced on the
  **clean base tree with these changes stashed** and are pre-existing, not caused here. They are not a full-suite
  pass. `uor-r4-core` library tests for the touched modules (28 `scoped_memory`, 5 `lexical_realization`) pass.
* Whole-path D0-b, broad prose, general reasoning/coding, geometric advantage and energy savings remain
  unqualified; energy is `UNAVAILABLE` without measurement. The D0-b statement is scoped to the realized emission
  path only.

## Recommended next architectural decision

The evidence supports **widening the learned lexical interface, not the geometry.** The next milestone should move
from a four-word shared slot set to a source-separated text fit with a real (if small) learned vocabulary
distribution and a variable-length learned prelude/suffix, keeping the same causal key, the same owned-span
invariants and the same versioned contract. Do not restart arithmetic fitting, do not re-open a scheduler redesign,
and do not add S7/harmonic machinery: no witnessed representational limit has been demonstrated here — the limit
measured is support and vocabulary width, not representation.
