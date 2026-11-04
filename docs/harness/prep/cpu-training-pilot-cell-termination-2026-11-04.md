# Writer-cell geometric termination gate

**Status:** FIT

## Question

Can a local terminal majority in the writer's 256 admission cells stop
complete replies after the single-radius point gate closed negative? At served
decode, name the BOS stop sentinel when a generated query maps to a learned top
writer cell. Compare the learned cell set with no gate and an equal-size
shuffled-cell control on both public model seeds.

This changes the termination decision from point distance to cell-density
evidence. It does not repeat point radius, rank margin, memory weighting,
target arbitration or point voting.

## Compute class

This is CPU development research under the owner-authorized D84 program in
`uor-r4-cpu-train`. It uses the local Ryzen 9 9950X and the exact integer FBS
forward. CPU output cannot substitute for GPU qualification.

## Base artifacts and inputs

| Input | Identity |
|---|---|
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |
| seed 1 trained artifact | `c856ce21cf46a4be5a0cc02e033d9c7d0cd674bc4ab96a1185a1b9ccf3b88de4` |
| seed 2 trained artifact | `10f91f7309c3ed36e3ce76878b2c75c07bae0102b3933571ccbd8f3b40b350e6` |
| seed 1 no-gate reply digest | `ea95ef96a4adad3b3964912591ae0b9b9dfecf484aaae85cad9cb4dcbc769eec` |
| seed 2 no-gate reply digest | `09920e95529cec1e4c87975d25cd9225a339c30e11a78853262fa918ed2d767c` |
| artifacts | `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-{1,2}-raw_both.uorlut` |
| corpus | `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-{tokens.u16,response-mask.u8,manifest.json}` |
| hardware | AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads |

## Data split and training

Replay deepseek-109 exactly:

- 16 repeated opening keys and up to 16 examples per key;
- 102 training and 114 evaluation examples;
- 32 attempted Q24 memory movement-coordinate updates, update seed 1;
- 32 Rayon workers and cached exact trajectory;
- 16 generated tokens per complete reply.

The trained artifact hash and deepseek-110 no-gate reply digest must match
before any cell-gate result is used. The corrected reference normalization
from deepseek-115 remains in force: one trailing corpus EOS is removed, while
decoder BOS remains the generated stop marker.

## Minimum test power

Two public model seeds and 114 document-disjoint evaluation examples per seed.
The corrected census has 1,212 content tokens and 104 references reachable
within the fixed cap and context. This is a bounded readiness screen, not a
capability claim.

## Cell gate derivation

For every prompt-final and reply-content training query:

- use the writer's exact admission scoring to choose its single highest-ranked
  cell among the 256 first-root / last-root coordinates;
- count prompt-final and earlier reply-content queries as nonterminal;
- count the final content row before corpus EOS as terminal.

This is a cell-local terminal-majority rule: a cell is learned only when its
terminal count is strictly greater than its nonterminal count. Equal and
nonterminal-majority cells are excluded. It is not a class-normalized rate.

The matched control has exactly the learned set's number of cells, sampled
deterministically without replacement from distinct observed cells outside
the learned set with the model seed. If the unselected observed-cell pool is
too small for an equal-size disjoint control, the run stops as an instrument
fault.

## Arms

Evaluate three arms on the same 114 evaluation prompts per seed:

1. `no_gate`: the existing trained memory artifact and exact writer;
2. `cell_gate`: replace a writer choice with BOS when the generated query maps
   to a learned cell;
3. `shuffled_cell_gate`: apply the same rule to the equal-size disjoint cell
   control.

Every arm uses the shared integer `reply_stop` contract and the same 16-token
cap. One trailing corpus EOS is removed from references. A BOS stop whose
tokens differ from the reference is premature.

## Steps

1. Build from the FIT-reviewed source head.
2. Pass the finite cell-gate tests before public data.
3. Run seed 1, then seed 2:

```text
env RAYON_NUM_THREADS=32 target/release/cpu-memory-panel-driver \
  /home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-SEED-raw_both.uorlut \
  /home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-tokens.u16 \
  /home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-response-mask.u8 \
  /home/callard/projects/codex-playground/uor-r4/local/deepseek/public-m1-parity/dev-manifest.json \
  OUT \
  --expected-artifact-sha256 ARTIFACT_PIN \
  --expected-tokens-sha256 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  --expected-mask-sha256 a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  --expected-manifest-sha256 ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab \
  --expected-trained-sha256 TRAINED_PIN \
  --expected-no-gate-reply-sha256 NO_GATE_REPLY_PIN \
  --steps 32 --keys 16 --per-key 16 --key-tokens 8 --reply-turn 8 \
  --reply-cap 16 --seed SEED --termination-cell-gate
```

4. Repeat seed 1 in a fresh output directory and compare deterministic fields.

## Instrument bar

- The trained artifacts, selection and no-gate digests reproduce exactly.
- Cell derivation uses training examples only; the mapping is the writer's
  top admitted cell and finite tests compare it with the shared
  writer-admission implementation on root-aligned, arbitrary and tie-shaped
  queries.
- Learned cells all have terminal count greater than nonterminal count.
- The shuffled control is the same size, disjoint from the learned set and
  byte-wise different for a nonempty learned gate.
- Reports include both cell-count maps, excluded tie/loss counts, gate hashes,
  artifact hash and ordered reply digests.
- A seed-1 repeat reproduces all gate counts, hashes, reply counters, per-row
  outcomes and ordered reply digests.
- Width mismatch, malformed training replies, an undersized control pool,
  underflow or nondeterminism refuses the run.

## Candidate bar

On each seed, `cell_gate` must strictly exceed both no-gate and shuffled-cell
controls in non-premature BOS completion
(`bos_complete - premature_stop`) and must not reduce exact-reference replies.
Raw BOS and premature counts remain reported.

A positive result is a bounded CPU development finding and justifies a fresh
phrasing follow-up. A negative closes this fixed density rule, not every
geometric termination mechanism.

## Cost and stop

Local Ryzen 9 9950X only; no GPU, pod or paid API. Two replays and three reply
arms are expected below five CPU minutes after build. Stop before seed 2 if
seed 1 projects the two-seed total beyond 30 CPU minutes.

Stop on source/input/artifact drift, leakage, missing no-gate parity, a
non-disjoint or unequal control, nondeterministic replay, counter underflow or
any edit to Claude-owned long-window files.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030
sheet: `docs/harness/prep/cpu-training-pilot-cell-termination-2026-11-04.md`
head: `bb4411ecd0c27431d946e4fb534e99a1297e4f71`
reviewer: Codex fresh-context non-author review
note: bounded two-seed D84 CPU writer-cell termination experiment; the gate
uses the writer's exact top admission cell, training-only local
terminal-majority counts, and an equal-size disjoint deterministic
shuffled-cell control; inherited corpus-EOS/decoder-BOS normalization and
non-premature BOS accounting remain fixed; 32-update replay, 114 prompts,
16-token cap, 32 Rayon workers, pinned artifacts/digests, two-seed bar, repeat
and 30-minute stop are registered.
date: 2026-11-04

This FIT clears only the registered repair at this exact source head. Any
source, hash, selection, arm, bar, control, gate-rule or cap change requires
re-review.
