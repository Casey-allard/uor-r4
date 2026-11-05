# REPORT: cpu-training-pilot, deepseek-196

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[confident suffix fallback sheet](../../../harness/prep/cpu-training-pilot-confident-suffix-fallback-2026-11-05.md).
Base: `ebbe084e9`.

## Status

Complete as **STOPPED_AT_PREFLIGHT_INSUFFICIENT_FALLBACK_COVERAGE**.

The content-only table and strict local priority were both active, but local
exact copy had a candidate at every reachable generated decision. The treatment
therefore made zero confident global suffix decisions against its required
minimum of 100. The wrapper stopped before generating seed 2773.

This is not a negative for content suffix memory. It shows that the proposed
fallback is structurally unreachable when local `Longest32To1` always has a
candidate.

## What did not work

- Local exact copy covered **25,202 / 25,202** reachable generated decisions in
  the TRAIN preflight.
- Treatment, depth-one fallback and shuffled fallback were byte-for-byte
  behaviorally identical to local longest: all produced **11** exact bodies,
  **409** correct body positions, 78 BOS stops and 41 premature stops.
- The treatment made **0** confident-suffix decisions against the registered
  minimum of 100, so its local-vs-treatment and treatment-vs-shuffle change
  counts were both zero.
- The content-only arm was active, with 12,684 confident suffix decisions, but
  it is not the treatment: it reached only **43** correct body positions. This
  confirms that removing local priority is not acceptable but does not assess
  the fallback configuration.
- The first TEST FITNESS review was NOT_FIT. It found incomplete decision
  accounting, a weak local-preservation assertion and no complete-reply
  fallback fixture. All were repaired.
- The first D67 review was NO-GO because RSS was recorded but not enforced and
  fresh power/control gates were not machine-checked. Both were repaired and
  the corrected review returned GO.
- The final stop is an instrument/power condition, not a mechanism result.

## What changed

- Added `derive_suffix_content_memory`, which excludes BOS and all terminal
  transitions.
- Added `predict_confident`, requiring at least two observations and a strict
  majority successor.
- Added `SuffixFallbackArm` and a seven-arm evaluator that preserves local
  exact copy, keeps stopping in the independent boundary memory, and uses a
  global content suffix only at uncovered decisions.
- Added the `cpu-copy-suffix-fallback` driver.
- Added focused tests for content/BOS separation, local-priority preservation,
  end-to-end confident fallback and shuffled trajectory divergence.
- Added machine-enforced run RSS and coverage/control gates.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

TRAIN preflight:
`local/deepseek/suffix-fallback/2026-11-05/reports/train-preflight.json`.

- 453 conversations and 453 reachable replies;
- 13,777 content suffixes, 28,640 content transitions, 0 BOS transitions;
- 13,506 shuffled successor associations changed;
- 25,280 common generated steps across treatment and controls;
- treatment confident fallback decisions: 0;
- treatment changes from local: 0;
- treatment changes from shuffled: 0.

| Arm | Exact | BOS | Premature | Body | Local | Suffix | Writer |
|---|---:|---:|---:|---:|---:|---:|---:|
| local longest-32 | 11 | 78 | 41 | 409 | 25,202 | 0 | 0 |
| local then content-32 | 11 | 78 | 41 | 409 | 25,202 | 0 | 0 |
| local then depth-1 | 11 | 78 | 41 | 409 | 25,202 | 0 | 0 |
| local then shuffled | 11 | 78 | 41 | 409 | 25,202 | 0 | 0 |
| content-32 only | 0 | 453 | 15 | 43 | 0 | 12,684 | 453 |
| writer | 0 | 0 | 0 | 60 | 0 | 0 | 28,992 |
| reference oracle | 11 | 453 | 0 | 409 | 4,661 | 0 | 0 |

Resources:

- release build: **0.48 s**, 207,668 KiB peak RSS;
- generator build: **10.11 s**, 1,293,092 KiB peak RSS;
- TRAIN preflight: **2.84 s**, 654,344 KiB peak RSS;
- seed 2773 generated: no;
- artifact runs and repeats: none;
- local raw evidence `SHA256SUMS` SHA-256:
  `e32e1a983b473d9f336c8714b6411584010ff37a44afdb98da3825be112e3d7d`;
- raw preflight SHA-256:
  `a9ecf7e276e03a4f69c78822ba615cd1d2c2baf6574c1b4ae1bce5db596dfb79`.

Compact evidence: [result.json](evidence/result.json).

## Checks run at the head

- first non-author TEST FITNESS: **NOT_FIT**, findings repaired;
- corrected non-author TEST FITNESS: **FIT**;
- first D67 pre-run review: **NO-GO**, enforcement gaps repaired;
- corrected D67 pre-run review: **GO**;
- focused fallback tests: **3 passed**;
- full CPU release suite: **371 passed**, 4 expected ignores (375 selected);
- strict scoped Clippy, formatting, source/input pins and wrapper `bash -n`:
  passed;
- per-command RSS checks and preflight coverage/control gates: enforced;
- no fresh panel was generated.

## Delivery

PR #3752 was reviewed at exact head
`ae6a6916e4af27eec578427f0606a21480a4af00` with a counted
`uor-review v1` APPROVE and all required checks green. The protected queue
merged it as `03c929896efe795a62745281b84ebafa7fba0c77`.

Fresh `main` has identical content for all 15 changed files. The CPU package
passed **371 tests** with four expected ignores, the compact evidence manifest
read back clean, and all three source hashes still match the registration. The
release record removes the completed source claim.

## Reading

The local exact-copy domain is too complete for a “local absent” fallback to be
tested. On this design, global content memory must be allowed to compete at
shallow local depths or as part of candidate arbitration; it cannot wait for
local copy to disappear.

The next attempt should use a train-only local-depth/global-depth rule, for
example allowing a deeper confident global suffix to replace only a shallow
local candidate while preserving deep local matches. It must use a new fresh
seed because this one was registered and stopped before reading; seed 2773 is
not used as evidence here.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
D84 CPU research and protected delivery procedures were used.

## Still open

- continuation arbitration at shallow local-copy depths;
- complete-reply/fresh-phrasing capability on the one-model path;
- Claude's independent scale and serving track.

## Plain summary

The conservative fallback never ran because local exact copy always had a
candidate. The content-only table itself was active and much weaker than local
copy. The next useful test is a depth-aware arbitration, not another “local
absent” fallback.
