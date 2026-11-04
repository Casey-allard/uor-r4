# ORDER: cpu-training-pilot, deepseek-132

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `e3959b140`.

## Goal

Test a CPU-only two-class Hamming stop candidate outside writer admission.
Convert each 256-coordinate Q12 writer query to a 256-bit signature, learn
terminal/nonterminal prototypes, fit a training-only margin, and emit BOS
before falling back to the exact writer token.

D84 CPU development evidence only; integer serving is unchanged.

## Cancel probe

Prior tests used an L1 radius, writer cell, 32-value orbit phase and final-gain
distance, not a full-query binary Hamming classifier. Deepseek-111 found BOS
absent from writer admission on 93.75-97.15% of generated decisions and outside
top-8 when admitted. Claude's P3a work does not overlap.

**Result at `e3959b140`:** not cancelled; this mechanism and admission bypass
are untested.

## Fixed construction

Use the public repeated-key split: even documents train, odd evaluate, 16
eight-token keys, up to eight per key. Derive memory from training only.

Evaluate each prompt-final and reply-content query. The final reply-content
decision is terminal; all included others are nonterminal.

Pack the exact Q12 query into four little-endian `u64` sign words;
`query[i] >= 0` sets its bit.

For each class, deduplicate signatures; select a Hamming medoid, then up to
seven farthest-first prototypes; break ties lexicographically. Score is
`d_nonterminal - d_terminal`; BOS fires at or above the fitted margin.

Fit on training rows only. Among thresholds with nonterminal false stops at or
below 1%, maximize terminal recall; ties choose the larger margin, then smaller
threshold. Refuse empty classes, duplicate-only prototypes, unchanged shuffles
or no valid margin.

## Arms and bar

Run no gate, true-label prototypes/margin and class-preserving shuffled-label
prototypes/margin. Shuffling must change the terminal subset. All arms use one
artifact.

Both seeds must pass:

- at least **26/103 (25%)** terminal rows stop correctly;
- nonterminal false stops are at most **1%**, and the gate fires at least once;
- true beats no-gate and shuffled terminal stops by
  `max(2, ceil(2 * sqrt(b + c)))`; and
- every ungated decision is exactly the writer token; every gated decision is
  exactly BOS.

Prompt-final rows are nonterminal. Free generation at 16 tokens reports BOS,
premature BOS, cap, context, exact, opening, distinct and repeated 4-grams.

## Instrument gates

- Pin source/inputs/binary; prove both splits contain both classes.
- Report training stops, margin, prototype hashes and gate exposure.
- Repeat seed 1 with identical non-timing fields/configuration.
- A pass is CPU candidate evidence only; integer integration is a later
  coordinated unit requiring exact stop/token parity.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-stop-signature-2026-11-04.md`
- `docs/labs/cpu-training-pilot-stop-signature-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-132/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No serving, integer-session, CUDA, Claude-owned or float-training source changes.

## Done when

1. The CPU crate exposes deterministic packing, prototypes, margin fitting,
   teacher-forced evaluation and gated free replies.
2. Tests cover Hamming order, prototype ties, refusals, shuffled labels,
   prompt-final accounting, token identity and deterministic construction.
3. FIT and a fresh GO precede two seeds and a repeat.
4. The REPORT preserves either outcome, free-generation readings and the
   serving-integration boundary; notebook and roadmap retain the result.

## Stop and ask if

- The mechanism requires writer admission or integer-serving changes.
- A class, control, prototype set or qualifying margin is vacuous.
- An ungated token differs from the exact writer decision.
- The repeat differs or source/input identity drifts.
