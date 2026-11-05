# REPORT: cpu-training-pilot, deepseek-171

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `03f220688`.

## Status

Experiment complete as **REJECT**. Delivery review and protected merge remain
pending.

## Question

Can continuation-sensitive one-step lookahead repair the target-invalid
distance-only copy score from deepseek-169?

## What did not work

Deepseek-169's lookahead ranked a candidate-conditioned next state by learned
nearest-example distance, so shuffling target identities could not change any
decision. This unit made predicted target identity primary: each unique
exact-copy candidate was fed through a cloned exact forward, and its predicted
next token was compared with the token that historically followed the retained
source occurrence.

The repaired control was non-vacuous: shuffled continuation lookahead changed
167 and 278 decisions. The treatment nevertheless remained far below the
existing exact-copy path. It scored **242 and 262/4,899** correct body
positions against **511** for both no-lookahead learned and longest-32. Only
**231 and 209/30,784** evaluated decisions found a predicted continuation that
matched the historical source continuation. The nearest-example readout is too
weak and too sparse to arbitrate this single-step objective.

## What changed

- Exposed exact-copy candidates with absolute source positions.
- Retained the deepest occurrence of duplicate candidate tokens.
- Fed each unique depth-1..4 candidate through a cloned exact forward.
- Ranked by continuation mismatch, next-state distance, source depth and lower
  token id.
- Bound predicted continuation and match status into the decision digest.
- Added tests for target-shuffle sensitivity, mismatch-before-distance
  ordering and deepest duplicate-source selection.
- Registered and ran a fresh seed-2765 panel after independent FIT and GO.

No serving, CUDA, float-training, termination or Claude-owned source changed.

## Measurements

- implementation pin: `3960dcb27a299e7de24747beb09cd75630776f3d`;
- fresh panel: seed 2765, `dev,dev`, 128 conversations and **481** reachable
  replies per artifact seed;
- decisions: **30,784** per artifact seed;
- continuation-lookahead body positions: **242/4,899** and **262/4,899**;
- no-lookahead learned body positions: **511/4,899** on both seeds;
- longest-32 body positions: **511/4,899** on both seeds;
- geometric-copy control body positions: **248** and **244**;
- candidate arbitrations: **27,318** and **26,594**;
- continuation matches: **231** and **209**;
- decisions changed versus longest-32: **4,581** and **4,914**;
- decisions changed versus no-lookahead learned: **4,467** and **4,900**;
- shuffled continuation lookahead changed **167** and **278** decisions;
- random-32 changed **3,070** decisions on both seeds;
- training retained **1,024/4,661** rows; leave-one-out top-1 was **116/125**,
  coverage **929/925**, radius **957/2,048**, and the shuffled mapping differed;
- lookahead throughput was **12.9k/12.6k** generated tokens/s, versus
  **56.8k/56.1k** for no-lookahead learned and **67.8k/66.3k** for longest-32;
- every arm had zero exact, BOS-complete and premature-BOS replies;
- fresh-process repeats were byte-identical after timing fields;
- compact evidence:
  [`continuation-2765/`](../../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/evidence/continuation-2765/),
  raw local manifest SHA-256
  `c2523410a33a4cbe130efd63440d4534f3b718443e07e4155d3cae0a95decc70`;
- sealed outcome: `REJECT`.

All eleven registered comparisons and every copy control were reachable. The
result is valid but negative; it is not an instrument or control fault.

## Checks run at the head

- CPU library: **308 passed**, 4 expected ignores.
- All CPU-training binary test targets: **83 passed**.
- Focused learned-copy tests: **12 passed**.
- Scoped Clippy with `--no-deps -D warnings`: passed.
- Rustfmt on the changed Rust files and `git diff --check`: passed.
- Independent TEST FITNESS and GO: passed before seed 2765.
- Local sealed evidence manifest and all copied evidence: verified.
- Published evidence sums and the no-corpus boundary: verified.

## What remains

This closes candidate-conditioned single-token learned lookahead with this
nearest-example readout. Do not retune it on seed 2765. Preserve longest-32 as
the stable body-selection baseline and prioritize the independent termination
and completion path; a later copy-read experiment would need a materially
stronger candidate-trajectory objective, not another scalar score over one
predicted token.

## Size

The implementation changes two source paths before documentation and compact
evidence. No `.u16` corpus is committed.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

Useful local chat remains open. This report makes no complete-reply, language
or serving claim.

## Plain summary

Making lookahead target-sensitive produced a real control and a real negative.
The learned readout matched the historical continuation on less than one
percent of evaluated decisions, while changing roughly 4,500 tokens and losing
about half of longest-32's correct body positions. Longest exact copy remains
the stronger content selector; completion work should proceed separately.
