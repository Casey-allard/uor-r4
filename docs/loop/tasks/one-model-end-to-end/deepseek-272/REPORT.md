# REPORT: one-model-end-to-end, deepseek-272 (aligned scenario readout)

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Base: `8b818936c`.

**In one line:** the aligned scenario readout now runs through the product
`StackChat` path, and it separates stopping from answering. The current
arm-r artifact completes 980/1,024 DEV replies and 957/1,024 Fresh replies,
but its 10/11 asked-value hits are all in the `copy` archetype; memorize,
paraphrase, selection, update, abstention and the three memory controls remain
at zero. A 16,384-step CPU rank-margin dose moved neither scenario readout and
its internal panel failed admission.

## What did not work, first

- **The first scenario readout used the wrong terminal contract.** The existing
  CPU `generate_reply` helper stops on token 0, while the scenario dialogue
  protocol terminates on EOS token 1. The public artifact emitted end-of-reply
  and was then forced to continue, producing 64-token EOS repetition. The
  first report therefore read `complete = 0`, which was an instrument fault.
  It is retained under
  `local/deepseek/scenario-training-2026-11-10/baseline/` and is not used as a
  model result.
- **A 256-step CPU smoke did not move the internal panel.** It accepted 36
  updates at 51.9 targets/s and left top-1, top-8 and target admission exactly
  unchanged.
- **The registered 16,384-step dose did not clear admission.** Top-1 rose
  181→183, but top-8 fell 296→293, target admission fell 314→313 and the
  driver returned `reject_admission`.
- **The bounded CPU dose did not change any DEV or Fresh scenario metric.**
  Both artifacts still produced the same two reply texts: 992 generic rows
  were `small Pretty good, I M coat.` and 32 general rows were
  `small Pur I G's greatice'm good,uc cobweb!.`. Exact references, asked
  values, distractors and abstentions stayed at zero.
- **Registering 32 workers did not make the training step use 32 threads.**
  `/usr/bin/time` measured 395.68 user-seconds over 308.22 wall-seconds,
  about 1.28 cores on average. The 16-core/32-thread workstation was mostly
  idle during the update loop.

## What worked

- The production corpus receipt, tokenizer, TRAIN/DEV/Fresh files and scenario
  labels regenerate and bind exactly. The CPU trainer accepted all 20,000
  TRAIN documents and 185,023 training targets.
- `cpu-scenario-grade` now encodes each held-out prompt with
  `DialogueProtocol::literal_roles_v2`, generates through the exact product
  `StackChat` path after validating any artifact-embedded tokenizer binding,
  and reports EOS, short-cycle and cap stops separately from exact references,
  asked values, distractors and abstentions.
- The full public-artifact DEV and Fresh passes use all 1,024 cases with no
  prompt-ineligible rows. The non-timing report repeated byte for byte after
  removing only `elapsed_us`; the normalized SHA-256 was
  `01931345bcdfc90dec01f5f892e12f94ac4521b2c926adc50eaa8883b8a0f565`.
- The isolated CPU run remained deterministic and artifact-bound. The final
  checkpoint and artifact reloaded, and the independent CPU/serving audit
  matched 16,384 Q12 query coordinates, 64 token decisions and 512 kept cells
  with zero mismatches.
- The synthetic end-to-end test uses the standard GPT-2 byte alphabet and a
  `/10` artifact carrying the matching tokenizer hash, so it exercises the
  separator encoding and the new binding check rather than a permissive
  legacy fixture.

## What changed

- `crates/uor-r4-cpu-train/src/scenario_grade.rs` adds receipt/tokenizer/
  corpus-bound scenario scoring, deterministic case selection and archetype
  summaries, including prompt-ineligible counts by archetype.
- `crates/uor-r4-cpu-train/src/bin/cpu-scenario-grade.rs` exposes the readout.
- `Cargo.toml` and `lib.rs` register the new component.
- The item scope and CPU research notebook record the new readout and result.

The change is about 1,020 new Rust lines plus documentation. The ORDER had no
line estimate because the readout shape was not known until the cancel probe.

No float trainer, CUDA source, serving schema, bar, milestone or Claude-owned
file changed.

## Measurements

### Scenario readout

Public trained FBS artifact:
`a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.

| Readout | DEV | Fresh |
|---|---:|---:|
| Cases | 1,024 | 1,024 |
| EOS stops | 1,024 | 1,024 |
| Short-cycle stops | 0 | 0 |
| Cap stops | 0 | 0 |
| Exact reference | 0 | 0 |
| Asked value present | 0 | 0 |
| Distractor present | 0 | 0 |
| Correct abstention | 0 | 0 |
| Distinct replies | 2 | 2 |

Current arm-r artifact:
`a1d705586b35d7da282913b1e21b242a74c0661b32d73e2ec89f56310a8140e8`.

| Readout | DEV | Fresh |
|---|---:|---:|
| Cases | 1,024 | 1,024 |
| EOS stops | 980 | 957 |
| Short-cycle stops | 33 | 67 |
| Cap stops | 11 | 0 |
| Exact reference | 0 | 1 |
| Asked value present | 10 | 11 |
| Correct direct answer | 10 | 11 |
| Distractor present | 0 | 0 |
| Correct abstention | 0 | 0 |
| Distinct replies | 428 | 536 |

Every asked-value hit was in `copy`; no `memorize`, `paraphrase`, `selection`,
`update`, `abstention`, `wrong-memory-control` or `stale-memory-control` row
produced the asked value. This is prompt copying, not grounded memory.

### CPU dose

| Field | Reading |
|---|---:|
| Target visits | 16,384 |
| Training targets available | 185,023 |
| Coverage | 8.9% |
| Accepted one-coordinate updates | 2,134 |
| Refused updates | 14,250 |
| Mean step | 18.59 ms |
| Target visits/s | 53.79 |
| Context tokens/s | 3,830.97 |
| Wall time | 308.22 s |
| User CPU time | 395.68 s |
| Peak RSS | 317,908 KiB |

The source artifact was
`a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
The final artifact was
`6989329b76ed0b3f3b72870ec161ca7ebb130fd7b2124031485255a84b4ca4ce`.
The final checkpoint was
`72f85c37962ecc58a6e03071d007985d71e4380e9f9e40e2237930377d5f1f13`.

