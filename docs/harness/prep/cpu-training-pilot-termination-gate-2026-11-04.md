# CPU geometric termination-point gate

**Status:** FIT

## Question

Can training-derived Q12 writer query points predict reply termination strongly
enough to stop complete replies? Corpus replies end in EOS id 1. At served
decode, remap an accepted terminal decision to the shared BOS id 0 stop sentinel
when a generated query lies inside a deterministic L1 radius around a terminal
training point. Compare the learned gate with the existing no-gate reply and an
equal-size control drawn from nonterminal training queries.

This tests the termination representation directly. It does not repeat memory
movement training, rank margin, exact-target arbitration, or point voting.

## Compute class

This is CPU development research under the owner-authorized D84 program in
`uor-r4-cpu-train`. It uses the local Ryzen 9 9950X and the exact integer FBS
forward. CPU output is development research and cannot substitute for GPU
qualification.

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

- 16 repeated opening keys, up to 16 examples per key;
- 102 training and 114 evaluation examples, with one trailing corpus EOS per
  reply;
- the fixed deepseek-109 corpus split and selected key order;
- 32 attempted Q24 opening/reply movement-coordinate updates, update seed 1;
- 32 Rayon workers and cached exact trajectory;
- 16 generated tokens per complete reply.

The trained artifact hash must match its pin before any gate result is used.
The no-gate `trained` complete-reply digest must match deepseek-110 on the same
seed before the gate readings are used.

## Minimum test power

Two public model seeds and 114 document-disjoint evaluation examples per seed,
including the 104 references reachable within the registered cap and context.
The corrected model-free reference census has 1,212 content tokens and a
maximum of 35 after removing one trailing EOS per reply.
This is a bounded readiness screen, not a decisive capability run.

## Gate derivation

For each training sample, run the trained artifact with its turn-memory reads
enabled over the prompt and all reply tokens except the final EOS:

- the query at every prompt-final or reply-content row is a nonterminal query;
- the query at the final reply-content row predicts corpus EOS and is a terminal
  query;
- remove every terminal point exactly equal to any nonterminal training point;
- for each remaining terminal point find its nearest distinct terminal point;
- let `max_terminal_neighbor_distance` be the maximum of those distances;
- let `nearest_nonterminal_distance` be the minimum terminal-to-nonterminal
  distance;
- set `radius = min(max_terminal_neighbor_distance,
  nearest_nonterminal_distance - 1)` with saturating arithmetic.

If no terminal point remains, the trained and random gates are empty. If there
is no nonterminal point, the random control is the zero point and radius uses
the terminal-neighbour bound. Gate derivation is deterministic and sees only
training samples, never evaluation rows.

The matched random control samples exactly as many nonterminal training query
points as the trained gate has retained points, with replacement, using the
model seed. It uses the same radius.

## Arms

Evaluate three arms on the same 114 evaluation prompts for each model seed:

1. `no_gate`: the existing trained memory artifact and exact writer;
2. `trained_gate`: replace each writer choice with the BOS stop sentinel when
   the generated query's minimum L1 distance to a retained terminal point is at
   most the derived radius;
3. `random_gate`: apply the identical replacement rule to the equal-size
   nonterminal-point control.

Every arm uses the shared integer `reply_stop` contract and the same 16-token
cap. A BOS marker is removed from generated tokens and counted as a complete
reply. One trailing corpus EOS (or legacy trailing BOS) is normalized from the
reference, not generated output. A BOS stop whose generated tokens differ from
the normalized reference is counted as premature. References longer than the
cap/context remain unreachable and are not counted as misses.

## Steps

1. Build from the FIT-reviewed source head.
2. Pass the finite gate tests before reading public data.
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
  --reply-cap 16 --seed SEED --termination-gate
```

4. Repeat seed 1 in a fresh output directory and compare every deterministic
   report field after removing wall time, throughput and RSS.

## Instrument bar

- Training and selection reproduce the deepseek-109 trained artifact and the
  deepseek-110 no-gate reply digest exactly. The digest binds generated tokens
  and stops, so the EOS reference normalization does not alter the pin.
- Every gate coordinate is a training-derived Q12 query of the model width and
  every radius and distance is deterministic.
- The random control has the same point count and radius as the trained gate
  and differs from it byte-wise for a nonempty retained gate.
- The report records terminal points before collision, removed collisions,
  maximum terminal-neighbour distance, nearest nonterminal distance, final
  radius, gate hashes, trained artifact hash and ordered reply digests.
- The seed-1 repeat reproduces all gate counts, radii, hashes, reply counters,
  per-row outcomes and ordered reply digests.
- A malformed/empty/non-EOS training sample, width mismatch, non-`i16`
  coordinate or counter overflow refuses the run rather than truncating.

## Candidate bar

On each of both model seeds, `trained_gate` must strictly exceed both
`no_gate` and `random_gate` in non-premature BOS completion
(`bos_complete - premature_stop`) and must not reduce exact-reference replies
relative to either control. Raw BOS completion and premature stops remain
reported, but a premature BOS does not count as success. A reading whose
premature stops exceed its BOS completions is refused.

A positive result advances only as a bounded CPU development finding. It
justifies a follow-up on held-out phrasing and an integer-serving
representation; it does not qualify the mechanism or claim useful chat.
A negative closes this fixed point-gate configuration, not all geometric
termination mechanisms.

## Cost and stop

Local Ryzen 9 9950X only; no GPU, pod or paid API. Two 32-update replays plus
three reply arms are expected below five CPU minutes after build. Stop before
seed 2 if seed 1 projects the total beyond 30 CPU minutes.

Stop on source/input/artifact drift, evaluation leakage, a difference from the
deepseek-110 no-gate digest, random-control duplicate construction, a
nondeterministic repeat, an unavailable dependency or a counter overflow.

## TEST FITNESS

TEST FITNESS: FIT - M2 #2030
sheet: `docs/harness/prep/cpu-training-pilot-termination-gate-2026-11-04.md`
head: `8af603b074bed0dc1f0485f0512fb91d3837fd99`
reviewer: Codex fresh-context non-author review
note: repaired success accounting now compares non-premature BOS completion
`bos_complete - premature_stop`, rejects underflow, and retains raw BOS,
premature-stop and exact-reference readings. A finite regression proves an
all-premature candidate cannot pass. Mechanism, EOS/BOS boundary, training
replay, selection, gates, radius, arms, cap, candidate bar, seed-1 repeat and
30-minute stop remain fixed.
date: 2026-11-04

This FIT clears only the repaired comparison source at the exact head. Any
source, hash, selection, arm, cap, bar, control or gate-rule change requires
re-review.
