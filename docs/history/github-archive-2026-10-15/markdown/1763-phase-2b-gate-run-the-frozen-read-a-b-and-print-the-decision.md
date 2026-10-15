# #1763 — Phase 2b gate: run the frozen read A/B and print the decision

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T21:41:32Z
- **Updated:** 2026-10-09T14:17:02Z
- **Closed:** 2026-10-05T21:42:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1763
- **Merged:** 2026-10-05T21:42:47Z
- **Merge commit:** 2d2bc5712a3d3bece2f660666b6d8c5479e02ce0
- **Base / Head:** main / UOR-Foundation:deepseek/bf16-phase2b-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

`docs/compute/bf16-phase2b-gate.md` freezes the Phase 2b decision rule — recipe, arms, parity bounds, acceptance and a ≥5% speed floor — but **no script implemented it**, so the gate could not be run reproducibly. This adds the two files that do, and carries the frozen protocol onto main.

```
scripts/pod/bf16-phase2b-gate.sh build|base|ab|tabulate|all
scripts/pod/bf16-phase2b-gate.py RUNS [--json OUT]
```

**Attribution.** The two `docs/compute/bf16-phase2*.md` files are from the DeepSeek bf16 session that closed out (author: that session, not me). They live on `codex/bf16-phase2b-flash-read`, which has **no PR of its own**, so this is the only route to main. The frozen rule is unchanged; only the implementation is new. If you would rather land the docs separately, say so and I will rebase this to carry just the scripts.

## The frozen arms, as implemented

One binary, `precision=bf16` in both arms (evaluation stays f32 in both), seeds 1 and 2, the 29M Arm A recipe at 70,609 steps. Arms differ **only** in `UOR_R4_CUDA_READ=flash`. The `ab` stage is the alternating 300-step × 3-round measurement on one GPU.

- **NLL axis** — `spread = |NLL(current,1) − NLL(current,2)|`, `d_s = |NLL(flash,s) − NLL(current,s)|`; PASS iff **both** seeds have `d_s ≤ spread` and `d_s < 0.01`.
- **Speed axis** — PASS iff the flash read is ≥5% faster end to end at 29M. The gate **fails on speed alone**; this encodes Phase 2a's lesson that a neutral rewrite must not ship.

## Validation — the tabulator is tested, not assumed

A checker that has never passed a known-good input is a broken instrument, so I ran it against the published Phase 1 numbers and against three synthetic negatives:

| case | result |
|---|---|
| Phase 1's published 29M numbers | spread **0.00374975**, deltas **0.00034982** / **0.00009112** — reproduced exactly → **PASS → ADOPT** |
| flash outside the seed spread (`d = 0.01463698`) | **FAIL → DO NOT ADOPT** |
| NLL passes, read speed-neutral (ratio 1.0024 < 1.05) | **speed axis FAIL → DO NOT ADOPT** |
| run capped at 40,000 steps, `stopped_early=true` | NLL axis **NOT RUN**, verdict **INCOMPLETE** — a partial root can never be read as a result |

`bash -n` and `py_compile` clean. The script refuses to reuse a sealed report root (every retry needs a new one), and by D20 §4 an NLL gap is reported as a localiser, never as a verdict on the mechanism.

## Status and what blocks the gate

Pod `ic5zsemlme7moh` (1× RTX 5090) is leased and verified: bootstrap parity PASS, the Phase 2b branch builds, and the op suite is **31 passed / 0 failed / 1 ignored**.

The gate itself **cannot run yet**: the frozen recipe needs `$D/corpora/ts-*`, which exist only on the canonical EUR-NO-1 volume, and this pod is on the non-canonical EU-RO-1 volume. A retry loop is cycling on EUR-NO-1 stock; a pod created there mounts the canonical volume directly and needs no seeding step. The `test_flash_read_parity` test this gate requires is still to be written, alongside the kernel.

References #820


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6003565827 — 2026-10-05T21:41:39Z — Casey-allard

## Exact-head self-review — `10e2577f6e9d6a69ddc2f945630dcb766b2c0b26`

Owner direction permits self-review with a check list.

**Scope.** `git diff --name-only origin/codex/bf16-phase2b-flash-read...HEAD` lists exactly the two scripts — the implementation delta is clean and separate from the docs. Against `origin/main` the PR carries 4 files because the docs branch has no PR of its own (flagged in the description for the owner's call).

**Frozen-rule fidelity.** The script implements, and does not redefine, `bf16-phase2b-gate.md`: arms `current`/`flash` on one binary with `precision=bf16`, seeds 1/2, 70,609 steps, arms differing only by `UOR_R4_CUDA_READ=flash`; `d_s ≤ spread ∧ d_s < 0.01`; ≥5% speed floor.

**Checks executed.**

| check | result |
|---|---|
| `bash -n` on the shell script | clean |
| `py_compile` on the tabulator | clean |
| known-good input (Phase 1 published numbers) | spread `0.00374975`, deltas `0.00034982`/`0.00009112` — reproduced exactly → PASS → ADOPT |
| flash outside spread | FAIL → DO NOT ADOPT |
| NLL pass, speed neutral (1.0024 < 1.05) | speed FAIL → DO NOT ADOPT |
| capped/stopped_early root | NOT RUN → INCOMPLETE |
| sealed-root reuse | refused |

The known-good case is the important one: a checker is only trustworthy once it passes an input known to pass. These are the real published Phase 1 numbers, not invented ones.

**Not done here.** The kernel and `test_flash_read_parity` are the implementation this gate will exercise; the gate cannot run until the canonical-volume corpora are reachable. Stated in the description rather than implied.

No blocking issues found.


---
_Comment/review ids in this file: 6003565827_

