# Structured argument binding: partial implementation and honest status

> Principal correction, September22: the original aligned-byte path **was exercised**; the missing test is a learned join across different BPE boundaries. The original23/24 answer loss was cue-role47/48 despite48/48 extents. Corrected draft restores24/24 answers and causal instruments, with18 module and22 runner tests passing (one legacy test ignored). Real supervision/identity/loader/restore regressions were repaired; original reports remain preserved. Ordinary-form and H4-comparison milestone remains incomplete and this PR remains draft. The principal review and completion brief are delivered separately to main; live PR #1344 owns corrected source status. The original account below is historical and corrected by the source-linked review.


September 22 UTC, 2026. **Partial.** This records what was implemented, what was measured, and what
remains unfinished. It does not claim the milestone.

## Base

`origin/main` was `6ee9d0f5` at recovery — PR #1343 merged, exactly the SHA given. Worked in the
isolated worktree `.worktrees/structured-argument-binding`; owner checkout preserved at `74fef088`.

## Implemented in `learner/observed_text_session.rs`

1. **Bounded any-order span assignment replaces marker-position derivation.** The previous
   `locate_marker` + "subject is the prefix, object is the suffix, a question is an empty suffix"
   representation is replaced by a semi-Markov tiling decode: every token belongs to exactly one
   segment, each of `subject`, `cue`, `object` may be placed **at most once and in any order**, and
   unassigned material is a scored `background` segment rather than a free skip. The decoder chooses
   the parse **by score**, so a question whose object hypothesis does not score strictly higher stays
   a question. `Segmentation { subject, cue, object, score }` is the output.
2. **Segment features add interior tokens** (`G_INTERIOR`) plus first/last/left/right/length/initial/
   final, addressing the review's "equal endpoints, length and neighbourhoods collide even if
   interiors differ" limit.
3. **Ordered H4 interval features** (off by default, `use_h4`): ordered group prefixes
   `P_j = g(x_0)...g(x_{j-1})` with `P_i^{-1}P_j` as the exact interval product, plus relative
   elements to the neighbouring intervals (`G_H4_INTERVAL`, `G_H4_LEFT`, `G_H4_RIGHT`). Per-token
   elements are learnable and are fitted by a bounded coordinate search.
4. **Exact surface separated from lexical identity.** `Clause` now carries the observed `text` and
   per-token `byte_lengths`; `lexical_key` returns the span's original bytes with only **exterior
   ASCII whitespace** removed, preserving case and interior bytes. Joins in the runtime use the key
   (`TextObjective.key`, `TextCapture.key`, `Observation.subject_key/object_key`), while the surface
   tokens remain the emission and provenance source. A clause whose recorded lengths do not tile the
   text is treated as unaligned and keys fall back to exact token identity rather than an approximate
   offset.

## Measured on the *previous* fixture (exposed regression)

| Quantity | Value |
| --- | ---: |
| Exact bounded span assignment, declared uninformed start | **0 / 48** |
| After fitting, same clauses | **48 / 48** |
| Learned segment weights | 77 |
| Development answers | 23 / 24 |
| Exposed-regression answers | 23 / 24 |
| Registry-membership continuation / max-two-read / reads disabled | 20 / 18 / 0 of 24 |

So the richer decoder learns **every declared span assignment exactly** and loses exactly one of the
24 answers on a fixture designed for the *old* fixed subject-prefix/object-suffix layout, with the same
control profile as the corrected replay (20/18/0).

## Not finished

- **The new readable multi-form fixture was not built.** Flexible argument order, real interrogatives
  ("Where does Mara work?"), a trailing adjunct outside the answer, and case-sensitive cue/name overlap
  are therefore **implemented in the decoder but not exercised**. The brief's central demonstration is
  not delivered.
- **The H4 arm was not executed.** The ordered interval features compile and are fittable, but no
  categorical-vs-H4 comparison on equal observations/support/supervision was run.
- **Five module tests still assert the previous observation contract and fail**: the old
  candidate-grammar rejection, the old report fields, and three that expect the old fixed-layout parse
  (multi-token objects on the legacy 3-token fixture). I did not weaken them to make the suite green.
  They must be reconciled by the next contributor against the new contract, deliberately and
  individually.
- The aligned-byte path is **not exercised**: the fixture's per-token bytes did not tile its text under
  the pinned tokenizer's `decode_bytes`, so keys fell back to token identity. The exact-offset identity
  claim is therefore **implemented but unverified on real clauses**.
- No new final draw, no controls under the new fixture, no evidence/issue/knowledge synchronization
  beyond this file.

## Honest conclusion

The decoder/identity change is real, compiles, runs end to end, and measurably learns the declared
span assignments. It is **not** the milestone: the new argument-support behaviour is untested, the
geometric comparison is unrun, and the library test suite is knowingly red on five previous-contract
tests.
