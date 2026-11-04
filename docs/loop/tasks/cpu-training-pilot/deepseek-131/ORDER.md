# ORDER: cpu-training-pilot, deepseek-131

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `7a77bce37`.

## Goal

Train the exported FBS final gain through the complete served path so decoder
BOS id 0 ranks first at training-terminal rows while content and nonterminal
BOS suppression remain intact. Grade a document-disjoint public panel on both
seeds against frozen, shuffled-label and equal-move random controls.

D84 CPU development evidence only.

## Cancel probe (run first)

1. Public source artifacts have no Hamming, matching the cached gain limit.
2. Deepseek-118/120/122 tested post-query gates, not query training.
3. Rank-margin requires admission; deepseek-111 found BOS absent on
   93.75-97.15% of generated decisions.
4. Claude's open P2b long-context files do not overlap this CPU experiment.

**Result at `7a77bce37`:** not cancelled. The direct final-gain BOS objective is
untested, and no open ORDER claims it.

## Fixed construction

Use the established public split: even document ordinals train, odd ordinals
evaluate, 16 keys, up to eight examples per key. Derive opening/reply memory
from training examples only.

The actual next token EOS id 1 labels a training decision terminal; every other
decision is nonterminal and keeps its content target. Replay candidate final
gain, Q24 memory movement, Q12 conversion, admission and point fallback.

Each candidate changes one final-gain coordinate by one signed unit. Search all
coordinates in both directions in deterministic order and accept at most eight
strict improvements. Refuse empty, unchanged, no-terminal and Hamming-bearing
artifacts.

## Arms and bar

Frozen, true-label, shuffled-label and equal-move random final gains run.
Shuffling preserves counts and move budget; random applies the true arm's move
count. Report free generation for all four at a 16-token cap.

Both seeds must pass:

- terminal BOS top-1 at least **25%**;
- terminal BOS top-1 beats both shuffled and random controls by at least
  `max(2, ceil(2 * sqrt(b + c)))`, where `b` and `c` are discordant wins;
- nonterminal BOS selections at most **1%** and no higher than frozen;
- nonterminal content top-1 not below frozen; and
- zero query-coordinate, token and kept-cell mismatches in every arm.

Free-generation stop, exact, opening and repetition readings are reported, not
the primary bar.

## Instrument gates and steps

- Pin inputs/binary; require training terminal/nonterminal and held-out terminal
  rows.
- Require class-preserving, changed shuffled labels and safety-gated strict
  terminal improvements.
- Reparse changed artifacts; repeat seed 1 with identical non-timing fields.

Implement and test first. Then obtain FIT and a fresh non-author GO before both
seeds. Use fresh targets/roots, hash pins, resource caps and retained evidence.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/claims.json`
- `docs/harness/prep/cpu-training-pilot-bos-gain-2026-11-04.md`
- `docs/labs/cpu-training-pilot-bos-gain-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-131/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No serving, integer-session, CUDA, Claude-owned, float-training or
model-artifact source change is in scope.

## Done when

1. The cached trainer exposes exact terminal/nonterminal BOS readings and
   deterministic objective search without changing the existing distance path.
2. Focused tests cover labels, shuffle, candidate safety, refusal paths,
   random-move parity, cached/full-forward agreement and exact serving parity.
3. FIT and a fresh GO precede both seeds and the repeat.
4. The REPORT preserves either outcome, free-generation readings, parity,
   resource use and compact hashed evidence; notebook and roadmap retain it.

## Stop and ask if

- Cached and full-forward replay differ.
- A control is count- or behaviorally vacuous.
- Any accepted artifact loses exact CPU/integer parity.
- The mechanism needs a serving, integer-session, CUDA or Claude-owned change.
