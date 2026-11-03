# REPORT: full-path memory-weight training, deepseek-93

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base:
`58470eb321d1c3ac37a6bc9f804f18eab0af9702`.

## What did not work

- **Top-1 did not move on the finite fixture.** Both trained arms held top-1 at
  zero of five teacher-forced decisions. The accepted updates lower the complete
  query's L1 distance to the actual target row; they are not a language gain.
- **This is one width-64 synthetic artifact and one sample.** It does not test a
  held-out panel, a public artifact, corpus-scale data, multi-step convergence
  or GPU qualification.
- **The first reply fixture did not hit its reply keys.** The implementation
  marked the reply one token too late. Moving the marker to the final prompt
  token, whose query predicts the first reply token, made the reply leg exercise
  its intended table and pass exact parity.

## What changed

Added `CpuMemoryWeightTrainer`. It treats the exported integer artifact as the
only model state:

1. It forms a complete teacher-forced prompt/reply sequence.
2. It marks the opening at the final prompt token, where the opening move
   predicts the first reply token, then executes every reply row with the
   opening and reply tables active.
3. It evaluates each decision against the actual next token's writer row using
   the complete served Q12 query.
4. It tries deterministic positive and negative Q24 movements of every selected
   opening or reply weight coordinate.
5. It writes the candidate weight into its named artifact table, reparses the
   complete artifact and reruns every decision.
6. It keeps one coordinate only when top-1 does not fall and the target-distance
   sum strictly falls. The accepted artifact then passes the complete
   CPU-versus-integer-serving audit.

The search order is opening then reply, coordinate ascent, and positive then
negative movement. The first strictly best equal candidate wins, so independent
constructions are deterministic.

## Measurements

The fixture is a synthetic width-64 FBS artifact with one opening row
`[10,11,12] -> 42` and two reply rows:

- `[10,11,12,PAD,1,PAD,42] -> 77`;
- `[10,11,12,PAD,2,PAD,77] -> 99`.

The one sample is prompt `[10,11,12]`, reply `[42,77,99]`.

| Arm | Initial hash | Final hash | Distance before → after | Top-1 | Weight |
|---|---|---|---:|---:|---:|
| opening | `70a1ca72…8bb5010e` | `09bdeb12…6fa296ee` | 1,129,804 → 1,128,232 | 0 → 0 | coord 62: 0 → 1,048,576 |
| reply | `da626c93…c9d11b4` | `ac5c799b…95d7776f` | 1,508,891 → 1,507,118 | 0 → 0 | coord 47: 0 → 1,048,576 |

`1,048,576` is one Q24 unit (`0.0625`). Each accepted arm checked six complete
positions: 384 Q12 query coordinates, six token decisions and 48 kept cells, all
with zero CPU/serving mismatches. An update changed exactly one byte in the
selected fixed-point coordinate; all keys, values, the other table and the
memory record stayed equal.

The unseen-key control was a byte-identical no-op, and two independent
constructions produced the same step receipt and final artifact.

The compact record is
[memory-weight-training.json](evidence/memory-weight-training.json).

## Reading

This is the first CPU trainer in the sequence whose update objective is measured
after the complete exported full path has applied opening/reply memory. It
closes the implementation question for a bounded finite-difference surrogate:
the memory movement weights can be changed through the served path and remain
bit-exact against integer serving.

It does not show that the update generalizes or improves a model. The next
comparison must train on one document-disjoint panel and grade another with
matched frozen-weight, random-move and no-memory controls.

## Checks run at the head

- Fresh-target release crate suite: **144 passed, 0 failed, 4 release
  benchmarks ignored**.
- Driver suites: **6, 1 and 5 tests passed** for the CPU corpus,
  counterfactual-corpus and discrete-corpus drivers.
- Focused memory-weight suite: **6 passed**.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- `cargo fmt --package uor-r4-cpu-train -- --check` and `git diff --check`:
  passed.
- The existing planted query-mismatch test remains in the full suite and still
  classifies the injected difference as an instrument fault.

## Skills used

No available Codex skill applied. The artifact patcher, complete CPU forward
and served audit were inspected directly.

## Still open

- A multi-step corpus comparison with held-out targets and matched controls.
- Checkpoint/resume for the memory-weight trainer.
- Public trained full-path artifacts whose Hamming, memory and point records
  coexist with trainable memory weights.
- GPU qualification of any resulting training configuration.

## Plain summary

The memory path no longer has to keep every movement weight at one. A bounded
CPU trainer can now change opening or reply weights, export the artifact, rerun
the complete served path and keep only an exact improvement. On the finite test,
both table types reduced query distance through the served memory move while
preserving exact CPU/serving decisions. Top-1 stayed unchanged, so this is a
training mechanism result, not a model-quality result.