Internal panel:

| Reading | Before | After | Delta |
|---|---:|---:|---:|
| Top-1 | 181 | 183 | +2 |
| Top-8 | 296 | 293 | -3 |
| Target admitted | 314 | 313 | -1 |
| Margin-loss sum | 10,855,392 | 10,761,373 | -94,019 |

The lower margin-loss sum did not buy ranking or admission. The driver's
decision was `reject_admission`.

### Commands

With `CORPUS` set to the production `scenario-corpus-v1` root, `TOKENIZER`
set to the pinned production `tokenizer.json`, `SOURCE` set to the public
dual-c131 seed-1 artifact, and `FINAL` set to this run's `final.uorlut`:

```sh
target/release/cpu-scenario-grade "$SOURCE" "$CORPUS" "$TOKENIZER" \
  --split development --max-cases 1024 --reply-tokens 64 --out baseline-dev.json
target/release/cpu-scenario-grade "$SOURCE" "$CORPUS" "$TOKENIZER" \
  --split fresh --max-cases 1024 --reply-tokens 64 --out baseline-fresh.json
target/release/cpu-corpus-driver "$SOURCE" "$CORPUS/train/tokens.u16" \
  "$CORPUS/train/response_mask.u8" "$CORPUS/train/manifest.json" \
  272 16384 2 1024 2048 rank-margin-16384
target/release/cpu-scenario-grade "$FINAL" "$CORPUS" "$TOKENIZER" \
  --split development --max-cases 1024 --reply-tokens 64 --out final-dev.json
target/release/cpu-scenario-grade "$FINAL" "$CORPUS" "$TOKENIZER" \
  --split fresh --max-cases 1024 --reply-tokens 64 --out final-fresh.json
```

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  scenario_grade`: 8 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-scenario-grade`: 2 passed.
- Full CPU library suite: **503 passed, 5 ignored, 0 failed** in 48.62 s.
- Release build of `cpu-scenario-grade`: passed.
- Production `scenario-corpus check`: passed.
- Zero-step TRAIN ingestion and served-forward parity: passed; the 256-step
  and 16,384-step CPU reports are recorded below.
- DEV scenario repeat after removing `elapsed_us`: byte-identical.
- Full public-artifact DEV/Fresh and arm-r DEV/Fresh scenario reports:
  completed.
- Clippy completed without a warning in either new file. Running with
  `-D warnings` still fails on 10 pre-existing warnings in
  `prompt_reply_rank.rs` and `suffix_arbitration.rs`, outside this unit.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: **PASS**. The scope
  check reads the item scope at the merge base and lists the three newly added
  CPU paths as advisory; the head updates the item scope. The single-commit
  order and pre-existing file-map advisories remain.

### Artifact paths

- Baseline and instrument-fault reports:
  `local/deepseek/scenario-training-2026-11-10/final-head/baseline/`
  and `.../baseline/` (the latter is the retained pre-StackChat instrument
  fault).
- Arm-r reports:
  `local/deepseek/scenario-training-2026-11-10/final-head/arm-r/`.
- CPU run: `local/deepseek/scenario-training-2026-11-10/rank-margin-16384/`.
- Final reports:
  `local/deepseek/scenario-training-2026-11-10/final-head/trained/`.

## Skills used

`working-discipline` defined the probe-first loop and kept the negative result
separate from the instrument result. `solution-integrity` caught that the
initial readout was measuring the wrong terminal contract. `scope-guard`
caught and reverted workspace-wide formatting changes outside the unit.

## Still open

- Parallelize candidate preflight or BVG-K proposal evaluation so a CPU
  training step uses more than about one core.
- Train mechanisms that can transform context, not only move one token-table
  coordinate at a time.
- Test whether Claude's state-keyed read can move the `memorize`,
  `paraphrase`, `selection`, `update` and abstention columns while leaving the
  wrong/stale/no-memory controls at zero.
- A GPU qualification run remains separate and is not started by this unit.

## Plain summary

The stopping problem seen in the first scenario run was partly a test bug: the
old CPU helper watched for the wrong terminal token. Through the real chat
serving path, both tested artifacts stop their replies. The current arm-r
model can repeat some values supplied directly in a copy prompt, but it cannot
answer stored-memory questions, distinguish old/new values, or abstain. A
five-minute, 16,384-step CPU update made small rank-margin movement and no
conversation change. More of the same one-coordinate update is not the next
useful investment; the next work must parallelize the CPU path and train the
actual context transform.
