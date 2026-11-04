# REPORT: generated-state stop gate, deepseek-139

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `202aa1e29`.

## What did not work

- **The gate did not learn enough endpoints.** Training recall was **8/84**
  and **9/84** against the registered **26/84** floor, while staying inside
  the 1% nonterminal false-stop limit.
- **No stopped reply was exact.** Both seeds produced **0** exact references.
  Seed 1 stopped 10 replies, but 6 were within the reference length and only
  2 reached that length; none matched. Seed 2 stopped 11, with 9 within the
  reference and 4 at its length; none matched.
- **The measured shuffled gain did not replicate.** Seed 1 produced 10 true
  versus 9 shuffled BOS replies and passed the registered control-only
  threshold of 9. Seed 2 produced 21 shuffled BOS replies against 11 true and
  failed its threshold of 14, so the control completed more replies.
- **Increasing prototypes did not rescue training recall.** The registered
  preflight at 8/16/32/64/84 prototypes per class recalled only
  **8/11/5/9/12** terminal rows. The fixed eight-prototype arm was retained
  rather than silently replacing the registered configuration.
- **This is not serving evidence.** The code content arm, artifact and integer
  session remain unchanged.

## What changed

The CPU content driver can now fit a separate two-class stop gate over the
positive modulo-256 content arm's actual generated trajectories. Terminal rows
are generated decisions at the normalized reference length; every other
generated decision, including a longer one, is nonterminal.

Each class deduplicates its complete 2,048-bit signatures, selects a Hamming
medoid and seven farthest-first prototypes with lexicographic ties, and scores
a query as `nearest_nonterminal - nearest_terminal`. A training-only margin
emits BOS before the unchanged code content choice. The class-preserving
shuffled control moves each terminal label by a deterministic nonzero shift.

The gate fired on a small number of replies and left every ungated token equal
to the code arm. The public run therefore isolates termination from the
positive content representation. See the
[FIT sheet](../../../harness/prep/cpu-training-pilot-content-gate-2026-11-04.md).

Attempt 1 used the wrong paired threshold denominator and treated seed 1's
true-vs-shuffled comparison as a failure. Exact-head review caught the
instrument defect, so attempt 1 remains under
[`evidence-attempt-1/`](../../../../labs/cpu-training-pilot-content-gate-2026-11-04/evidence-attempt-1/)
with its original ledger and checksums. Attempt 2 used implementation pin
`249b29607`, the registered control-only thresholds, fresh FIT/GO reviews and
the complete rerun reported below.

## Measurements

The public artifacts are
`dual-c131-seed-1-raw_both.uorlut` and
`dual-c131-seed-2-raw_both.uorlut`; the command's runner seed was `1` for all
three runs, as registered. The split was 98 training and 103 evaluation
samples per artifact.

Training-only gate fit:

| Public seed | True margin | Shuffled margin | True recall | False stops | Shuffled recall | Shuffled false stops |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 982 | 952 | **8/84** | 9/1,484 | 11/84 | 13/1,484 |
| 2 | 975 | 953 | **9/84** | 11/1,484 | 9/84 | 14/1,484 |

The one-percent nonterminal limit was 14 false stops. Both true gates passed
the paired training-recall comparison against their shuffled labels, but both
missed the registered 26/84 absolute floor.

Generated alignment:

| Public seed | Arm | Top-1 | Depth | Opening | Distinct | Repeat 4-gram |
|---:|---|---:|---:|---:|---:|---:|
| 1 | code | 73 | 64 | 33 | 25 | 69 |
| 1 | stop gate | 73 | 64 | 33 | 27 | 64 |
| 1 | shuffled gate | 73 | 64 | 33 | 26 | 69 |
| 2 | code | 58 | 39 | 29 | 26 | 24 |
| 2 | stop gate | 58 | 39 | 29 | 28 | 15 |
| 2 | shuffled gate | 58 | 39 | 29 | 29 | 19 |

Generated stop classes:

| Public seed | Arm | BOS | Exact | Shorter | At reference | Longer | Cap | Context |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | code | 0 | 0 | 0 | 0 | 0 | 103 | 0 |
| 1 | stop gate | 10 | **0** | 4 | 2 | 4 | 93 | 0 |
| 1 | shuffled gate | 9 | 0 | 1 | 0 | 8 | 94 | 0 |
| 2 | code | 0 | 0 | 0 | 0 | 0 | 103 | 0 |
| 2 | stop gate | 11 | **0** | 5 | 4 | 2 | 92 | 0 |
| 2 | shuffled gate | 21 | 0 | 1 | 0 | 20 | 82 | 0 |

Every ungated generated decision retained the code arm's token; both true and
shuffled gates recorded zero ungated token mismatches. Both true gates raised
the number of BOS replies over code. Seed 1 also passed the shuffled
comparison by 10 candidate-only versus 9 control-only, while seed 2 failed by
11 versus 21.

Build and runs:

- release build: **19 s**, peak RSS **1,209,680 KiB**, target
  **265,930,318 bytes**;
- walls: **1.11/1.07/1.12 s** at 32 Rayon workers for seed 1, seed 2 and the
  seed-1 repeat;
- run RSS maxima: **224,080/223,536/223,116 KiB**;
- binary SHA-256:
  `ce94a067984bd6fab5f40c889ab2b9f90fb1808ec80aa45476828aa463123394`;
- committed attempt-2 evidence: **37,143,818 bytes**; attempt-1 evidence is
  retained separately at 37,143,760 bytes;
- evidence `SHA256SUMS`:
  `94ff1b09cd6e364589924351b10ff4d94d04d93ee94fe1af1f3592149a6138a5`.

Compact evidence:
[comparison.json](../../../../labs/cpu-training-pilot-content-gate-2026-11-04/evidence/comparison.json).
The source pipeline occupied **39,970,074 bytes**.

## Checks run at the head

- Fresh FIT and GO preceded both seeds and the repeat after the paired-threshold
  repair.
- Full `uor-r4-cpu-train` release suite: **245 passed, 4 expected ignores**,
  plus **62 binary tests passed**.
- Strict CPU Clippy with `-D warnings --no-deps` and targeted rustfmt passed.
  Dependency crates retained their existing dead-code warnings.
- All five public inputs and implementation pin `249b29607` matched.
- Seed-1 normalized report and all nine generated JSON files matched the
  repeat byte-for-byte.
- Recursive evidence checksums passed.
- Claim wording, docs catalog, roadmap schema/rendering, D23 and D25 passed.
  The full harness check exited 0 with no errors and 158 repository-wide
  warnings, with no warning attached to this unit.
- `git diff --check` and all unit-relative link targets passed.
- No integer-session, serving, CUDA, float-training or Claude-owned source
  changed.

## Skills used

No system-provided Codex skill applies. Prior independent FIT, GO and delivery
reviews were performed by separate subagents.

## Still open

- Serving-artifact integration and integer parity for the positive
  modulo-256 content readout.
- A termination representation or continuation objective that transfers from
  generated states and beats a matched control.
- Complete replies, fresh phrasing and useful local chat.

## Plain summary

Separating BOS from content prevented the seed-1 content loss seen in
deepseek-138, but the two-class code-signature gate still did not identify
held-out endpoints. It stopped only a small fraction of training endpoints,
emitted no exact reply, and did not produce a replicated gain over its shuffled
label control. This closes the fixed eight-prototype code-signature gate at the
1% false-stop limit; it does not close geometric termination generally.
